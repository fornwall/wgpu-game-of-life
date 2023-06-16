use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::ControlFlow,
};

use crate::State;
use log::error;

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
                        error!("Space pressed - warn");
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
                        error!("Mouse pressed: {:?}", state.cursor_position);
                    }
                    _ => {}
                }
            }
        }

        &Event::RedrawRequested(window_id) if window_id == state.window.id() => {
            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        //Event::NewEvents(StartCause::Init) => {
        // From the winit README:
        // "A lot of functionality expects the application to be ready before you start doing anything;
        // this includes creating windows, fetching monitors, drawing, and so on, see issues #2238, #2051
        // and #2087.
        // If you encounter problems, you should try doing your initialization inside
        // Event::NewEvents(StartCause::Init)."
        //state .window .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        //state.window.focus_window();
        //}
        Event::MainEventsCleared => {
            state.window.request_redraw();
        }
        &Event::WindowEvent {
            ref event,
            window_id,
        } => {
            error!("INFO: {:?}, window = {:?}", event, window_id);
        }
        _ => {}
    }
}
