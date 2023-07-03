use winit::event_loop::EventLoopBuilder;

#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Debug),
    );

    let mut maybe_state: Option<crate::State> = None;

    let event_loop = EventLoopBuilder::new().with_android_app(app).build();

    event_loop.run(move |event, event_loop, control_flow| {
        // *control_flow = winit::event_loop::ControlFlow::Wait;
        match event {
            winit::event::Event::Resumed => {
                let window = winit::window::WindowBuilder::new()
                    .build(&event_loop)
                    .unwrap();

                pollster::block_on(setup(window, &mut maybe_state));
            }
            winit::event::Event::Suspended => {
                maybe_state = None;
            }
            _ => {
                if let Some(ref mut state) = &mut maybe_state {
                    crate::event_loop::handle_event_loop(&event, state, control_flow);
                }
            }
        }
    });
}

async fn setup(window: winit::window::Window, state: &mut Option<crate::State>) {
    *state = Some(
        crate::State::new(window, None, None, None, None, false, None)
            .await
            .unwrap(),
    );
}
