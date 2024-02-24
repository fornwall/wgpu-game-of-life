#[cfg(target_os = "android")]
mod android;
pub mod computer;
pub mod event_loop;
mod renderer;
pub mod rules;
#[cfg(target_family = "wasm")]
mod web;

use computer::{Computer, ComputerFactory};
use renderer::{Renderer, RendererFactory};
use std::sync::Arc;
use winit::window::Window;

pub struct State {
    cells_height: u32,
    cells_width: u32,
    computer: Computer,
    computer_factory: ComputerFactory,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    elapsed_time: f32,
    frame_count: u64,
    generations_per_second: u8,
    initial_density: u8,
    last_time: instant::Instant,
    paused: bool,
    queue: wgpu::Queue,
    renderer: Renderer,
    renderer_factory: RendererFactory,
    rule_idx: u32,
    seed: u32,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    texture_view_descriptor: wgpu::TextureViewDescriptor<'static>,
    window: Arc<Window>,
}
impl State {
    const ELIGIBLE_SIZES: [u32; 6] = [64, 128, 256, 512, 1024, 2048];

    pub async fn new(
        window: Window,
        rule_idx: Option<u32>,
        grid_size: Option<u32>,
        seed: Option<u32>,
        initial_density: Option<u8>,
        paused: bool,
        generations_per_second: Option<u8>,
    ) -> Result<Self, String> {
        let window = Arc::new(window);
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = instance
            .create_surface(Arc::clone(&window))
            .map_err(|_| "create_surface failed")?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or("request_adapter failed")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .map_err(|e| format!("request_device failed: {}", e))?;

        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        if size.height > 0 && size.width > 0 {
            // Necessary for android, which does not get resize event later:
            surface.configure(&device, &config);
        }

        let cells_width = match grid_size {
            Some(v) if Self::ELIGIBLE_SIZES.iter().any(|&e| e == v) => v,
            _ => 512,
        };
        let cells_height = cells_width;

        let rule_idx = match rule_idx {
            Some(idx) if idx < rules::RULES.len() as u32 => idx,
            _ => 0,
        };
        let rule = &rules::RULES[rule_idx as usize];

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
            rule,
            seed,
            initial_density,
            &queue,
        );

        let renderer_factory = RendererFactory::new(&device);
        let renderer = renderer_factory.create(
            &device,
            &computer,
            &computer_factory.size_buffer,
            cells_width,
            cells_height,
            surface_format,
        );

        let last_time = instant::Instant::now();
        let elapsed_time = 0.;

        let state = Self {
            generations_per_second,
            initial_density,
            paused,
            last_time,
            elapsed_time,
            seed,
            rule_idx,
            renderer_factory,
            renderer,
            frame_count: 0,
            device,
            queue,
            texture_view_descriptor: wgpu::TextureViewDescriptor::default(),
            window,
            config,
            size,
            surface,
            computer_factory,
            computer,
            cells_width,
            cells_height,
        };
        state.inform_ui_about_state();
        Ok(state)
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
        #[cfg(not(family = "wasm"))]
        self.window.set_title(&format!(
            "{} {}x{} 0.{} {} {}/s",
            rules::RULES[self.rule_idx as usize].name(),
            self.cells_width,
            self.cells_height,
            self.initial_density,
            self.seed,
            self.generations_per_second,
        ));

        #[cfg(target_family = "wasm")]
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
        use rand::prelude::RngCore;
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

        self.frame_count = 0;

        let rule = &rules::RULES[self.rule_idx as usize];

        self.computer = self.computer_factory.create(
            &self.device,
            self.cells_width,
            self.cells_height,
            rule,
            self.seed,
            self.initial_density,
            &self.queue,
        );

        self.renderer = self.renderer_factory.create(
            &self.device,
            &self.computer,
            &self.computer_factory.size_buffer,
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
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("command_encoder_descriptor"),
            });

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
