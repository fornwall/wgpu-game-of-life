use rand::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;

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

    //let event_loop = EventLoop::new();

    let state = State::new().await;

    //event_loop.run(move |event, _, control_flow| {
    //event_loop::handle_event_loop(event, &mut state, control_flow)
    //});
}

pub struct State<'a> {
    device: wgpu::Device,
    queue: wgpu::Queue,
    current_pipeline_idx: u8,
    clear_color: wgpu::Color,
    texture_view_descriptor: wgpu::TextureViewDescriptor<'a>,
    command_encoder: wgpu::CommandEncoderDescriptor<'a>,
}

impl<'a> State<'a> {
    async fn new() -> State<'a> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
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

        let mut rng = rand::thread_rng();
        let cells_width = 256;
        let mut cells_vec = vec![0_u32; cells_width * cells_width];
        for cell in cells_vec.iter_mut() {
            if rng.gen::<f32>() < 0.25 {
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

        let shader = device.create_shader_module(wgpu::include_wgsl!("game-of-life.compute.wgsl"));

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

        let compute_bind_group = device.create_bind_group({
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
                label: Some("compute_bind_group"),
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
            module: &shader,
            entry_point: "main",
        });

        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("TestCommandEncoder"),
        });

        let mut pass_encoder =
            command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        pass_encoder.set_pipeline(&compute_pipeline);
        pass_encoder.set_bind_group(0, &compute_bind_group, &[]);
        let workgroup_width = 8;
        let workgroup_count_x = cells_width / workgroup_width;
        let workgroup_count_y = cells_width / workgroup_width;
        let workgroup_count_z = cells_width / workgroup_width;
        pass_encoder.dispatch_workgroups(
            workgroup_count_x as u32,
            workgroup_count_y as u32,
            workgroup_count_z as u32,
        );
        drop(pass_encoder);

        Self {
            device,
            queue,
            current_pipeline_idx: 0,
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            texture_view_descriptor: wgpu::TextureViewDescriptor::default(),
            command_encoder: wgpu::CommandEncoderDescriptor {
                label: Some("TestCommandEncoder"),
            },
        }
    }
}
