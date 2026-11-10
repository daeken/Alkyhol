//! Ordinary CopyFileW into the explicitly configured Documents capability.
//! Paths/metadata outside this bounded profile remain explicit unsupported calls.
use super::{LoadedImage32, directory32, files32, path32, startup32, unsupported};
use crate::Result;
use std::{
    fs::{File, FileTimes, Metadata, OpenOptions},
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
};

fn io(error: std::io::Error) -> Result<u32> {
    match error.kind() {
        std::io::ErrorKind::NotFound => Ok(2),
        std::io::ErrorKind::NotADirectory => Ok(3),
        std::io::ErrorKind::PermissionDenied => Ok(5),
        std::io::ErrorKind::AlreadyExists => Ok(80),
        std::io::ErrorKind::StorageFull => Ok(112),
        _ => Err(format!(
            "CopyFileW host I/O condition is unsupported: {error}"
        )),
    }
}
fn copy_data(source: &mut File, target: &mut File, metadata: &Metadata) -> Result<()> {
    // All ownership, identities and sharing are checked before truncating an
    // existing target. Stream bytes rather than loading a save into host RAM.
    target.set_len(0).map_err(|e| e.to_string())?;
    let count = std::io::copy(source, target).map_err(|e| e.to_string())?;
    let after = source.metadata().map_err(|e| e.to_string())?;
    if count != metadata.len()
        || after.len() != count
        || after.modified().map_err(|e| e.to_string())?
            != metadata.modified().map_err(|e| e.to_string())?
    {
        return Err("CopyFileW source size/time changed during copy".into());
    }
    target
        .set_times(
            FileTimes::new()
                .set_accessed(metadata.accessed().map_err(|e| e.to_string())?)
                .set_modified(metadata.modified().map_err(|e| e.to_string())?),
        )
        .map_err(|e| e.to_string())?;
    // The existing ordinary-file profile exposes ARCHIVE/READONLY, not NTFS
    // security, encryption, alternate streams or inferred macOS attributes.
    if metadata.permissions().readonly() {
        let mut permissions = target.metadata().map_err(|e| e.to_string())?.permissions();
        permissions.set_mode(permissions.mode() & !0o222);
        target
            .set_permissions(permissions)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn perform(
    image: &LoadedImage32<'_>,
    source: &str,
    target: &str,
    exclusive: bool,
) -> Result<std::result::Result<(), u32>> {
    let docs = image
        .cpu
        .documents
        .as_ref()
        .ok_or("CopyFileW requires explicit Documents backing for its destination")?;
    let resolved = if image.cpu.casefold_file_opens {
        files32::existing_casefold(source)?
    } else {
        files32::existing(source)?
    };
    let (source_path, before) = match resolved {
        Ok(value) => value,
        Err(error) => return Ok(Err(error)),
    };
    if !before.is_file() {
        return Err("CopyFileW source must be an ordinary regular file".into());
    }
    let identity = (before.dev(), before.ino());
    for old in image
        .cpu
        .file_leases
        .borrow()
        .values()
        .filter(|v| v.identity == identity)
    {
        if old.share & 1 == 0 {
            return Ok(Err(32));
        }
        if old.access & 0x40000000 != 0 {
            return Err(
                "CopyFileW source with a live writer is outside the checked profile".into(),
            );
        }
    }
    let mut input = match OpenOptions::new()
        .read(true)
        .custom_flags(0x100 | 0x1000000)
        .open(source_path)
    {
        Ok(value) => value,
        Err(error) => return Ok(Err(io(error)?)),
    };
    let metadata = input.metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() || (metadata.dev(), metadata.ino()) != identity {
        return Err("CopyFileW source identity changed during checked open".into());
    }
    match files32::existing(target)? {
        Ok((_, dest)) => {
            if !dest.is_file() || (dest.dev(), dest.ino()) == identity {
                return Err("CopyFileW directory or same-file destination is unsupported".into());
            }
            if exclusive {
                return Ok(Err(80));
            }
            if dest.permissions().readonly() {
                return Ok(Err(5));
            }
            if image
                .cpu
                .file_leases
                .borrow()
                .values()
                .any(|v| v.identity == (dest.dev(), dest.ino()))
            {
                return Err("CopyFileW destination with a live guest handle is unsupported".into());
            }
        }
        Err(2) => (),
        Err(error) => return Ok(Err(error)),
    }
    let (mut output, _) = match docs.open_file(target, 0x40000000, if exclusive { 1 } else { 4 })? {
        Ok(value) => value,
        Err(error) => return Ok(Err(error)),
    };
    let dest = output.metadata().map_err(|e| e.to_string())?;
    if (dest.dev(), dest.ino()) == identity {
        return Err("CopyFileW destination aliased the source during open".into());
    }
    // Do not return successful completion after a partial copy/metadata failure.
    copy_data(&mut input, &mut output, &metadata)?;
    Ok(Ok(()))
}
pub(super) fn dispatch(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    let path = |address| {
        let units = startup32::read_units::<u16>(image, address, -1)
            .unwrap_or_else(|| unsupported("CopyFileW path outside checked UTF-16 storage"));
        let value = String::from_utf16(&units[..units.len() - 1])
            .unwrap_or_else(|_| unsupported("CopyFileW path is invalid UTF-16"));
        let cwd = directory32::current(image).unwrap_or_else(|e| unsupported(&e));
        path32::resolve(&cwd, &value).unwrap_or_else(|e| unsupported(&e))
    };
    let (source, target) = (path(args[0]), path(args[1]));
    if image.cpu.diagnostic.is_some() {
        eprintln!(
            "PE32 diagnostic copy file: source={source:?}; target={target:?}; fail_if_exists={}",
            args[2]
        );
    }
    match perform(image, &source, &target, args[2] != 0).unwrap_or_else(|e| unsupported(&e)) {
        Ok(()) => 1,
        Err(error) => {
            alkyhol::ffi::SetLastError(error);
            0
        }
    }
}
