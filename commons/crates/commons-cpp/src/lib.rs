//! engine-commons' C++ for hosts written in C++, such as the Unreal plugin: `commons/cpp/include`
//! holds header-only C++17 (`commons_wire.hpp`, the frame envelope; `timed_path.hpp`, positions
//! along timed paths). This crate has no Rust code of its own; its tests compile and run the C++
//! tests in `commons/cpp/tests` with the system's compiler (GCC or Clang, MSVC on Windows).

#![forbid(unsafe_code)]
