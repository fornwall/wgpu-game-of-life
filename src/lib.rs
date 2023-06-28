mod computer;
mod event_loop;
mod renderer;
mod rules;
#[cfg(target_arch = "wasm32")]
mod web;

use rand::prelude::*;

use computer::{Computer, ComputerFactory};
use renderer::{Renderer, RendererFactory};
use winit::{dpi::PhysicalPosition, window::Window};

#[cfg(not(target_arch = "wasm32"))]
pub async fn run() {
    env_logger::init();

    let event_loop = winit::event_loop::EventLoop::new();

    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(window, None, None, None, None, false, None)
        .await
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        event_loop::handle_event_loop(&event, &mut state, control_flow);
    });
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    pollster::block_on(android_run(app));
}

#[cfg(target_os = "android")]
async fn android_run(app: winit::platform::android::activity::AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    //android_logger::init_once(android_logger::Config::default().with_min_level(log::Level::Trace));

    let event_loop = winit::event_loop::EventLoopBuilder::with_user_event()
        .with_android_app(app)
        .build();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(window, None, None, None, None, false, None)
        .await
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        event_loop::handle_event_loop(&event, &mut state, control_flow);
    });
}

pub struct State {
    cells_height: u32,
    cells_width: u32,
    command_encoder_descriptor: wgpu::CommandEncoderDescriptor<'static>,
    computer: Computer,
    computer_factory: ComputerFactory,
    config: wgpu::SurfaceConfiguration,
    cursor_position: PhysicalPosition<f64>,
    device: wgpu::Device,
    elapsed_time: f32,
    frame_count: u64,
    generations_per_second: u8,
    initial_density: u8,
    last_time: instant::Instant,
    paused: bool,
    queue: wgpu::Queue,
    renderer: Renderer,
    renderer_factory: RendererFactory<'static>,
    rule_buffer: wgpu::Buffer,
    rule_idx: u32,
    seed: u32,
    size: winit::dpi::PhysicalSize<u32>,
    size_buffer: wgpu::Buffer,
    surface: wgpu::Surface,
    texture_view_descriptor: wgpu::TextureViewDescriptor<'static>,
    window: Window,
}
impl State {
    async fn new(
        window: Window,
        rule_idx: Option<u32>,
        grid_size: Option<u32>,
        seed: Option<u32>,
        initial_density: Option<u8>,
        paused: bool,
        generations_per_second: Option<u8>,
    ) -> Result<State, ()> {
        use wgpu::util::DeviceExt;

        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = unsafe { instance.create_surface(&window) }.map_err(|_| ())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(())?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .map_err(|_| ())?;

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            // "The usage field describes how SurfaceTextures will be used. RENDER_ATTACHMENT specifies
            // that the textures will be used to write to the screen (we'll talk about more TextureUsagess later).""
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            // "The format defines how SurfaceTextures will be stored on the gpu. We can get a supported format
            // from the SurfaceCapabilities."
            format: surface_format,
            // "width and height are the width and the height in pixels of a SurfaceTexture. This should
            // usually be the width and the height of the window.""
            width: size.width,
            height: size.height,
            // "present_mode uses wgpu::PresentMode enum which determines how to sync the surface with the display"
            present_mode: wgpu::PresentMode::Fifo,
            // "alpha_mode is honestly not something I'm familiar with. I believe it has something to do with
            // transparent windows, but feel free to open a pull request"
            alpha_mode: surface_caps.alpha_modes[0],
            // "view_formats is a list of TextureFormats that you can use when creating TextureViews"
            // "As of writing this means that if your surface is srgb color space, you can create a texture view
            // that uses a linear color space."
            view_formats: vec![],
        };

        let cells_width = match grid_size {
            Some(v) if [128, 256, 512, 1024, 2048].iter().any(|&e| e == v) => v,
            _ => 512,
        };
        let cells_height = cells_width;

        let size_array = [cells_width, cells_height];
        let size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("size_buffer"),
            contents: bytemuck::cast_slice(&size_array),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::VERTEX,
        });

        let rule_idx = match rule_idx {
            Some(idx) if idx < rules::RULES.len() as u32 => idx,
            _ => 0,
        };
        let rule = &rules::RULES[rule_idx as usize];
        let rule_array = rule.rule_array();
        let rule_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("rule_buffer"),
            contents: bytemuck::cast_slice(&rule_array),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::VERTEX,
        });

        let seed = seed.unwrap_or(0);

        let initial_density = match initial_density {
            Some(value) if value > 0 && value < 100 => value,
            _ => 12,
        };

        let generations_per_second = match generations_per_second {
            Some(value) if value > 0 && value < 100 => value,
            _ => 8,
        };

        let computer_factory = ComputerFactory::new(&device);
        let computer = computer_factory.create(
            &device,
            cells_width,
            cells_height,
            &size_buffer,
            &rule_buffer,
            seed,
            initial_density,
        );

        let renderer_factory = RendererFactory::new(&device);
        let renderer = renderer_factory.create(
            &device,
            &computer,
            &size_buffer,
            cells_width,
            cells_height,
            surface_format,
        );

        let last_time = instant::Instant::now();
        let elapsed_time = 0.;

        Ok(Self {
            generations_per_second,
            initial_density,
            paused,
            last_time,
            elapsed_time,
            seed,
            rule_idx,
            rule_buffer,
            size_buffer,
            renderer_factory,
            renderer,
            frame_count: 0,
            device,
            queue,
            texture_view_descriptor: wgpu::TextureViewDescriptor::default(),
            command_encoder_descriptor: wgpu::CommandEncoderDescriptor {
                label: Some("command_encoder_descriptor"),
            },
            window,
            config,
            size,
            surface,
            computer_factory,
            computer,
            cells_width,
            cells_height,
            cursor_position: PhysicalPosition { x: 0., y: 0. },
        })
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn set_initial_density(&mut self, new_density: u8) {
        if (1..=99).contains(&new_density) {
            self.initial_density = new_density;
            self.on_state_change();
        }
    }

    pub fn set_rule_idx(&mut self, new_rule_idx: u32) {
        self.rule_idx = new_rule_idx;
        let rule = &rules::RULES[self.rule_idx as usize];
        self.initial_density = rule.initial_density;
        self.queue.write_buffer(
            &self.rule_buffer,
            0,
            bytemuck::cast_slice(&rule.rule_array()),
        );
        self.on_state_change();
    }

    fn change_rule(&mut self, next: bool) {
        let new_rule_idx = if next {
            (self.rule_idx + 1) % (rules::RULES.len() as u32)
        } else if self.rule_idx == 0 {
            rules::RULES.len() as u32 - 1
        } else {
            self.rule_idx - 1
        };
        self.set_rule_idx(new_rule_idx);
    }

    fn inform_ui_about_state(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        self.window.set_title(&format!(
            "{} {}x{} 0.{} {} {}/s",
            rules::RULES[self.rule_idx as usize].name(),
            self.cells_width,
            self.cells_height,
            self.initial_density,
            self.seed,
            self.generations_per_second,
        ));

        #[cfg(target_arch = "wasm32")]
        web::set_new_state(
            self.rule_idx,
            self.cells_width,
            self.seed,
            self.initial_density,
            self.paused,
            self.generations_per_second,
            if self.paused { self.frame_count } else { 0 },
        );
    }

    fn reset_with_cells_width(&mut self, new_cells_width: u32, new_cells_height: u32) {
        self.cells_width = new_cells_width;
        self.cells_height = new_cells_height;
        self.on_state_change();
    }

    fn reset(&mut self) {
        self.seed = rand::thread_rng().next_u32();
        self.on_state_change();
    }

    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        self.inform_ui_about_state();
    }

    fn set_generations_per_second(&mut self, new_value: u8) {
        if new_value > 0 && new_value <= 100 {
            self.generations_per_second = new_value;
            self.inform_ui_about_state();
        }
    }

    fn on_state_change(&mut self) {
        self.inform_ui_about_state();

        let size_array = [self.cells_width, self.cells_height];
        self.queue
            .write_buffer(&self.size_buffer, 0, bytemuck::cast_slice(&size_array));

        self.frame_count = 0;

        self.computer = self.computer_factory.create(
            &self.device,
            self.cells_width,
            self.cells_height,
            &self.size_buffer,
            &self.rule_buffer,
            self.seed,
            self.initial_density,
        );

        self.renderer = self.renderer_factory.create(
            &self.device,
            &self.computer,
            &self.size_buffer,
            self.cells_width,
            self.cells_height,
            self.config.format,
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if self.window.inner_size().height < 10 {
            return Ok(());
        }

        let mut encoder = self
            .device
            .create_command_encoder(&self.command_encoder_descriptor);

        let frequency = 1.0 / f32::from(self.generations_per_second);
        loop {
            let advance_state = if self.paused {
                false
            } else {
                self.elapsed_time += self.last_time.elapsed().as_secs_f32();
                self.elapsed_time > frequency
            };
            self.last_time = instant::Instant::now();

            if advance_state {
                self.elapsed_time -= frequency;
                self.frame_count += 1;
            }

            if advance_state {
                self.computer.enqueue(&mut encoder);
            }
            if !advance_state {
                break;
            }
        }

        let output: wgpu::SurfaceTexture = self.surface.get_current_texture()?;

        self.renderer.enqueue(
            self.computer.currently_computed_is_0,
            &mut encoder,
            &output,
            &self.texture_view_descriptor,
        );

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
