use core::f32;

use crate::engine::gl_types::{matrices::MatN, vectors::VecN};

pub fn matrix_comp_mult<const N: usize, M: MatN<N>, R: AsRef<M>>(x: R, y: R) -> M {
    let a = x.as_ref().get_inner_matrix();
    let b = y.as_ref().get_inner_matrix();

    M::make(a.component_mul(&b))
}

pub fn outer_product<const N: usize, V: VecN<N>, M: MatN<N>, R: AsRef<V>>(x: R, y: R) -> M {
    let a = x.as_ref().get_inner_matrix();
    let b = y.as_ref().get_inner_matrix();

    let c = a * b.transpose();
    M::make(c)
}

pub fn transpose<const N: usize, M: MatN<N>, R: AsRef<M>>(mat: R) -> M {
    let m = mat.as_ref().get_inner_matrix();
    M::make(m.transpose())
}

pub fn determinant<const N: usize, M: MatN<N>, R: AsRef<M>>(mat: R) -> f32
    where
        nalgebra::Const<N>: nalgebra::DimMin<nalgebra::Const<N>, Output = nalgebra::Const<N>> {
    mat.as_ref().get_inner_matrix().determinant()
} 

pub fn inverse<const N: usize, M: MatN<N>, R: AsRef<M>>(mat: R) -> M {
    let mat = mat.as_ref().get_inner_matrix();

    match mat.try_inverse() {
        Some(m) => M::make(m),
        None => M::from_array([[f32::NAN; N]; N]),
    }
}