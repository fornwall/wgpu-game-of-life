mod event_loop;

use rand::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;
use winit::{
    event::WindowEvent,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

fn init_logging() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    init_logging();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        //.with_title("WGPU Start")
        //.with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| {
        event_loop::handle_event_loop(event, &mut state, control_flow)
    });
}

pub struct State<'a> {
    device: wgpu::Device,
    queue: wgpu::Queue,
    current_pipeline_idx: u8,
    clear_color: wgpu::Color,
    texture_view_descriptor: wgpu::TextureViewDescriptor<'a>,
    command_encoder_descriptor: wgpu::CommandEncoderDescriptor<'a>,
    // Window
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    render_pipeline: wgpu::RenderPipeline,
    compute_pipeline: wgpu::ComputePipeline,
    cells_buffer_0: wgpu::Buffer,
    cells_buffer_1: wgpu::Buffer,
    square_buffer: wgpu::Buffer,
    compute_bind_group_0: wgpu::BindGroup,
    compute_bind_group_1: wgpu::BindGroup,
    frame_count: u64,
}

impl<'a> State<'a> {
    async fn new(window: Window) -> State<'a> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = unsafe { instance.create_surface(&window).unwrap() };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

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
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
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
            // Probably want PresentMode::Fifo (VSync)?
            present_mode: surface_caps.present_modes[0],
            // "alpha_mode is honestly not something I'm familiar with. I believe it has something to do with
            // transparent windows, but feel free to open a pull request"
            alpha_mode: surface_caps.alpha_modes[0],
            // "view_formats is a list of TextureFormats that you can use when creating TextureViews"
            // "As of writing this means that if your surface is srgb color space, you can create a texture view
            // that uses a linear color space."
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let mut rng = rand::thread_rng();
        let cells_width = 256;
        let mut cells_vec = vec![0_u32; cells_width * cells_width];
        for cell in cells_vec.iter_mut() {
            if rng.gen::<f32>() < 0.05 {
                *cell = 1;
            }
        }

        let cells_buffer_0 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: unsafe {
                std::slice::from_raw_parts(cells_vec.as_ptr() as *const u8, cells_vec.len() * 4)
            },
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
        });

        let cells_buffer_1 = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (cells_vec.len() * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let compute_shader =
            device.create_shader_module(wgpu::include_wgsl!("game-of-life.compute.wgsl"));

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None, // None = Check at draw call, bad for perf/correctness?
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None, // None = Check at draw call, bad for perf/correctness?
                        },
                        count: None,
                    },
                ],
                label: Some("compute_bind_group_layout"),
            });

        let compute_bind_group_0 = device.create_bind_group({
            &wgpu::BindGroupDescriptor {
                layout: &compute_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &cells_buffer_0,
                            offset: 0,
                            size: None,
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &cells_buffer_1,
                            offset: 0,
                            size: None,
                        }),
                    },
                ],
                label: Some("compute_bind_group_0"),
            }
        });
        let compute_bind_group_1 = device.create_bind_group({
            &wgpu::BindGroupDescriptor {
                layout: &compute_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &cells_buffer_1,
                            offset: 0,
                            size: None,
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &cells_buffer_0,
                            offset: 0,
                            size: None,
                        }),
                    },
                ],
                label: Some("compute_bind_group_1"),
            }
        });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("compute_pipeline_layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("compute_pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
        });

        let square_vertices = [0, 0, 0, 1, 1, 0, 1, 1];
        let square_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("square_buffer"),
            contents: unsafe {
                std::slice::from_raw_parts(
                    square_vertices.as_ptr() as *const u8,
                    square_vertices.len() * 4,
                )
            },
            usage: wgpu::BufferUsages::VERTEX,
        });

        let square_stride = wgpu::VertexBufferLayout {
            array_stride: 2 * (u32::BITS / 8) as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                shader_location: 1,
                offset: 0,
                format: wgpu::VertexFormat::Uint32x2,
            }],
        };

        let cells_stride = wgpu::VertexBufferLayout {
            array_stride: (u32::BITS / 8) as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Uint32,
            }],
        };

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipeline_layout"),
                bind_group_layouts: &[], // &render_bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(wgpu::include_wgsl!("game-of-life.render.wgsl"));
        let render_pipeline = render_pipeline_from_shader(
            &device,
            &render_pipeline_layout,
            shader,
            &config,
            cells_stride,
            square_stride,
        );

        Self {
            frame_count: 0,
            device,
            queue,
            current_pipeline_idx: 0,
            clear_color: wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            texture_view_descriptor: wgpu::TextureViewDescriptor::default(),
            command_encoder_descriptor: wgpu::CommandEncoderDescriptor {
                label: Some("command_encoder_descriptor"),
            },
            window,
            config,
            size,
            surface,
            compute_pipeline,
            render_pipeline,
            compute_bind_group_0,
            compute_bind_group_1,
            cells_buffer_0,
            cells_buffer_1,
            square_buffer,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn update(&mut self) {
        // todo!()
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.frame_count += 1;
        //println!("render: {}", self.frame_count);

        // Note: About recreating the below objects each time:
        // https://stackoverflow.com/questions/70489849/in-webgpu-can-you-reuse-the-same-render-pass-in-multiple-frames
        // "As answered below, a render pass (or more specifically: a GPURenderPassEncoder) cannot be reused. However,
        // if your goal is to execute the same render commands repeatedly without re-encoding them each time you'll want to look at WebGPU's
        // Render Bundles API. It allows you to record most render commands into a resuable object that can be executed as part of a full render pass"

        // "The get_current_texture function will wait for the surface to provide a new
        // SurfaceTexture that we will render to. We'll store this in output for later."
        let output = self.surface.get_current_texture()?;

        // "This line creates a TextureView with default settings".
        let view = output.texture.create_view(&self.texture_view_descriptor);

        // "We also need to create a CommandEncoder to create the actual commands to send to the gpu.
        // Most modern graphics frameworks expect commands to be stored in a command buffer before being sent to the gpu.
        // The encoder builds a command buffer that we can then send to the gpu."
        let mut encoder = self
            .device
            .create_command_encoder(&self.command_encoder_descriptor);

        let mut pass_encoder = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        pass_encoder.set_pipeline(&self.compute_pipeline);
        pass_encoder.set_bind_group(
            0,
            if self.frame_count % 2 == 0 {
                &self.compute_bind_group_1
            } else {
                &self.compute_bind_group_0
            },
            &[],
        );
        let workgroup_width = 8;
        let cells_width = 256;
        let workgroup_count_x = cells_width / workgroup_width;
        let workgroup_count_y = cells_width / workgroup_width;
        let workgroup_count_z = 1;
        pass_encoder.dispatch_workgroups(
            workgroup_count_x as u32,
            workgroup_count_y as u32,
            workgroup_count_z as u32,
        );
        drop(pass_encoder);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(
            0,
            if self.frame_count % 2 == 0 {
                self.cells_buffer_0.slice(..)
            } else {
                self.cells_buffer_0.slice(..)
            },
        );
        render_pass.set_vertex_buffer(1, self.square_buffer.slice(..));
        render_pass.draw(0..4, 0..(256 * 256));
        drop(render_pass);

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }
}

fn render_pipeline_from_shader(
    device: &wgpu::Device,
    render_pipeline_layout: &wgpu::PipelineLayout,
    shader: wgpu::ShaderModule,
    config: &wgpu::SurfaceConfiguration,
    cells_stride: wgpu::VertexBufferLayout,
    square_stride: wgpu::VertexBufferLayout,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("render_pipeline"),
        layout: Some(render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vertex_main",
            buffers: &[cells_stride, square_stride],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fragment_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw, // 2.
            cull_mode: None,                  // Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}
