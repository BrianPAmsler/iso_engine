use error_union::error_union;
use thiserror::Error;
use vulkano::{command_buffer::CommandBufferExecError, pipeline::layout::IntoPipelineLayoutCreateInfoError, sync::HostAccessError};


use crate::{engine::graphics::{error::{GetBindingError, InvalidEntryPoint, NoLayout, SetIndirectBufferError}, texture::error::InvalidFrameDimensions}, error::EngineError};

#[derive(Error, Debug)]
#[error("Invalid sprite sheet \"{sheet}\"")]
pub struct UnknownSpriteSheet { pub sheet: String }

impl EngineError for UnknownSpriteSheet {}
impl EngineError for HostAccessError {}

type ValidatedVulkanError = vulkano::Validated<vulkano::VulkanError>;
type ValidatedAllocateBufferError = vulkano::Validated<vulkano::buffer::AllocateBufferError>;
type BoxedValidationError = Box<vulkano::ValidationError>;
type ValidatedAllocateImageError = vulkano::Validated<vulkano::image::AllocateImageError>;

error_union!(
    ValidatedAllocateImageError,
    ValidatedAllocateBufferError,
    BoxedValidationError,
    CommandBufferExecError,
    ValidatedVulkanError,
    UnknownSpriteSheet,
    NoLayout,
    HostAccessError,
    InvalidEntryPoint,
    IntoPipelineLayoutCreateInfoError,
    GetBindingError,
    as AddSpritesheetError
);
error_union!(GetBindingError, HostAccessError as SpriteRendererBufferError into SpriteRendererUpdateError);
error_union!(GetBindingError, HostAccessError, SetIndirectBufferError as SpriteRendererUpdateError);
error_union!(ValidatedVulkanError, ValidatedAllocateBufferError, NoLayout, InvalidEntryPoint, BoxedValidationError, IntoPipelineLayoutCreateInfoError as NewAnimatedSpriteError into AddAnimatedSpriteError);

error_union!(InvalidFrameDimensions, ValidatedAllocateImageError, BoxedValidationError, CommandBufferExecError, ValidatedVulkanError, ValidatedAllocateBufferError, NoLayout, InvalidEntryPoint, IntoPipelineLayoutCreateInfoError as AddAnimatedSpriteError);