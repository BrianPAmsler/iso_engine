use error::{Error, union};
use vulkano::{LoadingError, Validated, ValidationError, VulkanError, buffer::AllocateBufferError, command_buffer::CommandBufferExecError, image::AllocateImageError, pipeline::layout::IntoPipelineLayoutCreateInfoError, swapchain::FromWindowError, sync::HostAccessError};
use winit::raw_window_handle::HandleError;
use crate::error as errors_module;

#[derive(Error, Debug)]
#[error("Vulkan Error: No physical devices.")]
pub struct NoPhysicalDevices;

#[derive(Error, Debug)]
#[error("Vulkan Error: No layout.")]
pub struct NoLayout;

#[derive(Error, Debug)]
#[error("Invalid shader entry point.")]
pub struct InvalidEntryPoint;

#[derive(Error, Debug)]
#[error("Device does not support sRGB.")]
pub struct SRGBUnsupported;

#[derive(Error, Debug)]
#[error("Invalid pipeline handle.")]
pub struct InvalidPipelineHandle;

#[derive(Error, Debug)]
#[error("Invalid binding.")]
pub struct InvalidBinding;

union!(Validated<VulkanError> as GetRenderPassError);

union!(LoadingError, HandleError, NoPhysicalDevices, SRGBUnsupported, Validated<VulkanError>, VulkanError, FromWindowError, GetFramebuffersError, GetRenderPassError as NewGraphicsError);

union!(NoLayout, Validated<VulkanError>, as DescriptorSetError);

union!(GetPipelineError, DescriptorSetError, Validated<AllocateBufferError> as PipelineBuilderError);

union!(Validated<AllocateImageError>, Validated<VulkanError> as GetFramebuffersError);
union!(Validated<VulkanError>, GetPipelineError, GetCommandBuffersError, GetFramebuffersError as UpdatePipelinesError);
union!(InvalidEntryPoint, Validated<VulkanError>, Box<ValidationError>, IntoPipelineLayoutCreateInfoError as GetPipelineError);
union!(Validated<VulkanError>: ValidatedVulkanError, Box<ValidationError> as GetCommandBuffersError);
union!(CommandBufferExecError, Validated<VulkanError> as DrawError);
union!(InvalidPipelineHandle, HostAccessError as SetIndirectBufferError);
union!(InvalidBinding, InvalidPipelineHandle as GetBindingError);
union!(Validated<AllocateBufferError>, Box<ValidationError>, Validated<VulkanError>, CommandBufferExecError as BufferImageError);
