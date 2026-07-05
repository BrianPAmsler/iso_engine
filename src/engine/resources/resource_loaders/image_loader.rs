use std::sync::Arc;

use image::{ImageError, RgbaImage};

use crate::engine::resources::ResourceLoader;

pub struct ImageLoader;

impl ResourceLoader<RgbaImage, Arc<image::ImageError>> for ImageLoader {
    fn load(self, data: Box<[u8]>) -> Result<RgbaImage, Arc<ImageError>> {
        Ok(image::load_from_memory(&data).map_err(Arc::new)?.into_rgba8())
    }
}