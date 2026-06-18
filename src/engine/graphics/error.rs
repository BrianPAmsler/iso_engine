use error_union::error_union;
use thiserror::Error;
use vulkano::{command_buffer::CommandBufferExecError, pipeline::layout::IntoPipelineLayoutCreateInfoError, sync::HostAccessError};

use crate::{engine::{error::NewEngineErorr, graphics::{sprite_renderer::error::{AddSpritesheetError, NewAnimatedSpriteError}, terrain::{error::TerrainFromRawError, terrain_renderer::error::TerrainRendererUpdateError}, texture::error::TextureBuilderError}}, error::EngineError};

type ValidatedVulkanError = vulkano::Validated<vulkano::VulkanError>;
type ValidatedAllocateBufferError = vulkano::Validated<vulkano::buffer::AllocateBufferError>;
type ValidatedAllocateImageError = vulkano::Validated<vulkano::image::AllocateImageError>;
type BoxedValidationError = Box<vulkano::ValidationError>;

#[derive(Error, Debug)]
#[error("Vulkan Error: No physical devices.")]
pub struct NoPhysicalDevices;
impl EngineError for NoPhysicalDevices {}

#[derive(Error, Debug)]
#[error("Vulkan Error: No layout.")]
pub struct NoLayout;
impl EngineError for NoLayout {}

#[derive(Error, Debug)]
#[error("Invalid shader entry point.")]
pub struct InvalidEntryPoint;
impl EngineError for InvalidEntryPoint {}

#[derive(Error, Debug)]
#[error("Device does not support sRGB.")]
pub struct SRGBUnsupported;
impl EngineError for SRGBUnsupported {}

#[derive(Error, Debug)]
#[error("Invalid pipeline handle.")]
pub struct InvalidPipelineHandle;
impl EngineError for InvalidPipelineHandle {}

#[derive(Error, Debug)]
#[error("Invalid binding.")]
pub struct InvalidBinding;
impl EngineError for InvalidBinding {}

impl EngineError for vulkano::LoadingError {}
impl EngineError for winit::raw_window_handle::HandleError {}
impl EngineError for vulkano::Validated<vulkano::VulkanError> {}
impl EngineError for vulkano::VulkanError {}
impl EngineError for vulkano::swapchain::FromWindowError {}
impl EngineError for vulkano::Validated<vulkano::buffer::AllocateBufferError> {}
impl EngineError for IntoPipelineLayoutCreateInfoError {}

error_union!(ValidatedAllocateImageError, vulkano::LoadingError, winit::raw_window_handle::HandleError, ValidatedVulkanError, vulkano::VulkanError, vulkano::swapchain::FromWindowError, NoPhysicalDevices, SRGBUnsupported as NewGraphicsError into NewEngineErorr);
error_union!(ValidatedVulkanError, NoLayout, InvalidEntryPoint, BoxedValidationError as DescriptorSetError into PipelineBuilderError);
error_union!(ValidatedAllocateBufferError, ValidatedVulkanError, NoLayout, InvalidEntryPoint, BoxedValidationError, IntoPipelineLayoutCreateInfoError as PipelineBuilderError into AddSpritesheetError, TerrainFromRawError, NewAnimatedSpriteError);
error_union!(ValidatedAllocateImageError, ValidatedVulkanError, as GetFramebuffersError into NewGraphicsError, UpdatePipelinesError);
error_union!(ValidatedAllocateImageError, ValidatedVulkanError, InvalidEntryPoint, BoxedValidationError, IntoPipelineLayoutCreateInfoError as UpdatePipelinesError);
error_union!(ValidatedVulkanError, InvalidEntryPoint, BoxedValidationError, IntoPipelineLayoutCreateInfoError as GetPipelineError into PipelineBuilderError, UpdatePipelinesError);
error_union!(ValidatedVulkanError, BoxedValidationError as GetCommandBuffersError into UpdatePipelinesError);
error_union!(ValidatedVulkanError, CommandBufferExecError as DrawError);
error_union!(InvalidPipelineHandle, HostAccessError as SetIndirectBufferError into TerrainRendererUpdateError);
error_union!(InvalidPipelineHandle, InvalidBinding as GetBindingError into TerrainRendererUpdateError);
error_union!(ValidatedVulkanError, ValidatedAllocateBufferError, BoxedValidationError, CommandBufferExecError as BufferImageError into TextureBuilderError);