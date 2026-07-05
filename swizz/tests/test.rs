#![allow(unused)]
use swizz::generate_swizzles;

struct Vec2([f32; 2]);
struct Vec3([f32; 3]);
struct Vec4([f32; 4]);

impl Vec2 {
    pub fn _new(x: f32, y: f32) -> Self {
        Self([x, y])
    }
}
impl Vec3 {
    pub fn _new(x: f32, y: f32, z: f32) -> Self {
        Self([x, y, z])
    }
}
impl Vec4 {
    pub fn _new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self([x, y, z, w])
    }
}

generate_swizzles!(Vec2, xy, 4);
generate_swizzles!(Vec3, xyz, 4);
generate_swizzles!(Vec4, xyzw, 4);

#[test]
fn swizz() {
    let t = Vec4([0.0, 1.0, 2.0, 3.0]);
    let v2 = Vec2([1.0, 2.0]);

    panic!("{:?}", v2.xxyx().0);
}