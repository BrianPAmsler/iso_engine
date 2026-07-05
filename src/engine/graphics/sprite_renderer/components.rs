use std::{path::{Path, PathBuf}, sync::Arc};

use crate::{engine::{Engine, game_object::{ObjectID, component::Component}, graphics::sprite_renderer::{AnimatedSpriteID, SpriteDefinition, SpriteSheetID, animated_sprite::AnimatedSpriteData}, resources::{ResourceHandle, resource_loaders::ImageLoader}}, error::{DynamicMessageErorr, MessageErorr, Result, TryUnwrap as _, Uninitialized}};
use derive_serialize::Serialize;
use crate::engine::gl_types::vectors::Vec2;

use image::{ImageError, RgbaImage};

use super::SpriteData;

#[derive(Serialize)]
pub struct SpriteSheet {
    id: Option<SpriteSheetID>,
    #[serialized]
    resource: PathBuf,
    resource_handle: Option<ResourceHandle<RgbaImage, Arc<ImageError>>>,
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

impl Component for SpriteSheet {
    fn init(&mut self, engine: &mut Engine, _: ObjectID) -> crate::error::any::Result<()> {
        self.resource_handle = Some(engine.resource_manager.load(ImageLoader, &self.resource)?);

        Ok(())
    }

    fn update(&mut self, engine: &mut Engine, _: ObjectID, _: f32) -> crate::error::any::Result<()> {
        let Some(resource) = self.resource_handle.take_if(|resource| resource.can_take()) else { return Ok(()) };

        let Some(result) = resource.take().ok().unwrap() else { return Ok(()) };
        let image = result?;
        engine.sprite_renderer.add_sprite_sheet(self.resource.to_str().unwrap(), &mut engine.gfx, 1024, image, &self.sprite_definitions)?;

        Ok(())
    }

    fn on_remove(&mut self, engine: &mut Engine, _: ObjectID) -> crate::error::any::Result<()> {
        engine.sprite_renderer.remove_sprite_sheet(&mut engine.gfx, self.id.ok_or(Uninitialized)?);

        Ok(())
    }

    fn priority(&self) -> &'static i32 {
        &i32::MIN
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
    fn update(&mut self, engine: &mut Engine, owner: ObjectID, _delta_time: f32) -> crate::error::any::Result<()> {
        if self.sprite_sheet_id.is_none() {
            self.sprite_sheet_id = match engine.sprite_renderer.get_sprite_sheet_by_name(&self.name) {
                None => return Ok(()),
                v => v
            };
        }

        let Some(sprite_sheet) = self.sprite_sheet_id else { return Ok(()) };

        let transform = engine.world.get_transform(owner)?;

        engine.sprite_renderer.queue_sprite_instance(
            SpriteData { position: *transform.position(), anchor: self.anchor, dimensions: transform.scale().xy(), sprite_id: self.sprite_index },
            sprite_sheet
        );

        Ok(())
    }

    fn priority(&self) -> &'static i32 {
        &i32::MAX
    }
}

#[derive(Serialize)]
pub struct AnimatedSprite {
    #[serialized]
    name: String,
    pub anchor: Vec2,
    pub current_frame: f32,
    pub framerate: f32,
    pub paused: bool,
    animated_sprite_id: Option<AnimatedSpriteID>,
    total_frames: u32,
}

impl AnimatedSprite {
    pub fn new<S: ToOwned<Owned = String>>(name: S, anchor: Vec2, framerate: f32) -> AnimatedSprite {
        let name = name.to_owned();
        AnimatedSprite { name, anchor, current_frame: 0.0, framerate, paused: false, animated_sprite_id: None, total_frames: 0  }
    }
}

impl Component for AnimatedSprite {
    fn update(&mut self, engine: &mut Engine, owner: ObjectID, delta_time: f32) -> crate::error::any::Result<()> {
        if self.animated_sprite_id.is_none() {
            let id = match engine.sprite_renderer.get_animated_sprite_by_name(&self.name) {
                None => return Ok(()),
                Some(v) => v
            };
            self.animated_sprite_id = Some(id);
            
            self.total_frames = engine.sprite_renderer.get_total_frames(id).unwrap();
        }

        let Some(id) = self.animated_sprite_id else { unreachable!() };
        
        let transform = engine.world.get_transform(owner)?;

        if !self.paused {
            let advance_frames = delta_time * self.framerate;
            self.current_frame = (self.current_frame + advance_frames) % self.total_frames as f32;
        }

        let frame = self.current_frame as u32;
        engine.sprite_renderer.queue_animated_sprite_instance(
            id,
            AnimatedSpriteData {
                position: *transform.position(),
                anchor: self.anchor,
                dimensions: transform.scale().xy(),
                frame
            },
        );

        Ok(())
    }

    fn priority(&self) -> &'static i32 {
        &i32::MAX
    }
}

#[derive(Serialize)]
pub struct AnimatedSpriteLoader {
    #[serialized]
    name: String,
    #[serialized]
    frames_dir: PathBuf,
    sprite_id: Option<AnimatedSpriteID>
}

impl AnimatedSpriteLoader {
    pub fn new<S: Into<String>, P: Into<PathBuf>>(name: S, frames_dir: P) -> AnimatedSpriteLoader {
        let name = name.into();
        let frames_dir = frames_dir.into();
        AnimatedSpriteLoader { name, frames_dir, sprite_id: None }
    }
}

impl Component for AnimatedSpriteLoader {
    fn init(&mut self, engine: &mut Engine, _owner: ObjectID) -> crate::error::any::Result<()> {
        // let AnimatedSpriteLoaderEnum::Uninitialized { name, frames_dir } = self.inner.take() else { unreachable!("init called twice") };

        // let frames = std::fs::read_dir(frames_dir)?
        //     .filter_map(|result| result.ok())
        //     .map(|entry| entry.path())
        //     .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("png"))
        //     .map(|path| {
        //         Ok::<_, ImageError>(image::open(path)?.into_rgba8())
        //     })
        //     .try_collect()
        //     ?;

        // self.inner = AnimatedSpriteLoaderEnum::Initialized(engine.sprite_renderer.add_animated_sprite(&mut engine.gfx, name, frames)?);

        Ok(())
    }

    fn on_remove(&mut self, engine: &mut Engine, _owner: ObjectID) -> crate::error::any::Result<()> {
        // let AnimatedSpriteLoaderEnum::Initialized(id) = self.inner.take() else { unreachable!("uninitialized.") };

        // engine.sprite_renderer.remove_animated_sprite(&mut engine.gfx, id);

        Ok(())
    }
}