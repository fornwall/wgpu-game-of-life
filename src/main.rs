use wgpu_game_of_life::State;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

fn main() {
    #[cfg(not(any(target_os = "android", target_family = "wasm")))]
    {
        pub async fn create_state(event_loop: &ActiveEventLoop) -> State {
            let window_attributes = Window::default_attributes();
            let window = event_loop.create_window(window_attributes).unwrap();
            wgpu_game_of_life::State::new(window, None, None, None, None, false, None)
                .await
                .unwrap()
        }
        env_logger::init();

        let event_loop = winit::event_loop::EventLoop::new().unwrap();

        let mut state: Option<State> = None;

        event_loop
            .run(move |event, event_loop| {
                if matches!(event, winit::event::Event::Resumed) && state.is_none() {
                    state = Some(pollster::block_on(create_state(event_loop)));
                };

                if let Some(state) = &mut state {
                    wgpu_game_of_life::event_loop::handle_event_loop(&event, state, event_loop);
                }
            })
            .unwrap();
    }
}
