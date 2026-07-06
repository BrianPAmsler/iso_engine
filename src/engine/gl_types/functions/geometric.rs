use crate::engine::gl_types::vectors::VecN;

pub fn length<const N: usize, V: VecN<N>, R: AsRef<V>>(x: R) -> f32 {
    let mut sum = 0.0;
    let mat = x.as_ref().get_inner_matrix();
    mat.iter().for_each(|el| sum += el * el);

    sum.sqrt()
}

pub fn distance<const N: usize, V: VecN<N>, R: AsRef<V>>(x: R, y: R) -> f32 {
    let a = x.as_ref().get_inner_matrix();
    let b = y.as_ref().get_inner_matrix();

    let delta = b - a;
    let delta = V::make(delta);
    
    length(&delta)
}

pub fn dot<const N: usize, V: VecN<N>, R: AsRef<V>>(x: R, y: R) -> f32 {
    let a = x.as_ref().get_inner_matrix();
    let b = y.as_ref().get_inner_matrix();

    a.dot(b)
}

pub fn cross<const N: usize, V: VecN<N>, R: AsRef<V>>(x: R, y: R) -> V {
    let a = x.as_ref().get_inner_matrix();
    let b = y.as_ref().get_inner_matrix();

    V::make(a.cross(b))
}

pub fn normalize<const N: usize, V: VecN<N>, R: AsRef<V>>(x: R) -> V {
    let v = x.as_ref().get_inner_matrix();
    V::make(v.normalize())
}

pub fn face_forward<const N: usize, V: VecN<N>, R: AsRef<V>>(n: R, i: R) -> V {
    let n = n.as_ref().get_inner_matrix();
    let i = i.as_ref().get_inner_matrix();
    
    let dot = n.dot(i);
    if dot < 0.0 {
        V::make(*n)
    } else {
        V::make(-n)
    }
}

pub fn reflect<const N: usize, V: VecN<N>, R: AsRef<V>>(i: R, n: R) -> V {
    let i = i.as_ref().get_inner_matrix();
    let n = n.as_ref().get_inner_matrix();

    V::make(i - 2.0 * n.dot(i) * n)
}

pub fn refract<const N: usize, V: VecN<N>, R: AsRef<V>>(i: R, n: R, eta: f32) -> V {
    let i = i.as_ref().get_inner_matrix();
    let n = n.as_ref().get_inner_matrix();

    let n_dot_i = n.dot(i);
    let k = 1.0 - eta * eta * (1.0 - n_dot_i * n_dot_i);

    if k < 0.0 {
        V::from_array([0.0; N])
    } else {
        V::make(eta * i - (eta * n_dot_i + k.sqrt()) * n)
    }
}