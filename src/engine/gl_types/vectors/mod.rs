mod vec2;
mod vec3;
mod vec4;

use nalgebra::{ArrayStorage, Const, Matrix};
pub use vec2::*;
pub use vec3::*;
pub use vec4::*;

use crate::engine::gl_types::{Make, gl_types::inner_matrix::InnerMatrix};

pub trait VecN<const N: usize>: InnerMatrix<N, 1> + Make<Matrix<f32, Const<N>, Const<1>, ArrayStorage<f32, N, 1>>> + AsRef<Self> {
    fn as_array(self) -> [f32; N];
    fn from_array(array: [f32; N]) -> Self;
    fn as_slice(&self) -> &[f32; N];
    fn as_slice_mut(&mut self) -> &mut [f32; N];
    fn from_slice(slice: &[f32; N]) -> Self;
}

pub mod swizzles {
    use swizz::generate_swizzles;

    use super::{Vec2, Vec3, Vec4};

    generate_swizzles!(Vec2, xy, 4);
    generate_swizzles!(Vec3, xyz, 4);
    generate_swizzles!(Vec4, xyzw, 4);

    generate_swizzles!(Vec2, rg, 4);
    generate_swizzles!(Vec3, rgb, 4);
    generate_swizzles!(Vec4, rgba, 4);
}

impl<const N: usize, T: InnerMatrix<N, 1> + Make<Matrix<f32, Const<N>, Const<1>, ArrayStorage<f32, N, 1>>> + AsRef<T>> VecN<N> for T {
    fn as_array(self) -> [f32; N] {
        let mat = self.into_inner_matrix();

        mat.data.0[0]
    }
    
    fn from_array(array: [f32; N]) -> Self {
        Self::make(Matrix::<f32, Const<N>, Const<1>, ArrayStorage<f32, N, 1>>::from_data(
            ArrayStorage::<f32, N, 1>([array])
        ))
    }
    
    fn as_slice(&self) -> &[f32; N] {
        &self.get_inner_matrix().data.0[0]
    }
    
    fn as_slice_mut(&mut self) -> &mut [f32; N] {
        &mut self.get_inner_matrix_mut().data.0[0]
    }
    
    fn from_slice(slice: &[f32; N]) -> Self {
        Self::make(Matrix::<f32, Const<N>, Const<1>, ArrayStorage<f32, N, 1>>::from_data(
            ArrayStorage::<f32, N, 1>([slice.to_owned()])
        ))
    }

}