use std::{io::Write, path::Path};
fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    assert_eq!(args.len(), 2, "geometry_stream MODEL OUTPUT");
    let model =
        alkyhol_miniengine::h3d::parse(&std::fs::read(Path::new(&args[0])).unwrap()).unwrap();
    let mut file = std::fs::File::create(&args[1]).unwrap();
    for mesh in &model.meshes {
        file.write_all(&(mesh.material as u32).to_le_bytes())
            .unwrap();
        file.write_all(&(mesh.vertices.len() as u32).to_le_bytes())
            .unwrap();
        file.write_all(bytemuck::cast_slice(&mesh.vertices))
            .unwrap();
    }
}
