use std::sync::Arc;

use vulkano::image::{Image, sampler::Sampler, view::ImageView};

use crate::{engine::graphics::error::BufferImageError, error::Result};

use super::Graphics;

#[derive(Clone, Debug)]
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
        println!("texture updated");
        gfx.buffer_to_image(image_data, self.image.clone())
    }

    pub fn image(&self) -> &Arc<Image> {
        &self.image
    }

    pub fn view(&self) -> &Arc<ImageView> {
        &self.view
    }

    pub fn sampler(&self) -> &Arc<Sampler> {
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
    use std::sync::Arc;

    use image::{DynamicImage};
    use itertools::Itertools;
    use paste::paste;
    use vulkano::{command_buffer::allocator::StandardCommandBufferAllocator, device::{Device, Queue}, format::Format, image::{Image, ImageCreateInfo, ImageType, ImageUsage, sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode}, view::ImageView}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}};

    use crate::{engine::graphics::{self, Graphics, texture::{Texture, error::{InvalidFormat, InvalidFrameDimensions, NewTextureArrayError, TextureBuilderError, UnsupportedImageFormat}}}, error::Result};

    macro_rules! data_enum {
        ($($type:ident),*) => {
            paste! {
                enum Data {
                    $([<$type:upper>](Vec<$type>)),*
                }

                $(
                    impl From<Vec<$type>> for Data {
                        fn from(value: Vec<$type>) -> Self {
                            Self::[<$type:upper>](value)
                        }
                    }
                )*
            }
        };
    }

    data_enum!(u8, u16, f32);

    pub struct TextureBuilder {
        data: Data,
        width: u32,
        height: u32,
        depth: u32,
        format: Format,
        srgb: bool,
        wrap_s: SamplerAddressMode,
        wrap_t: SamplerAddressMode,
        min_filter: Filter,
        mag_filter: Filter,
        image_type: ImageType,
    }

    fn into_data(image: DynamicImage) -> std::result::Result<(Data, Format), UnsupportedImageFormat> {
        let image = match image {
            DynamicImage::ImageLuma8(_) => image,
            DynamicImage::ImageRgb8(_) => image,
            DynamicImage::ImageRgba8(_) => image,
            DynamicImage::ImageLuma16(_) => image,
            DynamicImage::ImageRgb16(_) => image,
            DynamicImage::ImageRgba16(_) => image,
            DynamicImage::ImageRgb32F(_) => image,
            DynamicImage::ImageRgba32F(_) => image,
            DynamicImage::ImageLumaA8(_) => return Err(UnsupportedImageFormat("ImageLumaA8")),
            DynamicImage::ImageLumaA16(_) => return Err(UnsupportedImageFormat("ImageLumaA16")),
            _ => return Err(UnsupportedImageFormat("Unknown")),
        };

        Ok(match image {
            DynamicImage::ImageLuma8(image) => (image.into_raw().into(), Format::R8_UNORM),
            DynamicImage::ImageRgb8(image) => (image.into_raw().into(), Format::R8G8B8_UNORM),
            DynamicImage::ImageRgba8(image) => (image.into_raw().into(), Format::R8G8B8A8_UNORM),
            DynamicImage::ImageLuma16(image) => (image.into_raw().into(), Format::R16_UNORM),
            DynamicImage::ImageRgb16(image) => (image.into_raw().into(), Format::R16G16B16_UNORM),
            DynamicImage::ImageRgba16(image) => (image.into_raw().into(), Format::R16G16B16A16_UNORM),
            DynamicImage::ImageRgb32F(image) => (image.into_raw().into(), Format::R32G32B32_SFLOAT),
            DynamicImage::ImageRgba32F(image) => (image.into_raw().into(), Format::R32G32B32A32_SFLOAT),
            _ => unreachable!(), 
        })
    }

    impl TextureBuilder {
        pub fn new(image: DynamicImage) -> Result<TextureBuilder, UnsupportedImageFormat> {
            let (width, height) = (image.width(), image.height());

            let (data, format) = into_data(image)?;

            Ok(TextureBuilder {
                data,
                width,
                height,
                depth: 1,
                format,
                srgb: true,
                wrap_s: SamplerAddressMode::Repeat,
                wrap_t: SamplerAddressMode::Repeat,
                min_filter: Filter::Linear,
                mag_filter: Filter::Linear,
                image_type: ImageType::Dim2d
            })
        }

        pub fn from_raw_pixels(data: Vec<u8>, width: u32, height: u32, format: Format) -> TextureBuilder {
            let data = Data::U8(data);
            TextureBuilder {
                data,
                width,
                height,
                depth: 1,
                format,
                srgb: true,
                wrap_s: SamplerAddressMode::Repeat,
                wrap_t: SamplerAddressMode::Repeat,
                min_filter: Filter::Linear,
                mag_filter: Filter::Linear,
                image_type: ImageType::Dim2d
            }
        }

        pub fn new_array(frames: Vec<DynamicImage>) -> Result<TextureBuilder, NewTextureArrayError> {
            if frames.is_empty() {
                Err(InvalidFrameDimensions)?;
            }

            let width = frames[0].width();
            let height = frames[0].height();
            let depth = frames.len() as u32;

            let frames = frames.into_iter()
                .map(into_data)
                .try_collect::<_, Vec<_>, _>()?;

            enum DataType {
                U8,
                U16,
                F32
            }

            let format = frames[0].1;
            let data_type = match &frames[0].0 {
                Data::U8(_) => DataType::U8,
                Data::U16(_) => DataType::U16,
                Data::F32(_) => DataType::F32,
            };

            let data = match data_type {
                DataType::U8 => {
                    Data::U8(
                        frames.into_iter()
                        .map(|(data, format)| {
                            match data {
                                Data::U8(data) => (data, format),
                                _ => unreachable!()
                            }
                        })
                        .map(|(data, data_format)| if data_format == format { Ok(data) } else { Err(InvalidFormat) })
                        .try_fold(Vec::new(), |mut prev, next| {
                            prev.extend(next?.into_iter());
                            Ok::<_, InvalidFormat>(prev)
                        })?
                    )
                },
                DataType::U16 => {
                    Data::U16(
                        frames.into_iter()
                        .map(|(data, format)| {
                            match data {
                                Data::U16(data) => (data, format),
                                _ => unreachable!()
                            }
                        })
                        .map(|(data, data_format)| if data_format == format { Ok(data) } else { Err(InvalidFormat) })
                        .try_fold(Vec::new(), |mut prev, next| {
                            prev.extend(next?.into_iter());
                            Ok::<_, InvalidFormat>(prev)
                        })?
                    )
                },
                DataType::F32 => {
                    Data::F32(
                        frames.into_iter()
                        .map(|(data, format)| {
                            match data {
                                Data::F32(data) => (data, format),
                                _ => unreachable!()
                            }
                        })
                        .map(|(data, data_format)| if data_format == format { Ok(data) } else { Err(InvalidFormat) })
                        .try_fold(Vec::new(), |mut prev, next| {
                            prev.extend(next?.into_iter());
                            Ok::<_, InvalidFormat>(prev)
                        })?
                    )
                },
            };

            Ok(TextureBuilder {
                data,
                width,
                height,
                depth,
                format,
                srgb: true,
                wrap_s: SamplerAddressMode::Repeat,
                wrap_t: SamplerAddressMode::Repeat,
                min_filter: Filter::Linear,
                mag_filter: Filter::Linear,
                image_type: ImageType::Dim2d
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

        pub fn finish_no_gfx(self, memory_allocator: Arc<StandardMemoryAllocator>, command_buffer_allocator: Arc<StandardCommandBufferAllocator>, queue: Arc<Queue>, device: Arc<Device>) -> Result<Texture, TextureBuilderError> {
            let Self { data, width, height, depth, format, srgb, wrap_s, wrap_t, min_filter, mag_filter, image_type } = self;

            let array_layers = depth;
            let depth_dim = if image_type == ImageType::Dim3d {
                depth
            } else {
                1
            };

            let format = if srgb {
                match format {
                    Format::R8_UNORM => Format::R8_SRGB,
                    Format::R8G8B8_UNORM => Format::R8G8B8_SRGB,
                    Format::R8G8B8A8_UNORM => Format::R8G8B8A8_SRGB,
                    _ => format
                }
            } else {
                format
            };

            let image = Image::new(
                memory_allocator.clone(),
                ImageCreateInfo {
                    image_type,
                    format,
                    extent: [width, height, depth_dim],
                    array_layers,
                    usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                    ..Default::default()
                },
            )?;

            match data {
                Data::U8(data) => graphics::buffer_to_image(memory_allocator, command_buffer_allocator, queue, device.clone(), data, image.clone())?,
                Data::U16(data) => graphics::buffer_to_image(memory_allocator, command_buffer_allocator, queue, device.clone(), data, image.clone())?,
                Data::F32(data) => graphics::buffer_to_image(memory_allocator, command_buffer_allocator, queue, device.clone(), data, image.clone())?
            }
            

            let sampler = Sampler::new(
                device,
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

        pub fn finish(self, gfx: &Graphics) -> Result<Texture, TextureBuilderError> {
            self.finish_no_gfx(gfx.memory_allocator(), gfx.command_buffer_allocator(), gfx.queue(), gfx.device())
        }
    }
}

pub mod error {
    use error::{Error, union};
    use crate::{engine::graphics::error::BufferImageError, error as errors_module};
    use vulkano::{Validated, ValidationError, VulkanError, buffer::AllocateBufferError, command_buffer::CommandBufferExecError, image::AllocateImageError};

    #[derive(Error, Debug)]
    #[error("All frames must have the same dimensions.")]
    pub struct InvalidFrameDimensions;

    #[derive(Error, Debug)]
    #[error("All frames must have the same format.")]
    pub struct InvalidFormat;

    #[derive(Error, Debug)]
    #[error("Unsupported image format: {0}")]
    pub struct UnsupportedImageFormat(pub &'static str);

    // #[allow(clippy::enum_variant_names, reason="Variant are generated from vulkan error names and should not be changed.")]
    union!(#[use_debug] Validated<AllocateImageError>, #[use_debug] Validated<AllocateBufferError>, #[use_debug] Box<ValidationError>, CommandBufferExecError, BufferImageError, #[use_debug] Validated<VulkanError> as TextureBuilderError);

    union!(InvalidFrameDimensions, UnsupportedImageFormat, InvalidFormat as NewTextureArrayError);
}