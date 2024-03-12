use jni::{objects::JObject, sys::JNIInvokeInterface_, JavaVM};
use winit::event_loop::EventLoop;

#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Warn),
    );

    let mut maybe_state: Option<crate::State> = None;

    enable_immersive(&app);

    let event_loop = EventLoop::builder().with_android_app(app).build().unwrap();

    #[allow(deprecated)]
    let _ = event_loop.run(|event, event_loop| match event {
        winit::event::Event::Resumed => {
            let window_attributes = winit::window::Window::default_attributes();
            let window = event_loop.create_window(window_attributes).unwrap();

            pollster::block_on(setup(window, &mut maybe_state));
        }
        winit::event::Event::Suspended => {
            maybe_state = None;
        }
        _ => {
            if let Some(ref mut state) = &mut maybe_state {
                crate::event_loop::handle_event_loop(&event, state, event_loop);
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

fn enable_immersive(app: &winit::platform::android::activity::AndroidApp) {
    const SYSTEM_UI_FLAG_FULLSCREEN: i32 = 4; // https://developer.android.com/reference/android/view/View#SYSTEM_UI_FLAG_FULLSCREEN
    const SYSTEM_UI_FLAG_HIDE_NAVIGATION: i32 = 2; // https://developer.android.com/reference/android/view/View#SYSTEM_UI_FLAG_HIDE_NAVIGATION
    const SYSTEM_UI_FLAG_IMMERSIVE_STICKY: i32 = 4096; // https://developer.android.com/reference/android/view/View#SYSTEM_UI_FLAG_IMMERSIVE_STICKY
    const SYSTEM_UI_VISIBILITY: i32 = SYSTEM_UI_FLAG_FULLSCREEN
        | SYSTEM_UI_FLAG_HIDE_NAVIGATION
        | SYSTEM_UI_FLAG_IMMERSIVE_STICKY;

    let vm =
        // SAFETY: Guaranteed by https://docs.rs/android-activity/latest/android_activity/struct.AndroidApp.html#method.vm_as_ptr
        unsafe { JavaVM::from_raw(app.vm_as_ptr().cast::<*const JNIInvokeInterface_>()) }.unwrap();
    let mut env = vm.attach_current_thread().unwrap();
    let activity =
        // SAFETY: Guaranteed by https://docs.rs/android-activity/latest/android_activity/struct.AndroidApp.html#method.activity_as_ptr
        unsafe { JObject::from_raw(app.activity_as_ptr().cast::<jni::sys::_jobject>()) };
    let window = env
        .call_method(activity, "getWindow", "()Landroid/view/Window;", &[])
        .unwrap()
        .l()
        .unwrap();
    let view = env
        .call_method(window, "getDecorView", "()Landroid/view/View;", &[])
        .unwrap()
        .l()
        .unwrap();
    if let Err(e) = env.call_method(
        view,
        "setSystemUiVisibility",
        "(I)V",
        &[jni::objects::JValue::Int(SYSTEM_UI_VISIBILITY)],
    ) {
        log::error!("Failed setting immersive mode: {}", e);
        if let Err(e) = env.exception_clear() {
            log::error!("Error in exception_clear(): {}", e);
        }
    } else {
        log::warn!("Managed to set immersive mode");
    }
}
