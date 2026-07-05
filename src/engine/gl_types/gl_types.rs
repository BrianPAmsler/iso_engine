

use num::cast::AsPrimitive;

pub(in crate::engine::gl_types) mod private {
    pub trait Seal {}

    impl Seal for i32 {}
    impl Seal for i64 {}
    impl Seal for u32 {}
    impl Seal for u64 {}
    impl Seal for f32 {}
    impl Seal for f64 {}
}

pub(in crate::engine::gl_types) mod inner_matrix {
    use nalgebra::{ArrayStorage, Const, Matrix};

    pub trait InnerMatrix<const R: usize, const C: usize> {
        fn get_inner_matrix(&self) -> &Matrix<f32, Const<R>, Const<C>, ArrayStorage<f32, R, C>>;
        fn get_inner_matrix_mut(&mut self) -> &mut Matrix<f32, Const<R>, Const<C>, ArrayStorage<f32, R, C>>;
        fn into_inner_matrix(self) -> Matrix<f32, Const<R>, Const<C>, ArrayStorage<f32, R, C>>;
    }
}

pub(in crate::engine::gl_types) trait Make<T>: private::Seal {
    fn make(inner: T) -> Self;
}

pub trait GLScalar: private::Seal + AsPrimitive<i32> + AsPrimitive<i64> + AsPrimitive<u32> + AsPrimitive<u64> + AsPrimitive<f32> + AsPrimitive<f64> {}

impl GLScalar for i32 {}
impl GLScalar for i64 {}
impl GLScalar for u32 {}
impl GLScalar for u64 {}
impl GLScalar for f32 {}
impl GLScalar for f64 {}

macro_rules! vector_arithmetic {
    ($t:tt) => {
        impl Add for $t {
            type Output = Self;
        
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }
        
        impl Sub for $t {
            type Output = Self;
        
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }
        
        impl Mul for $t {
            type Output = Self;
        
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0.component_mul(&rhs.0))
            }
        }
        
        impl Div for $t {
            type Output = Self;
        
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0.component_div(&rhs.0))
            }
        }
        
        impl AddAssign for $t {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }
        
        impl SubAssign for $t {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }
        
        impl MulAssign for $t {
            fn mul_assign(&mut self, rhs: Self) {
                self.0.component_mul_assign(&rhs.0);
            }
        }
        
        impl DivAssign for $t {
            fn div_assign(&mut self, rhs: Self) {
                self.0.component_div_assign(&rhs.0);
            }
        }
        
        // Scalar-Vector Operators
        
        impl<T: GLScalar> Add<T> for $t {
            type Output = Self;
        
            fn add(self, rhs: T) -> Self::Output {
                let rhs: f32 = rhs.as_();
                Self(self.0.add_scalar(rhs))
            }
        }
        
        impl<T: GLScalar> Sub<T> for $t {
            type Output = Self;
        
            fn sub(self, rhs: T) -> Self::Output {
                let rhs: f32 = rhs.as_();
                Self(self.0.add_scalar(-rhs))
            }
        }
        
        impl<T: GLScalar> Mul<T> for $t {
            type Output = Self;
        
            fn mul(self, rhs: T) -> Self::Output {
                let rhs: f32 = rhs.as_();
                Self(self.0 * rhs)
            }
        }
        
        impl<T: GLScalar> Div<T> for $t {
            type Output = Self;
        
            fn div(self, rhs: T) -> Self::Output {
                let rhs: f32 = rhs.as_();
                Self(self.0 / rhs)
            }
        }
        
        multi_impl! (Add<$t> for (i32, u32, i64, u64, f32, f64) {
            type Output = $t;
        
            fn add(self, rhs: $t) -> Self::Output {
                rhs + self
            }
        });
        
        multi_impl! (Sub<$t> for (i32, u32, i64, u64, f32, f64) {
            type Output = $t;
        
            fn sub(self, rhs: $t) -> Self::Output {
                rhs + self
            }
        });
        
        multi_impl! (Mul<$t> for (i32, u32, i64, u64, f32, f64) {
            type Output = $t;
        
            fn mul(self, rhs: $t) -> Self::Output {
                rhs + self
            }
        });
    };
}

macro_rules! matrix_arithmetic {
    ($t:tt) => {
        impl Add for $t {
            type Output = Self;
        
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }
        
        impl Sub for $t {
            type Output = Self;
        
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }
        
        impl Mul for $t {
            type Output = Self;
        
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }
        
        impl AddAssign for $t {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }
        
        impl SubAssign for $t {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }
        
        impl MulAssign for $t {
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }
        
        // Scalar-Vector Operators
        
        impl<T: GLScalar> Add<T> for $t {
            type Output = Self;
        
            fn add(self, rhs: T) -> Self::Output {
                let rhs: f32 = rhs.as_();
                Self(self.0.add_scalar(rhs))
            }
        }
        
        impl<T: GLScalar> Sub<T> for $t {
            type Output = Self;
        
            fn sub(self, rhs: T) -> Self::Output {
                let rhs: f32 = rhs.as_();
                Self(self.0.add_scalar(-rhs))
            }
        }
        
        impl<T: GLScalar> Mul<T> for $t {
            type Output = Self;
        
            fn mul(self, rhs: T) -> Self::Output {
                let rhs: f32 = rhs.as_();
                Self(self.0 * rhs)
            }
        }
        
        impl<T: GLScalar> Div<T> for $t {
            type Output = Self;
        
            fn div(self, rhs: T) -> Self::Output {
                let rhs: f32 = rhs.as_();
                Self(self.0 / rhs)
            }
        }
        
        multi_impl! (Add<$t> for (i32, u32, i64, u64, f32, f64) {
            type Output = $t;
        
            fn add(self, rhs: $t) -> Self::Output {
                rhs + self
            }
        });
        
        multi_impl! (Sub<$t> for (i32, u32, i64, u64, f32, f64) {
            type Output = $t;
        
            fn sub(self, rhs: $t) -> Self::Output {
                rhs + self
            }
        });
        
        multi_impl! (Mul<$t> for (i32, u32, i64, u64, f32, f64) {
            type Output = $t;
        
            fn mul(self, rhs: $t) -> Self::Output {
                rhs + self
            }
        });
    };
}

pub(in crate::engine::gl_types) use vector_arithmetic;
pub(in crate::engine::gl_types) use matrix_arithmetic;