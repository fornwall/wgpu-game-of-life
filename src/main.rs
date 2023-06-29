fn main() {
    #[cfg(not(target_arch = "android"))]
    #[cfg(not(target_arch = "wasm32"))]
    {
        pub async fn run() {
            env_logger::init();

            let event_loop = winit::event_loop::EventLoop::new();

            let window = winit::window::WindowBuilder::new()
                .build(&event_loop)
                .unwrap();

            let mut state =
                wgpu_game_of_life::State::new(window, None, None, None, None, false, None)
                    .await
                    .unwrap();

            event_loop.run(move |event, _, control_flow| {
                wgpu_game_of_life::event_loop::handle_event_loop(&event, &mut state, control_flow);
            });
        }

        pollster::block_on(run());
    }
}
