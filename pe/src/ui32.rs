//! Main-thread Win32 class/window bootstrap backed by actual AppKit windows.
use super::{
    CallingConvention32, Cpu32, LoadedImage32, bulk32, startup32, unsupported,
    window_host32 as host,
};
use crate::Result;
use std::collections::BTreeMap;
const VISIBLE: u32 = 0x10000000;
#[derive(Clone)]
struct Class {
    atom: u32,
    name: String,
    procedure: u32,
    style: u32,
    instance: u32,
    icon: u32,
    cursor: u32,
    background: u32,
    extra: usize,
}
struct Window {
    native: host::Window,
    class: u32,
    procedure: u32,
    style: u32,
    ex_style: u32,
    instance: u32,
    user: u32,
    id: u32,
    title: Vec<u16>,
    extra: Vec<u8>,
}
#[derive(Default)]
pub(super) struct State {
    app: Option<host::App>,
    classes: BTreeMap<u32, Class>,
    windows: BTreeMap<u32, Window>,
    next_atom: u32,
    desktop: u32,
    active: u32,
}
impl State {
    fn app(&mut self) -> Result<&host::App> {
        if self.app.is_none() {
            self.app = Some(host::App::new()?);
        }
        Ok(self.app.as_ref().unwrap())
    }
}
impl Drop for State {
    fn drop(&mut self) {
        self.windows.clear();
        self.app.take();
    }
}
fn stop<T>(r: Result<T>) -> T {
    r.unwrap_or_else(|e| unsupported(&e))
}
fn error(code: u32) -> u32 {
    alkyhol::ffi::SetLastError(code);
    0
}
fn units(image: &LoadedImage32<'_>, at: u32, ansi: bool) -> Vec<u16> {
    if ansi {
        bulk32::ascii(image, at).encode_utf16().collect()
    } else {
        let mut v = startup32::read_units::<u16>(image, at, -1)
            .unwrap_or_else(|| unsupported("window string outside checked UTF-16 storage"));
        v.pop();
        v
    }
}
fn string(image: &LoadedImage32<'_>, at: u32, ansi: bool) -> String {
    String::from_utf16(&units(image, at, ansi))
        .unwrap_or_else(|_| unsupported("window string is invalid UTF-16"))
}
fn read<const N: usize>(image: &LoadedImage32<'_>, at: u32) -> [u32; N] {
    if !at.is_multiple_of(4) || !image.permits(at as usize, N * 4, false) {
        unsupported("window input structure outside checked storage");
    }
    unsafe { (at as *const [u32; N]).read() }
}
fn write(image: &LoadedImage32<'_>, at: u32, v: &[u32]) {
    if !at.is_multiple_of(4) || !image.permits(at as usize, v.len() * 4, true) {
        unsupported("window output structure outside writable storage");
    }
    unsafe {
        std::ptr::copy_nonoverlapping(v.as_ptr(), at as *mut u32, v.len());
    }
}
fn executable(image: &LoadedImage32<'_>, at: u32) -> bool {
    image
        .regions
        .iter()
        .any(|&(a, b, r)| r & 0x20000000 != 0 && at as usize >= a && (at as usize) < b)
}
fn mask(style: u32) -> u64 {
    if style & 0x00c00000 == 0 {
        return 0;
    }
    1 | if style & 0x80000 != 0 { 2 } else { 0 }
        | if style & 0x20000 != 0 { 4 } else { 0 }
        | if style & 0x40000 != 0 { 8 } else { 0 }
}
fn register(image: &LoadedImage32<'_>, at: u32, ansi: bool) -> u32 {
    let (style, procedure, cls_extra, wnd_extra, instance, icon, cursor, background, menu, name) =
        if ansi {
            let v = read::<10>(image, at);
            (v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8], v[9])
        } else {
            let v = read::<12>(image, at);
            if v[0] != 48 {
                return error(87);
            }
            (v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8], v[9], v[10])
        };
    if !executable(image, procedure) || cls_extra > 4096 || wnd_extra > 4096 {
        return error(87);
    }
    if menu != 0 || cls_extra != 0 {
        unsupported("window class menus and class-extra storage are not implemented");
    }
    let name = string(image, name, ansi);
    if name.is_empty() || !name.is_ascii() {
        unsupported("window class names require nonempty ASCII spelling");
    }
    let mut ui = image.cpu.ui.borrow_mut();
    if ui
        .classes
        .values()
        .any(|c| c.instance == instance && c.name.eq_ignore_ascii_case(&name))
    {
        return error(1410);
    }
    if ui.next_atom == 0 {
        ui.next_atom = 0xc000;
    }
    if ui.next_atom > 0xffff {
        unsupported("window class atom space exhausted");
    }
    let atom = ui.next_atom;
    ui.next_atom += 1;
    ui.classes.insert(
        atom,
        Class {
            atom,
            name,
            procedure,
            style,
            instance,
            icon,
            cursor,
            background,
            extra: wnd_extra as usize,
        },
    );
    atom
}
struct Scratch<'a> {
    cpu: &'a Cpu32,
    heap: u32,
    address: u32,
}
impl<'a> Scratch<'a> {
    fn new(cpu: &'a Cpu32, words: &[u32]) -> Self {
        let mut heap = cpu.heap.borrow_mut();
        let h = stop(heap.handle());
        let address = stop(heap.allocate(h, 8, (words.len() * 4) as u32));
        if address == 0 {
            unsupported("window callback scratch allocation failed");
        }
        unsafe {
            std::ptr::copy_nonoverlapping(words.as_ptr(), address as *mut u32, words.len());
        }
        Self {
            cpu,
            heap: h,
            address,
        }
    }
}
impl Drop for Scratch<'_> {
    fn drop(&mut self) {
        let e = alkyhol::ffi::GetLastError();
        let _ = self.cpu.heap.borrow_mut().free(self.heap, 0, self.address);
        alkyhol::ffi::SetLastError(e);
    }
}
fn call(image: &LoadedImage32<'_>, hwnd: u32, message: u32, w: u32, l: u32) -> u32 {
    let procedure = match image.cpu.ui.borrow().windows.get(&hwnd) {
        Some(window) => window.procedure,
        None => return error(1400),
    };
    stop(unsafe {
        image.callback_frame(
            procedure,
            &[hwnd, message, w, l],
            CallingConvention32::Stdcall,
        )
    })
    .unwrap_or_default()
}
fn frame(rect: host::Rect) -> [u32; 4] {
    [
        rect.origin.x.round() as i32 as u32,
        rect.origin.y.round() as i32 as u32,
        (rect.origin.x + rect.size.width).round() as i32 as u32,
        (rect.origin.y + rect.size.height).round() as i32 as u32,
    ]
}
fn create(image: &LoadedImage32<'_>, a: &[u32; 16], ansi: bool) -> u32 {
    if a[8] != 0 || a[9] != 0 || a[3] & 0x40000000 != 0 {
        unsupported("only owned top-level windows without menus are supported");
    }
    if a[0] & !0x00040000 != 0 {
        unsupported("window extended styles outside the native bootstrap profile");
    }
    let name = if a[1] <= 0xffff {
        None
    } else {
        Some(string(image, a[1], ansi))
    };
    let title = if a[2] == 0 {
        Vec::new()
    } else {
        units(image, a[2], ansi)
    };
    let (_class, hwnd) = {
        let mut ui = image.cpu.ui.borrow_mut();
        let Some(class) = ui
            .classes
            .values()
            .find(|c| {
                (c.instance == a[10] || c.style & 0x4000 != 0)
                    && if let Some(name) = &name {
                        c.name.eq_ignore_ascii_case(name)
                    } else {
                        c.atom == a[1]
                    }
            })
            .cloned()
        else {
            return error(1407);
        };
        let app = stop(ui.app());
        let desktop = app.desktop;
        let width = if a[6] == 0x80000000 {
            (desktop.size.width * 0.75) as i32
        } else {
            a[6] as i32
        };
        let height = if a[7] == 0x80000000 {
            (desktop.size.height * 0.75) as i32
        } else {
            a[7] as i32
        };
        let x = if a[4] == 0x80000000 { 40 } else { a[4] as i32 };
        let y = if a[5] == 0x80000000 { 40 } else { a[5] as i32 };
        let native = stop(app.create(x, y, width, height, mask(a[3]), &title));
        let hwnd = alkyhol::kernel::reserve_handle_token()
            .unwrap_or_else(|_| unsupported("window handle namespace exhausted"))
            as u32;
        ui.windows.insert(
            hwnd,
            Window {
                native,
                class: class.atom,
                procedure: class.procedure,
                style: a[3],
                ex_style: a[0],
                instance: a[10],
                user: 0,
                id: a[9],
                title,
                extra: vec![0; class.extra],
            },
        );
        (class, hwnd)
    };
    let rect = {
        let ui = image.cpu.ui.borrow();
        ui.windows[&hwnd].native.frame()
    };
    let r = frame(rect);
    let minmax = Scratch::new(&image.cpu, &[0, 0, 0, 0, 1, 1, 0, 0, 32767, 32767]);
    call(image, hwnd, 0x24, 0, minmax.address);
    let params = Scratch::new(
        &image.cpu,
        &[
            a[11],
            a[10],
            a[9],
            a[8],
            r[3].wrapping_sub(r[1]),
            r[2].wrapping_sub(r[0]),
            r[1],
            r[0],
            a[3],
            a[2],
            a[1],
            a[0],
        ],
    );
    if call(image, hwnd, 0x81, 0, params.address) == 0 {
        image.cpu.ui.borrow_mut().windows.remove(&hwnd);
        return 0;
    }
    let calc = Scratch::new(&image.cpu, &r);
    call(image, hwnd, 0x83, 0, calc.address);
    if call(image, hwnd, 1, 0, params.address) == u32::MAX {
        destroy(image, hwnd);
        return 0;
    }
    if a[3] & VISIBLE != 0 {
        show(image, hwnd, 1);
    }
    if image.cpu.diagnostic.is_some() {
        let ui = image.cpu.ui.borrow();
        if let Some(w) = ui.windows.get(&hwnd) {
            eprintln!(
                "PE32 native window created: hwnd={hwnd:x}; native_window={}; visible={}; WindowServer_onscreen={:?}; frame={:?}; client={:?}",
                w.native.number(),
                w.native.visible(),
                w.native.onscreen(),
                w.native.frame(),
                w.native.client()
            );
        }
    }
    hwnd
}
fn show(image: &LoadedImage32<'_>, hwnd: u32, command: u32) -> u32 {
    if !matches!(command, 0 | 1 | 4 | 5 | 8 | 9 | 10) {
        unsupported("native window minimize/maximize commands are not implemented");
    }
    let previous = {
        let mut ui = image.cpu.ui.borrow_mut();
        let Some(w) = ui.windows.get_mut(&hwnd) else {
            return error(1400);
        };
        let previous = (w.style & VISIBLE != 0) as u32;
        w.native.show(command != 0);
        if command == 0 {
            w.style &= !VISIBLE;
        } else {
            w.style |= VISIBLE;
        }
        if command != 0 {
            ui.active = hwnd;
            if let Some(app) = &ui.app {
                app.activate();
                stop(app.pump());
            }
        }
        previous
    };
    call(image, hwnd, 0x18, (command != 0) as u32, 0);
    if command != 0 {
        let r = {
            let ui = image.cpu.ui.borrow();
            ui.windows[&hwnd].native.client()
        };
        call(
            image,
            hwnd,
            5,
            0,
            (r.size.width as u32 & 0xffff) | ((r.size.height as u32 & 0xffff) << 16),
        );
        call(image, hwnd, 6, 1, 0);
        call(image, hwnd, 7, 0, 0);
    }
    previous
}
fn destroy(image: &LoadedImage32<'_>, hwnd: u32) -> u32 {
    if !image.cpu.ui.borrow().windows.contains_key(&hwnd) {
        return error(1400);
    }
    call(image, hwnd, 2, 0, 0);
    call(image, hwnd, 0x82, 0, 0);
    let mut ui = image.cpu.ui.borrow_mut();
    ui.windows.remove(&hwnd);
    if ui.active == hwnd {
        ui.active = 0;
    }
    1
}
fn default_proc(image: &LoadedImage32<'_>, a: &[u32; 16]) -> u32 {
    match a[1] {
        0x81 => 1,
        0x10 => {
            destroy(image, a[0]);
            0
        }
        0x24 => {
            let ui = image.cpu.ui.borrow();
            let Some(w) = ui.windows.get(&a[0]) else {
                return error(1400);
            };
            let (min, max) = w.native.limits();
            let d = ui.app.as_ref().unwrap().desktop;
            let mut info = read::<10>(image, a[3]);
            info[2] = d.size.width as u32;
            info[3] = d.size.height as u32;
            info[4] = d.origin.x as i32 as u32;
            info[5] = d.origin.y as i32 as u32;
            info[6] = min.width.clamp(0., i32::MAX as f64) as u32;
            info[7] = min.height.clamp(0., i32::MAX as f64) as u32;
            info[8] = max.width.clamp(0., i32::MAX as f64) as u32;
            info[9] = max.height.clamp(0., i32::MAX as f64) as u32;
            write(image, a[3], &info);
            0
        }
        0x83 => {
            let rect = read::<4>(image, a[3]);
            let ui = image.cpu.ui.borrow();
            let Some(w) = ui.windows.get(&a[0]) else {
                return error(1400);
            };
            let f = w.native.frame();
            let c = stop(w.native.client_frame());
            let output = [
                (rect[0] as i32).wrapping_add((c.origin.x - f.origin.x) as i32) as u32,
                (rect[1] as i32).wrapping_add((c.origin.y - f.origin.y) as i32) as u32,
                (rect[2] as i32)
                    .wrapping_sub((f.origin.x + f.size.width - c.origin.x - c.size.width) as i32)
                    as u32,
                (rect[3] as i32)
                    .wrapping_sub((f.origin.y + f.size.height - c.origin.y - c.size.height) as i32)
                    as u32,
            ];
            write(image, a[3], &output);
            0
        }
        0 | 1 | 2 | 3 | 5 | 6 | 7 | 8 | 0xf | 0x14 | 0x18 | 0x1c | 0x46 | 0x47 | 0x7c | 0x7d
        | 0x82 | 0x85 | 0x86 | 0x215 => 0,
        0xe => image
            .cpu
            .ui
            .borrow()
            .windows
            .get(&a[0])
            .map_or(0, |w| w.title.len() as u32),
        _ => unsupported(&format!(
            "default window message 0x{:x} is outside the bootstrap implementation",
            a[1]
        )),
    }
}
pub(super) fn dispatch(image: &LoadedImage32<'_>, id: u32, a: &[u32; 16]) -> u32 {
    match id {
        0 => register(image, a[0], false),
        1 => register(image, a[0], true),
        2 => {
            let name = string(image, a[0], true);
            let mut ui = image.cpu.ui.borrow_mut();
            let Some(atom) = ui
                .classes
                .values()
                .find(|c| c.name.eq_ignore_ascii_case(&name) && c.instance == a[1])
                .map(|c| c.atom)
            else {
                return error(1411);
            };
            if ui.windows.values().any(|w| w.class == atom) {
                return error(1412);
            }
            ui.classes.remove(&atom);
            1
        }
        3 | 4 => create(image, a, id == 4),
        5 | 6 => default_proc(image, a),
        7 | 8 => {
            let rect = read::<4>(image, a[0]);
            if a[2] != 0 {
                unsupported("native window menu rectangle adjustment is unsupported");
            }
            let mut ui = image.cpu.ui.borrow_mut();
            let width = rect[2].wrapping_sub(rect[0]) as i32 as f64;
            let height = rect[3].wrapping_sub(rect[1]) as i32 as f64;
            let outer = stop(stop(ui.app()).frame_for_client(
                host::Rect {
                    origin: host::Point { x: 0., y: 0. },
                    size: host::Size { width, height },
                },
                mask(a[1]),
            ));
            let left = -outer.origin.x;
            let bottom = -outer.origin.y;
            let top = outer.size.height - height - bottom;
            let right = outer.size.width - width - left;
            write(
                image,
                a[0],
                &[
                    (rect[0] as i32).wrapping_sub(left as i32) as u32,
                    (rect[1] as i32).wrapping_sub(top as i32) as u32,
                    (rect[2] as i32).wrapping_add(right as i32) as u32,
                    (rect[3] as i32).wrapping_add(bottom as i32) as u32,
                ],
            );
            1
        }
        9 => show(image, a[0], a[1]),
        10 => {
            if !image.cpu.ui.borrow().windows.contains_key(&a[0]) {
                return error(1400);
            }
            call(image, a[0], 0xf, 0, 0);
            1
        }
        11 | 12 => {
            let mut ui = image.cpu.ui.borrow_mut();
            let rect = if a[0] == ui.desktop && ui.desktop != 0 {
                let d = stop(ui.app()).desktop;
                host::Rect {
                    origin: host::Point { x: 0., y: 0. },
                    size: d.size,
                }
            } else {
                let Some(w) = ui.windows.get(&a[0]) else {
                    return error(1400);
                };
                if id == 11 {
                    w.native.client()
                } else {
                    w.native.frame()
                }
            };
            write(image, a[1], &frame(rect));
            1
        }
        13 => {
            let ui = image.cpu.ui.borrow();
            let Some(w) = ui.windows.get(&a[0]) else {
                return error(1400);
            };
            if a[1] != 0 || a[6] & !0x737 != 0 {
                unsupported(
                    "window position ordering/flags are outside the native bootstrap profile",
                );
            }
            let r = w.native.frame();
            let x = if a[6] & 2 != 0 {
                r.origin.x as i32
            } else {
                a[2] as i32
            };
            let y = if a[6] & 2 != 0 {
                r.origin.y as i32
            } else {
                a[3] as i32
            };
            let width = if a[6] & 1 != 0 {
                r.size.width as i32
            } else {
                a[4] as i32
            };
            let height = if a[6] & 1 != 0 {
                r.size.height as i32
            } else {
                a[5] as i32
            };
            if width <= 0 || height <= 0 {
                unsupported("native window position requires positive dimensions");
            }
            w.native.position(x, y, width, height);
            drop(ui);
            if a[6] & 0x40 != 0 {
                show(image, a[0], 1);
            }
            1
        }
        14 => {
            let title = units(image, a[1], false);
            let mut ui = image.cpu.ui.borrow_mut();
            let Some(w) = ui.windows.get_mut(&a[0]) else {
                return error(1400);
            };
            stop(w.native.title(&title));
            w.title = title;
            1
        }
        15 => image
            .cpu
            .ui
            .borrow()
            .windows
            .get(&a[0])
            .map_or(0, |w| (w.style & VISIBLE != 0) as u32),
        16 => {
            let mut ui = image.cpu.ui.borrow_mut();
            stop(ui.app());
            if ui.desktop == 0 {
                ui.desktop = alkyhol::kernel::reserve_handle_token()
                    .unwrap_or_else(|_| unsupported("window handle namespace exhausted"))
                    as u32;
            }
            ui.desktop
        }
        17 => {
            let mut ui = image.cpu.ui.borrow_mut();
            let app = stop(ui.app());
            let d = app.desktop;
            match a[0] {
                0 => d.size.width as u32,
                1 => d.size.height as u32,
                76..=80 => {
                    let (v, n) = stop(app.screen_geometry());
                    match a[0] {
                        76 => v.origin.x as i32 as u32,
                        77 => v.origin.y as i32 as u32,
                        78 => v.size.width as u32,
                        79 => v.size.height as u32,
                        _ => n,
                    }
                }
                4 => {
                    let r = stop(app.frame_for_client(
                        host::Rect {
                            origin: host::Point { x: 0., y: 0. },
                            size: host::Size {
                                width: 100.,
                                height: 100.,
                            },
                        },
                        15,
                    ));
                    (r.size.height - 100.) as u32
                }
                _ => unsupported(&format!(
                    "system metric {} is outside the host display profile",
                    a[0]
                )),
            }
        }
        18 => image.cpu.ui.borrow().active,
        19..=21 => {
            let mut ui = image.cpu.ui.borrow_mut();
            let Some(w) = ui.windows.get_mut(&a[0]) else {
                return error(1400);
            };
            let value = match a[1] as i32 {
                -21 => &mut w.user,
                -4 => &mut w.procedure,
                -6 => &mut w.instance,
                -12 => &mut w.id,
                -16 => {
                    if id == 21 {
                        unsupported(
                            "window style replacement requires native frame transition support",
                        );
                    }
                    return w.style;
                }
                -20 => {
                    if id == 21 {
                        unsupported(
                            "window extended-style replacement requires native transitions",
                        );
                    }
                    return w.ex_style;
                }
                index if index >= 0 && (index as usize + 4) <= w.extra.len() => {
                    let at = index as usize;
                    let old = u32::from_le_bytes(w.extra[at..at + 4].try_into().unwrap());
                    if id == 21 {
                        w.extra[at..at + 4].copy_from_slice(&a[2].to_le_bytes());
                    }
                    return old;
                }
                _ => return error(1413),
            };
            let old = *value;
            if id == 21 {
                if a[1] as i32 == -4 && !executable(image, a[2]) {
                    return error(87);
                }
                *value = a[2];
            }
            old
        }
        22 => destroy(image, a[0]),
        23 => {
            if !image.cpu.ui.borrow().windows.contains_key(&a[0]) {
                return error(1400);
            }
            show(image, a[0], 1);
            image.cpu.ui.borrow().windows[&a[0]].native.key() as u32
        }
        24 => {
            write(image, a[0], &a[1..5]);
            1
        }
        25 | 26 => {
            let point = read::<2>(image, a[1]);
            let ui = image.cpu.ui.borrow();
            let Some(w) = ui.windows.get(&a[0]) else {
                return error(1400);
            };
            let f = w.native.frame();
            let c = w.native.client();
            let x = f.origin.x as i32;
            let y = (f.origin.y + f.size.height - c.size.height) as i32;
            let sign = if id == 25 { 1 } else { -1 };
            write(
                image,
                a[1],
                &[
                    (point[0] as i32).wrapping_add(sign * x) as u32,
                    (point[1] as i32).wrapping_add(sign * y) as u32,
                ],
            );
            1
        }
        27 => {
            if !image.cpu.ui.borrow().windows.contains_key(&a[0]) {
                return error(1400);
            }
            if a[1] != 0 {
                write(image, a[1], &[std::process::id()]);
            }
            alkyhol::ffi::GetCurrentThreadId()
        }
        28 => {
            let ui = image.cpu.ui.borrow();
            let Some(w) = ui.windows.get(&a[0]) else {
                return error(1400);
            };
            let c = &ui.classes[&w.class];
            match a[1] as i32 {
                -26 => c.style,
                -24 => c.procedure,
                -16 => c.instance,
                -14 => c.icon,
                -12 => c.cursor,
                -10 => c.background,
                _ => unsupported("class-long index is outside the registered metadata profile"),
            }
        }
        _ => unreachable!(),
    }
}
