use std::{fmt::Debug, ops::{Add, AddAssign, Div, Mul, MulAssign, Sub, SubAssign}};

use derive_serialize::Serialize;
use multi_impl::multi_impl;
use nalgebra::{Matrix3, Vector3};

use crate::engine::gl_types::{inner_matrix::InnerMatrix, matrix_arithmetic, private::Seal, vectors::Vec3, GLScalar, Make};

#[repr(C)]
#[derive(Clone, Copy, PartialEq, PartialOrd, Serialize)]
pub struct Mat3(pub(in crate::engine::gl_types) Matrix3<f32>);

impl Debug for Mat3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Mat3 {
    pub const ZERO: Mat3 = Mat3::_new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    pub const IDENTITY: Mat3 = Mat3::_new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);

    pub(in crate::engine::gl_types) const fn _new(m11: f32, m12: f32, m13: f32, m21: f32, m22: f32, m23: f32, m31: f32, m32: f32, m33: f32) -> Self {
        Self(Matrix3::new(m11, m12, m13, m21, m22, m23, m31, m32, m33))
    }
}

matrix_arithmetic!(Mat3);

impl Seal for Mat3 {}

pub trait Mat3Constructor<T>: Seal {
    fn new(args: T) -> Mat3;
}

impl<A: GLScalar> Mat3Constructor<A> for Mat3 {
    fn new(args: A) -> Mat3 {
        Mat3::_new(
            args.as_(), 0.0, 0.0,
            0.0, args.as_(), 0.0,
            0.0, 0.0, args.as_())
    }
}

impl Mat3Constructor<Vec3> for Mat3 {
    fn new(args: Vec3) -> Mat3 {
        Self(Matrix3::from_diagonal(&args.0))
    }
}

impl Mat3Constructor<(Vec3, Vec3, Vec3)> for Mat3 {
    fn new(args: (Vec3, Vec3, Vec3)) -> Mat3 {
        Self(Matrix3::from_columns(&[args.0.0, args.1.0, args.2.0]))
    }
}

impl<A, B, C, D, E, F, G, H, I> Mat3Constructor<(A, B, C, D, E, F, G, H, I)> for Mat3 
where 
    A: GLScalar, B: GLScalar, C: GLScalar,
    D: GLScalar, E: GLScalar, F: GLScalar,
    G: GLScalar, H: GLScalar, I: GLScalar
        {
    fn new(args: (A, B, C, D, E, F, G, H, I)) -> Mat3 {
        Mat3::_new(args.0.as_(), args.1.as_(), args.2.as_(), args.3.as_(), args.4.as_(), args.5.as_(), args.6.as_(), args.7.as_(), args.8.as_())
    }
}

impl Mat3Constructor<super::Mat2> for Mat3 {
    fn new(args: super::Mat2) -> Mat3 {
        let mut cols: Vec<_> = args.0.column_iter().map(|col| Vector3::new(col[0], col[1], 0.0f32)).collect();
        cols.push(Vector3::new(0.0f32, 0.0f32, 1.0f32));

        Self(Matrix3::from_columns(&cols[..]))
    }
}

impl Mat3Constructor<super::Mat4> for Mat3 {
    fn new(args: super::Mat4) -> Mat3 {
        let cols: Vec<_> = args.0.column_iter().map(|col| Vector3::new(col[0], col[1], col[2])).collect();

        Self(Matrix3::from_columns(&cols[0..2]))
    }
}

#[macro_export]
macro_rules! mat3 {
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr, $g:expr, $h:expr, $i:expr) => {
        {
            use $crate::engine::gl_types::matrices::Mat3Constructor;
            $crate::engine::gl_types::matrices::Mat3::new(($a, $b, $c, $d, $e, $f, $g, $h, $i))
        }
    };
    ($a:expr, $b:expr, $c:expr) => {
        {
            use $crate::engine::gl_types::matrices::Mat3Constructor;
            $crate::engine::gl_types::matrices::Mat3::new(($a, $b, $c))
        }
    };
    ($a:expr) => {
        {
            use $crate::engine::gl_types::matrices::Mat3Constructor;
            $crate::engine::gl_types::matrices::Mat3::new($a)
        }
    };
    () => {
        {
            use $crate::engine::gl_types::matrices::Mat3Constructor;
            $crate::engine::gl_types::matrices::Mat3::new(0)
        }
    };
}

pub use mat3;

impl InnerMatrix<3, 3> for Mat3 {
    fn get_inner_matrix(&self) -> &nalgebra::Matrix<f32, nalgebra::Const<3>, nalgebra::Const<3>, nalgebra::ArrayStorage<f32, 3, 3>> {
        &self.0
    }

    fn get_inner_matrix_mut(&mut self) -> &mut nalgebra::Matrix<f32, nalgebra::Const<3>, nalgebra::Const<3>, nalgebra::ArrayStorage<f32, 3, 3>> {
        &mut self.0
    }

    fn into_inner_matrix(self) -> nalgebra::Matrix<f32, nalgebra::Const<3>, nalgebra::Const<3>, nalgebra::ArrayStorage<f32, 3, 3>> {
        self.0
    }
}

impl Make<Matrix3<f32>> for Mat3 {
    fn make(inner: Matrix3<f32>) -> Self {
        Self(inner)
    }
}

impl AsRef<Mat3> for Mat3 {
    fn as_ref(&self) -> &Mat3 {
        self
    }
}