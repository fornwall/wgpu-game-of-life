use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use winit::event_loop::EventLoopProxy;
use winit::{event::Event, window::WindowBuilder};

#[derive(Debug, Clone, Copy)]
pub enum CustomWinitEvent {
    RuleChange(u32),
    SizeChange(u32),
    SetDensity(u8),
    SetGenerationsPerSecond(u8),
    Reset,
    TogglePause,
}

pub(crate) type EventTypeUsed<'a> = Event<'a, CustomWinitEvent>;

thread_local! {
    pub static EVENT_LOOP_PROXY: Mutex<Option<EventLoopProxy<CustomWinitEvent>>> = Mutex::new(None);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = setNewState)]
    pub fn set_new_state(
        rule_idx: u32,
        cells_width: u32,
        seed: u32,
        density: u8,
        paused: bool,
        generations_per_second: u8,
        frame: u64,
    );

    #[wasm_bindgen(js_name = toggleFullscreen)]
    pub fn toggle_fullscreen();

    #[wasm_bindgen(js_name = toggleControls)]
    pub fn toggle_controls();

    #[wasm_bindgen(js_name = downloadImage)]
    pub fn download_image();
}

#[wasm_bindgen(js_name = "setNewRule")]
pub fn set_new_rule(rule_idx: u32) {
    EVENT_LOOP_PROXY.with(|proxy| {
        if let Ok(unlocked) = proxy.lock() {
            if let Some(event_loop_proxy) = &*unlocked {
                event_loop_proxy
                    .send_event(CustomWinitEvent::RuleChange(rule_idx))
                    .ok();
            }
        }
    });
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = "setNewSize")]
pub fn set_new_size(size: u32) {
    EVENT_LOOP_PROXY.with(|proxy| {
        if let Ok(unlocked) = proxy.lock() {
            if let Some(event_loop_proxy) = &*unlocked {
                event_loop_proxy
                    .send_event(CustomWinitEvent::SizeChange(size))
                    .ok();
            }
        }
    });
}

#[wasm_bindgen(js_name = "resetGame")]
pub fn reset_game() {
    EVENT_LOOP_PROXY.with(|proxy| {
        if let Ok(unlocked) = proxy.lock() {
            if let Some(event_loop_proxy) = &*unlocked {
                event_loop_proxy.send_event(CustomWinitEvent::Reset).ok();
            }
        }
    });
}

#[wasm_bindgen(js_name = "togglePause")]
pub fn toggle_pause() {
    EVENT_LOOP_PROXY.with(|proxy| {
        if let Ok(unlocked) = proxy.lock() {
            if let Some(event_loop_proxy) = &*unlocked {
                event_loop_proxy
                    .send_event(CustomWinitEvent::TogglePause)
                    .ok();
            }
        }
    });
}

#[wasm_bindgen(js_name = "setDensity")]
pub fn set_density(density: u8) {
    EVENT_LOOP_PROXY.with(|proxy| {
        if let Ok(unlocked) = proxy.lock() {
            if let Some(event_loop_proxy) = &*unlocked {
                event_loop_proxy
                    .send_event(CustomWinitEvent::SetDensity(density))
                    .ok();
            }
        }
    });
}

#[wasm_bindgen(js_name = "setGenerationsPerSecond")]
pub fn set_generations_per_second(generations_per_second: u8) {
    EVENT_LOOP_PROXY.with(|proxy| {
        if let Ok(unlocked) = proxy.lock() {
            if let Some(event_loop_proxy) = &*unlocked {
                event_loop_proxy
                    .send_event(CustomWinitEvent::SetGenerationsPerSecond(
                        generations_per_second,
                    ))
                    .ok();
            }
        }
    });
}

#[wasm_bindgen]
pub async fn run(
    rule_idx: Option<u32>,
    size: Option<u32>,
    seed: Option<u32>,
    initial_density: Option<u8>,
    paused: bool,
    generations_per_second: Option<u8>,
) -> Result<(), String> {
    use winit::event_loop::EventLoopBuilder;
    use winit::platform::web::{EventLoopExtWebSys, WindowBuilderExtWebSys};

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info)
        .map_err(|e| format!("Couldn't initialize logger: {e}"))?;

    let event_loop = EventLoopBuilder::<CustomWinitEvent>::with_user_event().build();

    let event_loop_proxy = event_loop.create_proxy();
    EVENT_LOOP_PROXY.with(move |proxy| {
        if let Ok(mut proxy) = proxy.lock() {
            *proxy = Some(event_loop_proxy);
        }
    });

    let canvas_element = web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let canvas = doc.get_element_by_id("webgpu-canvas")?;
            canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok()
        })
        .ok_or("Could not get canvas element")?;

    let window = WindowBuilder::new()
        .with_canvas(Some(canvas_element))
        .with_prevent_default(false)
        .build(&event_loop)
        .unwrap();

    let mut state = crate::State::new(
        window,
        rule_idx,
        size,
        seed,
        initial_density,
        paused,
        generations_per_second,
    )
    .await?;

    event_loop.spawn(move |event, _, control_flow| {
        crate::event_loop::handle_event_loop(&event, &mut state, control_flow);
    });

    Ok(())
}
