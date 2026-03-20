fn main() {
    #[cfg(not(any(target_os = "android", target_family = "wasm")))]
    {
        use wgpu_game_of_life::event_loop::App;
        use winit::event_loop::EventLoop;

        env_logger::init();

        let event_loop = EventLoop::new().unwrap();
        let mut app = App::new();
        event_loop.run_app(&mut app).unwrap();
    }
}
