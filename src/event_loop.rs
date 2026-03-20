use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    window::WindowId,
};

use crate::State;

pub struct App {
    state: Option<State>,
}

impl App {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window_attributes = winit::window::Window::default_attributes();
            let window = event_loop.create_window(window_attributes).unwrap();
            self.state = Some(
                pollster::block_on(State::new(window, None, None, None, None, false, None))
                    .unwrap(),
            );
        }
        if let Some(state) = &mut self.state {
            state.last_time = web_time::Instant::now();
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else {
            return;
        };
        handle_window_event(&event, state, event_loop);
    }
}

#[allow(unused_variables)]
pub fn handle_window_event(event: &WindowEvent, state: &mut State, event_loop: &ActiveEventLoop) {
    match event {
        WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowDown),
                    state: ElementState::Pressed,
                    ..
                },
            ..
        } => {
            state.change_rule(true);
        }
        WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowUp),
                    state: ElementState::Pressed,
                    ..
                },
            ..
        } => {
            state.change_rule(false);
        }
        WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowLeft),
                    state: ElementState::Pressed,
                    ..
                },
            ..
        } => {
            state.set_initial_density(state.initial_density - 1);
        }
        WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowRight),
                    state: ElementState::Pressed,
                    ..
                },
            ..
        } => {
            state.set_initial_density(state.initial_density + 1);
        }
        WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    logical_key: winit::keyboard::Key::Character(c),
                    state: ElementState::Pressed,
                    ..
                },
            ..
        } => {
            if c == "f" || c == "F" {
                #[cfg(target_family = "wasm")]
                {
                    crate::web::toggle_fullscreen();
                }
                #[cfg(not(target_family = "wasm"))]
                {
                    if state.window.fullscreen().is_some() {
                        state.window.set_fullscreen(None);
                    } else {
                        state
                            .window
                            .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                    }
                }
            } else if c == "c" || c == "C" {
                #[cfg(target_family = "wasm")]
                crate::web::toggle_controls();
            } else if c == "i" || c == "I" {
                #[cfg(target_family = "wasm")]
                crate::web::download_image();
            } else if c == "r" || c == "R" {
                state.reset();
            } else if c == "q" || c == "Q" {
                state.set_generations_per_second(state.generations_per_second - 1);
            } else if c == "<" {
                state.set_initial_density(state.initial_density - 1);
            } else if c == ">" {
                state.set_initial_density(state.initial_density + 1);
            } else if c == "w" || c == "W" {
                state.set_generations_per_second(state.generations_per_second + 1);
            } else if c == "-" && state.cells_width < 2048 {
                let current_idx = State::ELIGIBLE_SIZES
                    .iter()
                    .position(|&s| s == state.cells_width)
                    .unwrap();
                let new_size = State::ELIGIBLE_SIZES[current_idx + 1];
                state.reset_with_cells_width(new_size, new_size);
            } else if c == "+" && state.cells_width > 64 {
                let current_idx = State::ELIGIBLE_SIZES
                    .iter()
                    .position(|&s| s == state.cells_width)
                    .unwrap();
                let new_size = State::ELIGIBLE_SIZES[current_idx - 1];
                state.reset_with_cells_width(new_size, new_size);
            }
        }
        #[cfg(not(target_family = "wasm"))]
        #[cfg(not(target_os = "android"))]
        WindowEvent::CloseRequested
        | WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    state: ElementState::Pressed,
                    logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
                    ..
                },
            ..
        } => event_loop.exit(),
        #[cfg(target_family = "wasm")]
        WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab),
                    state: ElementState::Pressed,
                    ..
                },
            ..
        } => {
            crate::web::toggle_controls();
        }
        WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space),
                    state: ElementState::Pressed,
                    ..
                },
            ..
        } => {
            state.toggle_pause();
        }
        WindowEvent::Resized(physical_size) => {
            state.resize(*physical_size);
        }
        WindowEvent::RedrawRequested => match state.render() {
            crate::RenderResult::Ok => {}
            crate::RenderResult::Lost => state.resize(state.size),
            crate::RenderResult::Other => {}
        },
        _ => {}
    }
}
