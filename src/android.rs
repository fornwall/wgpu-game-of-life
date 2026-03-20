use jni::{JavaVM, objects::JObject, sys::JNIInvokeInterface_};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::WindowId;

struct AndroidApp {
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

#[unsafe(no_mangle)]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Warn),
    );

    enable_immersive(&app);

    let event_loop = EventLoop::builder().with_android_app(app).build().unwrap();
    let mut android_app = AndroidApp { state: None };
    let _ = event_loop.run_app(&mut android_app);
}

fn enable_immersive(app: &winit::platform::android::activity::AndroidApp) {
    use jni::signature::RuntimeMethodSignature;
    use jni::strings::JNIString;

    const SYSTEM_UI_FLAG_FULLSCREEN: i32 = 4;
    const SYSTEM_UI_FLAG_HIDE_NAVIGATION: i32 = 2;
    const SYSTEM_UI_FLAG_IMMERSIVE_STICKY: i32 = 4096;
    const SYSTEM_UI_VISIBILITY: i32 = SYSTEM_UI_FLAG_FULLSCREEN
        | SYSTEM_UI_FLAG_HIDE_NAVIGATION
        | SYSTEM_UI_FLAG_IMMERSIVE_STICKY;

    let vm =
        // SAFETY: Guaranteed by https://docs.rs/android-activity/latest/android_activity/struct.AndroidApp.html#method.vm_as_ptr
        unsafe { JavaVM::from_raw(app.vm_as_ptr().cast::<*const JNIInvokeInterface_>()) };

    let get_window_name = JNIString::from("getWindow");
    let get_window_sig = RuntimeMethodSignature::from_str("()Landroid/view/Window;").unwrap();

    let get_decor_view_name = JNIString::from("getDecorView");
    let get_decor_view_sig = RuntimeMethodSignature::from_str("()Landroid/view/View;").unwrap();

    let set_ui_vis_name = JNIString::from("setSystemUiVisibility");
    let set_ui_vis_sig = RuntimeMethodSignature::from_str("(I)V").unwrap();

    let result: Result<(), jni::errors::Error> = vm.attach_current_thread(|env| {
        let activity =
            // SAFETY: Guaranteed by https://docs.rs/android-activity/latest/android_activity/struct.AndroidApp.html#method.activity_as_ptr
            unsafe {
                JObject::from_raw(env, app.activity_as_ptr().cast::<jni::sys::_jobject>())
            };
        let window = env
            .call_method(
                &activity,
                &get_window_name,
                &get_window_sig.method_signature(),
                &[],
            )?
            .l()?;
        let view = env
            .call_method(
                &window,
                &get_decor_view_name,
                &get_decor_view_sig.method_signature(),
                &[],
            )?
            .l()?;
        env.call_method(
            &view,
            &set_ui_vis_name,
            &set_ui_vis_sig.method_signature(),
            &[jni::objects::JValue::Int(SYSTEM_UI_VISIBILITY)],
        )?;
        log::warn!("Managed to set immersive mode");
        Ok(())
    });

    if let Err(e) = result {
        log::error!("Failed setting immersive mode: {}", e);
    }
}
