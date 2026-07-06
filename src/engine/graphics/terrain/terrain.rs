use std::{borrow::Cow, path::{Path, PathBuf}, sync::Arc};

use bytemuck::{Pod, Zeroable};
use derive_serialize::Serialize;
use image::{GrayImage, ImageBuffer, ImageError, Luma, RgbImage, RgbaImage};
use vulkano::{format::Format, image::sampler::Filter, pipeline::graphics::vertex_input::Vertex};

use crate::{engine::{game_object::component::Component, graphics::{BufferType, Graphics, PipelineBuilder, PipelineHandle, terrain::{error::{CellAccessError, TerrainFromRawError, UpdateTextureError}, terrain_renderer::{TerrainRenderer, fragment_shader::FragmentUniforms, vertex_shader::VertexUniforms}}, texture::{Texture, builder::TextureBuilder}}, resources::{ResourceHandle, resource_loaders::ImageLoader, serialization::AsSerialize}}, error::{OutOfBounds, Result, TryUnwrap, Uninitialized}};

const VERTEX_DATA: &[TerrainVertex] = &[
    // [0]: Bottom-Left Corner
    TerrainVertex { position: [0.0, 0.0, 0.0] },
    // [1]: Bottom-Right Corner
    TerrainVertex { position: [1.0, 0.0, 0.0] },
    // [2]: Top-Left Corner
    TerrainVertex { position: [0.0, 0.0, 1.0] },
    // [3]: Top-Right Corner
    TerrainVertex { position: [1.0, 0.0, 1.0] },
    // [4]: Center
    TerrainVertex { position: [0.5, 0.0, 0.5] }
];

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Vertex)]
struct TerrainVertex {
    #[format(R32G32B32_SFLOAT)]
    position: [f32; 3]
}

pub(in crate::engine::graphics::terrain) const INDEX_DATA: &[u32] = &[
    // -X side
    0, 4, 2,
    // +X side
    1, 3, 4,
    // -Z side
    0, 1, 4,
    // +Z side
    2, 4, 3,
];

pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    All
}

pub struct CellCorner<'a> {
    height: &'a mut u8,
    color: &'a mut [u8; 3],
    color_dirty: &'a mut bool,
    height_dirty: &'a mut bool
}

impl CellCorner<'_> {
    pub fn height(&mut self) -> &mut u8 {
        *self.height_dirty = true;
        self.height
    }

    pub fn color(&mut self) -> &mut [u8; 3] {
        *self.color_dirty = true;
        self.color
    }
}

// I'm not the biggest fan of this method of accessing terrain data,
// but I'll keep it for now.
pub struct TerrainCellMut<'a> {
    top_left_height: &'a mut u8,
    top_right_height: &'a mut u8,
    bottom_left_height: &'a mut u8,
    bottom_right_height: &'a mut u8,
    top_left_color: &'a mut [u8; 3],
    top_right_color: &'a mut [u8; 3],
    bottom_left_color: &'a mut [u8; 3],
    bottom_right_color: &'a mut [u8; 3],
    color_changed: &'a mut bool,
    height_changed: &'a mut bool
}

impl<'a> TerrainCellMut<'a> {
    pub fn top_left(&'a mut self) -> CellCorner<'a> {
        CellCorner { height: self.top_left_height, color: self.top_left_color, color_dirty: self.color_changed, height_dirty: self.height_changed }
    }

    pub fn top_right(&'a mut self) -> CellCorner<'a> {
        CellCorner { height: self.top_right_height, color: self.top_right_color, color_dirty: self.color_changed, height_dirty: self.height_changed }
    }

    pub fn bottom_left(&'a mut self) -> CellCorner<'a> {
        CellCorner { height: self.bottom_left_height, color: self.bottom_left_color, color_dirty: self.color_changed, height_dirty: self.height_changed }
    }

    pub fn bottom_right(&'a mut self) -> CellCorner<'a> {
        CellCorner { height: self.bottom_right_height, color: self.bottom_right_color, color_dirty: self.color_changed, height_dirty: self.height_changed }
    }
}

#[derive(Debug)]
struct TerrainInner {
    height_data: Vec<u8>,
    color_data: Vec<u8>,
    width: u32,
    height: u32,
    height_texture: Texture,
    color_texture: Texture,
    pipeline: PipelineHandle,
    height_dirty: bool,
    color_dirty: bool
}

#[derive(Serialize, Debug)]
pub struct Terrain {
    #[serialized]
    height_file: PathBuf,
    #[serialized]
    color_file: PathBuf,
    data: Option<TerrainInner>,
    height_handle: Option<ResourceHandle<GrayImage, Arc<ImageError>>>,
    color_handle: Option<ResourceHandle<RgbaImage, Arc<ImageError>>>,
}

const ALIGNED_BYTES_PER_COLOR: usize = 4;
const BYTES_PER_COLOR: usize = 3;
const COLORS_PER_CELL: usize = 4;

impl Terrain {
    pub fn new<P1: Into<PathBuf>, P2: Into<PathBuf>>(height_file: P1, color_file: P2) -> Terrain {
        Terrain { height_file: height_file.into(), color_file: color_file.into(), data: None, height_handle: None, color_handle: None }
    } 

    fn from_raw_unchecked(gfx :&mut Graphics, terrain_renderer: &TerrainRenderer, height_data: Vec<u8>, color_data: Vec<u8>, width: u32, height: u32) -> Result<TerrainInner, TerrainFromRawError> {
        let height_texture = TextureBuilder::from_raw_pixels(height_data.clone(), width + 1, height + 1, Format::R8_UNORM)
            .min_filter(Filter::Nearest)
            .mag_filter(Filter::Nearest)
            .finish(gfx)?;
        let color_texture = TextureBuilder::from_raw_pixels(color_data.clone(), width * 2, height * 2, Format::R8G8B8A8_SRGB)
            .min_filter(Filter::Nearest)
            .mag_filter(Filter::Nearest)
            .finish(gfx)?;

        let vertex_shader = terrain_renderer.vertex_shader().clone();
        let fragment_shader = terrain_renderer.fragment_shader().clone();

        let pipeline = PipelineBuilder::new(gfx)
            .vertex_shader(vertex_shader)
            .fragment_shader(fragment_shader)
            .vertex_data(VERTEX_DATA.to_vec(), INDEX_DATA.to_vec())?
            .add_uniform_buffer(0, VertexUniforms::default(), BufferType::Dynamic)?
            .add_uniform_buffer(1, FragmentUniforms::default(), BufferType::Dynamic)?
            .add_texture(2, height_texture.clone())
            .add_texture(3, color_texture.clone())
            .add_texture(4, terrain_renderer.noise_texture().clone())
            .finish()?;

        Ok(TerrainInner { height_data, color_data, width, height, height_texture, color_texture, pipeline, height_dirty: false, color_dirty: false })
    }

    fn from_raw(gfx :&mut Graphics, terrain_renderer: &TerrainRenderer, height_data: Vec<u8>, color_data: Vec<u8>, width: u32, height: u32) -> Result<TerrainInner, TerrainFromRawError> {
        // Height data is per corner, rather than per cell, so each dimension needs one extra value to represent all corners
        if height_data.len() != ((width + 1) * (height + 1)) as usize {
            panic!("Height data size does not match given dimensions. ({})", height_data.len()); // TODO: This should be an error, not a panic
        }

        if color_data.len() != (width * height) as usize * ALIGNED_BYTES_PER_COLOR * COLORS_PER_CELL {
            panic!("Color data size does not match given dimensions.");
        }

        Self::from_raw_unchecked(gfx, terrain_renderer, height_data, color_data, width, height)
    }

    pub fn get_raw_height(&self) -> Result<&[u8], Uninitialized> {
        let Some(TerrainInner { height_data, .. }) = &self.data else { Err(Uninitialized)? };
        Ok(height_data)
    }

    pub fn get_raw_colors(&self) -> Result<&[u8], Uninitialized> {
        let Some(TerrainInner { color_data, .. }) = &self.data else { Err(Uninitialized)? };

        Ok(color_data)
    }

    pub fn get_cell_mut<'a>(&'a mut self, x: u32, z: u32) -> Result<TerrainCellMut<'a>, CellAccessError> {
        let Some(TerrainInner { height_data , color_data, width, height, height_dirty, color_dirty, .. }) = &mut self.data else { return Err(Uninitialized)? };
        if x >= *width || z >= *height {
            return Err(OutOfBounds { index: (x, z), bounds: (0,0)..(*width, *height)})?;
        }

        #[allow(unsafe_code, reason="All of these point to different elements of the array, so this should be fine. Using slice.split_at_mut to do the same thing was way too complicated.")]
        #[allow(clippy::unwrap_used, reason="Slice is made with correct length, so unwrap will never fail.")]
        unsafe {
            let ptr = color_data[..].as_mut_ptr();
            let i = (x * 2 + z * *width * 4) as usize * ALIGNED_BYTES_PER_COLOR; // spooky numbers
            let bottom_left_color = (std::slice::from_raw_parts_mut(ptr.add(i), BYTES_PER_COLOR)).try_into().unwrap();
            let bottom_right_color = (std::slice::from_raw_parts_mut(ptr.add(i + ALIGNED_BYTES_PER_COLOR), BYTES_PER_COLOR)).try_into().unwrap();
            let top_left_color = (std::slice::from_raw_parts_mut(ptr.add(i + *width as usize * ALIGNED_BYTES_PER_COLOR * 2), BYTES_PER_COLOR)).try_into().unwrap();
            let top_right_color = (std::slice::from_raw_parts_mut(ptr.add(i + *width as usize * ALIGNED_BYTES_PER_COLOR * 2 + ALIGNED_BYTES_PER_COLOR), BYTES_PER_COLOR)).try_into().unwrap();

            let height_data_width = *width + 1;

            let ptr = height_data[..].as_mut_ptr();
            let i = x + z * height_data_width;
            let bottom_left_height = &mut *(ptr.add(i as usize));

            let i = (x + 1) + z * height_data_width;
            let bottom_right_height = &mut *(ptr.add(i as usize));

            let i = x + (z + 1) * height_data_width;
            let top_left_height = &mut *(ptr.add(i as usize));

            let i = (x + 1) + (z + 1) * height_data_width;
            let top_right_height = &mut *(ptr.add(i as usize));
            
            Ok(TerrainCellMut {
                top_left_height,
                top_right_height,
                bottom_left_height,
                bottom_right_height,
                top_left_color,
                top_right_color,
                bottom_left_color,
                bottom_right_color,
                color_changed: color_dirty,
                height_changed: height_dirty
            })
        }

    }

    pub fn width(&self) -> Result<u32, Uninitialized> {
        let Some(TerrainInner { width, .. }) = &self.data else { return Err(Uninitialized)? };
        Ok(*width)
    }

    pub fn height(&self) -> Result<u32, Uninitialized> {
        let Some(TerrainInner { height, .. }) = &self.data else { return Err(Uninitialized)? };
        Ok(*height)
    }

    pub(in crate::engine::graphics::terrain) fn update_textures(&mut self, gfx: &Graphics) -> Result<(), UpdateTextureError> {
        let Some(TerrainInner { height_dirty, height_texture, height_data, color_data, color_dirty, color_texture, .. }) = &mut self.data else { return Err(Uninitialized)? };

        if *height_dirty {
            // Terrain enforces correct data buffer size, so this is safe
            height_texture.update_texture(gfx, height_data.clone())?;
            *height_dirty = false;
        }

        if *color_dirty {
            // Terrain enforces correct data buffer size, so this is safe
            color_texture.update_texture(gfx, color_data.clone())?;
            *color_dirty = false;
        }

        Ok(())
    }
}

impl Component for Terrain {
    fn init(&mut self, engine: &mut crate::engine::Engine, _: crate::engine::game_object::ObjectID) -> crate::error::any::Result<()> {
        self.height_handle = Some(engine.resource_manager.load(ImageLoader::<GrayImage>::new(), &self.height_file)?);
        self.color_handle = Some(engine.resource_manager.load(ImageLoader::<RgbaImage>::new(), &self.color_file)?);

        Ok(())
    }

    fn update(&mut self, engine: &mut crate::engine::Engine, _owner: crate::engine::game_object::ObjectID, _delta_time: f32) -> crate::error::any::Result<()> {
        if self.data.is_none() {
            let can_take = 
                self.height_handle.as_ref().is_some_and(|handle| handle.can_take())
                && self.color_handle.as_ref().is_some_and(|handle| handle.can_take());

            if !can_take {
                return Ok(());
            }

            let height_handle = {#[allow(clippy::unwrap_used, reason = "Handle is already known to be Some.")] self.height_handle.take().unwrap()};
            let color_handle = {#[allow(clippy::unwrap_used, reason = "Handle is already known to be Some.")] self.color_handle.take().unwrap()};

            let grid = {#[allow(clippy::unwrap_used, reason = "Resource cannot be none if take() was successful.")] color_handle.take().ok().try_unwrap()?.unwrap()?};
            let height_map = {#[allow(clippy::unwrap_used, reason = "Resource cannot be none if take() was successful.")] height_handle.take().ok().try_unwrap()?.unwrap()?};

            // Height map uses offset pixel grid, so it ends up being +1 in each dimension.
            let (width, height) = (height_map.width() - 1, height_map.height() - 1);

            self.data = Some(Self::from_raw(&mut engine.gfx, &engine.terrain_renderer, height_map.into_raw(), grid.into_raw(), width, height)?);
        }

        self.update_textures(&engine.gfx)?;

        let Some(TerrainInner { width, height, pipeline, .. }) = &self.data else { Err(Uninitialized)? };
        engine.terrain_renderer.queue_terrain(*width, *height, *pipeline);
        Ok(())
    }

    fn on_remove(&mut self, _engine: &mut crate::engine::Engine, _owner: crate::engine::game_object::ObjectID) -> crate::error::any::Result<()> {
        unimplemented!("Terrain on_remove unimplemented.");
    }

    fn priority(&self) -> &'static i32 { &0 }
}

pub mod error {
    use error::union;
    use crate::{engine::graphics::{error::PipelineBuilderError, texture::error::TextureBuilderError}, error::{self as errors_module, OutOfBounds}};
    use vulkano::{Validated, buffer::AllocateBufferError};

    use crate::{engine::graphics::error::BufferImageError, error::Uninitialized};

    union!(#[use_debug] Validated<AllocateBufferError>, PipelineBuilderError, TextureBuilderError as TerrainFromRawError);
    union!(OutOfBounds<(u32, u32)>, Uninitialized as CellAccessError);
    union!(Uninitialized, BufferImageError as UpdateTextureError);
}