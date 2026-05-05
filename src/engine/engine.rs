
use gl46::{GL_COLOR_BUFFER_BIT, GL_DEPTH_BUFFER_BIT};
use glfw::{Action, WindowEvent};

use crate::engine::{errors::{Error, Result}, graphics::sprite_renderer::SpriteRenderer};

use super::{game_object::World, graphics::Graphics, input::Input};

pub struct Engine {
    pub gfx: Graphics,
    pub world: World,
    pub input: Input,
    pub(in crate::engine) sprite_renderer: SpriteRenderer,
    fixed_tick_duration: f64,
    fixed_input: Input,
    error_queue: Vec<Error>
}

#[derive(Debug)]
pub enum WindowMode {
    FullScreen(Option<u64>),
    Windowed
}

impl Engine {
    pub fn create_window(window_title: &str, width: u32, height: u32, window_mode: WindowMode) -> Result<Engine> {
        let gfx = Graphics::init(window_title, width, height, window_mode)?;

        let world = World::new();

        let sprite_renderer = SpriteRenderer::new(&gfx)?;
        
        Ok(Engine { gfx, world, sprite_renderer, fixed_tick_duration: 1.0 / 60.0, error_queue: Vec::new(), input: Input::new(), fixed_input: Input::new() })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut last_tick = self.gfx.get_glfw_time();
        let mut last_fixed_tick = last_tick;
        let mut fixed_tick_overflow = 0.0;

        self.log_errors();

        while !self.gfx.should_close() {
            self.gfx.poll_events();
            for msg in self.gfx.flush_messages() {
                match msg {
                    (_, WindowEvent::Key(key, _, Action::Press, _)) => {
                        let key_state = self.input.modify_key_state(key);
                        key_state.press = true;
                        key_state.is_down = true;

                        let fixed_key_state = self.fixed_input.modify_key_state(key);
                        fixed_key_state.press = true;
                        fixed_key_state.is_down = true;
                    },
                    (_, WindowEvent::Key(key, _, Action::Release, _)) => {
                        let key_state = self.input.modify_key_state(key);
                        key_state.release = true;
                        key_state.is_down = false;

                        let fixed_key_state = self.fixed_input.modify_key_state(key);
                        fixed_key_state.release = true;
                        fixed_key_state.is_down = false;
                    },
                    (_, WindowEvent::MouseButton(button, Action::Press, _)) => {
                        let key_state = self.input.modify_mouse_button_state(button as u32);
                        key_state.press = true;
                        key_state.is_down = true;

                        let fixed_key_state = self.fixed_input.modify_mouse_button_state(button as u32);
                        fixed_key_state.press = true;
                        fixed_key_state.is_down = true;
                    },
                    (_, WindowEvent::MouseButton(button, Action::Release, _)) => {
                        let key_state = self.input.modify_mouse_button_state(button as u32);
                        key_state.release = true;
                        key_state.is_down = false;

                        let fixed_key_state = self.fixed_input.modify_mouse_button_state(button as u32);
                        fixed_key_state.release = true;
                        fixed_key_state.is_down = false;
                    },
                    (_, WindowEvent::Scroll(x, y)) => {
                        self.input.add_scroll_delta(x, y);
                        self.fixed_input.add_scroll_delta(x, y);
                    }
                    // (_, WindowEvent::Key(Key::Escape, _, Action::Press, _)) => gfx.set_should_close(true),
                    // (_, WindowEvent::Key(Key::Space, _, Action::Press, _)) => gfx.set_fullscreen(Monitor::from_primary()),
                    _ => ()
                }
            }

            // TODO: move clear call to after game tick

            self.gfx.glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
            
            // Game tick
            let current_time = self.gfx.get_glfw_time();
            World::update(self, (current_time - last_tick) as f32)?; // TODO: This is not supposed to crash, catch and log errors
            last_tick = current_time;

            self.input.modify_all_key_states(|key| {
                key.press = false;
                key.release = false;
            });
            self.input.modify_all_mouse_button_states(|button| {
                button.press = false;
                button.release = false;
            });
            self.input.set_scroll_delta(0.0, 0.0);

            let fixed_diff = current_time - last_fixed_tick - self.fixed_tick_duration;

            // Add overflow to adjust for errors in timing
            if fixed_diff + fixed_tick_overflow >= 0.0 {
                fixed_tick_overflow = f64::max(0.0, fixed_diff * 2.0);
                World::fixed_update(self, (current_time - last_fixed_tick) as f32)?; // TODO: This is not supposed to crash, catch and log errors
                last_fixed_tick = current_time;

                self.fixed_input.modify_all_key_states(|key| {
                    key.press = false;
                    key.release = false;
                });
                self.fixed_input.modify_all_mouse_button_states(|button| {
                    button.press = false;
                    button.release = false;
                });
                self.fixed_input.set_scroll_delta(0.0, 0.0);
            }

            self.log_errors();
            match self.world.get_main_camera() {
                Some(camera) => {
                    let mut camera = camera.borrow_mut();
                    self.sprite_renderer.render(&self.gfx, &camera.view_matrix(), &camera.projection_matrix());
                },
                _ => ()
            }

            for (owner, mut component) in self.world.get_removed_components() {
                component.on_remove(self, owner)?; // TODO: This is not supposed to crash, catch and log errors
            }

            // Render
            // gfx.render();

            // Swap front and back buffers
            self.gfx.swap_buffers();
        }

        Ok(())
    }

    fn log_errors(&mut self) {
        // Take erorr queue from error_queue, turn it into a Box and log them
        let mut errors = Vec::new();
        std::mem::swap(&mut errors, &mut self.error_queue);
        let errors = errors.into_boxed_slice();

        errors.iter().for_each(|error| eprintln!("{}", error))
    }

    // fn init(&mut self) {
    //     let all_objs = self.world.get_root().get_all_children().unwrap_or_else(|err| {self.error_queue.push(err); Box::new([])});

    //     for obj in all_objs.to_vec().into_iter() {
    //         obj.init(&self).unwrap_or_else(|err| self.error_queue.push(err));
    //     }
    // }

    // fn game_tick(&mut self, delta_time: f32) {
    //     let all_objs = self.world.get_root().get_all_children().unwrap_or_else(|err| {self.error_queue.push(err); Box::new([])});

    //     for obj in all_objs.to_vec().into_iter() {
    //         obj.update(&self, delta_time).unwrap_or_else(|err| self.error_queue.push(err));
    //     }
    // }

    // fn fixed_game_tick(&mut self, delta_time: f32) {
    //     let all_objs = self.world.get_root().get_all_children().unwrap_or_else(|err| {self.error_queue.push(err); Box::new([])});

    //     for obj in all_objs.to_vec().into_iter() {
    //         obj.fixed_update(&self, delta_time).unwrap_or_else(|err| self.error_queue.push(err));
    //     }
    // }
}