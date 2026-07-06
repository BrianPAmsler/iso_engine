use crate::engine::gl_types::gl_types::inner_matrix::InnerMatrix;

pub trait ElementWise<const A: usize, const B: usize>: Copy + Clone {
    fn operate<F: FnMut(&mut f32)>(self, f: F) -> Self;
}

impl<const R: usize, const C: usize, T: InnerMatrix<R, C> + Copy + Clone> ElementWise<R, C> for T {
    fn operate<F: FnMut(&mut f32)>(mut self, f: F) -> T {
        self.get_inner_matrix_mut().iter_mut().for_each(f);

        self
    }
}

impl ElementWise<1, 1> for f32 {
    fn operate<F: FnMut(&mut f32)>(mut self, mut f: F) -> f32 {
        f(&mut self);

        self
    }
}