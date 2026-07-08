use vulkano::{Validated, VulkanError, buffer::AllocateBufferError, sync::HostAccessError};

use crate::{engine::graphics::{error::{GetBindingError, PipelineBuilderError, SetIndirectBufferError}, texture::error::{InvalidFrameDimensions, TextureBuilderError, UnsupportedImageFormat}}, error::{self as errors_module, Error, union}};

#[derive(Error, Debug)]
#[error("Invalid sprite sheet \"{sheet}\"")]
pub struct UnknownSpriteSheet { pub sheet: String }

union!(#[use_debug] Validated<VulkanError>, UnknownSpriteSheet, #[use_debug] Validated<AllocateBufferError>, PipelineBuilderError, GetBindingError, HostAccessError, TextureBuilderError, UnsupportedImageFormat as AddSpritesheetError);
union!(GetBindingError, HostAccessError as SpriteRendererBufferError);
union!(GetBindingError, HostAccessError, SetIndirectBufferError, SpriteRendererBufferError as SpriteRendererUpdateError);
union!(#[use_debug] Validated<VulkanError>, #[use_debug] Validated<AllocateBufferError>, PipelineBuilderError as NewAnimatedSpriteError);
union!(NewAnimatedSpriteError, InvalidFrameDimensions, TextureBuilderError as AddAnimatedSpriteError);
