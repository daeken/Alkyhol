use glam::{Quat, Vec3};
#[derive(Clone, Debug)]
pub struct Camera {
    pub eye: Vec3,
    pub heading: f32,
    pub pitch: f32,
}
impl Camera {
    #[allow(clippy::approx_constant)] // Retain the exact upstream preset literals.
    pub fn preset(index: usize) -> Result<Self, String> {
        let data = [
            ([-1100., 170., -30.], 1.5707, 0.),
            ([299., 208., -202.], -3.1111, 0.5953),
            ([-1237.61, 80.60, -26.02], -1.5707, 0.268),
            ([-977.90, 595.05, -194.97], -2.077, -0.450),
            ([-1463., 600., 394.52], -1.236, 0.),
        ];
        let &(eye, heading, pitch) = data.get(index).ok_or("camera preset must be 0 through 4")?;
        Ok(Self {
            eye: Vec3::from_array(eye),
            heading,
            pitch,
        })
    }
    pub fn basis(&self) -> [Vec3; 3] {
        let rotation = Quat::from_rotation_y(self.heading) * Quat::from_rotation_x(self.pitch);
        [rotation * Vec3::X, rotation * Vec3::Y, rotation * -Vec3::Z]
    }
    pub fn rotate(&mut self, yaw: f32, pitch: f32) {
        self.heading -= yaw;
        self.pitch = (self.pitch - pitch).clamp(-1.56, 1.56);
    }
}
