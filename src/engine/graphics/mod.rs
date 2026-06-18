mod graphics;
mod camera;

pub mod sprite_renderer;
pub mod error;
pub mod terrain;
pub mod texture;

pub use graphics::*;
pub use camera::*;

#[cfg(test)]
pub mod test_lock {
    use std::sync::Mutex;

    pub static LOCK: Mutex<()> = Mutex::new(());
}