use alkyhol_sakura::scene::instances;
use std::fmt::Write;
fn main() {
    let output = std::env::args().nth(1).expect("output path required");
    let scene = instances();
    let mut text = String::new();
    for i in &scene {
        writeln!(
            text,
            "{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
            i.mesh,
            i.material,
            i.scale.to_bits(),
            i.position[0].to_bits(),
            i.position[1].to_bits(),
            i.position[2].to_bits()
        )
        .unwrap();
    }
    std::fs::write(output, text).unwrap();
    println!("Rust placement: {} instances", scene.len());
}
