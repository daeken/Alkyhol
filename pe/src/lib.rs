//! Checked PE64 image layout and a deliberately bounded binary ABI runtime.
//! Input code is trusted native code, not sandboxed. See docs/pe.md.
mod command_line;
pub use command_line::windows_command_line;
#[cfg(all(
    target_os = "macos",
    any(target_arch = "x86_64", feature = "native-runtime32")
))]
mod activation_binding;
#[cfg(all(
    target_os = "macos",
    any(target_arch = "x86_64", feature = "native-runtime32")
))]
mod activation_manifest;
#[cfg(all(
    target_os = "macos",
    any(target_arch = "x86_64", feature = "native-runtime32")
))]
mod activation_policy;
mod image;
mod resource;
pub use resource::{Resource, ResourceName};
mod unwind_info;
pub use image::{
    Export, ExportTable, ExportTarget, Image, Import, ImportSymbol, LegacyLoadConfig32,
    RuntimeFunction, RuntimeFunctionTable, Section, StaticTls32,
};

#[cfg(all(
    target_os = "macos",
    any(target_arch = "x86_64", feature = "native-runtime32")
))]
mod abi;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
mod binary_com;
#[cfg(all(
    target_os = "macos",
    any(target_arch = "x86_64", feature = "native-runtime32")
))]
mod function_tables;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
mod module_context;
#[cfg(all(
    target_os = "macos",
    any(target_arch = "x86_64", feature = "native-runtime32")
))]
mod unicode;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
mod virtual_unwind;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub use module_context::ModuleContext;
#[cfg(all(
    target_os = "macos",
    any(target_arch = "x86_64", feature = "native-runtime32")
))]
mod process;
#[cfg(all(
    target_os = "macos",
    any(target_arch = "x86_64", feature = "native-runtime32")
))]
pub use process::ProcessContext;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
mod runtime;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub use runtime::{
    LinkedImage, LinkedLibrary, LoadedImage, LoadedLibrary, RegisteredImage, RegisteredLibrary,
};
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
mod library_set;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub use library_set::LibrarySet;

pub type Result<T> = std::result::Result<T, String>;

#[cfg(all(
    target_os = "macos",
    any(target_arch = "x86_64", feature = "native-runtime32")
))]
#[cfg_attr(target_arch = "aarch64", alkyhol_host_abi::native_runtime)]
mod runtime32;
#[cfg(all(
    target_os = "macos",
    any(target_arch = "x86_64", feature = "native-runtime32")
))]
pub use runtime32::{
    CallingConvention32, ExceptionImage32, IconImage32, LoadedImage32, LoadedLibrary32,
    ProcessImage32, ProcessOptions32, TimeZone32,
};

#[cfg(all(
    target_os = "macos",
    target_arch = "aarch64",
    feature = "native-runtime32"
))]
#[path = "runtime32_helpers.rs"]
mod runtime;

#[cfg(all(
    target_os = "macos",
    target_arch = "aarch64",
    feature = "native-runtime32"
))]
pub use runtime32::executor32::{Execute32, ExecutionContext32, install_executor32};
