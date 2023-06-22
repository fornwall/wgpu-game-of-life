use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::ControlFlow,
};

#[cfg(target_arch = "wasm32")]
use crate::CustomWinitEvent;
use crate::State;

pub fn handle_event_loop(
    event: &crate::EventTypeUsed,
    state: &mut State,
    control_flow: &mut ControlFlow,
) {
    match event {
        #[cfg(target_arch = "wasm32")]
        Event::UserEvent(event) => match event {
            &CustomWinitEvent::RuleChange(new_rule_idx) => {
                state.set_rule_idx(new_rule_idx);
            }
            CustomWinitEvent::Reset => {
                state.reset();
            }
        },
        &Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window.id() => match event {
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        logical_key: winit::keyboard::Key::ArrowDown,
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                state.change_rule(true);
            }
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        logical_key: winit::keyboard::Key::ArrowUp,
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                state.change_rule(false);
            }
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        logical_key: winit::keyboard::Key::ArrowLeft,
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if state.initial_density > 1 {
                    state.initial_density -= 1;
                    state.on_state_change();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        logical_key: winit::keyboard::Key::ArrowRight,
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if state.initial_density < 99 {
                    state.initial_density += 1;
                    state.on_state_change();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        logical_key: winit::keyboard::Key::Character(c),
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if c == "f" || c == "F" {
                    #[cfg(target_arch = "wasm32")]
                    {
                        crate::toggle_fullscreen();
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if state.window.fullscreen().is_some() {
                            state.window.set_fullscreen(None);
                        } else {
                            state
                                .window
                                .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                        }
                    }
                } else if c == "h" || c == "H" {
                    #[cfg(target_arch = "wasm32")]
                    crate::toggle_controls();
                } else if c == "r" || c == "R" {
                    state.reset();
                } else if c == "-" && state.cells_width < 2048 {
                    state.reset_with_cells_width(state.cells_width + 128, state.cells_height + 128);
                } else if c == "+" && state.cells_width > 128 {
                    state.reset_with_cells_width(state.cells_width - 128, state.cells_height - 128);
                }
            }
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: winit::keyboard::KeyCode::Escape,
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: winit::keyboard::KeyCode::Space,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                state.paused = !state.paused;
            }
            WindowEvent::Resized(physical_size) => {
                log::info!("resize: {:?}", physical_size);
                state.resize(*physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                log::info!("scale factor: {:?}", new_inner_size);
                state.resize(**new_inner_size);
            }
            WindowEvent::CursorMoved { position, .. } => {
                state.cursor_position = *position;
            }
            WindowEvent::MouseInput {
                state: winit::event::ElementState::Pressed,
                ..
            } => {
                log::error!("Mouse pressed: {:?}", state.cursor_position);
            }
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                println!("key = {:?}", event);
            }
            _ => {}
        },

        &Event::RedrawRequested(window_id) if window_id == state.window.id() => {
            match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => log::error!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            state.window.request_redraw();
        }
        &Event::WindowEvent {
            ref event,
            window_id,
        } => {
            log::error!("INFO: {:?}, window = {:?}", event, window_id);
        }
        _ => {}
    }
}
