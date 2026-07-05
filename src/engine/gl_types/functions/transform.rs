use crate::engine::gl_types::{matrices::Mat4, vectors::Vec3};

use super::geometric::{cross, dot, normalize};

pub fn look_at(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    let z_axis = normalize(center - eye);
    let x_axis = normalize(cross(up, z_axis));
    let y_axis = cross(z_axis, x_axis);

    let tx = -dot(x_axis, eye);
    let ty = -dot(y_axis, eye);
    let tz = -dot(z_axis, eye   );

    Mat4::_new(
        x_axis.x(), x_axis.y(), x_axis.z(), tx,
        y_axis.x(), y_axis.y(), y_axis.z(), ty,
        z_axis.x(), z_axis.y(), z_axis.z(), tz,
        0.0, 0.0, 0.0, 1.0
    )
}

// pub fn rotate(m: &Mat4, angle: f32, axis: &Vec3) -> Mat4 {

// }