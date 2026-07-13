use bytemuck::{Pod, Zeroable};
use crate::engine::gl_types::{matrices::{Mat4, MatN as _}, vectors::{Vec2, Vec3, VecN, vec4}};
use vulkano::{buffer::Subbuffer, command_buffer::DrawIndexedIndirectCommand, padded::Padded, pipeline::graphics::vertex_input::Vertex};

use crate::{engine::graphics::{Binding, BufferType, Graphics, PipelineBuilder, PipelineHandle, sprite_renderer::{animated_sprite::vertex_shader::{InputData, spriteSSBO}, error::{NewAnimatedSpriteError, SpriteRendererBufferError, SpriteRendererUpdateError}}, texture::Texture}, error::Result};


mod vertex_shader {
    vulkano_shaders::shader!{
        ty: "vertex",
        path: "src/engine/graphics/shaders/animated_sprite.vert",
        root_path_env: "CARGO_MANIFEST_DIR"
    }

    #[allow(clippy::derivable_impls, reason="Cannot add a derive attribute to generated code.")]
    impl Default for InputData {
        fn default() -> Self {
            Self { view: Default::default(), projection: Default::default() }
        }
    }
}

mod fragment_shader {
    vulkano_shaders::shader!{
        ty: "fragment",
        path: "src/engine/graphics/shaders/animated_sprite.frag",
        root_path_env: "CARGO_MANIFEST_DIR"
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Vertex)]
struct AnimatedSpriteVertex {
    #[format(R32G32B32_SFLOAT)]
    position: [f32; 3],
    #[format(R32G32_SFLOAT)]
    uv: [f32; 2]
}

const VERTEX_DATA: &[AnimatedSpriteVertex] = &[
    AnimatedSpriteVertex { position: [0.0, 0.0, 0.0], uv: [0.0, 1.0] }, // bottom left
    AnimatedSpriteVertex { position: [1.0, 0.0, 0.0], uv: [1.0, 1.0] },  // bottom right
    AnimatedSpriteVertex { position: [0.0, 1.0, 0.0], uv: [0.0, 0.0] },  // top left
    AnimatedSpriteVertex { position: [1.0, 1.0, 0.0], uv: [1.0, 0.0] },   // top right
];

const INDEX_DATA: &[u32] = &[
    0, 1, 2,
    2, 1, 3
];

pub(in crate::engine::graphics) struct AnimatedSpriteData {
    pub position: Vec3,
    pub anchor: Vec2,
    pub dimensions: Vec2,
    pub frame: u32
}

impl From<AnimatedSpriteData> for vertex_shader::Sprite {
    fn from(value: AnimatedSpriteData) -> Self {
        Self { position: value.position.into_array().into(), dimensions: vec4!(value.anchor, value.dimensions).into_array(), frame: value.frame }
    }
}

pub(in crate::engine::graphics) struct AnimatedSprite {
    pub name: String,
    pub pipeline: PipelineHandle,
    pub render_queue: Vec<Padded<vertex_shader::Sprite, 12>>,
    texture: Texture,
}

impl AnimatedSprite {
    pub fn new(gfx: &mut Graphics, texture: Texture, name: String) -> Result<AnimatedSprite, NewAnimatedSpriteError> {
        let vertex_shader = vertex_shader::load(gfx.device())?;
        let fragment_shader = fragment_shader::load(gfx.device())?;

        const animated_sprite_buffer_size: u64 = 1024; // Lowercase on purpose so I remember to change this from a constant
        let pipeline = PipelineBuilder::new(gfx)
            .vertex_shader(vertex_shader)
            .fragment_shader(fragment_shader)
            .vertex_data(VERTEX_DATA.to_vec(), INDEX_DATA.to_vec())?
            .add_texture(0, texture.clone())
            .add_uniform_buffer(1, InputData::default(), BufferType::Dynamic)?
            .add_storage_buffer_unsized::<spriteSSBO>(2, animated_sprite_buffer_size, BufferType::Dynamic)?
            .finish()?;

        Ok(AnimatedSprite { name, pipeline, render_queue: Vec::new(), texture })
    }

    fn buffer_sprite_data(&mut self, gfx: &Graphics)  -> Result<(), SpriteRendererBufferError>{
        // if self.render_queue.len() > self.buffersize {
        //     // Multiply new szie by 50% to give some wiggle room
        //     todo!("Implement uniform buffer resizing")
        // }

        let binding = gfx.get_binding(self.pipeline, 2)?;

        match binding {
            Binding::Buffer(buffer) => {
                let buffer = Subbuffer::from(buffer).reinterpret::<spriteSSBO>();
                let mut buffer = buffer.write()?;

                buffer.spriteCount = Padded(self.render_queue.len() as i32);
                buffer.sprites[..self.render_queue.len()].copy_from_slice(&self.render_queue);
            },
            _ => unreachable!("unexpected binding.")
        }

        Ok(())
    }

    pub fn update(&mut self, gfx: &Graphics, view_matrix: &Mat4, projection_matrix: &Mat4) -> Result<(), SpriteRendererUpdateError> {
        let draw_command = DrawIndexedIndirectCommand {
            index_count: INDEX_DATA.len() as u32,
            instance_count: self.render_queue.len() as u32,
            first_index: 0,
            vertex_offset: 0,
            first_instance: 0,
        };
        
        gfx.set_indirect_buffer(self.pipeline, draw_command)?;
        self.buffer_sprite_data(gfx)?;

        // Update uniforms
        let uniform_buffer = match gfx.get_binding(self.pipeline, 1)? {
            Binding::Buffer(buffer) => buffer,
            _ => unreachable!("invalid binding.")
        };

        let unifom_buffer = Subbuffer::new(uniform_buffer).reinterpret::<InputData>();

        let view = view_matrix.as_array();
        let projection = projection_matrix.as_array();
        *unifom_buffer.write()? = InputData {
            view,
            projection
        };

        self.render_queue.clear();

        Ok(())
    }

    pub fn width(&self) -> u32 {
        self.texture.width()
    }

    pub fn height(&self) -> u32 {
        self.texture.height()
    }

    pub fn depth(&self) -> u32 {
        self.texture.depth()
    }
}