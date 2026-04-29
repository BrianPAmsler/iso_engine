use std::collections::HashMap;

use gl_types::matrices::Mat4;
use gl_types::{vec2, vec3, vec4};
use gl_types::vectors::{Vec2, Vec3, Vec4};
use embed_shader_source::embed_shader_source;

use crate::engine::data_structures::{AllocationIndex, VecAllocator};
use crate::engine::graphics::gl_enums::{BufferTargetARB, BufferUsageARB, InternalFormat, PrimitiveType, TextureTarget, TextureUnit};
use crate::engine::graphics::image::Image;
use crate::engine::graphics::{BufferedMesh, FragmentShader, GlUniformLocation, Graphics, Mesh, ShaderProgram, ShaderProgramBuilder, Texture, UV, VBOBufferer, Vertex, VertexShader};

use crate::engine::errors::Result;

const SSBO_OFFSET: isize = 16;

#[repr(align(16))]
#[derive(Clone, Copy, PartialEq)]
struct AlignedVec3(gl_types::vectors::Vec3);

impl std::fmt::Debug for AlignedVec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy)]
struct GLSpriteStruct {
    position: AlignedVec3,
    dimensions: Vec4,
    id: u32
}

impl Default for GLSpriteStruct {
    fn default() -> Self {
        Self {
            position: AlignedVec3(vec3!(0)),
            dimensions: vec4!(0),
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
    buffersize: usize,
    sprite_ssbo: u32,
    spritesheet_ssbo: u32,
    sprite_sheet: Texture,
    sprite_map: Vec<Vec4>,
}

impl SpriteSheet {
    fn buffer_sprite_data(&mut self, gfx: &Graphics) {
        let data_size = self.render_queue.len() * std::mem::size_of::<GLSpriteStruct>() + SSBO_OFFSET as usize;
        gfx.glBindBuffer(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, self.sprite_ssbo);

        if data_size > self.buffersize {
            // Multiply new szie by 50% to give some wiggle room
            self.buffersize = (data_size * 3) / 2;
            gfx.glBufferNull(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, self.buffersize, BufferUsageARB::GL_DYNAMIC_DRAW);
            gfx.glBindBufferBase(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, 2, self.sprite_ssbo);
        }

        // Buffer length data
        gfx.glBufferSubData(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, 0, &[self.render_queue.len()]); 
        // Buffer sprite data
        gfx.glBufferSubData(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, SSBO_OFFSET, &self.render_queue[..]); 
        gfx.glBindBuffer(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, 0);
    }
}

#[derive(Clone, Copy)]
pub struct SpriteSheetID(AllocationIndex);

pub struct SpriteRenderer {
    program: ShaderProgram,
    mesh: BufferedMesh,
    sprite_sheets: VecAllocator<SpriteSheet>,
    sprite_sheet_index: HashMap<String, AllocationIndex>,
    view_location: GlUniformLocation,
    projection_location: GlUniformLocation,
    texel_offset_location: GlUniformLocation
}

impl SpriteRenderer {
    pub fn new(gfx: &Graphics) -> Result<SpriteRenderer> {
        let mut program = ShaderProgramBuilder::new(gfx);
        
        let vertex_shader_source = embed_shader_source!("sprite.vert");
        let fragment_shader_source = embed_shader_source!("sprite.frag");

        let vert_shader = VertexShader::compile_shader(gfx, vertex_shader_source)?;
        let frag_shader = FragmentShader::compile_shader(gfx, fragment_shader_source)?; 

        program.attach_shader(vert_shader);
        program.attach_shader(frag_shader);

        let program = program.finish();

        gfx.glUseProgram(program.program());

        let view_location = gfx.glGetUniformLocation(program.program(), "view");
        let projection_location = gfx.glGetUniformLocation(program.program(), "projection");
        let texel_offset_location = gfx.glGetUniformLocation(program.program(), "texelOffset");

        let vertex_data = Box::new([
            Vertex { x: 0.0, y: 0.0, z: 0.0 }, // bottom left
            Vertex { x: 1.0, y: 0.0, z: 0.0 },  // bottom right
            Vertex { x: 0.0, y: 1.0, z: 0.0 },  // top left
        
            Vertex { x: 0.0, y: 1.0, z: 0.0 },  // top left
            Vertex { x: 1.0, y: 0.0, z: 0.0 },  // bottom right
            Vertex { x: 1.0, y: 1.0, z: 0.0 },   // top right
        ]);

        let uv_data = Box::new([
            UV { u: 0.0 , v: 0.0 },
            UV { u: 1.0 , v: 0.0 },
            UV { u: 0.0 , v: 1.0 },

            UV { u: 0.0 , v: 1.0 },
            UV { u: 1.0 , v: 0.0 },
            UV { u: 1.0 , v: 1.0 },
        ]);

        let mesh = Mesh::new("Sprite Mesh".to_owned(), vertex_data, None, Some(uv_data), None, None);
        
        let mut vbo = VBOBufferer::new(gfx);
        let mesh = vbo.add_mesh(mesh);

        vbo.buffer_data(gfx);

        let mesh = mesh.take();

        Ok(SpriteRenderer { program, mesh, sprite_sheets: VecAllocator::new(), sprite_sheet_index: HashMap::new(), view_location, projection_location, texel_offset_location })
    }

    pub fn add_sprite_sheet(&mut self, name: &str, gfx: &Graphics, initial_buffer_size: usize, sprite_sheet: Image) -> Option<SpriteSheetID> {
        if self.sprite_sheet_index.contains_key(name) {
            return None;
        }

        let sprite_sheet = sprite_sheet.as_texture(gfx, InternalFormat::GL_RGBA);
        
        let mut sprite_ssbo = 0;
        gfx.glGenBuffer(&mut sprite_ssbo);
        gfx.glBindBuffer(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, sprite_ssbo);
        gfx.glBufferNull(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, initial_buffer_size, BufferUsageARB::GL_DYNAMIC_DRAW);
        gfx.glBindBufferBase(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, 2, sprite_ssbo);

        let mut spritesheet_ssbo = 0;
        gfx.glGenBuffer(&mut spritesheet_ssbo);

        let sprite_sheet = SpriteSheet {
            name: name.to_owned(),
            render_queue: Vec::new(),
            buffersize: initial_buffer_size,
            sprite_ssbo,
            spritesheet_ssbo,
            sprite_sheet,
            sprite_map: Vec::new(),
        };

        let id = self.sprite_sheets.insert(sprite_sheet);
        self.sprite_sheet_index.insert(name.to_owned(), id);

        Some(SpriteSheetID(id))
    }

    pub fn remove_sprite_sheet(&mut self, gfx: &Graphics, sprite_sheet: SpriteSheetID) {
        let Ok(old) = self.sprite_sheets.remove(sprite_sheet.0) else { return };

        self.sprite_sheet_index.remove(&old.name);
        old.sprite_sheet.delete(gfx);
        gfx.glDeleteBuffers(&[old.sprite_ssbo, old.spritesheet_ssbo]);
    }

    pub fn get_sprite_sheet_by_name(&self, name: &str) -> Option<SpriteSheetID> {
        self.sprite_sheet_index.get(name).map(|idx| SpriteSheetID(*idx))
    }

    pub fn add_sprite(&mut self, sprite_sheet: SpriteSheetID, x: u32, y: u32, width: u32, height: u32) -> Option<usize> {
        let Ok(sheet) = self.sprite_sheets.get_mut(sprite_sheet.0) else { return None; };
        // Convert pixel coordinates to uv coordinates
        let wh = vec2!(sheet.sprite_sheet.width(), sheet.sprite_sheet.height());
        let v = vec4!(x, sheet.sprite_sheet.height() - y - height, width, height) / vec4!(wh, wh);

        let idx = sheet.sprite_map.len();
        sheet.sprite_map.push(v);

        Some(idx)
    }

    pub fn update_sprite_map(&self, gfx: &Graphics, sprite_sheet: SpriteSheetID) {
        let Ok(sheet) = self.sprite_sheets.get(sprite_sheet.0) else { return; };

        gfx.glBindBuffer(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, sheet.spritesheet_ssbo);
        gfx.glBufferNull(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, sheet.sprite_map.len() * size_of::<Vec4>() + SSBO_OFFSET as usize, BufferUsageARB::GL_DYNAMIC_DRAW);
        gfx.glBindBufferBase(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, 3, sheet.spritesheet_ssbo);

        // Buffer length data
        gfx.glBufferSubData(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, 0, &[sheet.sprite_map.len()]); 
        // Buffer sprite data
        gfx.glBufferSubData(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, SSBO_OFFSET, &sheet.sprite_map[..]); 

        gfx.glBindBuffer(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, 0);
    }

    pub fn queue_sprite_instance(&mut self, sprite: SpriteData, sprite_sheet: SpriteSheetID) {
        let Ok(sheet) = self.sprite_sheets.get_mut(sprite_sheet.0) else { return; };

        let SpriteData { position, dimensions, anchor, sprite_id } = sprite;
        let dimensions = vec4!(anchor, dimensions);
        let position = AlignedVec3(position);
        let id = sprite_id;

        let sprite_data = GLSpriteStruct {
            position,
            dimensions,
            id
        };

        sheet.render_queue.push(sprite_data);
    }

    pub fn render(&mut self, gfx: &Graphics, view_matrix: &Mat4, projection_matrix: &Mat4) {
        for (_, sheet) in &mut self.sprite_sheets {
            gfx.glBindVertexArray(self.mesh.vao());
            gfx.glUseProgram(self.program.program());
            sheet.buffer_sprite_data(gfx);
            gfx.glActiveTexture(TextureUnit::GL_TEXTURE0);
            gfx.glBindTexture(TextureTarget::GL_TEXTURE_2D, sheet.sprite_sheet.texture_id());

            let texel_offset = vec2!(1.0) / (vec2!(sheet.sprite_sheet.width(), sheet.sprite_sheet.height()) * 2.0);

            gfx.glUniformMatrix4f(self.view_location, false, &view_matrix);
            gfx.glUniformMatrix4f(self.projection_location, false, &projection_matrix);
            gfx.glUniform2f(self.texel_offset_location, texel_offset.x(), texel_offset.y());

            gfx.glDrawArraysInstanced(PrimitiveType::GL_TRIANGLES, 0, self.mesh.len() as _, sheet.render_queue.len() as u32);
            sheet.render_queue.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use gl_types::{vec3, vec4};
    use embed_shader_source::embed_shader_source;
    
    impl PartialEq for GLSpriteStruct {
        fn eq(&self, other: &Self) -> bool {
            self.position == other.position && self.dimensions == other.dimensions && self.id == other.id
        }
    }

    use crate::engine::graphics::{FragmentShader, Graphics, ShaderProgramBuilder, VertexShader, gl_enums::{BufferTargetARB, BufferUsageARB, PrimitiveType}, sprite_renderer::sprite_renderer::{AlignedVec3, GLSpriteStruct, SSBO_OFFSET}};

    use super::SpriteRenderer;

    #[test]
    pub fn sprite_struct_test() {
        let lock = crate::engine::graphics::test_lock::LOCK.lock().unwrap();

        let gfx = Graphics::init("test_window", 1289, 720, crate::engine::WindowMode::Windowed).unwrap();

        let renderer = SpriteRenderer::new(&gfx).unwrap();

        let mut program = ShaderProgramBuilder::new(&gfx);
        
        let vertex_shader_source = embed_shader_source!("sprite_struct_test.vert");
        let fragment_shader_source = embed_shader_source!("sprite_struct_test.frag"); 

        let vert_shader = VertexShader::compile_shader(&gfx, vertex_shader_source).unwrap();
        let frag_shader = FragmentShader::compile_shader(&gfx, fragment_shader_source).unwrap();

        program.attach_shader(vert_shader);
        program.attach_shader(frag_shader);

        let program = program.finish();

        gfx.glUseProgram(program.program());

        let mut ssbo = 0;
        gfx.glGenBuffer(&mut ssbo);

        let sprite_structs = [GLSpriteStruct::default(); 2];

        let mut data_in = [GLSpriteStruct::default(); 2];

        gfx.glBindBuffer(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, ssbo);
        // Allocate space
        gfx.glBufferNull(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, std::mem::size_of::<GLSpriteStruct>() * sprite_structs.len() + SSBO_OFFSET as usize, BufferUsageARB::GL_DYNAMIC_DRAW); 
        // Buffer length data
        gfx.glBufferSubData(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, 0, std::slice::from_ref(&sprite_structs.len())); 
        // Buffer sprite data
        gfx.glBufferSubData(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, SSBO_OFFSET,&sprite_structs); 
        gfx.glBindBufferBase(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, 2, ssbo);

        gfx.glUseProgram(program.program());
        gfx.glBindVertexArray(renderer.mesh.vao());
        gfx.glDrawArrays(PrimitiveType::GL_TRIANGLES, 0, renderer.mesh.len() as _);

        unsafe { gfx.glGetBufferSubData(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, SSBO_OFFSET, std::mem::size_of::<GLSpriteStruct>() as isize * sprite_structs.len() as isize, data_in.as_mut_ptr() as *mut _) };

        gfx.glBindBuffer(BufferTargetARB::GL_SHADER_STORAGE_BUFFER, 0); // unbind
        
        let expected = [
            GLSpriteStruct {
                position: AlignedVec3(vec3!(1, 2, 3)),
                dimensions: vec4!(4, 5, 6, 7),
                id: 8
            },
            GLSpriteStruct {
                position: AlignedVec3(vec3!(9, 10, 11)),
                dimensions: vec4!(12, 13, 14, 15),
                id: 16
            },
        ];

        assert_eq!(data_in, expected);
        drop(gfx);
        drop(lock);
    }
}
