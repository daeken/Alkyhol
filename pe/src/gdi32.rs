//! Owned resource bitmaps and memory device contexts. No fabricated GDI handles.
use super::{LoadedImage32, unsupported};
use crate::Result;
use std::{collections::BTreeMap, rc::Rc};
#[derive(Clone)]
struct Bitmap {
    width: u32,
    height: u32,
    bits: u16,
    pixels: Vec<u8>,
    dib: Option<Dib>,
}
#[derive(Clone)]
struct Dib {
    memory: Rc<super::LowMapping>,
    size: usize,
    top_down: bool,
    header: [u32; 10],
}
impl Bitmap {
    fn read_pixels(&self) -> Vec<u8> {
        let Some(dib) = &self.dib else {
            return self.pixels.clone();
        };
        let mut pixels = vec![0; dib.size];
        let row = self.width as usize * 4;
        for y in 0..self.height as usize {
            let source = if dib.top_down {
                y
            } else {
                self.height as usize - 1 - y
            };
            unsafe {
                std::ptr::copy_nonoverlapping(
                    dib.memory.base.add(source * row),
                    pixels.as_mut_ptr().add(y * row),
                    row,
                );
            }
        }
        pixels
    }
    fn commit(&mut self, pixels: Vec<u8>) {
        if let Some(dib) = &self.dib {
            let row = self.width as usize * 4;
            for y in 0..self.height as usize {
                let dest = if dib.top_down {
                    y
                } else {
                    self.height as usize - 1 - y
                };
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        pixels.as_ptr().add(y * row),
                        dib.memory.base.add(dest * row),
                        row,
                    );
                }
            }
        } else {
            self.pixels = pixels;
        }
    }
    fn snapshot(&self) -> Self {
        Self {
            width: self.width,
            height: self.height,
            bits: self.bits,
            pixels: self.read_pixels(),
            dib: None,
        }
    }
}
#[derive(Clone, Copy)]
struct Brush {
    color: Option<u32>,
    stock: bool,
}
struct Dc {
    bitmap: u32,
    default: u32,
    window: Option<u32>,
    paint: bool,
    brush: u32,
}
#[derive(Default)]
pub(super) struct State {
    bitmaps: BTreeMap<u32, Bitmap>,
    dcs: BTreeMap<u32, Dc>,
    surfaces: BTreeMap<u32, u32>,
    brushes: BTreeMap<u32, Brush>,
    stocks: BTreeMap<u32, u32>,
}
fn u16at(b: &[u8], at: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(
        b.get(at..at + 2)
            .ok_or("truncated bitmap header")?
            .try_into()
            .unwrap(),
    ))
}
fn u32at(b: &[u8], at: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        b.get(at..at + 4)
            .ok_or("truncated bitmap header")?
            .try_into()
            .unwrap(),
    ))
}
fn decode(b: &[u8]) -> Result<Bitmap> {
    if u32at(b, 0)? != 40 || u16at(b, 12)? != 1 || u32at(b, 16)? != 0 {
        return Err("bitmap requires a BI_RGB BITMAPINFOHEADER".into());
    }
    let width = u32at(b, 4)? as i32;
    let signed = u32at(b, 8)? as i32;
    let bits = u16at(b, 14)?;
    if !(1..=8192).contains(&width)
        || signed == 0
        || signed == i32::MIN
        || signed.unsigned_abs() > 8192
        || ![1, 4, 8, 24, 32].contains(&bits)
    {
        return Err("bitmap dimensions or source format outside the bounded RGB profile".into());
    }
    let height = signed.unsigned_abs();
    let width = width as u32;
    let count = (width as usize)
        .checked_mul(height as usize)
        .filter(|&v| v <= 16 * 1024 * 1024)
        .ok_or("bitmap pixel budget exceeded")?;
    let colors = u32at(b, 32)? as usize;
    let palette = if bits <= 8 {
        if colors == 0 { 1usize << bits } else { colors }
    } else {
        colors
    };
    if (bits <= 8 && palette > (1usize << bits)) || palette > 256 {
        return Err("invalid bitmap palette extent".into());
    }
    let start = 40 + palette * 4;
    let stride = (width as usize * bits as usize).div_ceil(32) * 4;
    let required = start + stride * height as usize;
    if required > b.len() {
        return Err("truncated bitmap rows".into());
    }
    // BI_RGB scanline extent is derived from width/bpp, not biSizeImage. Some
    // valid resources carry file padding after the final row; retain/ignore it.
    let mut pixels = vec![0; count * 4];
    for y in 0..height as usize {
        let source = if signed > 0 {
            height as usize - 1 - y
        } else {
            y
        };
        let row = &b[start + source * stride..start + (source + 1) * stride];
        for x in 0..width as usize {
            let out = &mut pixels[(y * width as usize + x) * 4..(y * width as usize + x + 1) * 4];
            if bits <= 8 {
                let index = match bits {
                    1 => (row[x / 8] >> (7 - x % 8)) & 1,
                    4 => {
                        if x.is_multiple_of(2) {
                            row[x / 2] >> 4
                        } else {
                            row[x / 2] & 15
                        }
                    }
                    8 => row[x],
                    _ => unreachable!(),
                } as usize;
                if index >= palette {
                    return Err("bitmap index exceeds its palette".into());
                }
                out[..3].copy_from_slice(&b[40 + index * 4..43 + index * 4]);
            } else {
                let at = x * (bits as usize / 8);
                out[..3].copy_from_slice(&row[at..at + 3]);
            }
            out[3] = 255;
        }
    }
    Ok(Bitmap {
        width,
        height,
        bits: if bits == 1 { 1 } else { 32 },
        pixels,
        dib: None,
    })
}
fn token() -> u32 {
    let n = alkyhol::kernel::reserve_handle_token()
        .unwrap_or_else(|_| unsupported("GDI handle namespace exhausted"));
    if n >= 0x10000000 {
        unsupported("GDI handle namespace exhausted");
    }
    0x50000000 | n as u32
}
fn fail(error: u32) -> u32 {
    alkyhol::ffi::SetLastError(error);
    0
}
fn checked(image: &LoadedImage32<'_>, at: u32, bytes: usize) {
    if !at.is_multiple_of(4) || !image.permits(at as usize, bytes, true) {
        unsupported("GDI output outside checked writable guest storage");
    }
}
pub(super) fn load(image: &LoadedImage32<'_>, a: &[u32; 8]) -> u32 {
    if a[0] == 0 || a[1] == 0 || a[1] > u16::MAX as u32 {
        unsupported("LoadBitmapW requires a numeric resource in a registered guest module");
    }
    let source = image
        .cpu
        .icons
        .borrow()
        .bitmap_resource(a[0], a[1] as u16)
        .unwrap_or_else(|e| unsupported(&e));
    let Some(source) = source else {
        return fail(1814);
    };
    let bitmap = decode(&source).unwrap_or_else(|e| unsupported(&e));
    let mut gdi = image.cpu.gdi.borrow_mut();
    if gdi.bitmaps.len() >= 256 {
        unsupported("GDI bitmap budget exhausted");
    }
    let h = token();
    gdi.bitmaps.insert(h, bitmap);
    h
}
pub(super) fn dispatch(image: &LoadedImage32<'_>, id: u32, a: &[u32; 16]) -> u32 {
    match id {
        0 => {
            // GetObjectW, DDB BITMAP layout, not a host pointer.
            let state = image.cpu.gdi.borrow();
            if let Some(brush) = state.brushes.get(&a[0]) {
                if a[2] == 0 {
                    return 12;
                }
                if a[1] < 12 {
                    return fail(87);
                }
                checked(image, a[2], 12);
                let values = [
                    u32::from(brush.color.is_none()),
                    brush.color.unwrap_or(0),
                    0,
                ];
                unsafe {
                    std::ptr::copy_nonoverlapping(values.as_ptr(), a[2] as *mut u32, 3);
                }
                return 12;
            }
            let Some(b) = state.bitmaps.get(&a[0]) else {
                return fail(6);
            };
            if a[2] == 0 {
                return if b.dib.is_some() { 84 } else { 24 };
            }
            if a[1] < 24 {
                return fail(87);
            }
            checked(image, a[2], 24);
            let row = (b.width * b.bits as u32).div_ceil(16) * 2;
            let out = [
                0,
                b.width,
                b.height,
                row,
                1 | ((b.bits as u32) << 16),
                b.dib.as_ref().map_or(0, |d| d.memory.base as u32),
            ];
            if a[1] >= 84
                && let Some(dib) = &b.dib
            {
                checked(image, a[2], 84);
                let mut section = [0u32; 21];
                section[..6].copy_from_slice(&out);
                section[6..16].copy_from_slice(&dib.header);
                unsafe {
                    std::ptr::copy_nonoverlapping(section.as_ptr(), a[2] as *mut u32, 21);
                }
                84
            } else {
                unsafe {
                    std::ptr::copy_nonoverlapping(out.as_ptr(), a[2] as *mut u32, 6);
                }
                24
            }
        }
        1 => {
            // DeleteObject; a selected bitmap remains owned.
            let mut state = image.cpu.gdi.borrow_mut();
            if let Some(brush) = state.brushes.get(&a[0]) {
                if brush.stock {
                    return 1;
                }
                if state.dcs.values().any(|dc| dc.brush == a[0]) {
                    return 0;
                }
                state.brushes.remove(&a[0]);
                return 1;
            }
            if state
                .dcs
                .values()
                .any(|dc| dc.bitmap == a[0] || dc.default == a[0])
                || state.surfaces.values().any(|&h| h == a[0])
            {
                return 0;
            }
            state.bitmaps.remove(&a[0]).is_some() as u32
        }
        2 => {
            // CreateCompatibleDC with a real 1x1 monochrome default surface.
            let mut state = image.cpu.gdi.borrow_mut();
            if a[0] != 0 && !state.dcs.contains_key(&a[0]) {
                return fail(6);
            }
            if state.dcs.len() >= 128 {
                unsupported("GDI memory DC budget exhausted");
            }
            let default = token();
            state.bitmaps.insert(
                default,
                Bitmap {
                    width: 1,
                    height: 1,
                    bits: 1,
                    pixels: vec![255, 255, 255, 255],
                    dib: None,
                },
            );
            let h = token();
            let brush = state.stock(0).expect("white stock brush");
            state.dcs.insert(
                h,
                Dc {
                    bitmap: default,
                    default,
                    window: None,
                    paint: false,
                    brush,
                },
            );
            h
        }
        3 => {
            let mut state = image.cpu.gdi.borrow_mut();
            let Some(dc) = state.dcs.remove(&a[0]) else {
                return 0;
            };
            if dc.window.is_some() {
                unsupported("window DCs require ReleaseDC/EndPaint");
            }
            state.bitmaps.remove(&dc.default);
            1
        }
        4 => {
            let mut state = image.cpu.gdi.borrow_mut();
            if state.brushes.contains_key(&a[1]) {
                return state
                    .dcs
                    .get_mut(&a[0])
                    .map_or(0, |dc| std::mem::replace(&mut dc.brush, a[1]));
            }
            if !state.bitmaps.contains_key(&a[1]) {
                return 0;
            }
            if state
                .dcs
                .iter()
                .any(|(&h, dc)| h != a[0] && dc.bitmap == a[1])
            {
                return 0;
            }
            let Some(dc) = state.dcs.get_mut(&a[0]) else {
                return 0;
            };
            if dc.window.is_some() {
                return 0;
            }
            std::mem::replace(&mut dc.bitmap, a[1])
        }
        5 => {
            // GetDIBits, explicit 32-bit BI_RGB readback of a real bitmap.
            let state = image.cpu.gdi.borrow();
            if !state.dcs.contains_key(&a[0]) {
                return fail(6);
            }
            let Some(b) = state.bitmaps.get(&a[1]) else {
                return fail(6);
            };
            if a[6] != 0 {
                unsupported("palette-index DIB readback is unsupported");
            }
            checked(image, a[5], 40);
            let mut header = unsafe { (a[5] as *const [u32; 10]).read() };
            if header[0] != 40
                || header[1] != b.width
                || header[2] as i32 == i32::MIN
                || (header[2] as i32).unsigned_abs() != b.height
                || header[3] != ((32 << 16) | 1)
                || header[4] != 0
                || a[4] == 0
            {
                unsupported("DIB readback requires an explicit matching 32-bit BI_RGB buffer");
            }
            if a[2] >= b.height {
                return 0;
            }
            let lines = a[3].min(b.height - a[2]);
            let bytes = lines as usize * b.width as usize * 4;
            checked(image, a[4], bytes);
            let source_pixels = b.read_pixels();
            let out = unsafe { std::slice::from_raw_parts_mut(a[4] as *mut u8, bytes) };
            let row = b.width as usize * 4;
            for y in 0..lines as usize {
                let source = if header[2] as i32 > 0 {
                    b.height as usize - 1 - a[2] as usize - y
                } else {
                    a[2] as usize + y
                };
                out[y * row..(y + 1) * row]
                    .copy_from_slice(&source_pixels[source * row..(source + 1) * row]);
                for pixel in out[y * row..(y + 1) * row].as_chunks_mut::<4>().0 {
                    pixel[3] = 0;
                }
            }
            header[5] = b.width * b.height * 4;
            unsafe {
                (a[5] as *mut [u32; 10]).write(header);
            }
            lines
        }
        6 => {
            // CreateCompatibleBitmap follows the selected device format.
            let mut state = image.cpu.gdi.borrow_mut();
            let Some(dc) = state.dcs.get(&a[0]) else {
                return fail(6);
            };
            let bits = state.bitmaps.get(&dc.bitmap).unwrap().bits;
            if a[1] as i32 <= 0 || a[2] as i32 <= 0 {
                return fail(87);
            }
            let pixels = blank(a[1], a[2]).unwrap_or_else(|e| unsupported(&e));
            let h = token();
            state.bitmaps.insert(
                h,
                Bitmap {
                    width: a[1],
                    height: a[2],
                    bits,
                    pixels,
                    dib: None,
                },
            );
            h
        }
        7 => {
            // BitBlt, SRCCOPY with device clipping and overlap-safe source.
            if a[8] != 0x00cc0020 {
                unsupported("GDI raster operation requires SRCCOPY");
            }
            if (a[3] as i32) < 0 || (a[4] as i32) < 0 {
                return fail(87);
            }
            let mut state = image.cpu.gdi.borrow_mut();
            let Some(source_dc) = state.dcs.get(&a[5]) else {
                return fail(6);
            };
            let source = state.bitmaps[&source_dc.bitmap].snapshot();
            let Some(dest_dc) = state.dcs.get(&a[0]) else {
                return fail(6);
            };
            if dest_dc.window == Some(0) {
                unsupported("writing to the host desktop is unsupported");
            }
            let target_window = dest_dc.window;
            let destination = dest_dc.bitmap;
            let dest = state.bitmaps.get_mut(&destination).unwrap();
            let mut snapshot = dest.snapshot();
            blit(
                &mut snapshot,
                &source,
                [
                    a[1] as i32,
                    a[2] as i32,
                    a[3] as i32,
                    a[4] as i32,
                    a[6] as i32,
                    a[7] as i32,
                ],
            )
            .unwrap_or_else(|e| unsupported(&e));
            if dest.dib.is_none() {
                for p in snapshot.pixels.as_chunks_mut::<4>().0 {
                    p[3] = 255;
                }
            }
            dest.commit(snapshot.pixels);
            let result =
                target_window.map(|hwnd| (hwnd, dest.width, dest.height, dest.read_pixels()));
            drop(state);
            if let Some((hwnd, width, height, pixels)) = result {
                super::ui32::present(image, hwnd, width, height, &pixels)
                    .unwrap_or_else(|e| unsupported(&e));
            }
            1
        }
        8 => {
            // GetPixel reports actual backing values, COLORREF ordering.
            let state = image.cpu.gdi.borrow();
            let Some(dc) = state.dcs.get(&a[0]) else {
                return u32::MAX;
            };
            let b = &state.bitmaps[&dc.bitmap];
            if a[1] >= b.width || a[2] >= b.height {
                return u32::MAX;
            }
            let at = (a[2] as usize * b.width as usize + a[1] as usize) * 4;
            let pixels = b.read_pixels();
            let p = &pixels[at..at + 4];
            u32::from(p[2]) | (u32::from(p[1]) << 8) | (u32::from(p[0]) << 16)
        }
        9 => {
            let state = image.cpu.gdi.borrow();
            let Some(dc) = state.dcs.get(&a[0]) else {
                return 0;
            };
            match a[1] {
                7 => dc.bitmap,
                2 => dc.brush,
                _ => unsupported("GetCurrentObject type outside bitmap/brush profile"),
            }
        }
        _ => unreachable!(),
    }
}
fn blank(width: u32, height: u32) -> Result<Vec<u8>> {
    let count = (width as usize)
        .checked_mul(height as usize)
        .filter(|&v| width <= 8192 && height <= 8192 && v <= 16 * 1024 * 1024)
        .ok_or("GDI surface pixel budget exceeded")?;
    let mut pixels = vec![0; count * 4];
    for p in pixels.as_chunks_mut::<4>().0 {
        p[3] = 255;
    }
    Ok(pixels)
}
fn blit(dest: &mut Bitmap, source: &Bitmap, r: [i32; 6]) -> Result<()> {
    let [dx, dy, width, height, sx, sy] = r;
    if width < 0 || height < 0 {
        return Err("negative BitBlt extents are unsupported".into());
    }
    let left = 0i64.max(-(dx as i64));
    let top = 0i64.max(-(dy as i64));
    let right = (width as i64).min(dest.width as i64 - dx as i64);
    let bottom = (height as i64).min(dest.height as i64 - dy as i64);
    if left >= right || top >= bottom {
        return Ok(());
    }
    if sx as i64 + left < 0
        || sy as i64 + top < 0
        || sx as i64 + right > source.width as i64
        || sy as i64 + bottom > source.height as i64
    {
        return Err("BitBlt source rectangle outside owned bitmap".into());
    }
    for y in top..bottom {
        let from =
            ((sy as i64 + y) as usize * source.width as usize + (sx as i64 + left) as usize) * 4;
        let to = ((dy as i64 + y) as usize * dest.width as usize + (dx as i64 + left) as usize) * 4;
        let n = (right - left) as usize * 4;
        dest.pixels[to..to + n].copy_from_slice(&source.pixels[from..from + n]);
    }
    Ok(())
}
pub(super) fn get_dc(image: &LoadedImage32<'_>, hwnd: u32, paint: bool) -> u32 {
    if hwnd == 0 {
        if paint {
            unsupported("BeginPaint requires an owned window");
        }
        let (width, height, pixels) =
            super::window_host32::capture_desktop().unwrap_or_else(|e| unsupported(&e));
        let mut state = image.cpu.gdi.borrow_mut();
        let bitmap = token();
        let dc = token();
        state.bitmaps.insert(
            bitmap,
            Bitmap {
                width,
                height,
                bits: 32,
                pixels,
                dib: None,
            },
        );
        let brush = state.stock(0).expect("white stock brush");
        state.dcs.insert(
            dc,
            Dc {
                bitmap,
                default: 0,
                window: Some(0),
                paint: false,
                brush,
            },
        );
        return dc;
    }
    let Some((width, height)) = super::ui32::client_size(image, hwnd) else {
        return fail(1400);
    };
    let mut state = image.cpu.gdi.borrow_mut();
    let bitmap = if let Some(&bitmap) = state.surfaces.get(&hwnd) {
        let b = &state.bitmaps[&bitmap];
        if b.width != width || b.height != height {
            unsupported("resized window GDI backing requires a measured preservation policy");
        }
        bitmap
    } else {
        let h = token();
        state.bitmaps.insert(
            h,
            Bitmap {
                width,
                height,
                bits: 32,
                pixels: blank(width, height).unwrap_or_else(|e| unsupported(&e)),
                dib: None,
            },
        );
        state.surfaces.insert(hwnd, h);
        h
    };
    let h = token();
    let brush = state.stock(0).expect("white stock brush");
    state.dcs.insert(
        h,
        Dc {
            bitmap,
            default: 0,
            window: Some(hwnd),
            paint,
            brush,
        },
    );
    h
}
pub(super) fn release_dc(image: &LoadedImage32<'_>, hwnd: u32, hdc: u32, paint: bool) -> u32 {
    let (width, height, pixels) = {
        let mut state = image.cpu.gdi.borrow_mut();
        let Some(dc) = state.dcs.get(&hdc) else {
            return 0;
        };
        if dc.window != Some(hwnd) || dc.paint != paint {
            return 0;
        }
        let bitmap = dc.bitmap;
        let b = &state.bitmaps[&bitmap];
        let snapshot = (b.width, b.height, b.read_pixels());
        state.dcs.remove(&hdc);
        if hwnd == 0 {
            state.bitmaps.remove(&bitmap);
        }
        snapshot
    };
    if hwnd == 0 {
        return 1;
    }
    super::ui32::present(image, hwnd, width, height, &pixels).unwrap_or_else(|e| unsupported(&e));
    1
}

impl State {
    pub(super) fn permits(&self, at: usize, len: usize) -> bool {
        at.checked_add(len).is_some_and(|end| {
            self.bitmaps.values().any(|b| {
                b.dib.as_ref().is_some_and(|d| {
                    at >= d.memory.base as usize && end <= d.memory.base as usize + d.size
                })
            })
        })
    }
}
pub(super) fn dib_section(image: &LoadedImage32<'_>, a: &[u32; 8]) -> u32 {
    if a[2] != 0 || a[4] != 0 || a[5] != 0 {
        unsupported("DIB section requires an anonymous DIB_RGB_COLORS mapping");
    }
    if !a[1].is_multiple_of(4) || !image.permits(a[1] as usize, 40, false) {
        unsupported("DIB header outside checked readable storage");
    }
    checked(image, a[3], 4);
    let header = unsafe { (a[1] as *const [u32; 10]).read() };
    let width = header[1];
    let height = (header[2] as i32).unsigned_abs();
    if header[0] != 40
        || (width as i32) <= 0
        || header[2] as i32 == i32::MIN
        || height == 0
        || header[3] != 0x200001
        || header[4] != 0
        || header[8] != 0
    {
        unsupported("DIB section requires a bounded 32-bit BI_RGB header");
    }
    let bytes = blank(width, height)
        .unwrap_or_else(|e| unsupported(&e))
        .len();
    if a[0] != 0 && !image.cpu.gdi.borrow().dcs.contains_key(&a[0]) {
        return fail(6);
    }
    let memory = Rc::new(
        image
            .cpu
            .mapping(bytes.div_ceil(super::PAGE) * super::PAGE, super::PAGE, 0)
            .unwrap_or_else(|e| unsupported(&e)),
    );
    if image.cpu.gdi.borrow().bitmaps.len() >= 256 {
        return fail(8);
    }
    let address = memory.base as u32;
    let h = token();
    image.cpu.gdi.borrow_mut().bitmaps.insert(
        h,
        Bitmap {
            width,
            height,
            bits: 32,
            pixels: Vec::new(),
            dib: Some(Dib {
                memory,
                size: bytes,
                top_down: (header[2] as i32) < 0,
                header,
            }),
        },
    );
    unsafe {
        (a[3] as *mut u32).write(address);
    }
    h
}

impl State {
    fn stock(&mut self, index: u32) -> Option<u32> {
        if let Some(&h) = self.stocks.get(&index) {
            return Some(h);
        }
        let color = match index {
            0 => Some(0xffffff),
            4 => Some(0),
            5 => None,
            _ => return None,
        };
        let h = token();
        self.brushes.insert(h, Brush { color, stock: true });
        self.stocks.insert(index, h);
        Some(h)
    }
}
pub(super) fn stock(image: &LoadedImage32<'_>, index: u32) -> u32 {
    image
        .cpu
        .gdi
        .borrow_mut()
        .stock(index)
        .unwrap_or_else(|| fail(50))
}
pub(super) fn solid(image: &LoadedImage32<'_>, color: u32) -> u32 {
    if color & 0xff000000 != 0 {
        return fail(50);
    }
    let mut state = image.cpu.gdi.borrow_mut();
    if state.brushes.len() >= 256 {
        return fail(8);
    }
    let h = token();
    state.brushes.insert(
        h,
        Brush {
            color: Some(color),
            stock: false,
        },
    );
    h
}
fn fill_pixels(bitmap: &mut Bitmap, rect: [i32; 4], color: u32) {
    let left = rect[0].max(0).min(bitmap.width as i32);
    let right = rect[2].max(0).min(bitmap.width as i32);
    let top = rect[1].max(0).min(bitmap.height as i32);
    let bottom = rect[3].max(0).min(bitmap.height as i32);
    let mut pixels = bitmap.read_pixels();
    for y in top..bottom {
        for x in left..right {
            let at = (y as usize * bitmap.width as usize + x as usize) * 4;
            pixels[at..at + 4].copy_from_slice(&[
                (color >> 16) as u8,
                (color >> 8) as u8,
                color as u8,
                255,
            ]);
        }
    }
    bitmap.commit(pixels);
}
pub(super) fn fill(image: &LoadedImage32<'_>, dc: u32, rect: [i32; 4], brush: u32) -> u32 {
    let mut state = image.cpu.gdi.borrow_mut();
    let Some(brush) = state.brushes.get(&brush).copied() else {
        return fail(6);
    };
    let Some(dc) = state.dcs.get(&dc) else {
        return fail(6);
    };
    let window = dc.window;
    let bitmap = dc.bitmap;
    if window == Some(0) {
        unsupported("desktop GDI writes are unsupported");
    }
    let Some(color) = brush.color else {
        return 1;
    };
    let b = state.bitmaps.get_mut(&bitmap).expect("DC surface");
    if b.bits == 1 && !matches!(color, 0 | 0xffffff) {
        unsupported(
            "color brush mapping to a monochrome DC requires measured text/background behavior",
        );
    }
    fill_pixels(b, rect, color);
    let output = window.map(|hwnd| (hwnd, b.width, b.height, b.read_pixels()));
    drop(state);
    if let Some((hwnd, w, h, pixels)) = output {
        super::ui32::present(image, hwnd, w, h, &pixels).unwrap_or_else(|e| unsupported(&e));
    }
    1
}
pub(super) fn fill_rect(image: &LoadedImage32<'_>, a: &[u32; 8]) -> u32 {
    if !a[1].is_multiple_of(4) || !image.permits(a[1] as usize, 16, false) {
        unsupported("FillRect RECT outside checked storage");
    }
    fill(
        image,
        a[0],
        unsafe { (a[1] as *const [i32; 4]).read() },
        a[2],
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    fn rgb(width: u32, height: i32, bits: u16, extra: &[u8]) -> Vec<u8> {
        let stride = (width as usize * bits as usize).div_ceil(32) * 4;
        let mut b = vec![0; 40 + stride * height.unsigned_abs() as usize];
        b[..4].copy_from_slice(&40u32.to_le_bytes());
        b[4..8].copy_from_slice(&width.to_le_bytes());
        b[8..12].copy_from_slice(&height.to_le_bytes());
        b[12..14].copy_from_slice(&1u16.to_le_bytes());
        b[14..16].copy_from_slice(&bits.to_le_bytes());
        b.extend_from_slice(extra);
        b
    }
    #[test]
    fn rows_padding_orientation_and_unused_alpha_have_explicit_semantics() {
        let mut b = rgb(3, 2, 24, &[0x73, 0x25]);
        b[40..49].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
        b[52..61].copy_from_slice(&[10, 11, 12, 13, 14, 15, 16, 17, 18]);
        let d = decode(&b).unwrap();
        assert_eq!((d.width, d.height, d.bits), (3, 2, 32));
        assert_eq!(&d.pixels[..4], &[10, 11, 12, 255]);
        assert_eq!(&d.pixels[20..], &[7, 8, 9, 255]);
        b[8..12].copy_from_slice(&(-2i32).to_le_bytes());
        assert_eq!(&decode(&b).unwrap().pixels[..4], &[1, 2, 3, 255]);
        let mut a = rgb(1, 1, 32, &[]);
        a[40..].copy_from_slice(&[1, 2, 3, 17]);
        assert_eq!(decode(&a).unwrap().pixels, [1, 2, 3, 255]);
    }
    #[test]
    fn indexed_palette_and_bad_inputs_do_not_turn_into_blank_images() {
        let mut b = rgb(2, 1, 1, &[]);
        b.splice(40..40, [0, 0, 0, 0, 10, 20, 30, 0]);
        b[48] = 0x80;
        assert_eq!(decode(&b).unwrap().pixels, [10, 20, 30, 255, 0, 0, 0, 255]);
        for bad in [&b[..39], &b[..49]] {
            assert!(decode(bad).is_err());
        }
        b[16..20].copy_from_slice(&1u32.to_le_bytes());
        assert!(decode(&b).is_err());
    }
    #[test]
    fn device_clipping_preserves_source_coordinates_and_rejects_unbacked_reads() {
        let mut source = Bitmap {
            width: 3,
            height: 2,
            bits: 32,
            pixels: vec![0; 24],
            dib: None,
        };
        for (i, p) in source.pixels.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            *p = [i as u8 + 1, 2, 3, 255];
        }
        let mut dest = Bitmap {
            width: 3,
            height: 2,
            bits: 32,
            pixels: blank(3, 2).unwrap(),
            dib: None,
        };
        blit(&mut dest, &source, [-1, 0, 3, 2, 0, 0]).unwrap();
        assert_eq!(dest.pixels[0], 2);
        assert_eq!(dest.pixels[4], 3);
        assert_eq!(dest.pixels[8], 0);
        assert_eq!(dest.pixels[12], 5);
        let before = dest.pixels.clone();
        assert!(blit(&mut dest, &source, [0, 0, 3, 2, 1, 0]).is_err());
        assert_eq!(dest.pixels, before);
        blit(&mut dest, &source, [99, 99, 3, 2, -20, -20]).unwrap();
        assert_eq!(dest.pixels, before);
    }
}
