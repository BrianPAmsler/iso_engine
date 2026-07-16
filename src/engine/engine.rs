use std::{collections::HashMap, sync::Arc, time::{Duration, Instant}};

use winit::{application::ApplicationHandler, dpi::{PhysicalPosition, PhysicalSize}, event::{ElementState, KeyEvent, WindowEvent}, event_loop::{self, EventLoop}, monitor::MonitorHandle, platform::pump_events::EventLoopExtPumpEvents, window::{Fullscreen, Window, WindowAttributes}};

use crate::{engine::{error::{InvalidWindowState, NewEngineErorr}, game_object::{RootComponent, World}, gl_types::{matrices::{Mat2, Mat3, Mat4}, vectors::{Vec2, Vec3, Vec4}}, graphics::{Graphics, sprite_renderer::{SpriteRenderer, components::{AnimatedSprite, AnimatedSpriteLoader, Sprite, SpriteSheet}}, terrain::{Terrain, terrain_renderer::TerrainRenderer}}, hidden::__ComponentRegistration, input::{self, Input, Key}, resources::ResourceManager}, error::{MessageErorr, Result, any::Error}, register_serializable_types};

#[derive(Debug)]
pub enum WindowMode {
    FullScreen(Option<MonitorHandle>),
    Windowed
}

impl From<WindowMode> for Option<Fullscreen> {
    fn from(val: WindowMode) -> Self {
        match val {
            WindowMode::FullScreen(monitor_handle) => Some(Fullscreen::Borderless(monitor_handle)),
            WindowMode::Windowed => None,
        }
    }
}   

pub struct Engine {
    pub gfx: Graphics,
    pub world: World,
    pub input: Input,
    pub resource_manager: ResourceManager,
    pub(in crate::engine) sprite_renderer: SpriteRenderer,
    pub(in crate::engine) terrain_renderer: TerrainRenderer,
    fixed_tick_duration: f64,
    error_queue: Vec<Error>,
    pub(in crate::engine) window: Arc<Window>,
    initialization_time: Instant,
    last_tick: f64,
    last_fixed_tick: f64,
    fixed_tick_overflow: f64,
    should_close: bool,
    pub(in crate::engine) component_vec_constructors: HashMap<std::any::TypeId, Box<dyn Fn() -> Box<dyn super::game_object::ComponentVec>>>,
    _event_loop: Option<EventLoop<()>>
}

impl ApplicationHandler for Engine {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn new_events(&mut self, _event_loop: &event_loop::ActiveEventLoop, _cause: winit::event::StartCause) {
        self.input.reset();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::KeyboardInput { event, .. } => {
                match event {
                    KeyEvent { physical_key, state: ElementState::Pressed, .. } => {
                        let key_state = self.input.modify_key_state(Key(physical_key));
                        key_state.press = true;
                        key_state.is_down = true;

                    },
                    KeyEvent { physical_key, state: ElementState::Released, .. } => {
                        let key_state = self.input.modify_key_state(Key(physical_key));
                        key_state.release = true;
                        key_state.is_down = false;
                    }
                }
            },
            WindowEvent::MouseInput { button, state, .. } => {
                let button = match button {
                    winit::event::MouseButton::Left => 1,
                    winit::event::MouseButton::Right => 2,
                    winit::event::MouseButton::Middle => 3,
                    winit::event::MouseButton::Back => 4,
                    winit::event::MouseButton::Forward => 5,
                    winit::event::MouseButton::Other(button) => button as u32 + 1, // Winit starts at 0 (I like starting at 1)
                };

                let key_state = self.input.modify_mouse_button_state(button);
                match state {
                    ElementState::Pressed => {
                        key_state.press = true;
                        key_state.is_down = true;
                    },
                    ElementState::Released => {
                        key_state.release = true;
                        key_state.is_down = false;
                    },
                }
            },
            WindowEvent::MouseWheel { delta, .. } => {
                let (x, y) = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => (x as f64, y as f64),
                    winit::event::MouseScrollDelta::PixelDelta(physical_position) => {
                        let PhysicalPosition { x, y } = physical_position;
                        (x / input::LINE_HEIGHT, y / input::LINE_HEIGHT)
                    },
                };

                self.input.add_scroll_delta(x, y);
            },
            WindowEvent::Resized(size) => {
                self.gfx.window_resized();
                let new_aspect = size.width as f32 / size.height as f32;
                if let Some(main_camera) = self.world.get_main_camera_mut() {
                    main_camera.update_aspect(new_aspect);
                }
            },
            WindowEvent::RedrawRequested => {
                if self.should_close {
                    event_loop.exit();
                    return;
                }

                #[allow(clippy::unwrap_used, reason="Any errors that are not handled by this point should crash the program.")]
                self.update().unwrap();

                self.window.request_redraw();
            }
            _ => ()
        }
    }
}

impl Engine {
    pub fn new(window_title: &str, width: u32, height: u32, window_mode: WindowMode, component_registry: ComponentRegistry) -> Result<Engine, NewEngineErorr> {
        let mut event_loop = EventLoop::new()?;
        event_loop.set_control_flow(event_loop::ControlFlow::Poll);

        let window_attributes = WindowAttributes::default()
            .with_title(window_title)
            .with_inner_size(PhysicalSize::new(width, height))
            .with_fullscreen(window_mode.into())
            .with_resizable(true);

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
                self.0 = WindowStatus::Initialized(#[allow(clippy::unwrap_used, reason="No way to pass a result, plus this error is likely unrecoverable anyway.")] event_loop.create_window(*window_attributes).unwrap());
            }
        
            fn window_event(&mut self, _: &event_loop::ActiveEventLoop, _: winit::window::WindowId, _: WindowEvent) {}
        }

        let mut app = WindowInitializer(WindowStatus::Uninitialized(Box::new(window_attributes)));
        
        event_loop.pump_app_events(Some(Duration::ZERO), &mut app);

        let WindowStatus::Initialized(window) = app.0 else { return Err(InvalidWindowState.into()) };
        let window = Arc::new(window);

        let world = World::new();

        let mut gfx = Graphics::new(window.clone(), &event_loop)?;
        let sprite_renderer = SpriteRenderer::new();
        let terrain_renderer = TerrainRenderer::new(&mut gfx)?;

        // TODO: write a build script that finds all serializable types
        register_serializable_types!(SpriteSheet, Sprite, AnimatedSprite, AnimatedSpriteLoader, Terrain, RootComponent, Mat2, Mat3, Mat4, Vec2, Vec3, Vec4);
        let mut component_vec_constructors = register_components!(SpriteSheet, Sprite, AnimatedSprite, AnimatedSpriteLoader, Terrain, RootComponent).0;
        component_vec_constructors.extend(component_registry.0);

        let engine = Engine {
            window,
            gfx,
            world,
            input: Input::new(),
            resource_manager: ResourceManager::new(),
            sprite_renderer,
            terrain_renderer,
            error_queue: Vec::new(),fixed_tick_duration: 1.0 / 60.0,
            initialization_time: Instant::now(),
            last_tick: 0.0,
            last_fixed_tick: 0.0,
            fixed_tick_overflow: 0.0,
            should_close: false,
            _event_loop: Some(event_loop),
            component_vec_constructors
        };

        Ok(engine)
    }

    pub fn run(&mut self) -> crate::error::any::Result<()> {
        let event_loop = self._event_loop.take().ok_or(MessageErorr("No event loop"))?;

        self.window.request_redraw();
        event_loop.run_app(self)?;

        Ok(())
    }

    pub fn get_time(&self) -> f64 {
        (Instant::now() - self.initialization_time).as_secs_f64()
    }

    fn update(&mut self) -> crate::error::any::Result<()> {
        // Game tick
        let current_time = self.get_time();
        let errors = World::update(self, (current_time - self.last_tick) as f32);
        self.error_queue.extend(errors);
        self.last_tick = current_time;

        let fixed_diff = current_time - self.last_fixed_tick - self.fixed_tick_duration;

        // Add overflow to adjust for errors in timing
        if fixed_diff + self.fixed_tick_overflow >= 0.0 {
            self.fixed_tick_overflow = f64::max(0.0, fixed_diff * 2.0);
            let errors = World::fixed_update(self, (current_time - self.last_fixed_tick) as f32);
            self.error_queue.extend(errors);
            self.last_fixed_tick = current_time;
        }

        // Cleanup
        let errors = World::cleanup(self);
        self.error_queue.extend(errors);

        // Render
        self.gfx.update_pipelines(&self.window)?;
        
        if let Some(camera) = self.world.get_main_camera_mut() {
            let view_matrix = camera.view_matrix();
            let projection_matrix = camera.projection_matrix();
            let position = camera.position();

            let result = self.sprite_renderer.update(&self.gfx, &view_matrix, &projection_matrix);
            self.log_error(result);

            let result = self.terrain_renderer.update(&self.gfx, view_matrix, projection_matrix, position);
            self.log_error(result);
        }

        self.gfx.draw()?;

        self.log_errors();

        Ok(())
    }

    #[inline]
    fn log_error<T, E: Into<Error>>(&mut self, result: std::result::Result<T, E>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(err) => {
                self.error_queue.push(err.into());
                None
            }
        }
    }

    fn log_errors(&mut self) {
        // For now just print errors to console
        let mut errors = std::mem::take(&mut self.error_queue);

        errors.iter_mut().for_each(|error| {
            eprintln!("{}", error)
        })
    }

    pub fn set_should_close(&mut self, should_close: bool) {
        self.should_close = should_close;
    }
}

pub mod error {
    use std::fmt::Debug;
    use winit::error::EventLoopError;

    use crate::engine::graphics::error::NewGraphicsError;
    use crate::engine::graphics::terrain::terrain_renderer::error::NewTerrainRendererError;
    use crate::error as errors_module;

    use crate::error::{Error, union};

    #[derive(Error, Debug)]
    #[error("Invalid window state.")]
    pub struct InvalidWindowState;

    union!(
        EventLoopError,
        NewGraphicsError,
        NewTerrainRendererError,
        InvalidWindowState
        as NewEngineErorr
    );
}

pub struct ComponentRegistry(HashMap<std::any::TypeId, Box<dyn Fn() -> Box<dyn super::game_object::ComponentVec>>>);

impl ComponentRegistry {
    #[doc(hidden)]
    pub fn __new(registrations: impl IntoIterator<Item = __ComponentRegistration>) -> ComponentRegistry {
        ComponentRegistry(registrations.into_iter().map(|r| (r.0, r.1)).collect())
    }
}

#[macro_export]
macro_rules! register_components {
    ($($type:ty),*) => {
        ::opengl_engine::engine::ComponentRegistry::__new([
            $(::opengl_engine::engine::hidden::__register_component::<$type>()),*
        ])
    };
}

pub use register_components;

#[doc(hidden)]
pub mod hidden {
    use std::{any::TypeId, cell::RefCell};

    use crate::engine::{data_structures::VecAllocator, game_object::{ComponentVec, ObjectID, component::Component}};

    pub struct __ComponentRegistration(pub(in crate::engine) TypeId, pub(in crate::engine) Box<dyn Fn() -> Box<dyn ComponentVec>>);

    pub fn __register_component<C: Component>() -> __ComponentRegistration {
        __ComponentRegistration(TypeId::of::<C>(), Box::new(|| Box::new(VecAllocator::<(ObjectID, RefCell<(bool, C)>)>::new())))
    }
}