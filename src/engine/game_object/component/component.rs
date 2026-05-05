use crate::engine::{Engine, errors::Result};
use downcast_rs::{Downcast, impl_downcast};

use crate::engine::game_object::ObjectID;

#[allow(unused)]
pub trait Component: Downcast {
    fn init(&mut self, engine: &mut Engine, owner: ObjectID) -> Result<()> {Ok(())}
    fn update(&mut self, engine: &mut Engine, owner: ObjectID, delta_time: f32) -> Result<()> {Ok(())}
    fn fixed_update(&mut self, engine: &mut Engine, owner: ObjectID, delta_time: f32) -> Result<()> {Ok(())}
    fn on_remove(&mut self, engine: &mut Engine, owner: ObjectID) -> Result<()> {Ok(())}
    
    /// Priority determines execution order. The return value of this function should not change.
    fn priority(&self) -> &'static i32 { &0 }
}

impl_downcast!(Component);

// pub trait CloneRequirement {
//     fn clone_box(&self) -> Box<dyn Component>;
// }

// impl<T> CloneRequirement for T
// where
//     T: Component + Clone
// {   
//     fn clone_box(&self) -> Box<dyn Component> {
//         Box::new(self.clone())
//     }
// }

// impl Clone for Box<dyn Component> {
//     fn clone(&self) -> Box<dyn Component> {
//         self.clone_box()
//     }
// }
