//! Physical presentation lifetime regression; no game data or Windows parity claim.
fn unsupported(message: &str) -> ! {
    panic!("{message}")
}
type Result<T> = std::result::Result<T, String>;
#[path = "src/window_host32.rs"]
mod host;
unsafe extern "C" {
    fn alkyhol_pe32_resident_bytes() -> u64;
}
fn main() -> Result<()> {
    let app = host::App::new()?;
    let window = app.create(
        100,
        100,
        640,
        480,
        15,
        &"Alkyhol presentation lifetime check"
            .encode_utf16()
            .collect::<Vec<_>>(),
    )?;
    window.show(true);
    let mut pixels = vec![255u8; 640 * 480 * 4];
    for _ in 0..4 {
        window.present(640, 480, &pixels)?;
        app.pump()?;
    }
    let before = unsafe { alkyhol_pe32_resident_bytes() };
    for frame in 0..160 {
        for pixel in pixels.chunks_exact_mut(4) {
            pixel[1] = (frame & 255) as u8;
        }
        window.present(640, 480, &pixels)?;
        app.pump()?;
    }
    let after = unsafe { alkyhol_pe32_resident_bytes() };
    let growth = after.saturating_sub(before);
    println!(
        "physical AppKit 160-frame retained-memory check: before={before} after={after} growth={growth}"
    );
    assert_ne!(before, u64::MAX);
    assert_ne!(after, u64::MAX);
    if !std::env::args().any(|a| a == "--observe-only") {
        assert!(
            growth < 96 * 1024 * 1024,
            "frame images accumulated instead of releasing"
        );
    }
    window.show(false);
    app.pump()?;
    Ok(())
}
