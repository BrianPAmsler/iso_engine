use std::path::PathBuf;

use crate::{engine::{Engine, game_object::{ObjectID, component::Component}, graphics::{sprite_renderer::{AnimatedSpriteID, SpriteDefinition, SpriteSheetID, animated_sprite::AnimatedSpriteData}, texture::Texture}, resources::{ResourceHandle, resource_loaders::{TextureArrayLoader, TextureLoader, error::{TextureArrayLoadError, TextureLoadError}}}}, error::{Result, TryUnwrap, Uninitialized}, vec2};
use derive_serialize::Serialize;
use crate::engine::gl_types::vectors::Vec2;

use image::RgbaImage;

use super::SpriteData;

#[derive(Serialize)]
pub struct SpriteSheet {
    id: Option<SpriteSheetID>,
    #[serialized]
    resource: PathBuf,
    resource_handle: Option<ResourceHandle<Texture, TextureLoadError>>,
    #[serialized]
    sprite_definitions: Vec<SpriteDefinition>,
    count: usize
}

impl SpriteSheet {
    pub fn new<P: Into<PathBuf>>(resource: P) -> SpriteSheet {
        SpriteSheet { id: None, resource: resource.into(), resource_handle: None, sprite_definitions: Vec::new(), count: 0 }
    }

    pub fn add_sprite(&mut self, x: u32, y: u32, width: u32, height: u32) -> u32 {
        let idx = self.sprite_definitions.len() + self.count;

        self.sprite_definitions.push(SpriteDefinition {
            x,
            y,
            width,
            height,
        });

        idx as u32
    }
}

impl SpriteSheet {
    pub fn id(&self) -> Result<SpriteSheetID, Uninitialized> {
        Ok(self.id.ok_or(Uninitialized)?)
    }
}

// impl AsSerialize for SpriteSheet {
//     fn as_serialize(&self) -> Option<&dyn crate::engine::resources::serialization::Serialize> {
//         Some(self as &dyn crate::engine::resources::serialization::Serialize)
//     }
// }

impl Component for SpriteSheet {
    const PRIORITY: i32 = i32::MIN;

    fn init(&mut self, engine: &mut Engine, _: ObjectID) -> crate::error::any::Result<()> {
        self.resource_handle = Some(engine.resource_manager.load_file(TextureLoader::<RgbaImage>::new(&engine.gfx), &self.resource)?);

        Ok(())
    }

    fn update(&mut self, engine: &mut Engine, _: ObjectID, _: f32) -> crate::error::any::Result<()> {
        let Some(resource) = self.resource_handle.take_if(|resource| resource.can_take()) else { return Ok(()) };

        #[allow(clippy::unwrap_used, reason="Resource can_take() checked.")]
        let Some(result) = resource.take().ok().unwrap() else { return Ok(()) };
        let image = result?;
        engine.sprite_renderer.add_sprite_sheet(self.resource.to_string_lossy(), &mut engine.gfx, 1024, image, &self.sprite_definitions)?;

        Ok(())
    }

    fn on_remove(&mut self, engine: &mut Engine, _: ObjectID) -> crate::error::any::Result<()> {
        engine.sprite_renderer.remove_sprite_sheet(&mut engine.gfx, self.id.ok_or(Uninitialized)?);

        Ok(())
    }
}

#[derive(Serialize)]
pub struct Sprite {
    #[serialized]
    name: String,
    pub anchor: Vec2,
    pub sprite_index: u32,
    sprite_sheet_id: Option<SpriteSheetID>,
}

impl Sprite {
    pub fn new(sprite_sheet_name: &str, sprite_index: u32) -> Sprite {
        Sprite {
            name: sprite_sheet_name.to_owned(),
            sprite_sheet_id: None,
            anchor: Vec2::ZERO,
            sprite_index
        }
    }
}

impl Component for Sprite {
    const PRIORITY: i32 = i32::MAX;

    fn update(&mut self, engine: &mut Engine, owner: ObjectID, _delta_time: f32) -> crate::error::any::Result<()> {
        if self.sprite_sheet_id.is_none() {
            self.sprite_sheet_id = match engine.sprite_renderer.get_sprite_sheet_by_name(&self.name) {
                None => return Ok(()),
                v => v
            };
        }

        let Some(sprite_sheet) = self.sprite_sheet_id else { return Ok(()) };

        let object = engine.world.borrow_game_object(owner)?;

        engine.sprite_renderer.queue_sprite_instance(
            SpriteData { position: object.position, anchor: self.anchor, dimensions: object.scale.xy(), sprite_id: self.sprite_index },
            sprite_sheet
        );

        Ok(())
    }
}

#[derive(Serialize)]
pub struct AnimatedSprite {
    #[serialized]
    name: String,
    pub anchor: Vec2,
    #[non_serialized]
    pub current_frame: f32,
    pub framerate: f32,
    pub paused: bool,
    animated_sprite_id: Option<AnimatedSpriteID>,
    total_frames: u32,
}

impl AnimatedSprite {
    pub fn new<S: Into<String>>(name: S, framerate: f32) -> AnimatedSprite {
        let name = name.into();
        AnimatedSprite { name, anchor: vec2!(0, 0), current_frame: 0.0, framerate, paused: false, animated_sprite_id: None, total_frames: 0  }
    }
}

impl Component for AnimatedSprite {
    const PRIORITY: i32 = i32::MAX;
    
    fn update(&mut self, engine: &mut Engine, owner: ObjectID, delta_time: f32) -> crate::error::any::Result<()> {
        if self.animated_sprite_id.is_none() {
            let id = match engine.sprite_renderer.get_animated_sprite_by_name(&self.name) {
                None => return Ok(()),
                Some(v) => v
            };
            self.animated_sprite_id = Some(id);
            
            #[allow(clippy::unwrap_used, reason="id must be valid here.")]
            {self.total_frames = engine.sprite_renderer.get_total_frames(id).unwrap();}
        }

        let Some(id) = self.animated_sprite_id else { unreachable!() };
        
        let transform = engine.world.borrow_game_object(owner)?;

        if !self.paused {
            let advance_frames = delta_time * self.framerate;
            self.current_frame = (self.current_frame + advance_frames) % self.total_frames as f32;
        }

        let frame = self.current_frame as u32;
        engine.sprite_renderer.queue_animated_sprite_instance(
            id,
            AnimatedSpriteData {
                position: transform.position,
                anchor: self.anchor,
                dimensions: transform.scale.xy(),
                frame
            },
        );

        Ok(())
    }
}

#[derive(Serialize)]
pub struct AnimatedSpriteLoader {
    #[serialized]
    name: String,
    #[serialized]
    frames_dir: PathBuf,
    sprite_id: Option<AnimatedSpriteID>,
    resource: Option<ResourceHandle<Texture, TextureArrayLoadError>>
}

impl AnimatedSpriteLoader {
    pub fn new<S: Into<String>, P: Into<PathBuf>>(name: S, frames_dir: P) -> AnimatedSpriteLoader {
        let name = name.into();
        let frames_dir = frames_dir.into();
        AnimatedSpriteLoader { name, frames_dir, sprite_id: None, resource: None }
    }
}

impl Component for AnimatedSpriteLoader {
    fn init(&mut self, engine: &mut Engine, _: ObjectID) -> crate::error::any::Result<()> {
        let resource = engine.resource_manager.load_dir(TextureArrayLoader::<RgbaImage>::new(&engine.gfx), &self.frames_dir)?;

        self.resource = Some(resource);

        Ok(())
    }

    fn update(&mut self, engine: &mut Engine, _: ObjectID, _: f32) -> crate::error::any::Result<()> {
        if self.sprite_id.is_some() { return Ok(()) }

        let can_take = self.resource.as_ref().is_some_and(ResourceHandle::can_take);

        if !can_take { return Ok(()) }

        let resource = {#[allow(clippy::unwrap_used, reason = "resoures is known to be Some")] self.resource.take().unwrap()};

        let frames = resource.take().ok().try_unwrap()?.try_unwrap()??;

        self.sprite_id = Some(engine.sprite_renderer.add_animated_sprite(&mut engine.gfx, self.name.clone(), frames)?);

        Ok(())
    }

    fn on_remove(&mut self, engine: &mut Engine, _owner: ObjectID) -> crate::error::any::Result<()> {
        let Some(id) = self.sprite_id.take() else { return Ok(()) };

        engine.sprite_renderer.remove_animated_sprite(&mut engine.gfx, id);

        Ok(())
    }
}