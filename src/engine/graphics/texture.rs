use std::sync::Arc;

use image::{DynamicImage, GrayImage, Luma, Rgb, Rgb32FImage, RgbImage, Rgba, Rgba32FImage, RgbaImage};
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
        gfx.buffer_to_image(image_data, &self.image)
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

enum ImageContainer {
    /// Each pixel in this image is 8-bit Luma
    ImageLuma8(GrayImage),

    /// Each pixel in this image is 8-bit Rgb
    ImageRgb8(RgbImage),

    /// Each pixel in this image is 8-bit Rgb with alpha
    ImageRgba8(RgbaImage),

    /// Each pixel in this image is 16-bit Luma
    ImageLuma16(image::ImageBuffer<Luma<u16>, Vec<u16>>),

    /// Each pixel in this image is 16-bit Rgb
    ImageRgb16(image::ImageBuffer<Rgb<u16>, Vec<u16>>),

    /// Each pixel in this image is 16-bit Rgb with alpha
    ImageRgba16(image::ImageBuffer<Rgba<u16>, Vec<u16>>),

    /// Each pixel in this image is 32-bit float Rgb
    ImageRgb32F(Rgb32FImage),

    /// Each pixel in this image is 32-bit float Rgb with alpha
    ImageRgba32F(Rgba32FImage),
}

impl From<GrayImage> for ImageContainer {
    fn from(value: GrayImage) -> Self {
        Self::ImageLuma8(value)
    }
}

impl From<RgbImage> for ImageContainer {
    fn from(value: RgbImage) -> Self {
        Self::ImageRgb8(value)
    }    
}

impl From<RgbaImage> for ImageContainer {
    fn from(value: RgbaImage) -> Self {
        Self::ImageRgba8(value)
    }    
}

impl From<image::ImageBuffer<Luma<u16>, Vec<u16>>> for ImageContainer {
    fn from(value: image::ImageBuffer<Luma<u16>, Vec<u16>>) -> Self {
        Self::ImageLuma16(value)
    }    
}

impl From<image::ImageBuffer<Rgb<u16>, Vec<u16>>> for ImageContainer {
    fn from(value: image::ImageBuffer<Rgb<u16>, Vec<u16>>) -> Self {
        Self::ImageRgb16(value)
    }    
}

impl From<image::ImageBuffer<Rgba<u16>, Vec<u16>>> for ImageContainer {
    fn from(value: image::ImageBuffer<Rgba<u16>, Vec<u16>>) -> Self {
        Self::ImageRgba16(value)
    }    
}

impl From<Rgb32FImage> for ImageContainer {
    fn from(value: Rgb32FImage) -> Self {
        Self::ImageRgb32F(value)
    }    
}

impl From<Rgba32FImage> for ImageContainer {
    fn from(value: Rgba32FImage) -> Self {
        Self::ImageRgba32F(value)
    }    
}

impl TryFrom<DynamicImage> for ImageContainer {
    type Error = DynamicImage;
    fn try_from(value: DynamicImage) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            DynamicImage::ImageLuma8(image_buffer) => Self::ImageLuma8(image_buffer),
            DynamicImage::ImageRgb8(image_buffer) => Self::ImageRgb8(image_buffer),
            DynamicImage::ImageRgba8(image_buffer) => Self::ImageRgba8(image_buffer),
            DynamicImage::ImageLuma16(image_buffer) => Self::ImageLuma16(image_buffer),
            DynamicImage::ImageRgb16(image_buffer) => Self::ImageRgb16(image_buffer),
            DynamicImage::ImageRgba16(image_buffer) => Self::ImageRgba16(image_buffer),
            DynamicImage::ImageRgb32F(image_buffer) => Self::ImageRgb32F(image_buffer),
            DynamicImage::ImageRgba32F(image_buffer) => Self::ImageRgba32F(image_buffer),
            _ => {
                return Err(value);
            }, 
        })
    }
}

pub mod builder {
    use image::{DynamicImage, RgbaImage};
    use itertools::Itertools;
    use num::Zero;
    use paste::paste;
    use vulkano::{format::Format, image::{Image, ImageCreateInfo, ImageType, ImageUsage, sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode}, view::ImageView}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}};

    use crate::{engine::graphics::{Graphics, texture::{ImageContainer, Texture, error::{InvalidFrameDimensions, TextureBuilderError}}}, error::Result};

    fn pad_pixels<T: Copy + Zero>(pixels: Vec<T>, size: usize, align: usize, fill: T) -> Vec<T> {
        assert!(size > 0);
        assert!(align >= size);
        if size == align { return pixels }

        let len = (pixels.len() / size) * align;
        let padding = vec![fill; align - size];
        let mut out = vec![T::zero(); len];

        pixels.chunks(size).enumerate().for_each(|(i, chunk)| {
            let offset = i * align;

            let pixels = &mut out[offset..offset + size];
            pixels.copy_from_slice(chunk);

            let pad = &mut out[offset + size..offset + align];
            pad.copy_from_slice(&padding[..]);
        });

        out
    }

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
        wrap_s: SamplerAddressMode,
        wrap_t: SamplerAddressMode,
        min_filter: Filter,
        mag_filter: Filter,
        image_type: ImageType,
    }

    impl TextureBuilder {
        pub fn from_image(image: DynamicImage, srgb: bool) -> TextureBuilder {
            let image = match image {
                DynamicImage::ImageLuma8(_) => image,
                DynamicImage::ImageRgb8(_) => image,
                DynamicImage::ImageRgba8(_) => image,
                DynamicImage::ImageLuma16(_) => image,
                DynamicImage::ImageRgb16(_) => image,
                DynamicImage::ImageRgba16(_) => image,
                DynamicImage::ImageRgb32F(_) => image,
                DynamicImage::ImageRgba32F(_) => image,
                _ => image.into_rgba8().into(), 
            };

            let (width, height) = (image.width(), image.height());
            let (data, format) = match (image, srgb) {
                (DynamicImage::ImageLuma8(image), srgb) => (image.into_raw().into(), match srgb { true => Format::R8_SRGB, false => Format::R8_UNORM}),
                (DynamicImage::ImageRgb8(image), srgb) => (image.into_raw().into(), match srgb { true => Format::R8G8B8_SRGB, false => Format::R8G8B8_UNORM}),
                (DynamicImage::ImageRgba8(image), srgb) => (image.into_raw().into(), match srgb { true => Format::R8G8B8A8_SRGB, false => Format::R8G8B8A8_UNORM}),
                (DynamicImage::ImageLuma16(image), _) => (image.into_raw().into(), Format::R16_UNORM),
                (DynamicImage::ImageRgb16(image), _) => (image.into_raw().into(), Format::R16G16B16_UNORM),
                (DynamicImage::ImageRgba16(image), _) => (image.into_raw().into(), Format::R16G16B16A16_UNORM),
                (DynamicImage::ImageRgb32F(image), _) => (image.into_raw().into(), Format::R32G32B32_SFLOAT),
                (DynamicImage::ImageRgba32F(image), _) => (image.into_raw().into(), Format::R32G32B32A32_SFLOAT),
                _ => unreachable!(), 
            };

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

        pub fn from_raw_pixels(data: Vec<u8>, width: u32, height: u32, format: Format) -> TextureBuilder {
            let data = Data::U8(data);
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

            let data = Data::U8(data);

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

        pub fn finish(self, gfx: &Graphics) -> Result<Texture, TextureBuilderError> {
            let Self { data, width, height, depth, format, wrap_s, wrap_t, min_filter, mag_filter, image_type } = self;

            let array_layers = depth;
            let depth_dim = if image_type == ImageType::Dim3d {
                depth
            } else {
                1
            };

            let image = Image::new(
                gfx.memory_allocator(),
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
                Data::U8(data) => gfx.buffer_to_image(data, &image)?,
                Data::U16(data) => gfx.buffer_to_image(data, &image)?,
                Data::F32(data) => gfx.buffer_to_image(data, &image)?
            }
            

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

pub mod error {
    use error::{Error, union};
    use crate::{engine::graphics::error::BufferImageError, error as errors_module};
    use vulkano::{Validated, ValidationError, VulkanError, buffer::AllocateBufferError, command_buffer::CommandBufferExecError, image::AllocateImageError};

    #[derive(Error, Debug)]
    #[error("All frames must have the same dimensions.")]
    pub struct InvalidFrameDimensions;

    // #[allow(clippy::enum_variant_names, reason="Variant are generated from vulkan error names and should not be changed.")]
    union!(#[use_debug] Validated<AllocateImageError>, #[use_debug] Validated<AllocateBufferError>, #[use_debug] Box<ValidationError>, CommandBufferExecError, BufferImageError, #[use_debug] Validated<VulkanError> as TextureBuilderError);
}