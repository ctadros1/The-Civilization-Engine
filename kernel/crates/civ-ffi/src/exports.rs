//! The C functions of ABI 1 (ADR-0005). This is the only hand-written `unsafe` code in the
//! kernel: it turns the host's raw pointers into references and slices for one call each. Every
//! function catches panics; a panic faults the kernel it happened in.

#![allow(unsafe_code)]

use std::any::Any;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::ptr;

use crate::kernel::{Config, Health, Kernel, PanelConfig, Refused};

/// The ABI's major version. A host built for another major must not use this library.
pub const TCE_ABI_MAJOR: u32 = 1;
/// The ABI's minor version. Minors only append functions to the table and fields to structures.
pub const TCE_ABI_MINOR: u32 = 0;

/// Done.
pub const TCE_OK: i32 = 0;
/// Nothing is waiting: no frame to poll, or no snapshot newer than the one asked after.
pub const TCE_NONE: i32 = 1;
/// The output does not fit; `out_len` holds the size needed and the frame is kept.
pub const TCE_BUFFER_TOO_SMALL: i32 = 2;
/// A null pointer, a structure smaller than this ABI's, an empty or non-UTF-8 path.
pub const TCE_INVALID_ARGUMENT: i32 = -1;
/// Not a frame the kernel accepts (see `last_error`); nothing was queued.
pub const TCE_BAD_FRAME: i32 = -2;
/// The kernel could not start, or the panel server could not (see `last_error`).
pub const TCE_FAILED: i32 = -3;
/// The kernel failed while running and accepts nothing more; destroy it.
pub const TCE_FAULTED: i32 = -4;
/// The host wants another ABI major or a larger table than this library has.
pub const TCE_INCOMPATIBLE: i32 = -5;
/// The engine has stopped.
pub const TCE_STOPPED: i32 = -6;

/// Bytes the caller owns, lent for one call: UTF-8 text for strings and paths, no NUL needed.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TceBytes {
    /// The first byte; may be null when `len` is 0.
    pub ptr: *const u8,
    /// How many bytes.
    pub len: usize,
}

/// What `create` needs.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TceConfig {
    /// `sizeof(TceConfig)` as the host compiled it.
    pub struct_size: u32,
    /// Reserved: 0.
    pub flags: u32,
    /// The content folder (the repository's `content/`).
    pub content_dir: TceBytes,
    /// The saves folder; created when missing.
    pub saves_dir: TceBytes,
}

/// What `serve_panels` needs.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TcePanelConfig {
    /// `sizeof(TcePanelConfig)` as the host compiled it.
    pub struct_size: u32,
    /// Port on 127.0.0.1; 0 picks a free one.
    pub port: u16,
    /// Reserved: 0.
    pub reserved: u16,
    /// The built web shell (`web/dist`); empty when there is none.
    pub web_dir: TceBytes,
}

/// The versions this library speaks: of the ABI, and of the schemas and content it carries.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct TceVersions {
    /// `sizeof(TceVersions)` as the host compiled it.
    pub struct_size: u32,
    /// `TCE_ABI_MAJOR`.
    pub abi_major: u32,
    /// `TCE_ABI_MINOR`.
    pub abi_minor: u32,
    /// The boundary schema's major version (ADR-0001).
    pub wire_major: u32,
    /// The boundary schema's minor version.
    pub wire_minor: u32,
    /// The save schema version (ADR-0002).
    pub save_schema: u32,
    /// The content API version content packs must declare.
    pub content_api: u32,
}

/// A running kernel. Opaque: only pointers to it cross the boundary.
#[derive(Debug)]
pub struct TceKernel {
    kernel: Kernel,
}

/// The table of ABI 1 functions `tce_get_api` fills.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TceApiV1 {
    /// How many bytes of the table this library filled.
    pub struct_size: u32,
    /// `TCE_ABI_MAJOR`.
    pub abi_major: u32,
    /// `TCE_ABI_MINOR`.
    pub abi_minor: u32,
    /// Writes the versions this library speaks into `out` (its `struct_size` set by the host).
    pub versions: Option<unsafe extern "C" fn(out: *mut TceVersions) -> i32>,
    /// Creates a kernel. Writes it to `out` even when it could not start: `create` then returns
    /// `TCE_FAILED`, and the kernel must still be destroyed.
    pub create:
        Option<unsafe extern "C" fn(config: *const TceConfig, out: *mut *mut TceKernel) -> i32>,
    /// Stops and frees a kernel: the panel server, then the engine (saving a changed world).
    /// Blocks. Must not race any other call on the kernel, which must not be used again.
    pub destroy: Option<unsafe extern "C" fn(kernel: *mut TceKernel) -> i32>,
    /// `TCE_OK` while running, `TCE_FAILED` if it never started, `TCE_FAULTED` if it failed.
    pub status: Option<unsafe extern "C" fn(kernel: *const TceKernel) -> i32>,
    /// Copies the last failure, UTF-8, into `buf`; its length goes to `out_len`.
    pub last_error: Option<
        unsafe extern "C" fn(
            kernel: *const TceKernel,
            buf: *mut u8,
            cap: usize,
            out_len: *mut usize,
        ) -> i32,
    >,
    /// Queues one commons-wire frame: a `Command`, `Query` or `Heartbeat`. Copied before return.
    pub submit:
        Option<unsafe extern "C" fn(kernel: *mut TceKernel, frame: *const u8, len: usize) -> i32>,
    /// Copies the next ordered frame (`Welcome`, `Events`, `Response`, `Error`) into `buf`.
    pub poll: Option<
        unsafe extern "C" fn(
            kernel: *mut TceKernel,
            buf: *mut u8,
            cap: usize,
            out_len: *mut usize,
        ) -> i32,
    >,
    /// Copies the latest `Snapshot` frame into `buf` if its number is greater than `after`, and
    /// its number into `out_number`.
    pub copy_snapshot: Option<
        unsafe extern "C" fn(
            kernel: *mut TceKernel,
            after: u64,
            buf: *mut u8,
            cap: usize,
            out_len: *mut usize,
            out_number: *mut u64,
        ) -> i32,
    >,
    /// Serves the web panels and their socket on 127.0.0.1; `TCE_OK` if already serving.
    pub serve_panels:
        Option<unsafe extern "C" fn(kernel: *mut TceKernel, config: *const TcePanelConfig) -> i32>,
    /// Copies the panels' address, with port and token, into `buf`; `TCE_NONE` before
    /// `serve_panels`.
    pub panel_url: Option<
        unsafe extern "C" fn(
            kernel: *const TceKernel,
            buf: *mut u8,
            cap: usize,
            out_len: *mut usize,
        ) -> i32,
    >,
}

const TABLE: TceApiV1 = TceApiV1 {
    struct_size: size_of::<TceApiV1>() as u32,
    abi_major: TCE_ABI_MAJOR,
    abi_minor: TCE_ABI_MINOR,
    versions: Some(versions),
    create: Some(create),
    destroy: Some(destroy),
    status: Some(status),
    last_error: Some(last_error),
    submit: Some(submit),
    poll: Some(poll),
    copy_snapshot: Some(copy_snapshot),
    serve_panels: Some(serve_panels),
    panel_url: Some(panel_url),
};

/// Fills `out` with the table of ABI 1 functions: the library's only export (ADR-0005 §2).
/// Returns `TCE_INCOMPATIBLE`, writing nothing, when `abi_major` is not `TCE_ABI_MAJOR` or
/// `out_size` is smaller than this ABI's table.
///
/// # Safety
///
/// `out` must be null or point to `out_size` writable bytes, aligned for `TceApiV1`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tce_get_api(abi_major: u32, out: *mut TceApiV1, out_size: usize) -> i32 {
    if abi_major != TCE_ABI_MAJOR || out_size < size_of::<TceApiV1>() {
        return TCE_INCOMPATIBLE;
    }
    if out.is_null() {
        return TCE_INVALID_ARGUMENT;
    }
    // SAFETY: `out` is not null, and the host promises `out_size` writable, aligned bytes, at
    // least the size of the table.
    unsafe { ptr::write(out, TABLE) };
    TCE_OK
}

/// A panic's message, as far as it has one.
fn panic_message(panic: &(dyn Any + Send)) -> String {
    panic
        .downcast_ref::<&str>()
        .map(|s| (*s).to_owned())
        .or_else(|| panic.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "a panic without a message".to_owned())
}

/// Runs `f` on the kernel behind `kernel`, faulting it if `f` panics.
///
/// # Safety
///
/// `kernel` must be null or a kernel `create` returned that has not been destroyed.
unsafe fn with_kernel(kernel: *const TceKernel, f: impl FnOnce(&Kernel) -> i32) -> i32 {
    if kernel.is_null() {
        return TCE_INVALID_ARGUMENT;
    }
    // SAFETY: not null, and the host passes a live kernel from `create`; every function but
    // `destroy` only reads through it, and the kernel synchronises internally.
    let kernel = unsafe { &(*kernel).kernel };
    match catch_unwind(AssertUnwindSafe(|| f(kernel))) {
        Ok(code) => code,
        Err(panic) => {
            kernel.fault(format!("the kernel panicked: {}", panic_message(&*panic)));
            TCE_FAULTED
        }
    }
}

/// The bytes behind `bytes` as a path, or `None` when empty, null or not UTF-8.
///
/// # Safety
///
/// `bytes.ptr` must point to `bytes.len` readable bytes when `len` is not 0.
unsafe fn path_of(bytes: TceBytes) -> Option<PathBuf> {
    if bytes.len == 0 || bytes.ptr.is_null() {
        return None;
    }
    // SAFETY: not null, and the host lends `len` readable bytes for this call.
    let raw = unsafe { std::slice::from_raw_parts(bytes.ptr, bytes.len) };
    std::str::from_utf8(raw).ok().map(PathBuf::from)
}

/// The caller's output buffer as a slice; empty when `cap` is 0.
///
/// # Safety
///
/// `buf` must point to `cap` writable bytes when `cap` is not 0.
unsafe fn out_buffer<'a>(buf: *mut u8, cap: usize) -> Option<&'a mut [u8]> {
    if cap == 0 {
        return Some(&mut []);
    }
    if buf.is_null() {
        return None;
    }
    // SAFETY: not null, and the host lends `cap` writable bytes for this call.
    Some(unsafe { std::slice::from_raw_parts_mut(buf, cap) })
}

/// Writes `value` through `out` when it is not null.
///
/// # Safety
///
/// `out` must be null or valid for writing a `T`.
unsafe fn put<T>(out: *mut T, value: T) {
    if !out.is_null() {
        // SAFETY: not null, and the host promises it is valid for writing a `T`.
        unsafe { out.write(value) };
    }
}

fn code_of(health: Health) -> i32 {
    match health {
        Health::Running => TCE_OK,
        Health::Failed => TCE_FAILED,
        Health::Faulted => TCE_FAULTED,
    }
}

fn refused_code(refused: &Refused) -> i32 {
    match refused {
        Refused::Nothing => TCE_NONE,
        Refused::TooSmall(_) => TCE_BUFFER_TOO_SMALL,
        Refused::BadFrame(_) => TCE_BAD_FRAME,
        Refused::NotRunning(health) => code_of(*health),
        Refused::Stopped => TCE_STOPPED,
        Refused::Panels(_) => TCE_FAILED,
        Refused::Broken(_) => TCE_FAULTED,
    }
}

/// Copies `text` into the caller's buffer, as `poll` copies frames.
///
/// # Safety
///
/// `buf` must point to `cap` writable bytes when `cap` is not 0; `out_len` must be null or
/// writable.
unsafe fn copy_out(text: &[u8], buf: *mut u8, cap: usize, out_len: *mut usize) -> i32 {
    if out_len.is_null() {
        return TCE_INVALID_ARGUMENT;
    }
    // SAFETY: `out_len` is not null, and the host promises it is writable.
    unsafe { out_len.write(text.len()) };
    if text.len() > cap {
        return TCE_BUFFER_TOO_SMALL;
    }
    // SAFETY: the host lends `cap` writable bytes at `buf`.
    let Some(dest) = (unsafe { out_buffer(buf, cap) }) else {
        return TCE_INVALID_ARGUMENT;
    };
    dest[..text.len()].copy_from_slice(text);
    TCE_OK
}

/// # Safety
///
/// `out` must be null or point to a writable `TceVersions` whose `struct_size` the host set.
unsafe extern "C" fn versions(out: *mut TceVersions) -> i32 {
    if out.is_null() {
        return TCE_INVALID_ARGUMENT;
    }
    // SAFETY: not null; the host promises a writable `TceVersions`. Its size is checked before
    // anything is written.
    let size = unsafe { (*out).struct_size } as usize;
    if size < size_of::<TceVersions>() {
        return TCE_INVALID_ARGUMENT;
    }
    let versions = TceVersions {
        struct_size: size_of::<TceVersions>() as u32,
        abi_major: TCE_ABI_MAJOR,
        abi_minor: TCE_ABI_MINOR,
        wire_major: u32::from(civ_schema::WIRE_SCHEMA_MAJOR),
        wire_minor: u32::from(civ_schema::WIRE_SCHEMA_MINOR),
        save_schema: civ_schema::SAVE_SCHEMA_VERSION,
        content_api: civ_content::KERNEL_CONTENT_API,
    };
    // SAFETY: not null, writable and at least this size (checked above).
    unsafe { out.write(versions) };
    TCE_OK
}

/// # Safety
///
/// `config` must be null or point to a readable `TceConfig` whose byte strings are readable for
/// the call; `out` must be null or writable.
unsafe extern "C" fn create(config: *const TceConfig, out: *mut *mut TceKernel) -> i32 {
    if config.is_null() || out.is_null() {
        return TCE_INVALID_ARGUMENT;
    }
    // SAFETY: not null; the host lends a readable `TceConfig` for the call. Its size is checked
    // before any field past it is used.
    let config = unsafe { *config };
    if (config.struct_size as usize) < size_of::<TceConfig>() {
        return TCE_INVALID_ARGUMENT;
    }
    // SAFETY: the host lends the byte strings' bytes for the call.
    let (content_dir, saves_dir) =
        unsafe { (path_of(config.content_dir), path_of(config.saves_dir)) };
    let (Some(content_dir), Some(saves_dir)) = (content_dir, saves_dir) else {
        return TCE_INVALID_ARGUMENT;
    };
    let config = Config {
        content_dir,
        saves_dir,
    };
    let kernel = catch_unwind(|| Kernel::create(&config)).unwrap_or_else(|panic| {
        Kernel::failed(format!(
            "the kernel panicked while starting: {}",
            panic_message(&*panic)
        ))
    });
    let code = code_of(kernel.health());
    let boxed = Box::into_raw(Box::new(TceKernel { kernel }));
    // SAFETY: `out` is not null and the host promises it is writable.
    unsafe { out.write(boxed) };
    code
}

/// # Safety
///
/// `kernel` must be null or a kernel `create` returned, not yet destroyed, and not in use by
/// any other call.
unsafe extern "C" fn destroy(kernel: *mut TceKernel) -> i32 {
    if kernel.is_null() {
        return TCE_INVALID_ARGUMENT;
    }
    // SAFETY: the host passes a kernel from `create`, which made it with `Box::into_raw`, and
    // promises nothing else uses it now or later.
    let boxed = unsafe { Box::from_raw(kernel) };
    match catch_unwind(AssertUnwindSafe(move || boxed.kernel.destroy())) {
        Ok(()) => TCE_OK,
        Err(_) => TCE_FAULTED,
    }
}

/// # Safety
///
/// `kernel` must be null or a live kernel from `create`.
unsafe extern "C" fn status(kernel: *const TceKernel) -> i32 {
    // SAFETY: the host passes null or a live kernel.
    unsafe { with_kernel(kernel, |k| code_of(k.health())) }
}

/// # Safety
///
/// `kernel` must be null or a live kernel from `create`; `buf` must point to `cap` writable
/// bytes when `cap` is not 0; `out_len` must be writable.
unsafe extern "C" fn last_error(
    kernel: *const TceKernel,
    buf: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the host passes null or a live kernel and lends the buffers for the call.
    unsafe {
        with_kernel(kernel, |k| {
            let error = k.last_error();
            copy_out(error.as_bytes(), buf, cap, out_len)
        })
    }
}

/// # Safety
///
/// `kernel` must be null or a live kernel from `create`; `frame` must point to `len` readable
/// bytes when `len` is not 0.
unsafe extern "C" fn submit(kernel: *mut TceKernel, frame: *const u8, len: usize) -> i32 {
    if frame.is_null() && len > 0 {
        return TCE_INVALID_ARGUMENT;
    }
    // SAFETY: the host passes null or a live kernel and lends the frame's bytes for the call.
    unsafe {
        with_kernel(kernel, |k| {
            let bytes = if len == 0 {
                &[][..]
            } else {
                std::slice::from_raw_parts(frame, len)
            };
            match k.submit(bytes) {
                Ok(()) => TCE_OK,
                Err(refused) => refused_code(&refused),
            }
        })
    }
}

/// # Safety
///
/// `kernel` must be null or a live kernel from `create`; `buf` must point to `cap` writable
/// bytes when `cap` is not 0; `out_len` must be writable.
unsafe extern "C" fn poll(
    kernel: *mut TceKernel,
    buf: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    if out_len.is_null() {
        return TCE_INVALID_ARGUMENT;
    }
    // SAFETY: the host passes null or a live kernel and lends the buffers for the call.
    unsafe {
        with_kernel(kernel, |k| {
            let Some(dest) = out_buffer(buf, cap) else {
                return TCE_INVALID_ARGUMENT;
            };
            match k.poll(dest) {
                Ok(n) => {
                    out_len.write(n);
                    TCE_OK
                }
                Err(refused) => {
                    let needed = if let Refused::TooSmall(n) = refused {
                        n
                    } else {
                        0
                    };
                    out_len.write(needed);
                    refused_code(&refused)
                }
            }
        })
    }
}

/// # Safety
///
/// `kernel` must be null or a live kernel from `create`; `buf` must point to `cap` writable
/// bytes when `cap` is not 0; `out_len` must be writable; `out_number` null or writable.
unsafe extern "C" fn copy_snapshot(
    kernel: *mut TceKernel,
    after: u64,
    buf: *mut u8,
    cap: usize,
    out_len: *mut usize,
    out_number: *mut u64,
) -> i32 {
    if out_len.is_null() {
        return TCE_INVALID_ARGUMENT;
    }
    // SAFETY: the host passes null or a live kernel and lends the buffers for the call.
    unsafe {
        with_kernel(kernel, |k| {
            let Some(dest) = out_buffer(buf, cap) else {
                return TCE_INVALID_ARGUMENT;
            };
            match k.copy_snapshot(after, dest) {
                Ok((n, number)) => {
                    out_len.write(n);
                    put(out_number, number);
                    TCE_OK
                }
                Err(refused) => {
                    let needed = if let Refused::TooSmall(n) = refused {
                        n
                    } else {
                        0
                    };
                    out_len.write(needed);
                    refused_code(&refused)
                }
            }
        })
    }
}

/// # Safety
///
/// `kernel` must be null or a live kernel from `create`; `config` must be null or point to a
/// readable `TcePanelConfig` whose byte string is readable for the call.
unsafe extern "C" fn serve_panels(kernel: *mut TceKernel, config: *const TcePanelConfig) -> i32 {
    if config.is_null() {
        return TCE_INVALID_ARGUMENT;
    }
    // SAFETY: not null; the host lends a readable `TcePanelConfig` for the call. Its size is
    // checked before any field past it is used.
    let config = unsafe { *config };
    if (config.struct_size as usize) < size_of::<TcePanelConfig>() {
        return TCE_INVALID_ARGUMENT;
    }
    let web_dir = if config.web_dir.len == 0 {
        None
    } else {
        // SAFETY: the host lends the byte string's bytes for the call.
        match unsafe { path_of(config.web_dir) } {
            Some(dir) => Some(dir),
            None => return TCE_INVALID_ARGUMENT,
        }
    };
    let config = PanelConfig {
        port: config.port,
        web_dir,
    };
    // SAFETY: the host passes null or a live kernel.
    unsafe {
        with_kernel(kernel, |k| match k.serve_panels(&config) {
            Ok(()) => TCE_OK,
            Err(refused) => refused_code(&refused),
        })
    }
}

/// # Safety
///
/// As for `last_error`.
unsafe extern "C" fn panel_url(
    kernel: *const TceKernel,
    buf: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the host passes null or a live kernel and lends the buffers for the call.
    unsafe {
        with_kernel(kernel, |k| match k.panel_url() {
            Some(url) => copy_out(url.as_bytes(), buf, cap, out_len),
            None => {
                put(out_len, 0);
                TCE_NONE
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_panic_faults_the_kernel_and_says_why() {
        let kernel = TceKernel {
            kernel: Kernel::failed("never started".to_owned()),
        };
        // SAFETY: a live kernel on the stack.
        let code = unsafe { with_kernel(&kernel, |_| panic!("deliberately")) };
        assert_eq!(code, TCE_FAULTED);
        assert_eq!(kernel.kernel.health(), Health::Faulted);
        assert!(kernel.kernel.last_error().contains("deliberately"));
    }

    #[test]
    fn the_table_only_comes_for_this_major_and_a_big_enough_table() {
        let mut table = std::mem::MaybeUninit::<TceApiV1>::uninit();
        let size = size_of::<TceApiV1>();
        // SAFETY: `table` has room for a whole table.
        unsafe {
            assert_eq!(tce_get_api(2, table.as_mut_ptr(), size), TCE_INCOMPATIBLE);
            assert_eq!(
                tce_get_api(1, table.as_mut_ptr(), size - 8),
                TCE_INCOMPATIBLE
            );
            assert_eq!(tce_get_api(1, ptr::null_mut(), size), TCE_INVALID_ARGUMENT);
            assert_eq!(tce_get_api(1, table.as_mut_ptr(), size), TCE_OK);
            let table = table.assume_init();
            assert_eq!(table.struct_size as usize, size);
            assert_eq!((table.abi_major, table.abi_minor), (1, 0));
            assert!(table.panel_url.is_some());
        }
    }
}
