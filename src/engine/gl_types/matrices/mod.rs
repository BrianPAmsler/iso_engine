mod mat2;
mod mat3;
mod mat4;

pub use mat2::*;
pub use mat3::*;
pub use mat4::*;
use nalgebra::{ArrayStorage, Const, Matrix};

use crate::engine::gl_types::{inner_matrix::InnerMatrix, Make};

pub trait MatN<const N: usize>: InnerMatrix<N, N> + Make<Matrix<f32, Const<N>, Const<N>, ArrayStorage<f32, N, N>>> + AsRef<Self> {
    fn as_array(self) -> [[f32; N]; N];
    fn from_array(array: [[f32; N]; N]) -> Self;
    fn as_slice(&self) -> &[[f32; N]; N];
    fn as_slice_mut(&mut self) -> &mut [[f32; N]; N];
    fn from_slice(slice: &[[f32; N]; N]) -> Self;
}

impl<const N: usize, T: InnerMatrix<N, N> + Make<Matrix<f32, Const<N>, Const<N>, ArrayStorage<f32, N, N>>> + AsRef<T>> MatN<N> for T {
    fn as_array(self) -> [[f32; N]; N] {
        let mat = self.into_inner_matrix();

        mat.data.0
    }
    
    fn from_array(array: [[f32; N]; N]) -> Self {
        Self::make(Matrix::<f32, Const<N>, Const<N>, ArrayStorage<f32, N, N>>::from_data(
            ArrayStorage::<f32, N, N>(array)
        ))
    }
    
    fn as_slice(&self) -> &[[f32; N]; N] {
        &self.get_inner_matrix().data.0
    }
    
    fn as_slice_mut(&mut self) -> &mut [[f32; N]; N] {
        &mut self.get_inner_matrix_mut().data.0
    }
    
    fn from_slice(slice: &[[f32; N]; N]) -> Self {
        Self::make(Matrix::<f32, Const<N>, Const<N>, ArrayStorage<f32, N, N>>::from_data(
            ArrayStorage::<f32, N, N>(slice.to_owned())
        ))
    }

}