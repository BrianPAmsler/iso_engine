use std::{collections::VecDeque, i32, path::Path};

use gl_types::{vec2, vec3};

use crate::engine::{Engine, errors::Result, game_object::{ObjectID, component::Component}, graphics::{image::Image, sprite_renderer::SpriteSheetID}};

use super::SpriteData;

pub struct SpriteSheet {
    id: Option<SpriteSheetID>,
    filename: Option<String>,
    sprite_definitions: VecDeque<(u32, u32, u32, u32)>,
    count: usize
}

impl SpriteSheet {
    pub fn new(file_name: &str) -> SpriteSheet {
        SpriteSheet { id: None, filename: Some(file_name.to_owned()), sprite_definitions: VecDeque::new(), count: 0 }
    }

    pub fn add_sprite(&mut self, x: u32, y: u32, width: u32, height: u32) -> u32 {
        let idx = self.sprite_definitions.len() + self.count;

        self.sprite_definitions.push_back((x, y, width, height));

        idx as u32
    }
}

impl SpriteSheet {
    pub fn id(&self) -> SpriteSheetID {
        self.id.expect("Sprite sheet not initialized.")
    }
}

impl Component for SpriteSheet {
    fn init(&mut self, engine: &mut Engine, _owner: ObjectID) -> Result<()> {
        let path = Path::new(self.filename.as_ref().unwrap());
        let sprite_map = Image::load_from_file(path)?;

        // If add_sprite_sheet returns None it should panic, so rewrap the unwrapped result.
        self.id = Some(engine.sprite_renderer.add_sprite_sheet(path.file_name().unwrap().to_str().unwrap(), &engine.gfx, 1024, sprite_map).unwrap());
        self.filename = None;

        Ok(())
    }

    fn fixed_update(&mut self, engine: &mut Engine, _owner: ObjectID, _delta_time: f32) -> Result<()> {
        let mut update = false;
        while !self.sprite_definitions.is_empty() {
            let (x, y, width, height) = self.sprite_definitions.pop_front().unwrap();
            engine.sprite_renderer.add_sprite(self.id.unwrap(), x, y, width, height);
            self.count += 1;
            update = true;
        }

        if update {
            engine.sprite_renderer.update_sprite_map(&engine.gfx, self.id.unwrap());
        }

        Ok(())
    }

    fn on_remove(&mut self, engine: &mut Engine, _owner: ObjectID) -> Result<()> {
        engine.sprite_renderer.remove_sprite_sheet(&engine.gfx, self.id.unwrap());

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
    pub data: SpriteData
}

impl Sprite {
    pub fn new(sprite_sheet_name: &str, sprite_index: u32) -> Sprite {
        Sprite {
            sprite_sheet_id: SpriteSheetEnum::Name(sprite_sheet_name.to_owned()),
            data: SpriteData { 
                position: vec3!(0),
                anchor: vec2!(0),
                dimensions: vec2!(1),
                sprite_id: sprite_index
            }
        }
    }
}

impl Component for Sprite {
    fn init(&mut self, engine: &mut Engine, _owner: ObjectID) -> Result<()> {
        self.sprite_sheet_id = SpriteSheetEnum::ID(match &self.sprite_sheet_id {
            SpriteSheetEnum::ID(_) => panic!("no"),
            SpriteSheetEnum::Name(name) => engine.sprite_renderer.get_sprite_sheet_by_name(name).ok_or(format!("Sprite sheet \"{}\" not found.", name))?,
        });

        Ok(())
    }

    fn update(&mut self, engine: &mut Engine, _owner: ObjectID, _delta_time: f32) -> Result<()> {
        let SpriteSheetEnum::ID(sprite_sheet) = self.sprite_sheet_id else { return Ok(()); };
        engine.sprite_renderer.queue_sprite_instance(
            self.data,
            sprite_sheet
        );

        Ok(())
    }

    fn priority(&self) -> &'static i32 {
        &i32::MAX
    }
}