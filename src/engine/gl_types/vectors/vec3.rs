use std::{fmt::Debug, ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign}};

use derive_serialize::Serialize;
use multi_impl::multi_impl;
use nalgebra::{Matrix, Vector3};

use crate::engine::gl_types::{GLScalar, Make, inner_matrix::InnerMatrix, private::Seal, vector_arithmetic, vectors::VecN};

use super::{Vec2, Vec4};

#[repr(C)]
#[derive(Clone, Copy, PartialEq, PartialOrd, Serialize)]
pub struct Vec3(pub(in crate::engine::gl_types) Vector3<f32>);

impl serde::Serialize for Vec3 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        self.as_slice().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Vec3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        Ok(Self::from_array(<[_; _]>::deserialize(deserializer)?))
    }
}

impl Debug for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3::_new(0.0, 0.0, 0.0);
    pub const ONE: Vec3 = Vec3::_new(1.0, 1.0, 1.0);

    pub(in crate::engine::gl_types) const fn _new(x: f32, y: f32, z: f32) -> Vec3 {
        Self(Vector3::new(x, y, z))
    }
}

vector_arithmetic!(Vec3);

impl Seal for Vec3 {}

pub trait Vec3Constructor<T>: Seal {
    fn new(args: T) -> Self;
}

impl<A: GLScalar, B: GLScalar, C: GLScalar> Vec3Constructor<(A, B, C)> for Vec3 {
    fn new(args: (A, B, C)) -> Vec3 {
        let (a, b, c) = args;
        Self::_new(a.as_(), b.as_(), c.as_())
    }
}

impl<B: GLScalar> Vec3Constructor<(Vec2, B)> for Vec3 {
    fn new(args: (Vec2, B)) -> Vec3 {
        let (a, b) = args;
        Self::_new(a.x(), a.y(), b.as_())
    }
}

impl<A: GLScalar> Vec3Constructor<(A, Vec2)> for Vec3 {
    fn new(args: (A, Vec2)) -> Vec3 {
        let (a, b) = args;
        Self::_new(a.as_(), b.x(), b.y())
    }
}

impl<A: GLScalar> Vec3Constructor<A> for Vec3 {
    fn new(args: A) -> Vec3 {
        Self::_new(args.as_(), args.as_(), args.as_())
    }
}

impl Vec3Constructor<Vec2> for Vec3 {
    fn new(args: Vec2) -> Vec3 {
        Self::_new(args.x(), args.y(), 0.0f32)
    }
}

impl Vec3Constructor<Vec4> for Vec3 {
    fn new(args: Vec4) -> Vec3 {
        Self::_new(args.x(), args.y(), args.z())
    }
}

#[macro_export]
macro_rules! vec3 {
    ($a:expr, $b:expr, $c:expr) => {
        {
            use $crate::engine::gl_types::vectors::Vec3Constructor;
            $crate::engine::gl_types::vectors::Vec3::new(($a, $b, $c))
        }
    };
    ($a:expr, $b:expr) => {
        {
            use $crate::engine::gl_types::vectors::Vec3Constructor;
            $crate::engine::gl_types::vectors::Vec3::new(($a, $b))
        }
    };
    ($a:expr) => {
        {
            use $crate::engine::gl_types::vectors::Vec3Constructor;
            $crate::engine::gl_types::vectors::Vec3::new($a)
        }
    };
    () => {
        {
            use $crate::engine::gl_types::vectors::Vec3Constructor;
            $crate::engine::gl_types::vectors::Vec3::new(0)
        }
    };
}

pub use vec3;

impl InnerMatrix<3, 1> for Vec3 {
    fn get_inner_matrix(&self) -> &Matrix<f32, nalgebra::Const<3>, nalgebra::Const<1>, nalgebra::ArrayStorage<f32, 3, 1>> {
        &self.0
    }

    fn get_inner_matrix_mut(&mut self) -> &mut Matrix<f32, nalgebra::Const<3>, nalgebra::Const<1>, nalgebra::ArrayStorage<f32, 3, 1>> {
        &mut self.0
    }

    fn into_inner_matrix(self) -> Matrix<f32, nalgebra::Const<3>, nalgebra::Const<1>, nalgebra::ArrayStorage<f32, 3, 1>> {
        self.0
    }
}

impl Make<Vector3<f32>> for Vec3 {
    fn make(inner: Vector3<f32>) -> Self {
        Self(inner)
    }
}

impl AsRef<Vec3> for Vec3 {
    fn as_ref(&self) -> &Vec3 {
        self
    }
}