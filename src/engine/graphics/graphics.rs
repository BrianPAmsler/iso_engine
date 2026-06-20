use std::{collections::HashMap, iter, sync::Arc};

use itertools::Itertools;
use vulkano::{Validated, VulkanError, VulkanLibrary, buffer::{Buffer, BufferContents, BufferCreateInfo, Subbuffer}, command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferToImageInfo, DrawIndexedIndirectCommand, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo, allocator::StandardCommandBufferAllocator}, descriptor_set::{DescriptorSet, WriteDescriptorSet, allocator::StandardDescriptorSetAllocator}, device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags, physical::PhysicalDeviceType}, format::{ClearValue, Format}, image::{Image, ImageCreateInfo, ImageType, ImageUsage, view::ImageView}, instance::{Instance, InstanceCreateFlags, InstanceCreateInfo}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}, pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo, graphics::{GraphicsPipelineCreateInfo, color_blend::{ColorBlendAttachmentState, ColorBlendState}, depth_stencil::{DepthState, DepthStencilState}, input_assembly::InputAssemblyState, multisample::MultisampleState, rasterization::{CullMode, FrontFace, RasterizationState}, vertex_input::{VertexBufferDescription, VertexDefinition as _}, viewport::{Viewport, ViewportState}}, layout::PipelineDescriptorSetLayoutCreateInfo}, render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass}, shader::ShaderModule, swapchain::{self, PresentMode, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo}, sync::{self, GpuFuture}};
use winit::{event_loop::EventLoop, window::Window};
use crate::{engine::{data_structures::{AllocationIndex, VecAllocator}, graphics::{texture::Texture, error::{BufferImageError, DescriptorSetError, DrawError, GetBindingError, GetCommandBuffersError, GetFramebuffersError, GetPipelineError, GetRenderPassError, InvalidBinding, InvalidEntryPoint, InvalidPipelineHandle, NewGraphicsError, NoLayout, NoPhysicalDevices, SRGBUnsupported, SetIndirectBufferError, UpdatePipelinesError}}}, error::{ExplicitUnwrap, Result}};

unsafe fn exit<T> (status: i32) -> T {
    std::process::exit(status)
}

#[repr(C, align(16))]
#[derive(Debug, Default, BufferContents, Clone, Copy, PartialEq)]
pub struct AlignedVec2(pub [u32; 2]);

#[repr(C, align(16))]
#[derive(Debug, Default, BufferContents, Clone, Copy, PartialEq)]
pub struct AlignedVec3(pub [f32; 3]);

#[derive(Clone, Copy)]
pub struct PipelineHandle {
    handle: AllocationIndex 
}

struct PipelineCell {
    pipeline: Arc<GraphicsPipeline>,
    bindings: HashMap<u32, Binding>,
    vertex_buffer_description: VertexBufferDescription,
    vertex_buffer: Arc<Buffer>,
    index_buffer: Arc<Buffer>,
    indirect_buffer: Subbuffer<[DrawIndexedIndirectCommand]>,
    descriptor_set: Arc<DescriptorSet>,
    vertex_shader: Arc<ShaderModule>,
    intermediate_shaders: Vec<Arc<ShaderModule>>,
    fragment_shader: Arc<ShaderModule>
}

pub mod pipeline_builder {
    pub mod stage_1 {
        use std::sync::Arc;

        use vulkano::shader::ShaderModule;

        use crate::engine::graphics::Graphics;

        pub struct PipelineBuilder<'a> {
            pub(in crate::engine::graphics::graphics) gfx: &'a mut Graphics,
        }

        impl PipelineBuilder<'_> {
            pub fn new(gfx: &mut Graphics) -> PipelineBuilder<'_> {
                PipelineBuilder { gfx }
            }
        }

        impl<'a> PipelineBuilder<'a> {
            pub fn vertex_shader(self, shader_module: Arc<ShaderModule>) -> super::stage_2::PipelineBuilder<'a> {
                let Self { gfx } = self;
                let vertex_shader = shader_module;
                super::stage_2::PipelineBuilder {
                    gfx,
                    vertex_shader,
                    shaders: Vec::new()
                }
            }
        }
    }

    mod stage_2 {
        use std::sync::Arc;

        use vulkano::shader::ShaderModule;

        use crate::engine::graphics::Graphics;

        pub struct PipelineBuilder<'a> {
            pub(in crate::engine::graphics::graphics) gfx: &'a mut Graphics,
            pub(in crate::engine::graphics::graphics) vertex_shader: Arc<ShaderModule>,
            pub(in crate::engine::graphics::graphics) shaders: Vec<Arc<ShaderModule>>
        }

        impl<'a> PipelineBuilder<'a> {
            pub fn add_shader(mut self, shader_module: Arc<ShaderModule>) -> super::stage_2::PipelineBuilder<'a> {
                self.shaders.push(shader_module);

                self
            }

            pub fn fragment_shader(self, shader_module: Arc<ShaderModule>) -> super::stage_3::PipelineBuilder<'a> {
                let Self { gfx, vertex_shader, shaders } = self;
                let fragment_shader = shader_module;
                super::stage_3::PipelineBuilder {
                    gfx,
                    vertex_shader,
                    intermediate_shaders: shaders,
                    fragment_shader,
                }
            }
        }
    }

    mod stage_3 {
        use std::{collections::HashMap, sync::Arc};

        use vulkano::{Validated, buffer::{AllocateBufferError, Buffer, BufferContents, BufferCreateInfo, BufferUsage}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}, pipeline::graphics::vertex_input::Vertex, shader::ShaderModule};

        use crate::{engine::graphics::Graphics, error::Result};
        
        pub struct PipelineBuilder<'a> {
            pub(in crate::engine::graphics::graphics) gfx: &'a mut Graphics,
            pub(in crate::engine::graphics::graphics) vertex_shader: Arc<ShaderModule>,
            pub(in crate::engine::graphics::graphics) intermediate_shaders: Vec<Arc<ShaderModule>>,
            pub(in crate::engine::graphics::graphics) fragment_shader: Arc<ShaderModule>,
        }

        impl<'a> PipelineBuilder<'a> {
            pub fn vertex_data<T: Vertex + BufferContents>(self, vertices: Vec<T>, indices: Vec<u32>) -> Result<super::stage_4::PipelineBuilder<'a>, Validated<AllocateBufferError>> {
                let Self { gfx, vertex_shader, intermediate_shaders, fragment_shader } = self;

                let vertex_buffer = Buffer::from_iter(
                    gfx.memory_allocator.clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::VERTEX_BUFFER,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                            | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                        ..Default::default()
                    },
                    vertices,
                )?
                .buffer()
                .clone();

                let index_buffer = Buffer::from_iter(
                    gfx.memory_allocator.clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::INDEX_BUFFER,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                            | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                        ..Default::default()
                    },
                    indices,
                )?
                .buffer()
                .clone();

                let vertex_buffer_description = T::per_vertex();

                Ok(super::stage_4::PipelineBuilder {
                    gfx,
                    vertex_buffer,
                    index_buffer,
                    vertex_shader,
                    intermediate_shaders,
                    fragment_shader,
                    vertex_buffer_description,
                    bindings: HashMap::new(),
                })
            }
        }
    }

    mod stage_4 {
        use std::{collections::HashMap, sync::Arc};

        use vulkano::{Validated, buffer::{AllocateBufferError, Buffer, BufferContents, BufferCreateInfo, BufferUsage}, command_buffer::DrawIndexedIndirectCommand, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}, pipeline::graphics::vertex_input::VertexBufferDescription, shader::ShaderModule};

        use crate::{engine::graphics::{Binding, BufferType, Graphics, PipelineHandle, error::PipelineBuilderError, graphics::{PipelineCell, get_descriptor_set, get_pipeline}, texture::Texture}, error::Result};

        pub struct PipelineBuilder<'a> {
            pub(in crate::engine::graphics) gfx: &'a mut Graphics,
            pub(in crate::engine::graphics) vertex_buffer: Arc<Buffer>,
            pub(in crate::engine::graphics) index_buffer: Arc<Buffer>,
            pub(in crate::engine::graphics) vertex_shader: Arc<ShaderModule>,
            pub(in crate::engine::graphics) intermediate_shaders: Vec<Arc<ShaderModule>>,
            pub(in crate::engine::graphics) fragment_shader: Arc<ShaderModule>,
            pub(in crate::engine::graphics) vertex_buffer_description: VertexBufferDescription,
            pub(in crate::engine::graphics) bindings: HashMap<u32, Binding>,
        }

        impl<'a> PipelineBuilder<'a> {
            pub fn add_uniform_buffer<T: BufferContents>(mut self, binding: u32, buffer: T, buffer_type: BufferType) -> Result<Self, Validated<AllocateBufferError>> {
                let buffer = Buffer::from_data(
                    self.gfx.memory_allocator.clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::UNIFORM_BUFFER,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: buffer_type.into(),
                        ..Default::default()
                    },
                    buffer,
                )?
                .buffer()
                .clone();

                self.bindings.insert(binding, Binding::Buffer(buffer));

                Ok(self)
            }

            pub fn add_existing_buffer(mut self, binding: u32, buffer: Arc<Buffer>) -> Self {
                self.bindings.insert(binding, Binding::Buffer(buffer));

                self
            }

            pub fn add_existing_texture(mut self, binding: u32, texture: Texture) -> Self {
                self.bindings.insert(binding, Binding::Texture(texture));
                
                self
            }

            pub fn add_storage_buffer<T: BufferContents>(mut self, binding: u32, buffer: T, buffer_type: BufferType) -> Result<Self, Validated<AllocateBufferError>> {
                let buffer = Buffer::from_data(
                    self.gfx.memory_allocator.clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::STORAGE_BUFFER,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: buffer_type.into(),
                        ..Default::default()
                    },
                    buffer,
                )?
                .buffer()
                .clone();

                self.bindings.insert(binding, Binding::Buffer(buffer));

                Ok(self)
            }

            pub fn add_storage_buffer_unsized<T: BufferContents + ?Sized>(mut self, binding: u32, count: u64, buffer_type: BufferType) -> Result<Self, Validated<AllocateBufferError>> {
                let buffer = Buffer::new_unsized::<T>(
                    self.gfx.memory_allocator.clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::STORAGE_BUFFER,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: buffer_type.into(),
                        ..Default::default()
                    },
                    count,
                )?
                .buffer()
                .clone();

                self.bindings.insert(binding, Binding::Buffer(buffer));

                Ok(self)
            }

            pub fn add_texture(mut self, binding: u32, texture: Texture) -> Self {
                self.bindings.insert(binding, Binding::Texture(texture));

                self
            }

            pub fn finish(self) -> Result<PipelineHandle, PipelineBuilderError> {
                let Self { gfx, vertex_buffer, index_buffer, vertex_shader, intermediate_shaders, fragment_shader, vertex_buffer_description, bindings } = self;

                let pipeline = get_pipeline(
                    &gfx.device,
                    vertex_buffer_description.clone(),
                    &vertex_shader,
                    &intermediate_shaders,
                    &fragment_shader,
                    &gfx.render_pass,
                    gfx.viewport.clone()
                )?;

                let descriptor_set = get_descriptor_set(
                    &gfx.descriptor_set_allocator,
                    &pipeline,
                    &bindings
                )?;

                let indirect_buffer = Buffer::from_iter(
                    gfx.memory_allocator.clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::INDIRECT_BUFFER | BufferUsage::TRANSFER_DST,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                            | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                        ..Default::default()
                    },
                    [DrawIndexedIndirectCommand {
                        index_count: 0,
                        instance_count: 0,
                        first_index: 0,
                        vertex_offset: 0,
                        first_instance: 0,
                    }]
                )?;

                let cell = PipelineCell {
                    pipeline,
                    bindings,
                    vertex_buffer_description,
                    vertex_buffer,
                    index_buffer,
                    indirect_buffer,
                    descriptor_set,
                    vertex_shader,
                    intermediate_shaders,
                    fragment_shader,
                };

                gfx.recreate_command_buffers = true;
                let handle = gfx.pipelines.insert(cell);
                Ok(PipelineHandle { handle })
            }
        }
    }
}

pub use pipeline_builder::stage_1::PipelineBuilder;

#[derive(Clone)]
pub enum Binding {
    Buffer(Arc<Buffer>),
    Texture(Texture)
}

pub enum BufferType {
    Static,
    Dynamic,
    DynamicRandomAccess
}

impl From<BufferType> for MemoryTypeFilter {
    fn from(val: BufferType) -> Self {
        match val {
            BufferType::Static => MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            BufferType::Dynamic => MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            BufferType::DynamicRandomAccess => MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_RANDOM_ACCESS,
        }
    }
}

pub struct Graphics {
    vulkan_instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<Image>>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    pipelines: VecAllocator<PipelineCell>,
    command_buffers: Vec<Arc<PrimaryAutoCommandBuffer>>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    viewport: Viewport,
    window_resized: bool,
    recreate_swapchain: bool,
    recreate_command_buffers: bool
}

fn get_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Result<Arc<RenderPass>, GetRenderPassError> {
    Ok(vulkano::single_pass_renderpass!(
        device,
        attachments: {
            color: {
                // Set the format the same as the swapchain.
                format: swapchain.image_format(),
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
            depth: {
                format: Format::D16_UNORM,
                samples: 1,
                load_op: Clear,
                store_op: DontCare,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth},
        },
    )?)
}

fn get_framebuffers(memory_allocator: &Arc<StandardMemoryAllocator>, render_pass: Arc<RenderPass>, images: &[Arc<Image>]) -> Result<Vec<Arc<Framebuffer>>, GetFramebuffersError> {
    let [width, height, _] = images[0].extent();
    let depth_buffer = ImageView::new_default(
        Image::new(memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::D16_UNORM,
                extent: [width, height, 1],
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        )?
    )?;
    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone())?;
            Ok(Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view, depth_buffer.clone()],
                    ..Default::default()
                },
            )?)
        })
        .collect()
}

fn get_pipeline(device: &Arc<Device>, vertex_buffer_description: VertexBufferDescription, vertex_shader: &Arc<ShaderModule>, intermediate_shaders: &[Arc<ShaderModule>], fragment_shader: &Arc<ShaderModule>, render_pass: &Arc<RenderPass>, viewport: Viewport) -> Result<Arc<GraphicsPipeline>, GetPipelineError> {
    let vertex_shader = vertex_shader.entry_point("main").ok_or(InvalidEntryPoint)?;
    let fragment_shader = fragment_shader.entry_point("main").ok_or(InvalidEntryPoint)?;
    
    let vertex_input_state = vertex_buffer_description
        .definition(&vertex_shader)?;

    let stages = iter::once(Ok(vertex_shader))
        .chain(
            intermediate_shaders.iter()
                .map(|shader| shader.entry_point("main").ok_or(InvalidEntryPoint))
        )
        .chain(iter::once(Ok(fragment_shader)))
        .collect::<std::result::Result<Vec<_>, InvalidEntryPoint>>()?
        .into_iter()
        .map(PipelineShaderStageCreateInfo::new)
        .collect_vec();

    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())?,
    )?;

    let subpass = Subpass::from(render_pass.clone(), 0).explicit_unwrap();

    Ok(GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState {
                viewports: [viewport].into_iter().collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState {
                cull_mode: CullMode::None,
                front_face: FrontFace::Clockwise,
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState::default(),
            )),
            subpass: Some(subpass.into()),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState::reverse()),
                ..Default::default()
            }),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )?)
}

fn get_descriptor_set(descriptor_set_allocator: &Arc<StandardDescriptorSetAllocator>, pipeline: &Arc<GraphicsPipeline>, bindings: &HashMap<u32, Binding>) -> Result<Arc<DescriptorSet>, DescriptorSetError> {
    let descriptor_set_allocator = descriptor_set_allocator.clone();
    let pipeline_layout = pipeline.layout();
    let descriptor_set_layouts = pipeline_layout.set_layouts();

    let descriptor_set_layout_index = 0;
    let descriptor_set_layout = descriptor_set_layouts
        .get(descriptor_set_layout_index)
        .ok_or(NoLayout)?;
    let descriptor_writes = bindings.iter()
        .map(|(binding, buffer)| {
            match buffer {
                Binding::Buffer(buffer) => WriteDescriptorSet::buffer(*binding, Subbuffer::new(buffer.clone())),
                Binding::Texture(texture) => WriteDescriptorSet::image_view_sampler(*binding, texture.view().clone(), texture.sampler().clone()),
            }
        });
    
    Ok(DescriptorSet::new(
        descriptor_set_allocator,
        descriptor_set_layout.clone(),
        descriptor_writes,
        [],
    )?)
}

fn get_command_buffers(command_buffer_allocator: &Arc<StandardCommandBufferAllocator>, queue: &Arc<Queue>, pipelines: &VecAllocator<PipelineCell>, framebuffers: &[Arc<Framebuffer>]) -> Result<Vec<Arc<PrimaryAutoCommandBuffer>>, GetCommandBuffersError> {
    framebuffers
        .iter()
        .map(|framebuffer| {
            let command_buffer_allocator = command_buffer_allocator.clone();
            let mut builder = AutoCommandBufferBuilder::primary(
                command_buffer_allocator,
                queue.queue_family_index(),
                // Don't forget to write the correct buffer usage.
                CommandBufferUsage::MultipleSubmit,
            )?;

            unsafe { 
                builder
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![Some([0.75, 0.75, 0.75, 1.0].into()), Some(ClearValue::Depth(0.0))],
                            ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                        },
                        SubpassBeginInfo {
                            contents: SubpassContents::Inline,
                            ..Default::default()
                        },
                    )?;

                pipelines.iter().try_fold(&mut builder, |builder, (_, pipeline)| {
                    let PipelineCell { pipeline, vertex_buffer, index_buffer, indirect_buffer, descriptor_set, .. } = pipeline;
                    builder
                        .bind_pipeline_graphics((*pipeline).clone())?
                        .bind_vertex_buffers(0, Subbuffer::new(vertex_buffer.clone()))?
                        .bind_index_buffer(Subbuffer::new(index_buffer.clone()).cast_aligned::<u32>())?
                        .bind_descriptor_sets(PipelineBindPoint::Graphics, pipeline.layout().clone(), 0, vec![descriptor_set.clone()])?
                        .draw_indexed_indirect(indirect_buffer.clone())
                })?;
            }
            builder
                .end_render_pass(SubpassEndInfo::default())?;

            Ok(builder.build()?)
        })
        .collect()
}

impl Graphics {
    pub fn new(window: Arc<Window>, event_loop: &EventLoop<()>) -> Result<Graphics, NewGraphicsError> {
        let library = VulkanLibrary::new()?;
        let required_extensions = Surface::required_extensions(&event_loop)?;
        let vulkan_instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: required_extensions,
                ..Default::default()
            },
        )?;

        let dimensions = window.inner_size();
        let surface = Surface::from_window(vulkan_instance.clone(), window)?;

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };
        
        let (physical_device, queue_family_index) = vulkan_instance
            .enumerate_physical_devices()?
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    // Find the first first queue family that is suitable.
                    // If none is found, `None` is returned to `filter_map`,
                    // which disqualifies this physical device.
                    .position(|(i, q)| {
                        q.queue_flags.contains(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|q| (p, q as u32))
            }).min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,

                // Note that there exists `PhysicalDeviceType::Other`, however,
                // `PhysicalDeviceType` is a non-exhaustive enum. Thus, one should
                // match wildcard `_` to catch all unknown device types.
                _ => 4,
            })
            .ok_or(NoPhysicalDevices)?;

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: device_extensions,
                ..Default::default()
            },
        )?;

        let queue = queues.next().explicit_unwrap();

        let caps = physical_device
            .surface_capabilities(&surface, Default::default())?;
        
        let composite_alpha = caps.supported_composite_alpha.into_iter().next().explicit_unwrap();
        let image_format =  physical_device
            .surface_formats(&surface, Default::default())?
            .into_iter()
            .find_map(|(format, _)| match format {
                Format::R8G8B8A8_SRGB => Some(format),
                _ => None
            })
            .ok_or(SRGBUnsupported)?;
        
        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                present_mode: PresentMode::Immediate,
                min_image_count: caps.min_image_count + 1, // How many buffers to use in the swapchain
                image_format,
                image_extent: dimensions.into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT, // What the images are going to be used for
                composite_alpha,
                ..Default::default()
            },
        )?;

        let render_pass = get_render_pass(device.clone(), swapchain.clone())?;

        // Flip viewport so +y is up (vulkan defaults to +y down)
        let viewport = Viewport {
            offset: [0.0, dimensions.height as f32],
            extent: [dimensions.width as f32, -(dimensions.height as f32)],
            depth_range: 0.0..=1.0,
        };

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let frame_buffers = get_framebuffers(&memory_allocator, render_pass.clone(), &images)?;
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(device.clone(), Default::default()));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(device.clone(), Default::default()));

        Ok(Graphics {
            vulkan_instance,
            device,
            queue,
            swapchain,
            images,
            render_pass,
            memory_allocator,
            framebuffers: frame_buffers,
            pipelines: VecAllocator::new(),
            command_buffers: Vec::new(),
            command_buffer_allocator,
            descriptor_set_allocator,
            viewport,
            window_resized: false,
            recreate_swapchain: false,
            recreate_command_buffers: false
        })
    }

    pub fn update_pipelines(&mut self, window: &Window) -> Result<(), UpdatePipelinesError> {
        if self.window_resized || self.recreate_swapchain {
            self.recreate_swapchain = false;

            let new_dimensions = window.inner_size();

            let (new_swapchain, new_images) = self.swapchain
                .recreate(SwapchainCreateInfo {
                    image_extent: new_dimensions.into(),
                    ..self.swapchain.create_info()
                })?;
            self.swapchain = new_swapchain;
            self.framebuffers = get_framebuffers(&self.memory_allocator, self.render_pass.clone(), &new_images)?;
            self.viewport = Viewport {
                offset: [0.0, new_dimensions.height as f32],
                extent: [new_dimensions.width as f32, -(new_dimensions.height as f32)],
                depth_range: 0.0..=1.0,
            };

            if self.window_resized {
                self.window_resized = false;
                self.recreate_command_buffers = true;

                for (_, pipeline) in &mut self.pipelines {
                    pipeline.pipeline = get_pipeline(
                        &self.device,
                        pipeline.vertex_buffer_description.clone(),
                        &pipeline.vertex_shader,
                        &pipeline.intermediate_shaders,
                        &pipeline.fragment_shader,
                        &self.render_pass,
                        self.viewport.clone(),
                    )?;
                }
            }
        }

        if self.recreate_command_buffers && self.pipelines.count() > 0 {
            self.recreate_command_buffers = false;
            self.command_buffers = get_command_buffers(
                &self.command_buffer_allocator,
                &self.queue,
                &self.pipelines,
                &self.framebuffers
            )?;
        }

        Ok(())
    }

    pub fn remove_pipeline(&mut self, pipeline: PipelineHandle) {
        self.recreate_command_buffers = true;
        self.pipelines.remove(pipeline.handle).ok();
    }

    pub fn draw(&mut self) -> Result<(), DrawError> {
        if self.command_buffers.is_empty() {
            return Ok(());
        }

        let (image_i, suboptimal, acquire_future) =
        match swapchain::acquire_next_image(self.swapchain.clone(), None)
            .map_err(Validated::unwrap)
        {
            Ok(r) => r,
            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
                return Ok(());
            }
            Err(e) => panic!("failed to acquire next image: {e}"),
        };

        if suboptimal {
            self.recreate_swapchain = true;
        }

        let execution = sync::now(self.device.clone())
            .join(acquire_future)
            .then_execute(self.queue.clone(), self.command_buffers[image_i as usize].clone())?
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_i),
            )
            .then_signal_fence_and_flush();

        match execution.map_err(Validated::unwrap) {
            Ok(future) => {
                // Wait for the GPU to finish.
                future.wait(None)?;
            }
            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
            }
            Err(e) => {
                println!("failed to flush future: {e}");
            }
        }

        Ok(())
    }

    pub fn set_indirect_buffer(&self, pipeline: PipelineHandle, draw_command: DrawIndexedIndirectCommand) -> Result<(), SetIndirectBufferError> {
        let pipeline = self.pipelines.get(pipeline.handle).map_err(|_| InvalidPipelineHandle)?;
        let mut buffer = pipeline.indirect_buffer.write()?;

        buffer[0] = draw_command;

        Ok(())
    }

    pub fn buffer_to_image(&self, image_data: Vec<u8>, image: &Arc<Image>) -> Result<(), BufferImageError> {
        let staging_buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: vulkano::buffer::BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            image_data,
        )?;
        
        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )?;

        builder.copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
            staging_buffer.clone(),
            image.clone(),
        ))?;

        let command_buffer = builder.build()?;
        let future = vulkano::sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)?
            .then_signal_fence_and_flush()?;

        future.wait(None)?;

        Ok(())
    }

    pub fn get_binding(&self, pipeline: PipelineHandle, binding: u32) -> Result<Binding, GetBindingError> {
        let pipeline = self.pipelines.get(pipeline.handle).map_err(|_| InvalidPipelineHandle)?;

        Ok(pipeline.bindings.get(&binding).ok_or(InvalidBinding)?.clone())
    }

    pub fn window_resized(&mut self) {
        self.window_resized = true;
    }

    pub fn vulkan_instance(&self) -> Arc<Instance> {
        self.vulkan_instance.clone()
    }

    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }

    pub fn memory_allocator(&self) -> Arc<StandardMemoryAllocator> {
        self.memory_allocator.clone()
    }

    pub fn command_buffer_allocator(&self) -> Arc<StandardCommandBufferAllocator> {
        self.command_buffer_allocator.clone()
    }

    pub fn descriptor_set_allocator(&self) -> Arc<StandardDescriptorSetAllocator> {
        self.descriptor_set_allocator.clone()
    }

    pub fn viewport(&self) -> Viewport {
        self.viewport.clone()
    }
}