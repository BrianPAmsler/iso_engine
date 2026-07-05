use std::f32::consts::PI;

use crate::engine::gl_types::ElementWise;

pub fn radians<const R: usize, const C:  usize, T: ElementWise<R, C>>(degrees: T) -> T {
    const RATIO: f32 = PI / 180.0;

    degrees.operate(|el| *el *= RATIO)
}

pub fn degrees<const R: usize, const C:  usize, T: ElementWise<R, C>>(radians: T) -> T {
    const RATIO: f32 = 180.0 / PI;

    radians.operate(|el| *el *= RATIO)
}

pub fn sin<const R: usize, const C:  usize, T: ElementWise<R, C>>(angle: T) -> T {
    angle.operate(|el| *el = el.sin())
}

pub fn cos<const R: usize, const C:  usize, T: ElementWise<R, C>>(angle: T) -> T {
    angle.operate(|el| *el = el.cos())
}

