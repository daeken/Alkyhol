//! Checked PE64 image layout and a deliberately bounded binary ABI runtime.
//! Input code is trusted native code, not sandboxed. See docs/pe.md.
mod command_line;
pub use command_line::windows_command_line;
mod image;
pub use image::{Export, ExportTable, ExportTarget, Image, Import, ImportSymbol, Section};

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
mod abi;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
mod binary_com;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
mod function_tables;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
mod module_context;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
mod virtual_unwind;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub use module_context::ModuleContext;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
mod process;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub use process::ProcessContext;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
mod runtime;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub use runtime::{LinkedImage, LinkedLibrary, LoadedImage, LoadedLibrary};
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
mod library_set;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub use library_set::LibrarySet;

pub type Result<T> = std::result::Result<T, String>;
