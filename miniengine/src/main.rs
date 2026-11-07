use alkyhol_miniengine::{
    camera::Camera,
    renderer::{RenderMode, Renderer},
};
use metal::*;
use objc::{msg_send, sel, sel_impl};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::{collections::HashSet, path::PathBuf, time::Instant};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

struct Options {
    assets: PathBuf,
    width: u32,
    height: u32,
    capture: Option<PathBuf>,
    headless: bool,
    frames: Option<u64>,
    camera: usize,
    mode: RenderMode,
}
fn options() -> Result<Options, String> {
    let mut o = Options {
        assets: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../local/miniengine/assets"),
        width: 960,
        height: 540,
        capture: None,
        headless: false,
        frames: None,
        camera: 2,
        mode: RenderMode::Geometry,
    };
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--assets" => o.assets = args.next().ok_or("--assets requires a path")?.into(),
            "--width" => {
                o.width = args
                    .next()
                    .ok_or("--width requires a number")?
                    .parse()
                    .map_err(|_| "invalid width")?
            }
            "--height" => {
                o.height = args
                    .next()
                    .ok_or("--height requires a number")?
                    .parse()
                    .map_err(|_| "invalid height")?
            }
            "--capture" => o.capture = Some(args.next().ok_or("--capture requires a path")?.into()),
            "--headless" => o.headless = true,
            "--frames" => {
                o.frames = Some(
                    args.next()
                        .ok_or("--frames requires a number")?
                        .parse()
                        .map_err(|_| "invalid frames")?,
                )
            }
            "--camera" => {
                o.camera = args
                    .next()
                    .ok_or("--camera requires 0..4")?
                    .parse()
                    .map_err(|_| "invalid camera")?
            }
            "--mode" => {
                o.mode = match args
                    .next()
                    .ok_or("--mode requires geometry, bary, normals or depth")?
                    .as_str()
                {
                    "geometry" => RenderMode::Geometry,
                    "bary" => RenderMode::Bary,
                    "normals" => RenderMode::Normals,
                    "depth" => RenderMode::Depth,
                    _ => return Err("unimplemented render mode".into()),
                }
            }
            "--help" => {
                println!(
                    "MiniEngine Sponza — native Metal ray tracing; geometry diagnostics\n--headless --capture path.png --width 960 --height 540 --mode geometry --camera 2\n--assets directory --frames N\nWASD move; Q/E down/up; arrows or left-drag look; PageUp/PageDown cycle camera presets; R reset. 1/G geometry; 2 Bary Rays; N normals; Z depth; Escape quit. Other upstream modes remain unimplemented."
                );
                std::process::exit(0);
            }
            _ => return Err(format!("unknown option: {a}")),
        }
    }
    if o.width == 0
        || o.height == 0
        || o.width > 4096
        || o.height > 4096
        || o.camera > 4
        || o.frames == Some(0)
    {
        return Err("invalid extent, time or frame count".into());
    }
    Ok(o)
}
struct App {
    options: Options,
    window: Option<Window>,
    layer: Option<MetalLayer>,
    renderer: Renderer,
    camera: Camera,
    keys: HashSet<KeyCode>,
    last: Instant,
    start: Instant,
    frame: u64,
    camera_index: usize,
    dragging: bool,
    cursor: Option<(f64, f64)>,
    failure: Option<String>,
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title("MiniEngine Sponza — Metal ray tracing · Geometry diagnostic")
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.options.width,
                self.options.height,
            ));
        let result = (|| -> Result<(), String> {
            let window = event_loop
                .create_window(attributes)
                .map_err(|e| e.to_string())?;
            let layer = MetalLayer::new();
            layer.set_device(self.renderer.device.metal_device());
            layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
            layer.set_presents_with_transaction(false);
            layer.set_framebuffer_only(true);
            let size = window.inner_size();
            layer.set_drawable_size(core_graphics_types::geometry::CGSize::new(
                size.width as f64,
                size.height as f64,
            ));
            if let RawWindowHandle::AppKit(handle) =
                window.window_handle().map_err(|e| e.to_string())?.as_raw()
            {
                unsafe {
                    let view = handle.ns_view.as_ptr() as *mut objc::runtime::Object;
                    let _: () = msg_send![view,setWantsLayer:objc::runtime::YES];
                    let _: () = msg_send![view,setLayer:layer.as_ref()];
                }
            } else {
                return Err("MiniEngine requires an AppKit window".into());
            }
            self.layer = Some(layer);
            window.request_redraw();
            self.window = Some(window);
            Ok(())
        })();
        if let Err(e) = result {
            self.failure = Some(e);
            event_loop.exit();
        }
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::MouseInput {
                state,
                button: winit::event::MouseButton::Left,
                ..
            } => {
                self.dragging = state == ElementState::Pressed;
                self.cursor = None;
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.dragging
                    && let Some((x, y)) = self.cursor
                {
                    self.camera.rotate(
                        (position.x - x) as f32 * 0.003,
                        (position.y - y) as f32 * 0.003,
                    );
                }
                self.cursor = Some((position.x, position.y));
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Focused(false) => self.keys.clear(),
            WindowEvent::Resized(size) => {
                if size.width > 0
                    && size.height > 0
                    && let Some(layer) = &self.layer
                {
                    layer.set_drawable_size(core_graphics_types::geometry::CGSize::new(
                        size.width as f64,
                        size.height as f64,
                    ));
                    let width = self.options.width;
                    let height = ((width as u64 * size.height as u64) / size.width as u64)
                        .clamp(1, 4096) as u32;
                    if (width, height) != self.renderer.extent()
                        && let Err(e) = self.renderer.resize(width, height)
                    {
                        self.failure = Some(e);
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    if event.state == ElementState::Pressed {
                        self.keys.insert(key);
                        if !event.repeat {
                            match key {
                                KeyCode::Escape => event_loop.exit(),
                                KeyCode::Digit1 | KeyCode::KeyG => {
                                    self.renderer.set_mode(RenderMode::Geometry)
                                }
                                KeyCode::Digit2 => self.renderer.set_mode(RenderMode::Bary),
                                KeyCode::KeyN => self.renderer.set_mode(RenderMode::Normals),
                                KeyCode::KeyZ => self.renderer.set_mode(RenderMode::Depth),
                                KeyCode::PageUp => {
                                    self.camera_index = (self.camera_index + 1) % 5;
                                    self.camera = Camera::preset(self.camera_index).unwrap();
                                }
                                KeyCode::PageDown => {
                                    self.camera_index = (self.camera_index + 4) % 5;
                                    self.camera = Camera::preset(self.camera_index).unwrap();
                                }
                                KeyCode::KeyR => {
                                    self.camera = Camera::preset(self.camera_index).unwrap()
                                }
                                _ => {}
                            }
                        }
                    } else {
                        self.keys.remove(&key);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - self.last).as_secs_f32().min(0.1);
                self.last = now;
                let [right, _, forward] = self.camera.basis();
                let mut delta = glam::Vec3::ZERO;
                for key in &self.keys {
                    delta += match key {
                        KeyCode::KeyW => forward,
                        KeyCode::KeyS => -forward,
                        KeyCode::KeyA => -right,
                        KeyCode::KeyD => right,
                        KeyCode::KeyQ => -glam::Vec3::Y,
                        KeyCode::KeyE => glam::Vec3::Y,
                        _ => glam::Vec3::ZERO,
                    };
                }
                self.camera.eye += delta * 1000. * dt;
                let yaw = (self.keys.contains(&KeyCode::ArrowRight) as i32
                    - self.keys.contains(&KeyCode::ArrowLeft) as i32)
                    as f32;
                let pitch = (self.keys.contains(&KeyCode::ArrowDown) as i32
                    - self.keys.contains(&KeyCode::ArrowUp) as i32)
                    as f32;
                self.camera.rotate(
                    yaw * dt * std::f32::consts::PI / 6.,
                    pitch * dt * std::f32::consts::PI / 6.,
                );
                let result = objc::rc::autoreleasepool(|| {
                    let Some(drawable) = self.layer.as_ref().and_then(|l| l.next_drawable()) else {
                        return Ok(None);
                    };
                    self.renderer.render(&self.camera, Some(drawable)).map(Some)
                });
                match result {
                    Ok(Some(counts)) => {
                        self.frame += 1;
                        if self.frame == 1 {
                            eprintln!("First live frame: ray statistics = {counts:?}");
                        }
                        if self.options.frames == Some(self.frame) {
                            if let Some(path) = &self.options.capture
                                && let Err(e) = self.renderer.capture(path)
                            {
                                self.failure = Some(e);
                            }
                            event_loop.exit();
                        }
                        if let Some(w) = &self.window {
                            w.set_title(&format!(
                                "Sponza · {} · {:.1} FPS · {}×{} · 1:Geometry 2:Bary N:Normals Z:Depth",
                                self.renderer.mode().label(),
                                self.frame as f64 / self.start.elapsed().as_secs_f64(),
                                self.renderer.extent().0,
                                self.renderer.extent().1
                            ));
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        self.failure = Some(e);
                        event_loop.exit();
                    }
                }
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
fn run() -> Result<(), String> {
    let options = options()?;
    let mut renderer = objc::rc::autoreleasepool(|| {
        Renderer::new(&options.assets, options.width, options.height)
    })?;
    renderer.set_mode(options.mode);
    if options.headless {
        let start = Instant::now();
        let counts = renderer.render(&Camera::preset(options.camera)?, None)?;
        eprintln!(
            "Rendered {}×{} in {:.3}s; ray statistics = {counts:?}",
            options.width,
            options.height,
            start.elapsed().as_secs_f64()
        );
        if let Some(path) = &options.capture {
            renderer.capture(path)?;
            eprintln!("Saved {}", path.display());
        }
        return Ok(());
    }
    let now = Instant::now();
    let camera_index = options.camera;
    let mut app = App {
        options,
        window: None,
        layer: None,
        renderer,
        camera: Camera::preset(camera_index)?,
        keys: HashSet::new(),
        last: now,
        start: now,
        frame: 0,
        camera_index,
        dragging: false,
        cursor: None,
        failure: None,
    };
    let event_loop = EventLoop::new().map_err(|e| e.to_string())?;
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).map_err(|e| e.to_string())?;
    if let Some(e) = app.failure {
        return Err(e);
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("MiniEngine error: {e}");
        std::process::exit(1);
    }
}
