mod event_loop;

use rand::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;
use winit::{
    event::{ElementState, WindowEvent},
    event_loop::EventLoop,
    keyboard::KeyCode,
    window::{Window, WindowBuilder},
};

fn init_logging() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
}

#[cfg(target_arch = "wasm32")]
fn setup_html_canvas() -> web_sys::HtmlCanvasElement {
    use web_sys::HtmlCanvasElement;
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let canvas = doc.get_element_by_id("webgpu-canvas")?;
            canvas.dyn_into::<HtmlCanvasElement>().ok()
        })
        .expect("Could not get canvas")
}

//#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn run() {
    init_logging();

    let event_loop = EventLoop::new();

    #[cfg(target_arch = "wasm32")]
    use winit::platform::web::WindowBuilderExtWebSys;
    #[cfg(target_arch = "wasm32")]
    let window = WindowBuilder::new()
        .with_canvas(Some(setup_html_canvas()))
        .build(&event_loop)
        .unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(window).await;

    #[cfg(target_arch = "wasm32")]
    use winit::platform::web::EventLoopExtWebSys;
    #[cfg(target_arch = "wasm32")]
    event_loop.spawn(move |event, _, control_flow| {
        event_loop::handle_event_loop(&event, &mut state, control_flow);
    });
    #[cfg(not(target_arch = "wasm32"))]
    event_loop.run(move |event, _, control_flow| {
        event_loop::handle_event_loop(&event, &mut state, control_flow);
    });
}

pub struct State<'a> {
    device: wgpu::Device,
    queue: wgpu::Queue,
    texture_view_descriptor: wgpu::TextureViewDescriptor<'a>,
    command_encoder_descriptor: wgpu::CommandEncoderDescriptor<'a>,
    // Window
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    computer_factory: ComputerFactory,
    computer: Computer,
    frame_count: u64,
    renderer_factory: RendererFactory<'a>,
    renderer: Renderer,
    size_buffer: wgpu::Buffer,
    cells_width: usize,
}

pub struct RendererFactory<'a> {
    shader: wgpu::ShaderModule,
    bind_group_layout: wgpu::BindGroupLayout,
    square_buffer: wgpu::Buffer,
    pipeline_layout: wgpu::PipelineLayout,
    cells_stride: wgpu::VertexBufferLayout<'a>,
    square_stride: wgpu::VertexBufferLayout<'a>,
}

impl<'a> RendererFactory<'a> {
    fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("game-of-life.render.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("bind_group_layout_render"),
        });

        let cells_stride = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<u32>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                shader_location: 0,
                offset: 0,
                format: wgpu::VertexFormat::Uint32,
            }],
        };

        let square_stride = wgpu::VertexBufferLayout {
            array_stride: 2 * std::mem::size_of::<u32>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                shader_location: 1,
                offset: 0,
                format: wgpu::VertexFormat::Uint32x2,
            }],
        };

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let square_vertices = [0, 0, 0, 1, 1, 0, 1, 1];
        let square_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("square_buffer"),
            contents: bytemuck::cast_slice(&square_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            shader,
            bind_group_layout,
            square_buffer,
            pipeline_layout,
            cells_stride,
            square_stride,
        }
    }

    fn create(
        &self,
        device: &wgpu::Device,
        computer: &Computer,
        size_buffer: &wgpu::Buffer,
        cells_width: usize,
        texture_format: wgpu::TextureFormat,
    ) -> Renderer {
        let render_pipeline = render_pipeline_from_shader(
            &device,
            &self.pipeline_layout,
            &self.shader,
            texture_format,
            self.cells_stride.clone(),
            self.square_stride.clone(),
        );

        let size_bind_group = device.create_bind_group({
            {
                &wgpu::BindGroupDescriptor {
                    layout: &self.bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &size_buffer,
                            offset: 0,
                            size: None,
                        }),
                    }],
                    label: Some("compute_bind_group_1"),
                }
            }
        });

        let create_render_bundle = |cells_buffer: &wgpu::Buffer| {
            let mut render_bundle_encoder =
                device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
                    sample_count: 1,
                    color_formats: &[Some(texture_format)],
                    ..Default::default()
                });
            render_bundle_encoder.set_pipeline(&render_pipeline);
            render_bundle_encoder.set_vertex_buffer(0, cells_buffer.slice(..));
            render_bundle_encoder.set_vertex_buffer(1, self.square_buffer.slice(..));
            render_bundle_encoder.set_bind_group(0, &size_bind_group, &[]);
            render_bundle_encoder.draw(0..4, 0..((cells_width * cells_width) as u32));
            render_bundle_encoder.finish(&wgpu::RenderBundleDescriptor::default())
        };

        let render_bundle_0 = create_render_bundle(&computer.cells_buffer_0);
        let render_bundle_1 = create_render_bundle(&computer.cells_buffer_1);

        Renderer {
            render_bundle_0,
            render_bundle_1,
        }
    }
}

pub struct Renderer {
    render_bundle_0: wgpu::RenderBundle,
    render_bundle_1: wgpu::RenderBundle,
}

impl Renderer {
    fn enqueue(
        &self,
        is_even: bool,
        encoder: &mut wgpu::CommandEncoder,
        surface_texture: &wgpu::SurfaceTexture,
        texture_view_descriptor: &wgpu::TextureViewDescriptor,
    ) {
        let view = surface_texture.texture.create_view(texture_view_descriptor);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.,
                        g: 0.,
                        b: 0.,
                        a: 1.,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        render_pass.execute_bundles(std::iter::once(if is_even {
            &self.render_bundle_0
        } else {
            &self.render_bundle_1
        }));
        drop(render_pass);
    }
}

pub struct ComputerFactory {
    shader: wgpu::ShaderModule,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl ComputerFactory {
    fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("game-of-life.compute.wgsl"));
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("compute_bind_group_layout"),
        });
        Self {
            shader,
            bind_group_layout,
        }
    }

    fn create(
        &self,
        device: &wgpu::Device,
        cells_width: usize,
        size_buffer: &wgpu::Buffer,
    ) -> Computer {
        let mut rng = rand::thread_rng();
        let mut cells_vec = vec![0_u32; cells_width * cells_width];
        for cell in cells_vec.iter_mut() {
            if rng.gen::<f32>() < 0.50 {
                *cell = 1;
            }
        }

        let cells_buffer_0 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&cells_vec),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
        });

        let cells_buffer_1 = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (cells_vec.len() * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let compute_bind_group_0 = device.create_bind_group({
            &wgpu::BindGroupDescriptor {
                layout: &self.bind_group_layout,
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
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &size_buffer,
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
                layout: &self.bind_group_layout,
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
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &size_buffer,
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
                bind_group_layouts: &[&self.bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("compute_pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &self.shader,
            entry_point: "main",
        });

        Computer {
            cells_width: cells_width as u32,
            compute_pipeline,
            compute_bind_group_0,
            compute_bind_group_1,
            cells_buffer_0,
            cells_buffer_1,
        }
    }
}

pub struct Computer {
    cells_width: u32,
    compute_pipeline: wgpu::ComputePipeline,
    compute_bind_group_0: wgpu::BindGroup,
    compute_bind_group_1: wgpu::BindGroup,
    cells_buffer_0: wgpu::Buffer,
    cells_buffer_1: wgpu::Buffer,
}

impl Computer {
    fn enqueue(&self, is_even: bool, command_encoder: &mut wgpu::CommandEncoder) {
        let mut pass_encoder =
            command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        pass_encoder.set_pipeline(&self.compute_pipeline);
        pass_encoder.set_bind_group(
            0,
            if is_even {
                &self.compute_bind_group_1
            } else {
                &self.compute_bind_group_0
            },
            &[],
        );
        let workgroup_width = 8;
        let workgroup_count_x = self.cells_width / workgroup_width;
        let workgroup_count_y = self.cells_width / workgroup_width;
        let workgroup_count_z = 1;
        pass_encoder.dispatch_workgroups(workgroup_count_x, workgroup_count_y, workgroup_count_z);
        drop(pass_encoder);
    }
}

impl<'a> State<'a> {
    async fn new(window: Window) -> State<'a> {
        let size = window.inner_size();
        log::info!("size = {:?}", size);
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

        let cells_width = 256;

        let size_array = [cells_width as u32, cells_width as u32];
        let size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("size_buffer"),
            contents: bytemuck::cast_slice(&size_array),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::VERTEX,
        });

        let computer_factory = ComputerFactory::new(&device);
        let computer = computer_factory.create(&device, cells_width, &size_buffer);

        let renderer_factory = RendererFactory::new(&device);
        let renderer = renderer_factory.create(
            &device,
            &computer,
            &size_buffer,
            cells_width,
            surface_format,
        );

        Self {
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

    fn reset_with_cells_width(&mut self, new_cells_width: usize) {
        self.cells_width = new_cells_width;
        let size_array = [self.cells_width as u32, self.cells_width as u32];
        self.queue
            .write_buffer(&self.size_buffer, 0, bytemuck::cast_slice(&size_array));

        self.frame_count = 0;

        self.computer =
            self.computer_factory
                .create(&self.device, self.cells_width, &self.size_buffer);

        self.renderer = self.renderer_factory.create(
            &self.device,
            &self.computer,
            &self.size_buffer,
            self.cells_width,
            self.config.format,
        )
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if self.window.inner_size().height < 10 {
            return Ok(());
        }
        self.frame_count += 1;
        let is_even = self.frame_count % 2 == 0;

        let mut encoder = self
            .device
            .create_command_encoder(&self.command_encoder_descriptor);

        let output: wgpu::SurfaceTexture = self.surface.get_current_texture().unwrap();

        self.computer.enqueue(is_even, &mut encoder);
        self.renderer.enqueue(
            is_even,
            &mut encoder,
            &output,
            &self.texture_view_descriptor,
        );

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        //error!("input: {:?}", event);
        match event {
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: KeyCode::NumpadAdd,
                        ..
                    },
                ..
            } => {
                self.window.set_title("plus");
                true
            }
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: KeyCode::Minus,
                        ..
                    },
                ..
            } => {
                self.window.set_title("minus");
                true
            }
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: KeyCode::ArrowUp,
                        ..
                    },
                ..
            } => {
                self.window.set_title("up");
                true
            }
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        logical_key: winit::keyboard::Key::Character(c),
                        ..
                    },
                ..
            } => {
                self.window.set_title(&format!("char: {}", c));
                if c == "f" || c == "F" {
                    if self.window.fullscreen().is_some() {
                        self.window.set_fullscreen(None);
                    } else {
                        self.window
                            .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                    }
                } else if c == "r" || c == "R" {
                    self.reset_with_cells_width(self.cells_width);
                } else if c == "+" {
                    if self.cells_width < 2048 {
                        self.reset_with_cells_width(self.cells_width + 128);
                    }
                } else if c == "-" {
                    if self.cells_width > 128 {
                        self.reset_with_cells_width(self.cells_width - 128);
                    }
                }
                true
            }
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                println!("key = {:?}", event);
                true
            }
            _ => false,
        }
    }
}

fn render_pipeline_from_shader(
    device: &wgpu::Device,
    render_pipeline_layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    cells_stride: wgpu::VertexBufferLayout,
    square_stride: wgpu::VertexBufferLayout,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("render_pipeline"),
        layout: Some(render_pipeline_layout),
        vertex: wgpu::VertexState {
            buffers: &[cells_stride, square_stride],
            entry_point: "vertex_main",
            module: shader,
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: "fragment_main",
            targets: &[Some(wgpu::ColorTargetState {
                blend: Some(wgpu::BlendState::REPLACE),
                format: format,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            conservative: false,
            cull_mode: Some(wgpu::Face::Back),
            front_face: wgpu::FrontFace::Cw,
            polygon_mode: wgpu::PolygonMode::Fill,
            strip_index_format: None,
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            unclipped_depth: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            alpha_to_coverage_enabled: false,
            count: 1,
            mask: !0,
        },
        multiview: None,
    })
}
