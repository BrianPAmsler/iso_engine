use std::any::Any;

use crate::{engine::{Engine, game_object::component::component::seal::Sealed, resources::serialization::AsSerialize}, error::any::Result};

use crate::engine::game_object::ObjectID;

#[allow(unused, reason="Variables are to be used by implementors.")]
pub trait Component: AsSerialize + 'static {
    /// Priority determines execution order.
    const PRIORITY: i32 = 0;

    fn init(&mut self, engine: &mut Engine, owner: ObjectID) -> Result<()> {Ok(())}
    fn update(&mut self, engine: &mut Engine, owner: ObjectID, delta_time: f32) -> Result<()> {Ok(())}
    fn fixed_update(&mut self, engine: &mut Engine, owner: ObjectID, delta_time: f32) -> Result<()> {Ok(())}
    fn on_remove(&mut self, engine: &mut Engine, owner: ObjectID) -> Result<()> {Ok(())}
}

mod seal {
    use crate::engine::game_object::component::Component;

    pub trait Sealed {}

    impl<T: Component> Sealed for T {}
}

pub trait DynComponent: Any + AsSerialize + Sealed {
    fn init(&mut self, engine: &mut Engine, owner: ObjectID) -> Result<()>;
    fn update(&mut self, engine: &mut Engine, owner: ObjectID, delta_time: f32) -> Result<()>;
    fn fixed_update(&mut self, engine: &mut Engine, owner: ObjectID, delta_time: f32) -> Result<()>;
    fn on_remove(&mut self, engine: &mut Engine, owner: ObjectID) -> Result<()>;

    fn priority(&self) -> i32;
}

impl<T: Component + Any> DynComponent for T {
    fn init(&mut self, engine: &mut Engine, owner: ObjectID) -> Result<()> {
        <Self as Component>::init(self, engine, owner)
    }

    fn update(&mut self, engine: &mut Engine, owner: ObjectID, delta_time: f32) -> Result<()> {
        <Self as Component>::update(self, engine, owner, delta_time)
    }

    fn fixed_update(&mut self, engine: &mut Engine, owner: ObjectID, delta_time: f32) -> Result<()> {
        <Self as Component>::fixed_update(self, engine, owner, delta_time)
    }

    fn on_remove(&mut self, engine: &mut Engine, owner: ObjectID) -> Result<()> {
        <Self as Component>::on_remove(self, engine, owner)
    }

    fn priority(&self) -> i32 {
        Self::PRIORITY
    }
}
