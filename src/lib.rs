extern crate self as opengl_engine;

pub mod engine;

pub mod error {
    pub use ::error::*;

    create_error!();

    #[derive(Error, Debug)]
    #[error("Uninitialized.")]
    pub struct Uninitialized;

    #[derive(Error, Debug)]
    #[error("Index {index:?} out of bounds ({bounds:?}).")]
    pub struct OutOfBounds<Idx: std::fmt::Debug> { pub index: Idx, pub bounds: std::ops::Range<Idx> }

    #[derive(Error, Debug)]
    #[error("Unwrap called on a None value.")]
    pub struct NoneValue;

    pub trait TryUnwrap<T>
    where
        Self: Sized
    {
        fn try_unwrap(self) -> Result<T, NoneValue>;
    }

    impl<T> TryUnwrap<T> for Option<T> {
        fn try_unwrap(self) -> Result<T, NoneValue> {
            Ok(self.ok_or(NoneValue)?)
        }
    }
}