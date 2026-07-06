use std::{fmt::Debug, ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign}};

use derive_serialize::Serialize;
use multi_impl::multi_impl;
use nalgebra::Vector4;

use crate::engine::gl_types::{inner_matrix::InnerMatrix, vector_arithmetic, private::Seal, GLScalar, Make};

use super::{Vec2, Vec3};

#[repr(C)]
#[derive(Clone, Copy, PartialEq, PartialOrd, Serialize)]
pub struct Vec4(pub(in crate::engine::gl_types) Vector4<f32>);

impl Debug for Vec4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Vec4 {
    pub const ZERO: Vec4 = Vec4::_new(0.0, 0.0, 0.0, 0.0);
    pub const ONE: Vec4 = Vec4::_new(1.0, 1.0, 1.0, 1.0);

    pub(in crate::engine::gl_types) const fn _new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self(Vector4::new(x, y, z, w))
    }
}

vector_arithmetic!(Vec4);

impl Seal for Vec4 {}

pub trait Vec4Constructor<T>: Seal {
    fn new(args: T) -> Self;
}

impl<A: GLScalar, B: GLScalar, C: GLScalar, D: GLScalar> Vec4Constructor<(A, B, C, D)> for Vec4 {
    fn new(args: (A, B, C, D)) -> Vec4 {
        let (a, b, c, d) = args;
        Vec4::_new(a.as_(), b.as_(), c.as_(), d.as_())
    }
}

impl<B: GLScalar, C: GLScalar> Vec4Constructor<(Vec2, B, C)> for Vec4 {
    fn new(args: (Vec2, B, C)) -> Vec4 {
        let (a, b, c) = args;
        Self::_new(a.x(), a.y(), b.as_(), c.as_())
    }
}

impl<A: GLScalar, C: GLScalar> Vec4Constructor<(A, Vec2, C)> for Vec4 {
    fn new(args: (A, Vec2, C)) -> Vec4 {
        let (a, b, c) = args;
        Self::_new(a.as_(), b.x(), b.y(), c.as_())
    }
}

impl<A: GLScalar, B: GLScalar> Vec4Constructor<(A, B, Vec2)> for Vec4 {
    fn new(args: (A, B, Vec2)) -> Vec4 {
        let (a, b, c) = args;
        Self::_new(a.as_(), b.as_(), c.x(), c.y())
    }
}

impl<B: GLScalar> Vec4Constructor<(Vec3, B)> for Vec4 {
    fn new(args: (Vec3, B)) -> Vec4 {
        let (a, b) = args;
        Self::_new(a.x(), a.y(), a.z(), b.as_())
    }
}

impl<A: GLScalar> Vec4Constructor<(A, Vec3)> for Vec4 {
    fn new(args: (A, Vec3)) -> Vec4 {
        let (a, b) = args;
        Self::_new(a.as_(), b.x(), b.y(), b.z())
    }
}

impl Vec4Constructor<(Vec2, Vec2)> for Vec4 {
    fn new(args: (Vec2, Vec2)) -> Vec4 {
        let (a, b) = args;
        Self::_new(a.x(), a.y(), b.x(), b.y())
    }
}

impl<A: GLScalar> Vec4Constructor<A> for Vec4 {
    fn new(args: A) -> Vec4 {
        Self::_new( args.as_(), args.as_(), args.as_(), args.as_())
    }
}

impl Vec4Constructor<Vec2> for Vec4 {
    fn new(args: Vec2) -> Vec4 {
        Self::_new(args.x(), args.y(), 0.0f32, 0.0f32)
    }
}

impl Vec4Constructor<Vec3> for Vec4 {
    fn new(args: Vec3) -> Vec4 {
        Self::_new(args.x(), args.y(), args.z(), 0.0f32)
    }
}

#[macro_export]
macro_rules! vec4 {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        {
            use $crate::engine::gl_types::vectors::Vec4Constructor;
            $crate::engine::gl_types::vectors::Vec4::new(($a, $b, $c, $d))
        }
    };
    ($a:expr, $b:expr, $c:expr) => {
        {
            use $crate::engine::gl_types::vectors::Vec4Constructor;
            $crate::engine::gl_types::vectors::Vec4::new(($a, $b, $c))
        }
    };
    ($a:expr, $b:expr) => {
        {
            use $crate::engine::gl_types::vectors::Vec4Constructor;
            $crate::engine::gl_types::vectors::Vec4::new(($a, $b))
        }
    };
    ($a:expr) => {
        {
            use $crate::engine::gl_types::vectors::Vec4Constructor;
            $crate::engine::gl_types::vectors::Vec4::new($a)
        }
    };
    () => {
        {
            use $crate::engine::gl_types::vectors::Vec4Constructor;
            $crate::engine::gl_types::vectors::Vec4::new(0)
        }
    };
}

pub use vec4;

impl InnerMatrix<4, 1> for Vec4 {
    fn get_inner_matrix(&self) -> &nalgebra::Matrix<f32, nalgebra::Const<4>, nalgebra::Const<1>, nalgebra::ArrayStorage<f32, 4, 1>> {
        &self.0
    }

    fn get_inner_matrix_mut(&mut self) -> &mut nalgebra::Matrix<f32, nalgebra::Const<4>, nalgebra::Const<1>, nalgebra::ArrayStorage<f32, 4, 1>> {
        &mut self.0
    }

    fn into_inner_matrix(self) -> nalgebra::Matrix<f32, nalgebra::Const<4>, nalgebra::Const<1>, nalgebra::ArrayStorage<f32, 4, 1>> {
        self.0
    }
}

impl Make<Vector4<f32>> for Vec4 {
    fn make(inner: Vector4<f32>) -> Self {
        Self(inner)
    }
}

impl AsRef<Vec4> for Vec4 {
    fn as_ref(&self) -> &Vec4 {
        self
    }
}