fn main() {
    #[cfg(not(any(target_os = "android", target_family = "wasm")))]
    {
        pub async fn run() {
            env_logger::init();

            let event_loop = winit::event_loop::EventLoop::new().unwrap();

            let window = winit::window::WindowBuilder::new()
                .build(&event_loop)
                .unwrap();

            let mut state =
                wgpu_game_of_life::State::new(window, None, None, None, None, false, None)
                    .await
                    .unwrap();

            event_loop
                .run(move |event, event_loop_window_target| {
                    wgpu_game_of_life::event_loop::handle_event_loop(
                        &event,
                        &mut state,
                        event_loop_window_target,
                    );
                })
                .unwrap();
        }

        pollster::block_on(run());
    }
}
