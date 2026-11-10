//! Closed, explicitly supplied dormant DLLs: real deferred initializers/refcounts.
//! Mappings are retained by ProcessImage32. No unregistered file is called loaded.
use super::{
    CallingConvention32, LoadedImage32, Module32, Phase32, bulk32, path32, startup32, thread32,
    unsupported,
};
use crate::Result;
fn error(code: u32) -> u32 {
    alkyhol::ffi::SetLastError(code);
    0
}
fn module(image: &LoadedImage32<'_>, base: u32) -> Module32 {
    image
        .cpu
        .modules
        .borrow()
        .iter()
        .find(|m| m.base == base)
        .cloned()
        .unwrap_or_else(|| unsupported("loader dependency lacks an owned mapping"))
}
fn update(image: &LoadedImage32<'_>, base: u32, change: impl FnOnce(&mut Module32)) {
    let mut modules = image.cpu.modules.borrow_mut();
    change(
        modules
            .iter_mut()
            .find(|m| m.base == base)
            .expect("owned module"),
    );
}
fn entry(image: &LoadedImage32<'_>, m: &Module32, reason: u32) -> Result<u32> {
    if m.entry == 0 {
        return Ok(1);
    }
    let _scope = super::activation32::enter_scope(&image.cpu, m.base);
    unsafe { image.callback_frame(m.entry, &[m.base, reason, 0], CallingConvention32::Stdcall) }?
        .ok_or_else(|| "DLL loader callback exited nonlocally".into())
}
fn acquire(image: &LoadedImage32<'_>, base: u32, depth: usize) -> Result<bool> {
    if depth > 64 {
        return Err("deferred DLL dependency depth exceeds budget".into());
    }
    let m = module(image, base);
    if m.loaded {
        let refs = m
            .references
            .checked_add(1)
            .ok_or("DLL reference count overflow")?;
        update(image, base, |m| m.references = refs);
        return Ok(true);
    }
    if m.retired {
        return Err("reloading a detached DLL requires fresh image mappings".into());
    }
    let mut acquired = Vec::new();
    for &dep in &m.dependencies {
        if !acquire(image, dep, depth + 1)? {
            for d in acquired.into_iter().rev() {
                release(image, d, depth + 1)?;
            }
            return Ok(false);
        }
        acquired.push(dep);
    }
    update(image, base, |m| {
        m.loaded = true;
        m.references = 1;
        m.thread_notifications = true;
    });
    if entry(image, &m, 1)? == 0 {
        entry(image, &m, 0)?;
        update(image, base, |m| {
            m.loaded = false;
            m.references = 0;
            m.retired = true;
            m.thread_notifications = false;
        });
        for d in acquired.into_iter().rev() {
            release(image, d, depth + 1)?;
        }
        return Ok(false);
    }
    image.cpu.dynamic_order.borrow_mut().push(base);
    Ok(true)
}
fn release(image: &LoadedImage32<'_>, base: u32, depth: usize) -> Result<()> {
    if depth > 64 {
        return Err("DLL release dependency depth exceeds budget".into());
    }
    let m = module(image, base);
    if !m.loaded || m.references == 0 {
        return Err("DLL release has no acquired reference".into());
    }
    if m.pinned && m.references == 1 {
        return Err("releasing the static process DLL pin is unsupported".into());
    }
    if m.references > 1 {
        update(image, base, |m| m.references -= 1);
        return Ok(());
    }
    entry(image, &m, 0)?;
    update(image, base, |m| {
        m.loaded = false;
        m.references = 0;
        m.retired = true;
        m.thread_notifications = false;
    });
    for d in m.dependencies.into_iter().rev() {
        release(image, d, depth + 1)?;
    }
    Ok(())
}
pub(super) fn load(image: &LoadedImage32<'_>, address: u32, ansi: bool) -> u32 {
    let name = if ansi {
        bulk32::ascii(image, address)
    } else {
        startup32::module_name(image, address).unwrap_or_else(|e| unsupported(&e))
    };
    eprintln!("PE32 LoadLibrary requested: {name:?}");
    if image.cpu.phase.get() != Phase32::Running {
        return error(50);
    }
    let basename = !name.contains(['\\', '/', ':']);
    let sought = if basename {
        let mut name = crate::runtime::dll_basename(&if name.contains('.') {
            name
        } else {
            format!("{name}.dll")
        })
        .unwrap_or_else(|e| unsupported(&e));
        name.make_ascii_lowercase();
        name
    } else {
        let cwd = if name.as_bytes().get(1) == Some(&b':')
            && name
                .as_bytes()
                .get(2)
                .is_some_and(|c| matches!(c, b'\\' | b'/'))
        {
            "Z:\\".into()
        } else {
            super::directory32::current(image).unwrap_or_else(|e| unsupported(&e))
        };
        path32::resolve(&cwd, &name).unwrap_or_else(|e| unsupported(&e))
    };
    if basename && sought == "dsound.dll" {
        return super::dsound32::load(image);
    }
    let target = image
        .cpu
        .modules
        .borrow()
        .iter()
        .find(|m| {
            if basename {
                m.name.eq_ignore_ascii_case(&sought)
            } else {
                m.origin.as_ref().is_some_and(|o| {
                    String::from_utf16_lossy(&o.windows).eq_ignore_ascii_case(&sought)
                })
            }
        })
        .map(|m| m.base);
    let Some(base) = target else {
        // Unknown is not a fabricated missing file or successful DLL. Full Windows
        // filesystem/system/PATH search and unprovided DLLs remain unsupported.
        return error(50);
    };
    let _lock = thread32::loader_lock(image);
    match acquire(image, base, 0).unwrap_or_else(|e| unsupported(&e)) {
        true => base,
        false => error(1114),
    }
}
pub(super) fn free(image: &LoadedImage32<'_>, base: u32) -> u32 {
    if let Some(result) = super::dsound32::free(image, base) {
        return result;
    }
    if !image
        .cpu
        .modules
        .borrow()
        .iter()
        .any(|m| m.base == base && m.loaded)
    {
        return error(6);
    }
    if image.cpu.phase.get() != Phase32::Running {
        return error(50);
    }
    let m = module(image, base);
    if m.pinned && m.references == 1 {
        return error(50);
    }
    let _lock = thread32::loader_lock(image);
    release(image, base, 0).unwrap_or_else(|e| unsupported(&e));
    1
}
