

use std::{fs::{File, OpenOptions}, io::{BufReader, BufWriter, Read}, sync::Arc};

use derive_serialize::Serialize;
use opengl_engine::{engine::{gl_types::{geometric::normalize, vectors::{vec2, vec3}}, graphics::{sprite_renderer::components::{AnimatedSprite, AnimatedSpriteLoader}, terrain::Terrain}, resources::serialization::AsSerialize}, register_serializable_types};
use regex::Regex;

use opengl_engine::{engine::{Engine, WindowMode, game_object::{ObjectID, component::Component}, graphics::{Camera, Projection, sprite_renderer::components::{Sprite, SpriteSheet}}, input::Key}, error::{TryUnwrap, any::{Error, Result}}};
use serde_json::de::IoRead;

#[derive(Clone, Default, Serialize)]
pub struct FPSCounter {
    pub test: Arc<u32>,
    count: i64,
    fixed_count: i64,
    last_update: f32,
    last_fixed_update: f32
}

impl FPSCounter {
    pub fn new(count: i64, fixed_count: i64, last_update: f32, last_fixed_update: f32) -> FPSCounter {
        FPSCounter { test: Arc::new(0), count, fixed_count, last_update, last_fixed_update }
    }
}

impl Component for FPSCounter {
    fn init(&mut self, _: &mut Engine, _: ObjectID) -> Result<()> {
        println!("first frame");

        Ok(())
    }
    
    fn update(&mut self, engine: &mut Engine, _: ObjectID, _: f32) -> Result<()> {
        if engine.input.get_key_state(Key::Escape).press {
            engine.set_should_close(true);
        }
        
        if engine.input.get_key_state(Key::KeyO).press {
            let root = engine.world.get_root();
            let file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open("target/test_serialize.json")?;
            let writer = BufWriter::new(file);
            let mut serializer = serde_json::Serializer::new(writer);
            engine.world.serialize_object(root, &mut serializer)?.try_unwrap()?;

            println!("Saved.");
        }

        self.count += 1;
        let current_tick = engine.get_time() as f32;

        let delta = current_tick - self.last_update;

        if delta >= 1.0 {
            let fps = self.count as f32 / delta;
            println!("FPS: {}\n", fps);

            self.count = 0;
            self.last_update = current_tick;
        }

        Ok(())
    }

    fn fixed_update(&mut self, engine: &mut Engine, _: ObjectID, _: f32) -> Result<()> {

        self.fixed_count += 1;
        let current_tick = engine.get_time() as f32;

        let delta = current_tick - self.last_fixed_update;

        if delta >= 1.0 {
            let fps = self.fixed_count as f32 / delta;
            println!("Fixed FPS: {}\n", fps);

            self.fixed_count = 0;
            self.last_fixed_update = current_tick;
        }

        Ok(())
    }
}

#[derive(Serialize)]
pub struct Renderer {
    pub camera_size: f32,
    sprite1: Option<ObjectID>,
    sprite2: Option<ObjectID>
}

impl Renderer {
    pub fn new(camera_size: f32, sprite1: Option<ObjectID>, sprite2: Option<ObjectID>) -> Renderer {
        Renderer { camera_size, sprite1, sprite2 }
    }
}

impl Component for Renderer {
    fn init(&mut self, engine: &mut Engine, _: ObjectID) -> Result<()> {
        let sprite1 = engine.world.find_child(engine.world.get_root(), "Sprite 1")?.try_unwrap()?;
        let sprite2 = engine.world.find_child(engine.world.get_root(), "Sprite 2")?.try_unwrap()?;

        self.sprite1 = Some(sprite1);
        self.sprite2 = Some(sprite2);

        Ok(())
    }

    fn update(&mut self, engine: &mut Engine, _: ObjectID, delta_time: f32) -> Result<()> {
        let Some(camera) = engine.world.get_main_camera_mut() else { return Ok(()) };
        // println!("asdf");
        let speed = 10.0;

        if engine.input.get_key_state(Key::KeyW).is_down {
            let pos = camera.position();
            camera.set_position(pos + normalize(vec3!(1, 0, 1)) * delta_time * speed);
        }

        if engine.input.get_key_state(Key::KeyA).is_down {
            let pos = camera.position();
            camera.set_position(pos + normalize(vec3!(-1, 0, 1)) * delta_time * speed);
        }
        if engine.input.get_key_state(Key::KeyS).is_down {
            let pos = camera.position();
            camera.set_position(pos + normalize(vec3!(-1, 0, -1)) * delta_time * speed);
        }
        if engine.input.get_key_state(Key::KeyD).is_down {
            let pos = camera.position();
            camera.set_position(pos + normalize(vec3!(1, 0, -1)) * delta_time * speed);
        }
        if engine.input.get_key_state(Key::Space).is_down {
            let pos = camera.position();
            camera.set_position(pos + vec3!(0, 1, 0) * delta_time * speed);
        }
        if engine.input.get_key_state(Key::ControlLeft).is_down {
            let pos = camera.position();
            camera.set_position(pos + vec3!(0, -1, 0) * delta_time * speed);
        }

        let mut sprite = engine.world.get_transform(self.sprite2.try_unwrap()?)?;
        if engine.input.get_key_state(Key::ArrowUp).is_down {
            *sprite.position_mut() += vec3!(0, 0, 1) * delta_time * speed;
        }

        if engine.input.get_key_state(Key::ArrowLeft).is_down {
            *sprite.position_mut() += vec3!(-1, 0, 0) * delta_time * speed;
        }
        if engine.input.get_key_state(Key::ArrowDown).is_down {
            *sprite.position_mut() += vec3!(0, 0, -1) * delta_time * speed;
        }
        if engine.input.get_key_state(Key::ArrowRight).is_down {
            *sprite.position_mut() += vec3!(1, 0, 0) * delta_time * speed;
        }
        if engine.input.get_key_state(Key::ShiftRight).is_down {
            *sprite.position_mut() += vec3!(0, 1, 0) * delta_time * speed;
        }
        if engine.input.get_key_state(Key::ControlRight).is_down {
            *sprite.position_mut() += vec3!(0, -1, 0) * delta_time * speed;
        }

        if engine.input.get_key_state(Key::KeyT).press {
            let terrain = engine.world.find_child(engine.world.get_root(), "a")?.try_unwrap()?;
            let terrain = engine.world.get_component::<Terrain>(terrain)?.try_unwrap()?;
            let mut terrain = engine.world.borrow_component_mut(terrain)?;

            let mut cell = terrain.get_cell_mut(0, 0)?;
            *cell.bottom_left().color() = [255, 0, 0];
            *cell.bottom_right().color() = [0, 255, 0];
            *cell.top_left().color() = [0, 0, 255];
            *cell.top_right().color() = [255, 255, 255];
        }

        let Some(camera) = engine.world.get_main_camera_mut() else { return Ok(()) };
        self.camera_size -= engine.input.get_scroll_y() as f32;

        if let Projection::Orthographic { width, .. } = camera.projection_mut() { *width = self.camera_size }

        Ok(())   
    }

    fn priority(&self) -> &'static i32 {
        &-1
    }
}

fn start_game() -> Result<()> {
    let mut engine = Engine::new("Test Window", 1280, 720, WindowMode::Windowed)?;

    register_serializable_types!(FPSCounter, Renderer);

    let root = engine.world.get_root();
    let save_file = BufReader::new(File::open("target/save.json")?);
    let mut deserializer = serde_json::Deserializer::new(IoRead::new(save_file));
    engine.world.deserialize_object(root, &mut deserializer)?;

    // let a = engine.world.create_game_object("a", engine.world.get_root())?;

    // let mut sprite_sheet = SpriteSheet::new("assets/sprite_sheet.png");
    // sprite_sheet.add_sprite(0, 0, 512, 512);
    // sprite_sheet.add_sprite(512, 512, 1024, 1024);

    

    // engine.world.add_component(a, sprite_sheet)?;

    // let animation = AnimatedSpriteLoader::new("test animation", "assets/test frames");
    
    // engine.world.add_component(a, animation)?;
    
    // let sprite1 = engine.world.create_game_object("Sprite 1", engine.world.get_root())?;
    // let sprite2 = engine.world.create_game_object("Sprite 2", engine.world.get_root())?;
    // let sprite3 = engine.world.create_game_object("Sprite 3", engine.world.get_root())?;
    // let sprite4 = engine.world.create_game_object("Sprite 4", engine.world.get_root())?;

    // let mut transform = engine.world.get_transform(sprite3)?;
    // *transform.position_mut() = vec3!(2, 0, 0);

    // let mut transform = engine.world.get_transform(sprite4)?;
    // *transform.position_mut() = vec3!(0, 0, 2);
    // *transform.scale_mut() = vec3!(2, 2, 2);

    // let mut sprite_component1 = Sprite::new("assets/sprite_sheet.png", 0);
    // sprite_component1.anchor = vec2!(0.5, 0);
    // let mut sprite_component2 = Sprite::new("assets/sprite_sheet.png", 1);
    // sprite_component2.anchor = vec2!(0.5, 0);
    // let mut sprite_component3 = AnimatedSprite::new("test animation", 60.0);
    // sprite_component3.anchor = vec2!(0.5, 0);
    // let mut sprite_component4 = AnimatedSprite::new("test animation", 60.0);
    // sprite_component4.anchor = vec2!(0.5, 0);
    // sprite_component4.current_frame = 15.0;

    // engine.world.add_component(sprite1, sprite_component1)?;
    // engine.world.add_component(sprite2, sprite_component2)?;
    // engine.world.add_component(sprite3, sprite_component3)?;
    // engine.world.add_component(sprite4, sprite_component4)?;
    
    let camera = Camera::new(
        Projection::Orthographic {
            width: 1.0,
            aspect: 16.0 /9.0,
            z_near: -100.0,
            z_far: 100.0,
        },
        vec3!(0, 1, 0),
        normalize(vec3!(1, -1, 1)),
        vec3!(0, 1, 0)
    );

    engine.world.set_main_camera(camera);

    // let terrain = Terrain::new("assets/height_map.png", "assets/ground.png");
    // engine.world.add_component(a, terrain)?;

    // let renderer = Renderer { camera_size: 10.0, sprite1: None, sprite2: None  };

    // engine.world.add_component(a, FPSCounter::default())?;
    // engine.world.add_component(a, renderer)?;

    engine.run()?;

    Ok(())
}

fn main() {
    match start_game() {
        Ok(_) => {},
        Err(err) => { eprint!("{}", clean_backtrace(&err, "opengl_engine")); }
    }
}

#[allow(clippy::unwrap_used, reason="Regex is valid.")]
pub fn clean_backtrace(error: &Error, crate_name: &'static str) -> String {
    let str = format!("{:?}", error.backtrace());

    let mut clean_str = String::new();
    clean_str.reserve(str.len());

    clean_str += &format!("Error: {}\n\nStack Backtrace\n", error);
    
    let is_error_line = Regex::new("^ +[0-9]+:").unwrap();
    let in_crate = Regex::new(&format!("^ +[0-9]+: {}::", crate_name)).unwrap();

    let mut count = 0;
    let mut adding = false;
    for line in str.split('\n') {
        let result = is_error_line.find(line);

        if adding {
            if result.is_some() {
                adding = false;
            } else {
                clean_str += line;
                clean_str += "\n";
            }
        }
        if !adding {
            if let Some(line_number) = result {
                if in_crate.find(line).is_some() {
                    adding = true;
                    
                    let new_line = format!("   {}: ", count) + &line[line_number.end()..];
                    clean_str += &new_line;
                    clean_str += "\n";
    
                    count += 1;
                }
            }
        }
    }

    clean_str
}