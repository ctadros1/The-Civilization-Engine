//! The kernel's C interface ([ADR-0005]): the `tce_kernel` library that Unreal loads. It runs the
//! same engine thread `civ-host` runs and speaks the same frames as the WebSocket ([ADR-0001]),
//! through one exported function, `tce_get_api`, which hands back a versioned table of functions.
//!
//! - [`kernel`]: the kernel in safe Rust (an engine thread, the host's link to it, the panel
//!   server).
//! - [`exports`]: the C functions and types, the only hand-written `unsafe` code in the kernel.
//!   `include/tce_kernel.h` is generated from them by `cbindgen`; never edit it.
//!
//! [ADR-0001]: ../../../../decisions/0001-boundary-schema.md
//! [ADR-0005]: ../../../../decisions/0005-kernel-c-interface.md

pub mod exports;
pub mod kernel;

pub use exports::{
    TCE_ABI_MAJOR, TCE_ABI_MINOR, TCE_BAD_FRAME, TCE_BUFFER_TOO_SMALL, TCE_FAILED, TCE_FAULTED,
    TCE_INCOMPATIBLE, TCE_INVALID_ARGUMENT, TCE_NONE, TCE_OK, TCE_STOPPED, TceApiV1, TceBytes,
    TceConfig, TceKernel, TcePanelConfig, TceVersions, tce_get_api,
};
