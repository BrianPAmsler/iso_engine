use std::{marker::PhantomData, sync::Arc};

use image::{DynamicImage, ImageError};

use crate::engine::resources::ResourceLoader;

pub struct TextureLoader() where DynamicImage: Into<T>;

impl<T: Send + Sync> ResourceLoader<T, Arc<image::ImageError>> for TextureLoader<T>
where
    DynamicImage: Into<T>
{
    fn load(self, data: Box<[u8]>) -> Result<T, Arc<ImageError>> {
        Ok(image::load_from_memory(&data).map_err(Arc::new)?.into())
    }
}