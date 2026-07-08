use std::{collections::BTreeMap, marker::PhantomData, path::PathBuf, sync::Arc};
use image::DynamicImage;
use itertools::Itertools;
use vulkano::{command_buffer::allocator::StandardCommandBufferAllocator, device::{Device, Queue}, memory::allocator::StandardMemoryAllocator};

use crate::engine::{graphics::{Graphics, texture::{Texture, builder::TextureBuilder}}, resources::{MultiResourceLoader, ResourceLoader, resource_loaders::error::{TextureArrayLoadError, TextureLoadError}}};

pub struct TextureLoader<T>
where
    DynamicImage: Into<T> + From<T>
{
    build: Box<dyn FnOnce(TextureBuilder) -> TextureBuilder + Send + Sync>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    queue: Arc<Queue>,
    device: Arc<Device>,
    _pd: PhantomData<fn() -> T>
}

impl<T> TextureLoader<T>
where
    DynamicImage: Into<T> + From<T>
{
    pub fn new(gfx: &Graphics) -> TextureLoader<T> {
        Self::with_build(|builder| builder, gfx)
    }

    pub fn with_build<F: FnOnce(TextureBuilder) -> TextureBuilder + Send + Sync + 'static>(build: F, gfx: &Graphics) -> TextureLoader<T> {
        let build = Box::new(build);
        let memory_allocator = gfx.memory_allocator();
        let command_buffer_allocator = gfx.command_buffer_allocator();
        let queue = gfx.queue();
        let device = gfx.device();

        TextureLoader { build, memory_allocator, command_buffer_allocator, queue, device, _pd: PhantomData }
    }
}

impl<T> ResourceLoader<Texture, TextureLoadError> for TextureLoader<T>
where
    DynamicImage: Into<T> + From<T>
{
    fn load(self, data: Box<[u8]>) -> Result<Texture, TextureLoadError> {
        let image = image::load_from_memory(&data)?;
        let image = image.into();
        let image = image.into();

        let builder = TextureBuilder::new(image).map_err(crate::error::Error::into_inner)?;
        let builder = (self.build)(builder);

        Ok(builder.finish_no_gfx(self.memory_allocator, self.command_buffer_allocator, self.queue, self.device).map_err(crate::error::Error::into_inner)?)
    }
}

pub struct TextureArrayLoader<T>
where
    DynamicImage: Into<T> + From<T>
{
    build: Box<dyn FnOnce(TextureBuilder) -> TextureBuilder + Send + Sync>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    queue: Arc<Queue>,
    device: Arc<Device>,
    _pd: PhantomData<fn() -> T>
}

impl<T> TextureArrayLoader<T>
where
    DynamicImage: Into<T> + From<T>
{
    pub fn new(gfx: &Graphics) -> TextureArrayLoader<T> {
        Self::with_build(|builder| builder, gfx)
    }

    pub fn with_build<F: FnOnce(TextureBuilder) -> TextureBuilder + Send + Sync + 'static>(build: F, gfx: &Graphics) -> TextureArrayLoader<T> {
        let build = Box::new(build);
        let memory_allocator = gfx.memory_allocator();
        let command_buffer_allocator = gfx.command_buffer_allocator();
        let queue = gfx.queue();
        let device = gfx.device();

        TextureArrayLoader { build, memory_allocator, command_buffer_allocator, queue, device, _pd: PhantomData }
    }
}

impl<T> MultiResourceLoader<Texture, TextureArrayLoadError> for TextureArrayLoader<T>
where
    DynamicImage: Into<T> + From<T>
{
    fn load(self, data: BTreeMap<PathBuf, Box<[u8]>>) -> Result<Texture, TextureArrayLoadError> {
        let frames = data.into_values()
            .map(|frame| {
                let image = image::load_from_memory(&frame)?;
                let image = image.into();
                let image = image.into();

                Ok::<_, image::ImageError>(image)
            })
            .try_collect()?;
        let builder = TextureBuilder::new_array(frames).map_err(crate::error::Error::into_inner)?;
        let builder = (self.build)(builder);

        Ok(builder.finish_no_gfx(self.memory_allocator, self.command_buffer_allocator, self.queue, self.device).map_err(crate::error::Error::into_inner)?)
    }
}

pub mod error {
    use crate::{engine::graphics::texture::error::{NewTextureArrayError, TextureBuilderError, UnsupportedImageFormat}, error as errors_module};
    use error::union;

    union!(image::ImageError, TextureBuilderError, UnsupportedImageFormat as TextureLoadError);
    union!(image::ImageError, TextureBuilderError, NewTextureArrayError as TextureArrayLoadError);
}
