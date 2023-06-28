pub struct ComputerFactory {
    shader: wgpu::ShaderModule,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl ComputerFactory {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
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
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
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

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create(
        &self,
        device: &wgpu::Device,
        cells_width: u32,
        cells_height: u32,
        size_buffer: &wgpu::Buffer,
        rule_buffer: &wgpu::Buffer,
        seed: u32,
        initial_density: u8,
    ) -> Computer {
        use rand::prelude::{Rng, SeedableRng};
        use wgpu::util::DeviceExt;

        let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(u64::from(seed));
        let mut cells_vec = vec![0_u32; cells_width as usize * cells_height as usize];
        let initial_density = f32::from(initial_density) * 0.01;
        for cell in cells_vec.iter_mut() {
            if rng.gen::<f32>() < initial_density {
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
            size: (cells_vec.len() * std::mem::size_of::<u32>()) as u64,
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
                            buffer: size_buffer,
                            offset: 0,
                            size: None,
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: rule_buffer,
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
                            buffer: size_buffer,
                            offset: 0,
                            size: None,
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: rule_buffer,
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
            cells_width,
            cells_height,
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
    cells_height: u32,
    compute_pipeline: wgpu::ComputePipeline,
    compute_bind_group_0: wgpu::BindGroup,
    compute_bind_group_1: wgpu::BindGroup,
    pub(crate) cells_buffer_0: wgpu::Buffer,
    pub(crate) cells_buffer_1: wgpu::Buffer,
}

impl Computer {
    pub(crate) fn enqueue(&self, is_even: bool, command_encoder: &mut wgpu::CommandEncoder) {
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
        let workgroup_count_x = (self.cells_width + workgroup_width - 1) / workgroup_width;
        let workgroup_count_y = (self.cells_height + workgroup_width - 1) / workgroup_width;
        let workgroup_count_z = 1;
        pass_encoder.dispatch_workgroups(workgroup_count_x, workgroup_count_y, workgroup_count_z);
    }
}
