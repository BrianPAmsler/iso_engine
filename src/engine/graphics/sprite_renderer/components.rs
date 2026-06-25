use std::path::Path;

use crate::{engine::{Engine, game_object::{ObjectID, component::Component}, graphics::sprite_renderer::{AnimatedSpriteID, SpriteDefinition, SpriteSheetID, animated_sprite::AnimatedSpriteData}}, error::{DynamicMessageErorr, ExplicitUnwrap, Result, TryUnwrap as _, Uninitialized}};
use gl_types::vectors::Vec2;

use gl_types::{vec2};
use image::ImageError;
use itertools::Itertools;

use super::SpriteData;

pub struct SpriteSheet {
    id: Option<SpriteSheetID>,
    filename: Option<String>,
    sprite_definitions: Vec<SpriteDefinition>,
    count: usize
}

impl SpriteSheet {
    pub fn new(file_name: &str) -> SpriteSheet {
        SpriteSheet { id: None, filename: Some(file_name.to_owned()), sprite_definitions: Vec::new(), count: 0 }
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
    fn init(&mut self, engine: &mut Engine, _owner: ObjectID) -> crate::error::any::Result<()> {
        let path = Path::new(self.filename.as_ref().ok_or(Uninitialized)?);
        let sprite_sheet = image::open(path)?;
        let sprite_map = std::mem::take(&mut self.sprite_definitions);
        let name = path.file_name().and_then(|path| path.to_str());

        // If add_sprite_sheet returns None it should panic, so rewrap the unwrapped result.
        self.id = Some(engine.sprite_renderer.add_sprite_sheet(name.try_unwrap()?, &mut engine.gfx, 1024, sprite_sheet, &sprite_map)?);
        self.filename = None;

        Ok(())
    }

    fn fixed_update(&mut self, _engine: &mut Engine, _owner: ObjectID, _delta_time: f32) -> crate::error::any::Result<()> { Ok(()) }

    fn on_remove(&mut self, engine: &mut Engine, _owner: ObjectID) -> crate::error::any::Result<()> {
        engine.sprite_renderer.remove_sprite_sheet(&mut engine.gfx, self.id.ok_or(Uninitialized)?);

        Ok(())
    }

    fn priority(&self) -> &'static i32 {
        &i32::MIN
    }
}

enum SpriteSheetEnum {
    ID(SpriteSheetID),
    Name(String)
}

pub struct Sprite {
    sprite_sheet_id: SpriteSheetEnum,
    pub anchor: Vec2,
    sprite_index: u32
}

impl Sprite {
    pub fn new(sprite_sheet_name: &str, sprite_index: u32) -> Sprite {
        Sprite {
            sprite_sheet_id: SpriteSheetEnum::Name(sprite_sheet_name.to_owned()),
            anchor: Vec2::ZERO,
            sprite_index
        }
    }
}

impl Component for Sprite {
    fn init(&mut self, engine: &mut Engine, _owner: ObjectID) -> crate::error::any::Result<()> {
        self.sprite_sheet_id = SpriteSheetEnum::ID(match &self.sprite_sheet_id {
            SpriteSheetEnum::ID(_) => panic!("no"),
            SpriteSheetEnum::Name(name) => engine.sprite_renderer.get_sprite_sheet_by_name(name).ok_or(DynamicMessageErorr(format!("Sprite sheet \"{}\" not found.", name)))?,
        });

        Ok(())
    }

    fn update(&mut self, engine: &mut Engine, owner: ObjectID, _delta_time: f32) -> crate::error::any::Result<()> {
        let SpriteSheetEnum::ID(sprite_sheet) = self.sprite_sheet_id else { return Ok(()); };
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

enum AnimatedSpriteEnum {
    ID(AnimatedSpriteID),
    Name(String)
}

pub struct AnimatedSprite {
    animated_sprite_id: AnimatedSpriteEnum,
    pub anchor: Vec2,
    pub current_frame: f32,
    framerate: f32,
    total_frames: u32,
    pub paused: bool,
    pub interpolate: bool
}

impl AnimatedSprite {
    pub fn new(name: &str, framerate: f32) -> AnimatedSprite {
        AnimatedSprite { animated_sprite_id: AnimatedSpriteEnum::Name(name.to_owned()), anchor: vec2!(0), current_frame: 0.0, framerate, total_frames: 0, paused: false, interpolate: false }
    }
}

impl Component for AnimatedSprite {
    fn init(&mut self, engine: &mut Engine, _owner: ObjectID) -> crate::error::any::Result<()> {
        self.animated_sprite_id = AnimatedSpriteEnum::ID(match &self.animated_sprite_id {
            AnimatedSpriteEnum::ID(_) => panic!("no"),
            AnimatedSpriteEnum::Name(name) => engine.sprite_renderer.get_animated_sprite_by_name(name).ok_or(DynamicMessageErorr(format!("Animated sprite \"{}\" not found.", name)))?,
        });

        let AnimatedSpriteEnum::ID(id) = self.animated_sprite_id else { unreachable!() };

        self.total_frames = engine.sprite_renderer.get_total_frames(id).explicit_unwrap();

        Ok(())
    }

    fn update(&mut self, engine: &mut Engine, owner: ObjectID, delta_time: f32) -> crate::error::any::Result<()> {
        let AnimatedSpriteEnum::ID(sprite) = self.animated_sprite_id else { return Ok(()); };
        let transform = engine.world.get_transform(owner)?;

        if !self.paused {
            let advance_frames = delta_time * self.framerate;
            self.current_frame = (self.current_frame + advance_frames) % self.total_frames as f32;
        }

        let frame = self.current_frame as u32;
        engine.sprite_renderer.queue_animated_sprite_instance(
            sprite,
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

enum AnimatedSpriteLoaderEnum {
    Uninitialized {
        name: String,
        frames_dir: String
    },
    Initialized(AnimatedSpriteID),
    Null
}

impl AnimatedSpriteLoaderEnum {
    pub fn take(&mut self) -> AnimatedSpriteLoaderEnum {
        let mut out = AnimatedSpriteLoaderEnum::Null;

        std::mem::swap(self, &mut out);

        out
    }
}

pub struct AnimatedSpriteLoader {
    inner: AnimatedSpriteLoaderEnum
}

impl AnimatedSpriteLoader {
    pub fn new<S1: Into<String>, S2: Into<String>>(name: S1, frames_dir: S2) -> AnimatedSpriteLoader {
        let name = name.into();
        let frames_dir = frames_dir.into();
        AnimatedSpriteLoader { inner: AnimatedSpriteLoaderEnum::Uninitialized { name, frames_dir } }
    }
}

impl Component for AnimatedSpriteLoader {
    fn init(&mut self, engine: &mut Engine, _owner: ObjectID) -> crate::error::any::Result<()> {
        let AnimatedSpriteLoaderEnum::Uninitialized { name, frames_dir } = self.inner.take() else { unreachable!("init called twice") };

        let frames = std::fs::read_dir(frames_dir)?
            .filter_map(|result| result.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("png"))
            .map(|path| {
                Ok::<_, ImageError>(image::open(path)?.into_rgba8())
            })
            .try_collect()
            ?;

        self.inner = AnimatedSpriteLoaderEnum::Initialized(engine.sprite_renderer.add_animated_sprite(&mut engine.gfx, name, frames)?);

        Ok(())
    }

    fn on_remove(&mut self, engine: &mut Engine, _owner: ObjectID) -> crate::error::any::Result<()> {
        let AnimatedSpriteLoaderEnum::Initialized(id) = self.inner.take() else { unreachable!("uninitialized.") };

        engine.sprite_renderer.remove_animated_sprite(&mut engine.gfx, id);

        Ok(())
    }
}