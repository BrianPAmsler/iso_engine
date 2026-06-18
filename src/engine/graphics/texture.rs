use std::sync::Arc;

use vulkano::image::{Image, sampler::Sampler, view::ImageView};

use crate::{engine::graphics::error::BufferImageError, error::Result};

use super::Graphics;

#[derive(Clone)]
pub struct Texture {
    image: Arc<Image>,
    view: Arc<ImageView>,
    sampler: Arc<Sampler>,
    width: u32,
    height: u32,
    depth: u32
}

impl Texture {
    pub fn update_texture(&self, gfx: &Graphics, image_data: Vec<u8>) -> Result<(), BufferImageError> {
        gfx.buffer_to_image(image_data, &self.image)
    }

    pub(in crate::engine::graphics) fn image(&self) -> &Arc<Image> {
        &self.image
    }

    pub(in crate::engine::graphics) fn view(&self) -> &Arc<ImageView> {
        &self.view
    }

    pub(in crate::engine::graphics) fn sampler(&self) -> &Arc<Sampler> {
        &self.sampler
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn depth(&self) -> u32 {
        self.depth
    }
}

pub mod builder {
    use image::RgbaImage;
    use itertools::Itertools;
    use vulkano::{format::Format, image::{Image, ImageCreateInfo, ImageType, ImageUsage, sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode}, view::ImageView}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}};

    use crate::{engine::graphics::{Graphics, texture::{Texture, error::{InvalidFrameDimensions, TextureBuilderError}}}, error::Result};

    pub struct TextureBuilder {
        data: Vec<u8>,
        width: u32,
        height: u32,
        depth: u32,
        format: Format,
        wrap_s: SamplerAddressMode,
        wrap_t: SamplerAddressMode,
        min_filter: Filter,
        mag_filter: Filter,
        image_type: ImageType
    }

    impl TextureBuilder {
        pub fn from_image(image: RgbaImage) -> TextureBuilder {
            let (width, height) = image.dimensions();
            let data = image.into_raw();
            TextureBuilder {
                data,
                width,
                height,
                depth: 1,
                format: Format::R8G8B8A8_SRGB,
                wrap_s: SamplerAddressMode::Repeat,
                wrap_t: SamplerAddressMode::Repeat,
                min_filter: Filter::Linear,
                mag_filter: Filter::Linear,
                image_type: ImageType::Dim2d
            }
        }

        pub fn from_raw_pixels(data: Vec<u8>, width: u32, height: u32, format: Format) -> TextureBuilder {
            TextureBuilder {
                data,
                width,
                height,
                depth: 1,
                format,
                wrap_s: SamplerAddressMode::Repeat,
                wrap_t: SamplerAddressMode::Repeat,
                min_filter: Filter::Linear,
                mag_filter: Filter::Linear,
                image_type: ImageType::Dim2d
            }
        }

        pub fn from_frames(frames: Vec<RgbaImage>) -> Result<TextureBuilder, InvalidFrameDimensions> {
            if frames.is_empty() {
                Err(InvalidFrameDimensions)?;
            }

            let (width, height) = frames[0].dimensions();

            for frame in &frames {
                if frame.dimensions() != (width, height) {
                    Err(InvalidFrameDimensions)?;
                }
            }

            let depth = frames.len() as u32;

            let data = frames.into_iter()
                .flat_map(|frame| frame.into_raw())
                .collect_vec();

            Ok(TextureBuilder {
                data,
                width,
                height,
                depth,
                format: Format::R8G8B8A8_SRGB,
                wrap_s: SamplerAddressMode::Repeat,
                wrap_t: SamplerAddressMode::Repeat,
                min_filter: Filter::Linear,
                mag_filter: Filter::Linear,
                image_type: ImageType::Dim3d
            })
        }

        pub fn wrap_s(mut self, wrap_s: SamplerAddressMode) -> Self {
            self.wrap_s = wrap_s;
            self
        }

        pub fn wrap_t(mut self, wrap_t: SamplerAddressMode) -> Self {
            self.wrap_t = wrap_t;
            self
        }

        pub fn min_filter(mut self, min_filter: Filter) -> Self {
            self.min_filter = min_filter;
            self
        }

        pub fn mag_filter(mut self, mag_filter: Filter) -> Self {
            self.mag_filter = mag_filter;
            self
        }

        pub fn finish(self, gfx: &Graphics) -> Result<Texture, TextureBuilderError> {
            let Self { data, width, height, depth, format, wrap_s, wrap_t, min_filter, mag_filter, image_type } = self;

            let image = Image::new(
                gfx.memory_allocator(),
                ImageCreateInfo {
                    image_type,
                    format,
                    extent: [width, height, depth],
                    usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                    ..Default::default()
                },
            )?;

            gfx.buffer_to_image(data, &image)?;

            let sampler = Sampler::new(
                gfx.device(),
                SamplerCreateInfo {
                    mag_filter,
                    min_filter,
                    mipmap_mode: SamplerMipmapMode::Nearest,
                    address_mode: [wrap_s, wrap_t, SamplerAddressMode::Repeat],
                    mip_lod_bias: 0.0,
                    ..Default::default()
                },
            )?;

            let view = ImageView::new_default(image.clone())?;

            Ok(Texture { image, view, sampler, width, height, depth  })
        }
    }
}

#[allow(clippy::enum_variant_names)]
pub mod error {
    use error_union::error_union;
    use thiserror::Error;
    use vulkano::command_buffer::CommandBufferExecError;

    use crate::{engine::graphics::{sprite_renderer::error::{AddAnimatedSpriteError, AddSpritesheetError}, terrain::{error::TerrainFromRawError, terrain_renderer::error::NewTerrainRendererError}}, error::EngineError};
    type ValidatedVulkanError = vulkano::Validated<vulkano::VulkanError>;
    type ValidatedAllocateBufferError = vulkano::Validated<vulkano::buffer::AllocateBufferError>;
    type BoxedValidationError = Box<vulkano::ValidationError>;
    type ValidatedAllocateImageError = vulkano::Validated<vulkano::image::AllocateImageError>;

    impl EngineError for Box<vulkano::ValidationError> {}
    impl EngineError for CommandBufferExecError {}
    impl EngineError for ValidatedAllocateImageError {}

    #[derive(Error, Debug)]
    #[error("InvalidFrameDimensions: Frames must all have same dimensions.")]
    pub struct InvalidFrameDimensions;
    impl EngineError for InvalidFrameDimensions {}

    error_union!(ValidatedAllocateImageError, ValidatedAllocateBufferError, BoxedValidationError, CommandBufferExecError, ValidatedVulkanError as TextureBuilderError into AddSpritesheetError, NewTerrainRendererError, TerrainFromRawError, AddAnimatedSpriteError);
}