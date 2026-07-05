use crate::engine::gl_types::matrices::Mat4;

pub fn frustum(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4 {
    Mat4::_new(
        2.0 * near / (right - left), 0.0                        , (right + left) / (right - left), 0.0,
        0.0                        , 2.0 * near / (top - bottom), (top + bottom) / (top - bottom), 0.0,
        0.0                        , 0.0                        , -(far + near) / (far - near)   , -2.0 * far * near / (far - near),
        0.0                        , 0.0                        , 1.0                           , 0.0
    )
}

pub fn ortho(width: f32, height: f32, mut z_near: f32, mut z_far: f32) -> Mat4 {
    // Swap to invert depth in order to be consistent with the
    // inverted depth of the perspective projection (caused by left-handed coordinates).
    std::mem::swap(&mut z_near, &mut z_far);
    
    let right = width / 2.0;
    let left = -right;
    let top = height / 2.0;
    let bottom = -top;

    Mat4::_new(
        2.0 / (right - left), 0.0                 , 0.0                  , -(right + left) / (right - left),
        0.0                 , 2.0 / (top - bottom), 0.0                  , -(top + bottom) / (top - bottom),
        0.0                 , 0.0                 , 2.0 / (z_far - z_near), -(z_far + z_near) / (z_far - z_near),
        0.0                 , 0.0                 , 0.0                  , 1.0
    )
}

pub fn ortho_aspect(width: f32, aspect: f32, z_near: f32, z_far: f32) -> Mat4 {
    ortho(width, width / aspect, z_near, z_far)
}

pub fn perspective(fovx: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let fovy = 2.0 * f32::atan(f32::tan(fovx / 2.0) / aspect); // Google gemnini formula (i'm trusting that its correct)
    let near = -near;
    let far = -far;
    let top = near * (fovy / 2.0); 
    let bottom = -top;
    let right = top * aspect;
    let left = -right;

    frustum(left, right, bottom, top, near, far)
}