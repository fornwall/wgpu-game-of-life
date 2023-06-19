use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::ControlFlow,
};

use crate::State;

pub fn handle_event_loop(event: &Event<()>, state: &mut State, control_flow: &mut ControlFlow) {
    // use std::ops::Add;
    // *control_flow = ControlFlow::WaitUntil(Instant::now().add(Duration::from_millis(1000)));
    match event {
        &Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window.id() => {
            if !state.input(event) {
                match event {
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
                    _ => {}
                }
            }
        }

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
