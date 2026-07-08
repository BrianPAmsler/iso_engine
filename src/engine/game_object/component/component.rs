use crate::{engine::{Engine, resources::serialization::AsSerialize}, error::any::Result};
use downcast_rs::{Downcast, impl_downcast};

use crate::engine::game_object::ObjectID;

#[allow(unused, reason="Variables are to be used by implementors.")]
pub trait Component: Downcast + AsSerialize {
    fn init(&mut self, engine: &mut Engine, owner: ObjectID) -> Result<()> {Ok(())}
    fn update(&mut self, engine: &mut Engine, owner: ObjectID, delta_time: f32) -> Result<()> {Ok(())}
    fn fixed_update(&mut self, engine: &mut Engine, owner: ObjectID, delta_time: f32) -> Result<()> {Ok(())}
    fn on_remove(&mut self, engine: &mut Engine, owner: ObjectID) -> Result<()> {Ok(())}
    
    /// Priority determines execution order. The return value of this function should not change.
    fn priority(&self) -> &'static i32 { &0 }
}

impl_downcast!(Component);