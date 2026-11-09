//! Checked staging and color transfers. All admitted copies reach real storage.
use super::*;

fn rectangle(image: &LoadedImage32<'_>, p: u32, width: u32, height: u32) -> Option<[u32; 4]> {
    let r = if p == 0 {
        [0, 0, width, height]
    } else {
        read::<4>(image, p)
    };
    valid_rect(r, width, height).then_some(r)
}
fn valid_rect(r: [u32; 4], width: u32, height: u32) -> bool {
    r[0] < r[2] && r[1] < r[3] && r[2] <= width && r[3] <= height
}
fn destination_rect(r: [u32; 4], point: [u32; 2], width: u32, height: u32) -> Option<[u32; 4]> {
    let d = [
        point[0],
        point[1],
        point[0].checked_add(r[2].checked_sub(r[0])?)?,
        point[1].checked_add(r[3].checked_sub(r[1])?)?,
    ];
    valid_rect(d, width, height).then_some(d)
}
fn bpp(format: u32) -> Option<u32> {
    match format {
        21 | 22 => Some(4),
        26 => Some(2),
        28 | 50 => Some(1),
        _ => None,
    }
}

pub(super) fn create_render_target(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    if a[7] == 0 {
        return INVALID;
    }
    write(image, a[7], &[0]);
    // Lockable render targets require a separate GPU/CPU coherence contract.
    if ![21, 22].contains(&a[3]) || a[4] != 0 || a[5] != 0 || a[6] != 0 || a[8] != 0 {
        return UNAVAILABLE;
    }
    let Body::Device { gpu, .. } = &s.objects[&a[0]].body else {
        unreachable!()
    };
    let mut target = match gpu.create_target(a[1], a[2]) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("D3D9 render target: {e}");
            return UNAVAILABLE;
        }
    };
    // Same real initialization policy as freshly created render-target textures.
    gpu.clear(&mut target, if a[3] == 22 { 0xff000000 } else { 0 })
        .unwrap_or_else(|e| unsupported(&e));
    addref(s, a[0]);
    let p = object(
        &image.cpu,
        s,
        Body::Surface {
            parent: a[0],
            back: false,
            attachment: Some(target),
            depth: None,
            implicit_depth: false,
            container: 0,
            texture_level: 0,
            bindings: 0,
            data: 0,
            width: a[1],
            height: a[2],
            format: a[3],
            initialized: false,
            locked: false,
        },
        1,
    );
    write(image, a[7], &[p]);
    0
}

pub(super) fn lock_surface(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    if a[1] == 0 || a[3] & !0x810 != 0 {
        return INVALID;
    }
    let Body::Surface {
        back,
        attachment,
        depth,
        data,
        width,
        height,
        initialized,
        locked,
        ..
    } = &mut s.objects.get_mut(&a[0]).unwrap().body
    else {
        unreachable!()
    };
    if *back || attachment.is_some() || depth.is_some() || *locked {
        return INVALID;
    }
    let Some(r) = rectangle(image, a[2], *width, *height) else {
        return INVALID;
    };
    if a[3] & 16 != 0 && !*initialized {
        unsupported("D3D9 read of uninitialized system-memory surface");
    }
    write(
        image,
        a[1],
        &[*width * 4, *data + r[1] * *width * 4 + r[0] * 4],
    );
    *locked = true;
    0
}

struct Plane {
    data: u32,
    width: u32,
    height: u32,
    pitch: u32,
    format: u32,
    pool: u32,
    initialized: bool,
    container: u32,
    level: u32,
    target: Option<Target>,
}
fn plane(s: &State, device: u32, p: u32) -> Option<Plane> {
    let Object {
        refs: 1..,
        body:
            Body::Surface {
                parent,
                back,
                attachment,
                depth: None,
                container,
                texture_level,
                data,
                width,
                height,
                format,
                initialized,
                locked: false,
                ..
            },
    } = s.objects.get(&p)?
    else {
        return None;
    };
    if *parent != device {
        return None;
    }
    let mut result = Plane {
        data: *data,
        width: *width,
        height: *height,
        pitch: *width * 4,
        format: *format,
        pool: 2,
        initialized: *initialized,
        container: *container,
        level: *texture_level,
        target: attachment.clone(),
    };
    if *back {
        let Body::Device { target, .. } = &s.objects[&device].body else {
            unreachable!()
        };
        result.target = Some((**target).clone());
    }
    if result.target.is_some() {
        result.pool = 0;
    }
    if *container != 0 {
        let Body::Texture {
            pool,
            levels,
            volume: false,
            ..
        } = &s.objects[container].body
        else {
            return None;
        };
        let mip = levels.get(*texture_level as usize)?;
        if mip.lock.is_some() {
            return None;
        }
        result.pool = *pool;
        result.pitch = mip.pitch;
        result.initialized = mip.initialized;
    }
    Some(result)
}

pub(super) fn color_fill(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    let Some(p) = plane(s, a[0], a[1]) else {
        return INVALID;
    };
    let Some(mut target) = p.target else {
        return UNAVAILABLE;
    };
    if p.pool != 0 || ![21, 22].contains(&p.format) {
        return INVALID;
    }
    let Some(r) = rectangle(image, a[2], p.width, p.height) else {
        return INVALID;
    };
    let Body::Device { gpu, .. } = &s.objects[&a[0]].body else {
        unreachable!()
    };
    gpu.fill_target_region(
        &mut target,
        r,
        if p.format == 22 {
            a[3] | 0xff000000
        } else {
            a[3]
        },
    )
    .unwrap_or_else(|e| unsupported(&e));
    0
}

pub(super) fn update_surface(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    if a[1] == a[3] {
        return INVALID;
    }
    let (Some(src), Some(dst)) = (plane(s, a[0], a[1]), plane(s, a[0], a[3])) else {
        return INVALID;
    };
    if src.pool != 2 || dst.pool != 0 || src.format != dst.format {
        return INVALID;
    }
    let Some(bytes_per_pixel) = bpp(src.format) else {
        return UNAVAILABLE;
    };
    if !src.initialized {
        unsupported("UpdateSurface source shadow has not been populated");
    }
    let Some(r) = rectangle(image, a[2], src.width, src.height) else {
        return INVALID;
    };
    let point = if a[4] == 0 {
        [0, 0]
    } else {
        read::<2>(image, a[4])
    };
    let Some(d) = destination_rect(r, point, dst.width, dst.height) else {
        return INVALID;
    };
    let pitch = (r[2] - r[0]) * bytes_per_pixel;
    let mut bytes = Vec::with_capacity((pitch * (r[3] - r[1])) as usize);
    for y in r[1]..r[3] {
        let p = src.data + y * src.pitch + r[0] * bytes_per_pixel;
        checked(image, p, pitch as usize, false);
        bytes.extend_from_slice(unsafe {
            std::slice::from_raw_parts(p as *const u8, pitch as usize)
        });
    }
    let Body::Device { gpu, .. } = &s.objects[&a[0]].body else {
        unreachable!()
    };
    let gpu = gpu.clone();
    if let Some(mut target) = dst.target {
        if dst.format == 22 {
            for p in bytes.as_chunks_mut::<4>().0 {
                p[3] = 255;
            }
        }
        gpu.write_target_region(&mut target, d, &bytes, pitch)
            .unwrap_or_else(|e| unsupported(&e));
    } else if dst.container != 0 {
        for (row, y) in (d[1]..d[3]).enumerate() {
            let p = dst.data + y * dst.pitch + d[0] * bytes_per_pixel;
            checked(image, p, pitch as usize, true);
            unsafe {
                std::ptr::copy_nonoverlapping(
                    bytes.as_ptr().add(row * pitch as usize),
                    p as *mut u8,
                    pitch as usize,
                );
            }
        }
        upload_level(image, s, &gpu, dst.container, dst.level);
    } else {
        return UNAVAILABLE;
    }
    if dst.container != 0 {
        let Body::Texture { levels, gpu, .. } =
            &mut s.objects.get_mut(&dst.container).unwrap().body
        else {
            unreachable!()
        };
        let face_base = dst.level / gpu.borrow().levels() * gpu.borrow().levels();
        levels[face_base as usize].dirty = true;
    }
    0
}

fn upload_level(image: &LoadedImage32<'_>, s: &mut State, device: &Gpu, p: u32, index: u32) {
    let Body::Texture {
        gpu,
        levels,
        format,
        attachment,
        ..
    } = &mut s.objects.get_mut(&p).unwrap().body
    else {
        unreachable!()
    };
    let level = &mut levels[index as usize];
    checked(image, level.data, level.length as usize, false);
    let mut bytes =
        unsafe { std::slice::from_raw_parts(level.data as *const u8, level.length as usize) }
            .to_vec();
    if *format == 22 {
        for p in bytes.as_chunks_mut::<4>().0 {
            p[3] = 255;
        }
    }
    if let Some(target) = attachment {
        device
            .write_target_region(
                target,
                [0, 0, level.width, level.height],
                &bytes,
                level.pitch,
            )
            .unwrap_or_else(|e| unsupported(&e));
    } else {
        let mut texture = gpu.borrow_mut();
        let mips = texture.levels();
        let result = if texture.is_volume() {
            device.upload_volume(&mut texture, index, &bytes)
        } else if texture.is_cube() {
            device.upload_cube_face(&mut texture, index / mips, index % mips, &bytes)
        } else {
            device.upload_texture(&mut texture, index, &bytes)
        };
        result.unwrap_or_else(|e| unsupported(&e));
    }
    level.initialized = true;
}

pub(super) fn add_dirty(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16], kind: u32) -> u32 {
    let Body::Texture { gpu, levels, .. } = &mut s.objects.get_mut(&a[0]).unwrap().body else {
        unreachable!()
    };
    let (face, p) = if kind == 10 { (a[1], a[2]) } else { (0, a[1]) };
    if face >= if kind == 10 { 6 } else { 1 } {
        return INVALID;
    }
    let level = &mut levels[(face * gpu.borrow().levels()) as usize];
    if kind == 12 {
        let b = if p == 0 {
            [0, 0, level.width, level.height, 0, level.depth]
        } else {
            read::<6>(image, p)
        };
        if !valid_rect([b[0], b[1], b[2], b[3]], level.width, level.height)
            || b[4] >= b[5]
            || b[5] > level.depth
        {
            return INVALID;
        }
    } else if rectangle(image, p, level.width, level.height).is_none() {
        return INVALID;
    }
    // The API permits expanding dirty copies. Keep one dirty flag per face and
    // copy complete matching mips, including scaled sublevels, on UpdateTexture.
    level.dirty = true;
    0
}

pub(super) fn update_texture(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    if a[1] == a[2] {
        return INVALID;
    }
    let texture = |p| match s.objects.get(&p) {
        Some(Object {
            refs: 1..,
            body:
                Body::Texture {
                    parent,
                    cube,
                    volume,
                    gpu,
                    levels,
                    format,
                    pool,
                    usage,
                    ..
                },
        }) if *parent == a[0] => Some((
            *cube,
            *volume,
            gpu.borrow().levels(),
            levels,
            *format,
            *pool,
            *usage,
        )),
        _ => None,
    };
    let (Some(src), Some(dst)) = (texture(a[1]), texture(a[2])) else {
        return INVALID;
    };
    if src.5 != 2
        || dst.5 != 0
        || src.0 != dst.0
        || src.1 != dst.1
        || src.4 != dst.4
        || src.2 < dst.2
        || src.6 != 0
        || dst.6 != 0
    {
        return INVALID;
    }
    if src.3.iter().chain(dst.3).any(|l| l.lock.is_some()) {
        return INVALID;
    }
    let skip = src.2 - dst.2;
    let mut copies = Vec::new();
    for face in 0..if src.0 { 6 } else { 1 } {
        for mip in 0..dst.2 {
            let from = &src.3[(face * src.2 + skip + mip) as usize];
            let to = &dst.3[(face * dst.2 + mip) as usize];
            if (from.width, from.height, from.depth, from.length)
                != (to.width, to.height, to.depth, to.length)
            {
                return INVALID;
            }
            if src.3[(face * src.2) as usize].dirty {
                if !from.initialized {
                    unsupported("UpdateTexture source mip has not been populated");
                }
                checked(image, from.data, from.length as usize, false);
                copies.push((
                    face * dst.2 + mip,
                    unsafe {
                        std::slice::from_raw_parts(from.data as *const u8, from.length as usize)
                    }
                    .to_vec(),
                ));
            }
        }
    }
    let (faces, source_mips) = (if src.0 { 6 } else { 1 }, src.2);
    let Body::Device { gpu, .. } = &s.objects[&a[0]].body else {
        unreachable!()
    };
    let gpu = gpu.clone();
    for (index, bytes) in copies {
        let Body::Texture { levels, .. } = &s.objects[&a[2]].body else {
            unreachable!()
        };
        let to = &levels[index as usize];
        checked(image, to.data, bytes.len(), true);
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), to.data as *mut u8, bytes.len());
        }
        upload_level(image, s, &gpu, a[2], index);
    }
    let Body::Texture { levels, .. } = &mut s.objects.get_mut(&a[1]).unwrap().body else {
        unreachable!()
    };
    for face in 0..faces {
        levels[(face * source_mips) as usize].dirty = false;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transfer_signatures_preserve_win32_stdcall_cleanup() {
        for (kind, slot, bytes) in [
            (1, 28, 36),
            (1, 30, 20),
            (1, 31, 12),
            (1, 35, 16),
            (7, 21, 8),
            (10, 21, 12),
            (12, 21, 8),
        ] {
            assert_eq!(signature(BASE + kind * 256 + slot).unwrap().1, bytes);
        }
    }
    #[test]
    fn rectangular_copy_rejects_signed_wrapped_and_overflowing_coordinates() {
        assert_eq!(
            destination_rect([2, 3, 6, 8], [7, 4], 11, 9),
            Some([7, 4, 11, 9])
        );
        for (r, p) in [
            ([0, 0, 0, 2], [0, 0]),
            ([0, 0, 4, 5], [8, 4]),
            ([0, 0, 1, 1], [u32::MAX, 0]),
            ([0, 0, 1, 1], [0, u32::MAX]),
        ] {
            assert!(destination_rect(r, p, 11, 9).is_none());
        }
        assert!(!valid_rect([u32::MAX, 0, 4, 1], 4, 1));
    }
}
