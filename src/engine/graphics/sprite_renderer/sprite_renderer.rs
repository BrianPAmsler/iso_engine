use std::{borrow::Cow, collections::HashMap};

use bytemuck::{Pod, Zeroable};
use derive_serialize::Serialize;
use crate::{engine::{gl_types::vectors::{vec2, vec4}, graphics::{sprite_renderer::{animated_sprite::{AnimatedSprite, AnimatedSpriteData}, error::AddAnimatedSpriteError}, texture::Texture}}, error::Result};
use crate::engine::gl_types::{matrices::{Mat4, MatN}, vectors::{Vec2, Vec3, VecN}};
use itertools::Itertools;
use vulkano::{buffer::{BufferContents, Subbuffer}, padded::Padded};
use vulkano::command_buffer::DrawIndexedIndirectCommand;
use vulkano::pipeline::graphics::vertex_input::Vertex;

use crate::engine::{data_structures::{AllocationIndex, VecAllocator}, graphics::{AlignedVec3, Binding, BufferType, Graphics, PipelineBuilder, PipelineHandle, sprite_renderer::error::{AddSpritesheetError, SpriteRendererBufferError, SpriteRendererUpdateError, UnknownSpriteSheet}}};

const UNIFORMS_BINDING: u32 = 1;
const SPRITE_SHEET_BINDING: u32 = 2;
const SPRITE_MAP_BINDING: u32 = 3;

mod vertex_shader {
    vulkano_shaders::shader!{
        ty: "vertex",
        path: "src/engine/graphics/shaders/sprite.vert",
        root_path_env: "CARGO_MANIFEST_DIR"
    }
}

mod fragment_shader {
    vulkano_shaders::shader!{
        ty: "fragment",
        path: "src/engine/graphics/shaders/sprite.frag",
        root_path_env: "CARGO_MANIFEST_DIR"
    }
}

const VERTEX_DATA: &[SpriteVertex] = &[
    SpriteVertex { position: [0.0, 0.0, 0.0], uv: [0.0, 1.0] }, // bottom left
    SpriteVertex { position: [1.0, 0.0, 0.0], uv: [1.0, 1.0] },  // bottom right
    SpriteVertex { position: [0.0, 1.0, 0.0], uv: [0.0, 0.0] },  // top left
    SpriteVertex { position: [1.0, 1.0, 0.0], uv: [1.0, 0.0] },   // top right
];

const INDEX_DATA: &[u32] = &[
    0, 1, 2,
    2, 1, 3
];

#[derive(Debug, Clone, Copy, BufferContents)]
#[repr(C)]
struct GLSpriteStruct {
    position: AlignedVec3,
    dimensions: [f32; 4],
    id: u32
}

impl Default for GLSpriteStruct {
    fn default() -> Self {
        Self {
            position: AlignedVec3([0.0, 0.0, 0.0]),
            dimensions: [0.0, 0.0, 0.0, 0.0],
            id: 0
        }
    }
}

#[derive(Clone, Copy)]
pub struct SpriteData {
    pub position: Vec3,
    pub anchor: Vec2,
    pub dimensions: Vec2,
    pub sprite_id: u32
}

struct SpriteSheet {
    name: String,
    render_queue: Vec<GLSpriteStruct>,
    pipeline: PipelineHandle,
    buffersize: usize,
    width: u32,
    height: u32
}

impl SpriteSheet {
    fn buffer_sprite_data(&mut self, gfx: &Graphics)  -> Result<(), SpriteRendererBufferError>{
        if self.render_queue.len() > self.buffersize {
            // Multiply new szie by 50% to give some wiggle room
            todo!("Implement uniform buffer resizing")
        }

        let binding = gfx.get_binding(self.pipeline, SPRITE_SHEET_BINDING)?;

        match binding {
            Binding::Buffer(buffer) => {
                let buffer = Subbuffer::from(buffer).reinterpret::<SpriteSSBO>();
                let mut buffer = buffer.write()?;

                buffer.count = self.render_queue.len() as i32;
                buffer.data[..self.render_queue.len()].copy_from_slice(&self.render_queue);
            },
            _ => unreachable!("unexpected binding.")
        }

        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Vertex)]
struct SpriteVertex {
    #[format(R32G32B32_SFLOAT)]
    position: [f32; 3],
    #[format(R32G32_SFLOAT)]
    uv: [f32; 2]
}

#[derive(Serialize)]
pub struct SpriteDefinition {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32
}

#[repr(C)]
#[derive(BufferContents)]
struct SpriteSSBO {
    count: i32,
    data: [GLSpriteStruct]
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, Default, BufferContents, PartialEq)]
struct Vec4Aligned([f32; 4]);

#[repr(C)]
#[derive(BufferContents)]
struct SpriteSheetSSBO {
    count: i32,
    data: [Vec4Aligned]
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, BufferContents)]
struct InputData {
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
    texel_offset: [f32; 2]
}

#[derive(Clone, Copy)]
pub struct SpriteSheetID(AllocationIndex);

#[derive(Clone, Copy)]
pub struct AnimatedSpriteID(AllocationIndex);

pub struct SpriteRenderer {
    sprite_sheets: VecAllocator<SpriteSheet>,
    animated_sprites: VecAllocator<AnimatedSprite>,
    sprite_sheet_index: HashMap<String, AllocationIndex>,
    animated_sprite_index: HashMap<String, AllocationIndex>
}

impl SpriteRenderer {
    pub fn new() -> SpriteRenderer {
        SpriteRenderer { sprite_sheets: VecAllocator::new(), animated_sprites: VecAllocator::new(), sprite_sheet_index: HashMap::new(), animated_sprite_index: HashMap::new() }
    }

    pub fn add_sprite_sheet<'a, S: Into<Cow<'a, str>>>(&mut self, name: S, gfx: &mut Graphics, initial_buffer_size: usize, sprite_sheet: Texture, sprite_map: &[SpriteDefinition]) -> Result<SpriteSheetID, AddSpritesheetError> {
        let name = name.into().into_owned();
        if self.sprite_sheet_index.contains_key(& *name) {
            return Err(UnknownSpriteSheet { sheet: name })?;
        }

        let vertex_shader = vertex_shader::load(gfx.device())?;
        let fragment_shader = fragment_shader::load(gfx.device())?;

        let sprite_map = sprite_map.iter().map(|sprite| {
            let SpriteDefinition { x, y, width, height } = *sprite;
            // Convert pixel coordinates to uv coordinates
            let wh = vec2!(sprite_sheet.width(), sprite_sheet.height());
            vec4!(x, y, width, height) / vec4!(wh, wh)
        })
        .map(|vec| Vec4Aligned(vec.as_array()))
        .collect_vec();

        let pipeline = PipelineBuilder::new(gfx)
            .vertex_shader(vertex_shader)
            .fragment_shader(fragment_shader)
            .vertex_data(VERTEX_DATA.to_owned(), INDEX_DATA.to_owned())?
            .add_texture(0, sprite_sheet.clone())
            .add_uniform_buffer(UNIFORMS_BINDING, InputData::default(), BufferType::Dynamic)?
            .add_storage_buffer_unsized::<SpriteSSBO>(SPRITE_SHEET_BINDING, initial_buffer_size as u64, BufferType::Dynamic)?
            .add_storage_buffer_unsized::<SpriteSheetSSBO>(SPRITE_MAP_BINDING, sprite_map.len() as u64, BufferType::Static)?
            .finish()?;

        match gfx.get_binding(pipeline, SPRITE_MAP_BINDING)? {
            Binding::Buffer(buffer) => {
                let buffer = Subbuffer::new(buffer).reinterpret::<SpriteSheetSSBO>();
                let mut value = buffer.write()?;
                value.count = sprite_map.len() as i32;
                value.data.copy_from_slice(&sprite_map);
            },
            _ => unreachable!()
        }

        let sprite_sheet = SpriteSheet {
            name: name.clone(),
            render_queue: Vec::new(),
            buffersize: initial_buffer_size,
            pipeline,
            width: sprite_sheet.width(),
            height: sprite_sheet.height()
        };

        let id = self.sprite_sheets.insert(sprite_sheet);
        self.sprite_sheet_index.insert(name, id);

        Ok(SpriteSheetID(id))
    }

    pub fn add_animated_sprite(&mut self, gfx: &mut Graphics, name: String, texture_array: Texture) -> Result<AnimatedSpriteID, AddAnimatedSpriteError> {
        let sprite = AnimatedSprite::new(gfx, texture_array, name.clone())?;

        let id = self.animated_sprites.insert(sprite);
        self.animated_sprite_index.insert(name, id);

        Ok(AnimatedSpriteID(id))
    }

    pub fn remove_sprite_sheet(&mut self, gfx: &mut Graphics, sprite_sheet: SpriteSheetID) {
        let Ok(old) = self.sprite_sheets.remove(sprite_sheet.0) else { return };

        self.sprite_sheet_index.remove(&old.name);
        gfx.remove_pipeline(old.pipeline);
    }

    pub fn get_sprite_sheet_by_name(&self, name: &str) -> Option<SpriteSheetID> {
        self.sprite_sheet_index.get(name).map(|idx| SpriteSheetID(*idx))
    }

    pub fn get_animated_sprite_by_name(&self, name: &str) -> Option<AnimatedSpriteID> {
        self.animated_sprite_index.get(name).map(|idx| AnimatedSpriteID(*idx))
    }

    pub fn remove_animated_sprite(&mut self, gfx: &mut Graphics, animated_sprite: AnimatedSpriteID) {
        let Ok(old) = self.animated_sprites.remove(animated_sprite.0) else { return };

        self.animated_sprite_index.remove(&old.name);
        gfx.remove_pipeline(old.pipeline);
    }

    pub fn queue_sprite_instance(&mut self, sprite: SpriteData, sprite_sheet: SpriteSheetID) {
        let Ok(sheet) = self.sprite_sheets.get_mut(sprite_sheet.0) else { return; };

        let SpriteData { position, dimensions, anchor, sprite_id } = sprite;
        let dimensions = vec4!(anchor, dimensions).as_array();
        let position = AlignedVec3(position.as_array());
        let id = sprite_id;

        let sprite_data = GLSpriteStruct {
            position,
            dimensions,
            id
        };

        sheet.render_queue.push(sprite_data);
    }

    pub(in crate::engine::graphics) fn queue_animated_sprite_instance(&mut self, sprite: AnimatedSpriteID, sprite_data: AnimatedSpriteData) {
        let Ok(sprite) = self.animated_sprites.get_mut(sprite.0) else { return };
        
        sprite.render_queue.push(Padded(sprite_data.into()));
    }

    pub fn get_total_frames(&self, sprite: AnimatedSpriteID) -> Option<u32> {
        self.animated_sprites.get(sprite.0).ok().map(|sprite| {
            sprite.depth()
        })
    }

    pub fn update(&mut self, gfx: &Graphics, view_matrix: &Mat4, projection_matrix: &Mat4) -> Result<(), SpriteRendererUpdateError> {
        for (_, sheet) in &mut self.sprite_sheets {
            let draw_command = DrawIndexedIndirectCommand {
                index_count: INDEX_DATA.len() as u32,
                instance_count: sheet.render_queue.len() as u32,
                first_index: 0,
                vertex_offset: 0,
                first_instance: 0,
            };
            
            gfx.set_indirect_buffer(sheet.pipeline, draw_command)?;
            sheet.buffer_sprite_data(gfx)?;

            let texel_offset = vec2!(1.0) / (vec2!(sheet.width, sheet.height) * 2.0);

            // Update uniforms
            let uniform_buffer = match gfx.get_binding(sheet.pipeline, UNIFORMS_BINDING)? {
                Binding::Buffer(buffer) => buffer,
                _ => unreachable!("invalid binding.")
            };

            let unifom_buffer = Subbuffer::new(uniform_buffer).reinterpret::<InputData>();

            let view = view_matrix.as_array();
            let projection = projection_matrix.as_array();
            let texel_offset = texel_offset.as_array();
            *unifom_buffer.write()? = InputData {
                view,
                projection,
                texel_offset,
            };
            sheet.render_queue.clear();
        }

        for (_, sprite) in &mut  self.animated_sprites {
            sprite.update(gfx, view_matrix, projection_matrix)?;
        }

        Ok(())
    }
}

impl Default for SpriteRenderer {
    fn default() -> Self {
        SpriteRenderer::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason="test")]
mod tests {
    use std::{sync::Arc, time::Duration};

    use vulkano::{buffer::Subbuffer, command_buffer::DrawIndexedIndirectCommand};
    use winit::{application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent, event_loop::{self, EventLoop}, platform::pump_events::EventLoopExtPumpEvents as _, window::{Window, WindowAttributes}};

    mod vertex_shader {
        vulkano_shaders::shader!{
            ty: "vertex",
            path: "src/engine/graphics/shaders/sprite_struct_test.vert",
            root_path_env: "CARGO_MANIFEST_DIR"
        }
    }

    mod fragment_shader {
        vulkano_shaders::shader!{
            ty: "fragment",
            path: "src/engine/graphics/shaders/sprite_struct_test.frag",
            root_path_env: "CARGO_MANIFEST_DIR"
        }
    }
    
    impl PartialEq for GLSpriteStruct {
        fn eq(&self, other: &Self) -> bool {
            self.position == other.position && self.dimensions == other.dimensions && self.id == other.id
        }
    }

    impl PartialEq for InputData {
        fn eq(&self, other: &Self) -> bool {
            self.view == other.view && self.projection == other.projection && self.texel_offset == other.texel_offset
        }
    }

    use crate::{engine::graphics::{Binding, BufferType, Graphics, PipelineBuilder, sprite_renderer::sprite_renderer::{AlignedVec3, GLSpriteStruct, INDEX_DATA, InputData, SPRITE_MAP_BINDING, SPRITE_SHEET_BINDING, SpriteSSBO, SpriteSheetSSBO, UNIFORMS_BINDING, VERTEX_DATA, Vec4Aligned}}, error::{MessageErorr, any::Result}};
    
    
    
    #[cfg(target_os = "windows")] // The EXT traits are platform dependent
    use winit::platform::windows::EventLoopBuilderExtWindows;

    #[test]
    pub fn sprite_struct_test() -> Result<()> {
        let _lock = crate::engine::graphics::test_lock::LOCK.lock().unwrap();

        let mut event_loop = EventLoop::builder()
        .with_any_thread(true)
        .build()?;
    
        event_loop.set_control_flow(event_loop::ControlFlow::Poll);

        let window_attributes = WindowAttributes::default()
            .with_title("Test Window")
            .with_inner_size(PhysicalSize::new(1280, 720))
            .with_fullscreen(None);

        #[derive(Default)]
        enum WindowStatus {
            Uninitialized(Box<WindowAttributes>),
            Initialized(Window),
            #[default]
            Null
        }

        struct WindowInitializer(WindowStatus);

        impl ApplicationHandler for WindowInitializer {
            fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
                let WindowStatus::Uninitialized(window_attributes) = std::mem::take(&mut self.0) else { return };
                self.0 = WindowStatus::Initialized(event_loop.create_window(*window_attributes).unwrap());
            }
        
            fn window_event(&mut self, _: &event_loop::ActiveEventLoop, _: winit::window::WindowId, _: WindowEvent) {}
        }

        let mut app = WindowInitializer(WindowStatus::Uninitialized(Box::new(window_attributes)));
        
        event_loop.pump_app_events(Some(Duration::ZERO), &mut app);

        let WindowStatus::Initialized(window) = app.0 else { Err(MessageErorr("d"))? };
        let window = Arc::new(window);

        let mut gfx = Graphics::new(window, &event_loop).unwrap();

        let vertex_shader = vertex_shader::load(gfx.device())?;
        let fragment_shader = fragment_shader::load(gfx.device())?;

        let pipeline = PipelineBuilder::new(&mut gfx)
            .vertex_shader(vertex_shader)
            .fragment_shader(fragment_shader)
            .vertex_data(VERTEX_DATA.to_owned(), INDEX_DATA.to_owned())?
            .add_storage_buffer(UNIFORMS_BINDING, InputData::default(), BufferType::Dynamic)?
            .add_storage_buffer_unsized::<SpriteSSBO>(SPRITE_SHEET_BINDING, 2, BufferType::Dynamic)?
            .add_storage_buffer_unsized::<SpriteSheetSSBO>(SPRITE_MAP_BINDING, 2, BufferType::Dynamic)?
            .finish()?;

        let mut sprite_data = [GLSpriteStruct::default(); 2];
        let sprite_count;

        let mut sprite_sheet_data = [Vec4Aligned::default(); 2];
        let sprite_sheet_count;

        gfx.set_indirect_buffer(pipeline, DrawIndexedIndirectCommand {
            index_count: INDEX_DATA.len() as u32,
            instance_count: 1,
            first_index: 0,
            vertex_offset: 0,
            first_instance: 0,
        })?;
        gfx.draw()?;

        match gfx.get_binding(pipeline, SPRITE_SHEET_BINDING)? {
            Binding::Buffer(buffer) => {
                let buffer = Subbuffer::new(buffer);
                let buffer = buffer.reinterpret::<SpriteSSBO>();
                let buffer = buffer.read()?;
                sprite_count = buffer.count;
                sprite_data.copy_from_slice(&buffer.data);
            },
            _ => unreachable!()
        }

        let uniform_data = match gfx.get_binding(pipeline, UNIFORMS_BINDING)? {
            Binding::Buffer(buffer) => {
                let buffer = Subbuffer::new(buffer).reinterpret::<InputData>();
                let buffer = buffer.read()?;
                *buffer
            },
            _ => unreachable!()
        };

        match gfx.get_binding(pipeline, SPRITE_MAP_BINDING)? {
            Binding::Buffer(buffer) => {
                let buffer = Subbuffer::new(buffer);
                let buffer = buffer.reinterpret::<SpriteSheetSSBO>();
                let buffer = buffer.read()?;
                sprite_sheet_count = buffer.count;
                sprite_sheet_data.copy_from_slice(&buffer.data);
            },
            _ => unreachable!()
        }
        
        let sprite_expected = [
            GLSpriteStruct {
                position: AlignedVec3([1.0, 2.0, 3.0]),
                dimensions: [4.0, 5.0, 6.0, 7.0],
                id: 8
            },
            GLSpriteStruct {
                position: AlignedVec3([9.0, 10.0, 11.0]),
                dimensions: [12.0, 13.0, 14.0, 15.0],
                id: 16
            },
        ];

        let uniform_expected = InputData {
            view: [
                [ 1.0,  2.0,  3.0,  4.0],
                [ 5.0,  6.0,  7.0,  8.0],
                [ 9.0, 10.0, 11.0, 12.0],
                [13.0, 14.0, 15.0, 16.0]
            ],
            projection: [
                [17.0, 18.0, 19.0, 20.0],
                [21.0, 22.0, 23.0, 24.0],
                [25.0, 26.0, 27.0, 28.0],
                [29.0, 30.0, 31.0, 32.0]
            ],
            texel_offset: [33.0, 34.0],
        };

        let sprite_sheet_expected = [
            Vec4Aligned([1.0, 2.0, 3.0, 4.0]),
            Vec4Aligned([5.0, 6.0, 7.0, 8.0])
        ];

        assert_eq!(sprite_data, sprite_expected);
        assert_eq!(sprite_count, 69);
        assert_eq!(uniform_expected, uniform_data);
        assert_eq!(sprite_sheet_data, sprite_sheet_expected);
        assert_eq!(sprite_sheet_count, 420);

        Ok(())
    }
}
