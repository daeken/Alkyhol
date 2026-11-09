//! Owned BGRA volume mip storage and shared texture/volume COM identities.
use super::*;
fn layout(
    width: u32,
    height: u32,
    depth: u32,
    levels: u32,
    format: u32,
) -> Option<(Vec<TextureLevel>, u32)> {
    if ![21, 22, 50].contains(&format)
        || width == 0
        || height == 0
        || depth == 0
        || width.max(height).max(depth) > 512
    {
        return None;
    }
    let maximum = 32 - width.max(height).max(depth).leading_zeros();
    let levels = if levels == 0 { maximum } else { levels };
    if levels > maximum {
        return None;
    }
    let mut total = 0u32;
    let mut result = Vec::new();
    for level in 0..levels {
        let (w, h, d) = (
            (width >> level).max(1),
            (height >> level).max(1),
            (depth >> level).max(1),
        );
        let pitch = w.checked_mul(if format == 50 { 1 } else { 4 })?;
        let length = pitch.checked_mul(h)?.checked_mul(d)?;
        let data = total;
        total = total.checked_add(length)?.checked_add(15)? & !15;
        if total > 64 * 1024 * 1024 {
            return None;
        }
        result.push(TextureLevel {
            dirty: true,
            data,
            width: w,
            height: h,
            depth: d,
            pitch,
            length,
            surface: 0,
            lock: None,
            initialized: false,
        });
    }
    Some((result, total))
}
pub(super) fn create(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    if a[8] == 0 {
        return INVALID;
    }
    write(image, a[8], &[0]);
    if a[5] != 0 || ![21, 22, 50].contains(&a[6]) || ![0, 1, 2].contains(&a[7]) || a[9] != 0 {
        return UNAVAILABLE;
    }
    let Some((mut levels, total)) = layout(a[1], a[2], a[3], a[4], a[6]) else {
        return UNAVAILABLE;
    };
    let Body::Device { gpu, .. } = &s.objects[&a[0]].body else {
        unreachable!()
    };
    let texture = match gpu.create_sampled_volume(
        a[1],
        a[2],
        a[3],
        a[4],
        if a[6] == 50 {
            SampleFormat::Luminance8
        } else {
            SampleFormat::Bgra8
        },
    ) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("D3D9 volume allocation: {e}");
            return UNAVAILABLE;
        }
    };
    let base = memory(&image.cpu, &vec![0; total as usize], true, false)
        .unwrap_or_else(|e| unsupported(&e));
    for level in &mut levels {
        level.data += base;
    }
    addref(s, a[0]);
    let p = object(
        &image.cpu,
        s,
        Body::Texture {
            cube: false,
            volume: true,
            parent: a[0],
            gpu: Rc::new(RefCell::new(texture)),
            levels,
            format: a[6],
            pool: a[7],
            usage: 0,
            bindings: 0,
            attachment: None,
        },
        1,
    );
    write(image, a[8], &[p]);
    0
}
fn box_offset(level: &TextureLevel, b: [u32; 6]) -> Option<u32> {
    // D3DBOX is Left,Top,Right,Bottom,Front,Back.
    if b[0] >= b[2]
        || b[1] >= b[3]
        || b[4] >= b[5]
        || b[2] > level.width
        || b[3] > level.height
        || b[5] > level.depth
    {
        return None;
    }
    b[4].checked_mul(level.pitch)?
        .checked_mul(level.height)?
        .checked_add(b[1].checked_mul(level.pitch)?)?
        .checked_add(b[0].checked_mul(level.pitch / level.width)?)
}
pub(super) fn operation(
    image: &LoadedImage32<'_>,
    s: &mut State,
    a: &[u32; 16],
    kind: u32,
    slot: u32,
) -> u32 {
    let (texture, index) = if kind == 12 {
        (a[0], a[1])
    } else {
        let Body::Volume {
            container, level, ..
        } = s.objects[&a[0]].body
        else {
            unreachable!()
        };
        (container, level)
    };
    if kind == 13 && slot == 7 {
        if a[2] == 0 {
            return INVALID;
        }
        write(image, a[2], &[0]);
        if a[1] == 0 {
            return POINTER;
        }
        checked(image, a[1], 16, false);
        let iid = unsafe { (a[1] as *const [u8; 16]).read_unaligned() };
        if ![IID_UNKNOWN, IID_RESOURCE, IID_BASE_TEXTURE, IIDS[12]].contains(&iid) {
            return NO_INTERFACE;
        }
        expose(s, texture);
        write(image, a[2], &[texture]);
        return 0;
    }
    let Body::Texture {
        parent,
        gpu,
        levels,
        ..
    } = &s.objects[&texture].body
    else {
        unreachable!()
    };
    if kind == 12 && slot == 13 {
        return gpu.borrow().levels();
    }
    let parent = *parent;
    let Some(level) = levels.get(index as usize) else {
        return INVALID;
    };
    if kind == 12 && slot == 18 {
        if a[2] == 0 {
            return INVALID;
        }
        write(image, a[2], &[0]);
        let mut p = level.surface;
        if p == 0 {
            p = object(
                &image.cpu,
                s,
                Body::Volume {
                    parent,
                    container: texture,
                    level: index,
                },
                0,
            );
            let Body::Texture { levels, .. } = &mut s.objects.get_mut(&texture).unwrap().body
            else {
                unreachable!()
            };
            levels[index as usize].surface = p;
        }
        expose(s, p);
        write(image, a[2], &[p]);
        return 0;
    }
    let Body::Device { gpu: device, .. } = &s.objects[&parent].body else {
        unreachable!()
    };
    let device = device.clone();
    let Body::Texture {
        gpu,
        levels,
        format,
        pool,
        usage,
        ..
    } = &mut s.objects.get_mut(&texture).unwrap().body
    else {
        unreachable!()
    };
    let level = &mut levels[index as usize];
    match (kind, slot) {
        (12, 17) | (13, 8) => {
            let output = if kind == 12 { a[2] } else { a[1] };
            if output == 0 {
                return INVALID;
            }
            write(
                image,
                output,
                &[
                    *format,
                    2,
                    *usage,
                    *pool,
                    level.width,
                    level.height,
                    level.depth,
                ],
            );
            0
        }
        (12, 19) | (13, 9) => {
            let (output, region, flags) = if kind == 12 {
                (a[2], a[3], a[4])
            } else {
                (a[1], a[2], a[3])
            };
            if output == 0 || level.lock.is_some() {
                return INVALID;
            }
            if flags & !0x8810 != 0 {
                return UNAVAILABLE;
            }
            let b = if region == 0 {
                [0, 0, level.width, level.height, 0, level.depth]
            } else {
                read::<6>(image, region)
            };
            let Some(offset) = box_offset(level, b) else {
                return INVALID;
            };
            let readonly = flags & 16 != 0;
            if index == 0 && !readonly && flags & 0x8000 == 0 {
                level.dirty = true;
            }
            if readonly && !level.initialized {
                unsupported("read of uninitialized volume shadow");
            }
            write(
                image,
                output,
                &[level.pitch, level.pitch * level.height, level.data + offset],
            );
            level.lock = Some(readonly);
            0
        }
        (12, 20) | (13, 10) => {
            let Some(readonly) = level.lock.take() else {
                return INVALID;
            };
            if !readonly {
                checked(image, level.data, level.length as usize, false);
                let mut bytes = unsafe {
                    std::slice::from_raw_parts(level.data as *const u8, level.length as usize)
                }
                .to_vec();
                if *format == 22 {
                    for p in bytes.as_chunks_mut::<4>().0 {
                        p[3] = 255;
                    }
                }
                if *pool == 2 {
                    level.initialized = true;
                    return 0;
                }
                device
                    .upload_volume(&mut gpu.borrow_mut(), index, &bytes)
                    .unwrap_or_else(|e| unsupported(&e));
                level.initialized = true;
            }
            0
        }
        _ => unreachable!(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn volume_layout_retains_all_axes_pitches_alignment_and_budget() {
        let (levels, total) = layout(4, 2, 8, 0, 21).unwrap();
        assert_eq!(levels.len(), 4);
        assert_eq!(total, 320);
        assert_eq!(
            (levels[0].pitch, levels[0].length, levels[0].depth),
            (16, 256, 8)
        );
        assert_eq!(
            (
                levels[1].width,
                levels[1].height,
                levels[1].depth,
                levels[1].data
            ),
            (2, 1, 4, 256)
        );
        assert_eq!(box_offset(&levels[0], [1, 1, 3, 2, 2, 4]), Some(84));
        for b in [
            [0, 0, 4, 2, 0, 9],
            [3, 0, 3, 1, 0, 1],
            [0, 0, 4, 3, 0, 1],
            [0, 0, 4, 2, 4, 4],
        ] {
            assert!(box_offset(&levels[0], b).is_none());
        }
        for extent in [
            (0, 2, 2, 1),
            (2, 0, 2, 1),
            (2, 2, 0, 1),
            (513, 2, 2, 1),
            (512, 512, 512, 1),
            (4, 4, 4, 4),
        ] {
            assert!(layout(extent.0, extent.1, extent.2, extent.3, 21).is_none());
        }
        let (luminance, _) = layout(4, 2, 8, 0, 50).unwrap();
        assert_eq!((luminance[0].pitch, luminance[0].length), (4, 64));
        assert_eq!(box_offset(&luminance[0], [1, 1, 3, 2, 2, 4]), Some(21));
        assert!(layout(4, 2, 8, 0, 100).is_none());
        assert_eq!(signature(BASE + 12 * 256 + 19).unwrap().1, 20);
        assert_eq!(signature(BASE + 13 * 256 + 9).unwrap().1, 16);
    }
}
