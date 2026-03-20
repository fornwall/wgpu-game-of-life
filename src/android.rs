use jni::{JavaVM, objects::JObject, sys::JNIInvokeInterface_};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::WindowId;

struct AndroidApp {
    app: winit::platform::android::activity::AndroidApp,
    state: Option<crate::State>,
}

impl ApplicationHandler for AndroidApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = winit::window::Window::default_attributes();
        let window = event_loop.create_window(window_attributes).unwrap();
        self.state = Some(
            pollster::block_on(crate::State::new(
                window, None, None, None, None, false, None,
            ))
            .unwrap(),
        );
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.state = None;
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(state) = &mut self.state {
            crate::event_loop::handle_window_event(&event, state, event_loop);
        }
    }
}

#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Warn),
    );

    enable_immersive(&app);

    let event_loop = EventLoop::builder()
        .with_android_app(app.clone())
        .build()
        .unwrap();
    let mut android_app = AndroidApp { app, state: None };
    let _ = event_loop.run_app(&mut android_app);
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
