//! Upstream scene data and draw order. No simplified or procedurally substituted meshes.
use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use std::path::Path;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}
#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>, // Fully expanded, original primitive order.
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Instance {
    pub mesh: u32,
    pub material: u32,
    pub scale: f32,
    pub position: [f32; 3],
}

/// Positive OBJ indices only, as in the pinned loader/assets. Repeated object names combine.
/// Extra polygon corners are ignored by upstream; malformed/missing attributes fail explicitly.
pub fn load_obj(text: &str, object: &str) -> Result<Mesh, String> {
    let mut positions = Vec::<[f32; 3]>::new();
    let mut normals = Vec::<[f32; 3]>::new();
    let mut uvs = Vec::<[f32; 2]>::new();
    let mut faces = Vec::new();
    let mut current = false;
    let mut found = false;
    for (line_number, line) in text.lines().enumerate() {
        let mut tokens = line.split_whitespace();
        let Some(kind) = tokens.next() else { continue };
        let error = || format!("invalid OBJ at line {}", line_number + 1);
        if kind == "o" {
            current = tokens.next() == Some(object);
            found |= current;
            continue;
        }
        if !current {
            continue;
        }
        match kind {
            "v" | "vn" | "vt" => {
                let n = if kind == "vt" { 2 } else { 3 };
                let values = tokens
                    .take(n)
                    .map(str::parse::<f32>)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| error())?;
                if values.len() != n || values.iter().any(|v| !v.is_finite()) {
                    return Err(error());
                }
                match kind {
                    "v" => {
                        positions.push([values[0] * 0.007, values[1] * 0.007, values[2] * 0.007])
                    }
                    "vn" => normals.push([values[0], values[1], values[2]]),
                    _ => uvs.push([values[0], values[1]]),
                }
            }
            "f" => {
                let corners = tokens.take(3).map(|s| s.to_owned()).collect::<Vec<_>>();
                if corners.len() != 3 {
                    return Err(error());
                }
                faces.push(corners);
            }
            _ => {}
        }
    }
    if !found {
        return Err(format!("OBJ object not found: {object}"));
    }
    let mut vertices = Vec::with_capacity(faces.len() * 3);
    for face in faces {
        for corner in face {
            let parts = corner.split('/').collect::<Vec<_>>();
            let index = |s: &str| {
                s.parse::<usize>()
                    .ok()
                    .and_then(|i| i.checked_sub(1))
                    .ok_or_else(|| format!("invalid OBJ corner: {corner}"))
            };
            let p = *positions
                .get(index(parts[0])?)
                .ok_or("position out of bounds")?;
            let normal = if let Some(s) = parts.get(2).filter(|s| !s.is_empty()) {
                *normals.get(index(s)?).ok_or("normal out of bounds")?
            } else {
                [0.; 3]
            };
            let uv = if let Some(s) = parts.get(1).filter(|s| !s.is_empty()) {
                *uvs.get(index(s)?).ok_or("UV out of bounds")?
            } else {
                [0.5; 2]
            };
            vertices.push(Vertex {
                position: p,
                normal,
                uv,
            });
        }
    }
    if vertices.is_empty() {
        return Err("empty OBJ geometry".into());
    }
    Ok(Mesh { vertices })
}

pub fn load_meshes(directory: &Path) -> Result<Vec<Mesh>, String> {
    let mut meshes = vec![cube()];
    for (file, object) in [
        ("trunk.obj", "tree"),
        ("leaves.obj", "leaves"),
        ("bush.obj", "bush"),
    ] {
        let text =
            std::fs::read_to_string(directory.join(file)).map_err(|e| format!("{file}: {e}"))?;
        meshes.push(load_obj(&text, object)?);
    }
    Ok(meshes)
}

pub const CUBE_INDICES: [usize; 36] = [
    3, 1, 0, 2, 1, 3, 6, 4, 5, 7, 4, 6, 11, 9, 8, 10, 9, 11, 14, 12, 13, 15, 12, 14, 19, 17, 16,
    18, 17, 19, 22, 20, 21, 23, 20, 22,
];
pub fn cube() -> Mesh {
    let faces = [
        (
            [[-1., 1., -1.], [1., 1., -1.], [1., 1., 1.], [-1., 1., 1.]],
            [0., 1., 0.],
        ),
        (
            [
                [-1., -1., -1.],
                [1., -1., -1.],
                [1., -1., 1.],
                [-1., -1., 1.],
            ],
            [0., -1., 0.],
        ),
        (
            [
                [-1., -1., 1.],
                [-1., -1., -1.],
                [-1., 1., -1.],
                [-1., 1., 1.],
            ],
            [-1., 0., 0.],
        ),
        (
            [[1., -1., 1.], [1., -1., -1.], [1., 1., -1.], [1., 1., 1.]],
            [1., 0., 0.],
        ),
        (
            [
                [-1., -1., -1.],
                [1., -1., -1.],
                [1., 1., -1.],
                [-1., 1., -1.],
            ],
            [0., 0., -1.],
        ),
        (
            [[-1., -1., 1.], [1., -1., 1.], [1., 1., 1.], [-1., 1., 1.]],
            [0., 0., 1.],
        ),
    ];
    let vertices = faces
        .into_iter()
        .flat_map(|(points, normal)| {
            points.into_iter().map(move |position| Vertex {
                position,
                normal,
                uv: [0.; 2],
            })
        })
        .collect::<Vec<_>>();
    Mesh {
        vertices: CUBE_INDICES.iter().map(|&i| vertices[i]).collect(),
    }
}

/// Standard mt19937. Float conversion follows current MSVC generate_canonical<float> (top 24 bits).
pub struct Mt19937 {
    state: [u32; 624],
    index: usize,
}
impl Mt19937 {
    pub fn new(seed: u32) -> Self {
        let mut state = [0; 624];
        state[0] = seed;
        for i in 1..624 {
            state[i] = 1812433253u32
                .wrapping_mul(state[i - 1] ^ (state[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        Self { state, index: 624 }
    }
    pub fn next_u32(&mut self) -> u32 {
        if self.index == 624 {
            for i in 0..624 {
                let x = (self.state[i] & 0x80000000) | (self.state[(i + 1) % 624] & 0x7fffffff);
                self.state[i] = self.state[(i + 397) % 624]
                    ^ (x >> 1)
                    ^ (if x & 1 != 0 { 0x9908b0df } else { 0 });
            }
            self.index = 0;
        }
        let mut y = self.state[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c5680;
        y ^= (y << 15) & 0xefc60000;
        y ^= y >> 18;
        y
    }
    pub fn uniform(&mut self, min: f32, max: f32) -> f32 {
        // Current MSVC discards eight low bits before conversion, avoiding rounding to one.
        let unit = (self.next_u32() >> 8) as f32 * (1.0 / 16777216.0f32);
        unit * (max - min) + min
    }
}

pub fn instances() -> Vec<Instance> {
    let mut out = Vec::with_capacity(13525);
    let mut rng = Mt19937::new(42);
    let mut bush_rng = Mt19937::new(42);
    for x in -10..=10 {
        for z in -10..=10 {
            out.push(Instance {
                mesh: 0,
                material: 0,
                scale: 2.,
                position: [x as f32 * 4. + 10., 0., z as f32 * 4.],
            });
        }
    }
    for x in -15..=15 {
        for z in -15..=15 {
            out.push(Instance {
                mesh: 0,
                material: 1,
                scale: 0.01,
                position: [
                    0.5 + x as f32 * 0.1,
                    1.7 + rng.uniform(-0.1, 0.7),
                    z as f32 * 0.1,
                ],
            });
        }
    }
    let mut trees = Vec::with_capacity(961);
    for x in -15..=15 {
        for z in -15..=15 {
            let position = [
                x as f32 * 1.9 + rng.uniform(-0.1, 0.7) / 6.,
                2. - rng.uniform(-0.1, 0.7) / 3.,
                z as f32 * 1.9 + rng.uniform(-0.1, 0.7) / 6.,
            ];
            trees.push(position);
            out.push(Instance {
                mesh: 1,
                material: 2,
                scale: 35.,
                position,
            });
        }
    }
    for position in trees {
        out.push(Instance {
            mesh: 2,
            material: 3,
            scale: 35.,
            position,
        });
    }
    for x in -50..=50 {
        for z in -50..=50 {
            out.push(Instance {
                mesh: 3,
                material: 4,
                scale: 2.1,
                position: [
                    x as f32 * 0.2 + rng.uniform(-0.1, 0.7),
                    1.7 + bush_rng.uniform(0.7, 0.75) / 3.,
                    z as f32 * 0.2 + rng.uniform(-0.1, 0.7),
                ],
            });
        }
    }
    out
}

#[derive(Clone, Debug)]
pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: glam::Mat3::from_rotation_y(std::f32::consts::FRAC_PI_4) * Vec3::new(0., 2.2, -2.),
            target: Vec3::new(1., 2.5, -6.),
            up: Vec3::Y,
        }
    }
}
impl Camera {
    pub fn basis(&self) -> [Vec3; 3] {
        let forward = (self.target - self.eye).normalize();
        let right = self.up.cross(forward).normalize();
        let up = forward.cross(right).normalize();
        [right, up, forward]
    }
    pub fn move_by(&mut self, delta: Vec3) {
        self.eye += delta;
        self.target += delta;
        let d = (2.02 - self.eye.y).max(0.);
        if d > 0. {
            self.eye.y = 2.02;
            self.target.y += d;
        }
    }
    pub fn rotate(&mut self, yaw: f32, pitch: f32) {
        let right = self.basis()[0];
        let rotation =
            glam::Quat::from_axis_angle(self.up, yaw) * glam::Quat::from_axis_angle(right, pitch);
        self.target = self.eye + rotation * (self.target - self.eye);
    }
}
