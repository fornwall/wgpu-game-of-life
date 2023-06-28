#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    pollster::block_on(android_run(app));
}

async fn android_run(app: winit::platform::android::activity::AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    //android_logger::init_once(android_logger::Config::default().with_min_level(log::Level::Trace));

    let event_loop = winit::event_loop::EventLoopBuilder::with_user_event()
        .with_android_app(app)
        .build();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();

    let mut state = crate::State::new(window, None, None, None, None, false, None)
        .await
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        crate::event_loop::handle_event_loop(&event, &mut state, control_flow);
    });
}
