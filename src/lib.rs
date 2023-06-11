use std::num::NonZeroU64;

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

        let gpu_write_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: {
                let data = [11_u32, 23];
                unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4) }
            },
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });
        /*
        let gpu_write_buffer_slice = gpu_write_buffer.slice(..);
        let data: wgpu::BufferViewMut = gpu_write_buffer_slice.get_mapped_range_mut();
        let data_u32: &mut [u32] =
            unsafe { std::slice::from_raw_parts_mut(data.as_ptr() as *mut u32, 2) };
        data_u32[0] = 11;
        data_u32[1] = 23;
        drop(data);
        gpu_write_buffer.unmap();
        */

        let gpu_result_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: 8,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("compute_shader.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None, // None = Check at draw call, bad for perf/correctness?
                    },
                    //count: NonZeroU32::new(4),
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
                    //count: NonZeroU32::new(4),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let bind_group = device.create_bind_group({
            &wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &gpu_write_buffer,
                            offset: 0,
                            size: NonZeroU64::new(8),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &gpu_result_buffer,
                            offset: 0,
                            size: NonZeroU64::new(8),
                        }),
                    },
                ],
                label: Some("diffuse_bind_group"),
            }
        });

        let reverse_bind_group = device.create_bind_group({
            &wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &gpu_write_buffer,
                            offset: 0,
                            size: NonZeroU64::new(8),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &gpu_result_buffer,
                            offset: 0,
                            size: NonZeroU64::new(8),
                        }),
                    },
                ],
                label: Some("Reverse bind group"),
            }
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        // In WebGPU, the GPU command encoder returned by device.createCommandEncoder()
        // is the JavaScript object that builds a batch of "buffered" commands that will
        // be sent to the GPU at some point. The methods on GPUBuffer, on the other hand,
        // are "unbuffered", meaning they execute atomically at the time they are called.
        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("TestCommandEncoder"),
        });

        let mut pass_encoder =
            command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        pass_encoder.set_pipeline(&compute_pipeline);
        pass_encoder.set_bind_group(0, &bind_group, &[]);
        let workgroup_count_x = 2;
        let workgroup_count_y = 1;
        let workgroup_count_z = 1;
        pass_encoder.dispatch_workgroups(workgroup_count_x, workgroup_count_y, workgroup_count_z);
        drop(pass_encoder);

        let mut pass_encoder =
            command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        pass_encoder.set_pipeline(&compute_pipeline);
        pass_encoder.set_bind_group(0, &reverse_bind_group, &[]);
        let workgroup_count_x = 2;
        let workgroup_count_y = 1;
        let workgroup_count_z = 1;
        pass_encoder.dispatch_workgroups(workgroup_count_x, workgroup_count_y, workgroup_count_z);
        drop(pass_encoder);

        let mut pass_encoder =
            command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        pass_encoder.set_pipeline(&compute_pipeline);
        pass_encoder.set_bind_group(0, &bind_group, &[]);
        let workgroup_count_x = 2;
        let workgroup_count_y = 1;
        let workgroup_count_z = 1;
        pass_encoder.dispatch_workgroups(workgroup_count_x, workgroup_count_y, workgroup_count_z);
        drop(pass_encoder);

        // To end the compute pass encoder, call passEncoder.end(). Then, create a GPU buffer to use as
        // a destination to copy the result matrix buffer with copyBufferToBuffer. Finally, finish encoding
        // commands with copyEncoder.finish() and submit those to the GPU device queue by calling
        // device.queue.submit() with the GPU commands.
        let gpu_read_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: 8,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        });
        // Add to command queue for later execution:
        command_encoder.copy_buffer_to_buffer(&gpu_result_buffer, 0, &gpu_read_buffer, 0, 8);
        // Submit copy commands.
        let commands = command_encoder.finish();
        queue.submit(std::iter::once(commands));

        let gpu_read_buffer_slice = gpu_read_buffer.slice(..);
        gpu_read_buffer_slice.map_async(wgpu::MapMode::Read, |result| {
            result.unwrap();
        });
        instance.poll_all(true);
        let gpu_read_buffer_range = gpu_read_buffer_slice.get_mapped_range();
        let gg: &[u32] =
            unsafe { std::slice::from_raw_parts(gpu_read_buffer_range.as_ptr() as *const u32, 2) };
        println!("read buffer: {}, {}", gg[0], gg[1]);

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
