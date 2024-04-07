use crate::rules::Rule;

pub struct ComputerFactory {
    shader: wgpu::ShaderModule,
    bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) size_buffer: wgpu::Buffer,
    pub(crate) rule_buffer: wgpu::Buffer,
}

impl ComputerFactory {
    pub fn new(device: &wgpu::Device) -> Self {
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

        let size_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("size_buffer"),
            size: (2 * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let rule_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rule_buffer"),
            size: (2 * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            shader,
            bind_group_layout,
            size_buffer,
            rule_buffer,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create(
        &self,
        device: &wgpu::Device,
        cells_width: u32,
        cells_height: u32,
        rule: &Rule,
        seed: u32,
        initial_density: u8,
        queue: &wgpu::Queue,
    ) -> Computer {
        use rand::prelude::{Rng, SeedableRng};
        use wgpu::util::DeviceExt;

        let size_array = [cells_width, cells_height];
        queue.write_buffer(&self.size_buffer, 0, bytemuck::cast_slice(&size_array));

        queue.write_buffer(
            &self.rule_buffer,
            0,
            bytemuck::cast_slice(&rule.rule_array()),
        );

        let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(u64::from(seed));
        let mut cells_vec = vec![0_u32; cells_width as usize * cells_height as usize];
        let initial_density = f32::from(initial_density) * 0.01;
        for cell in cells_vec.iter_mut() {
            if rng.gen::<f32>() < initial_density {
                *cell = 1;
            }
        }

        let cells_buffer_usages = {
            #[cfg(not(test))]
            {
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX
            }
            #[cfg(test)]
            {
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_SRC
            }
        };

        let cells_buffer_0 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&cells_vec),
            usage: cells_buffer_usages,
        });

        let cells_buffer_1 = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (cells_vec.len() * std::mem::size_of::<u32>()) as u64,
            usage: cells_buffer_usages,
            mapped_at_creation: false,
        });

        let create_bind_group = |from_buffer, to_buffer, bind_group_name| {
            device.create_bind_group({
                &wgpu::BindGroupDescriptor {
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: from_buffer,
                                offset: 0,
                                size: None,
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: to_buffer,
                                offset: 0,
                                size: None,
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &self.size_buffer,
                                offset: 0,
                                size: None,
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &self.rule_buffer,
                                offset: 0,
                                size: None,
                            }),
                        },
                    ],
                    label: Some(bind_group_name),
                }
            })
        };

        let compute_bind_group_from_0_to_1 =
            create_bind_group(&cells_buffer_0, &cells_buffer_1, "compute_bind_group_0");
        let compute_bind_group_from_1_to_0 =
            create_bind_group(&cells_buffer_1, &cells_buffer_0, "compute_bind_group_1");

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
            constants: &Default::default(),
        });

        Computer {
            cells_width,
            cells_height,
            compute_pipeline,
            currently_computed_is_0: true,
            compute_bind_group_from_0_to_1,
            compute_bind_group_from_1_to_0,
            cells_buffer_0,
            cells_buffer_1,
        }
    }
}

pub struct Computer {
    cells_width: u32,
    cells_height: u32,
    compute_pipeline: wgpu::ComputePipeline,
    compute_bind_group_from_0_to_1: wgpu::BindGroup,
    compute_bind_group_from_1_to_0: wgpu::BindGroup,
    pub currently_computed_is_0: bool,
    pub cells_buffer_0: wgpu::Buffer,
    pub cells_buffer_1: wgpu::Buffer,
}

impl Computer {
    pub fn enqueue(&mut self, command_encoder: &mut wgpu::CommandEncoder) {
        let mut pass_encoder =
            command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        pass_encoder.set_pipeline(&self.compute_pipeline);

        pass_encoder.set_bind_group(
            0,
            if self.currently_computed_is_0 {
                &self.compute_bind_group_from_0_to_1
            } else {
                &self.compute_bind_group_from_1_to_0
            },
            &[],
        );

        self.currently_computed_is_0 = !self.currently_computed_is_0;

        let workgroup_width = 8;
        assert_eq!(self.cells_width % workgroup_width, 0);
        assert_eq!(self.cells_height % workgroup_width, 0);
        let workgroup_count_x = (self.cells_width + workgroup_width - 1) / workgroup_width;
        let workgroup_count_y = (self.cells_height + workgroup_width - 1) / workgroup_width;
        let workgroup_count_z = 1;
        pass_encoder.dispatch_workgroups(workgroup_count_x, workgroup_count_y, workgroup_count_z);
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    struct CpuBasedGameOfLife {
        cells: Vec<u32>,
        width: usize,
        height: usize,
    }

    impl CpuBasedGameOfLife {
        fn live_neighbours_at(&self, x: usize, y: usize) -> u32 {
            let mut result = 0;
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let n_x = (x as i32 - dx).rem_euclid(self.width as i32) as usize;
                    let n_y = (y as i32 - dy).rem_euclid(self.height as i32) as usize;
                    if self.cells[n_x + n_y * self.width] > 0 {
                        result += 1;
                    }
                }
            }
            result
        }
        fn next_generation(&mut self, rule: &Rule) {
            let mut new_cells = vec![0; self.cells.len()];
            for x in 0..self.width {
                for y in 0..self.height {
                    let current_generation = self.cells[x + y * self.width];
                    let was_alive = current_generation > 0;
                    let num_live_neighbours = self.live_neighbours_at(x, y);
                    let new_is_alive = if was_alive { rule.survives } else { rule.born }
                        & (1 << num_live_neighbours)
                        > 0;
                    new_cells[x + y * self.width] = if new_is_alive {
                        current_generation + 1
                    } else {
                        0
                    };
                }
            }
            let _ = std::mem::replace(&mut self.cells, new_cells);
        }
    }

    #[test]
    fn test_computer() {
        async fn async_test_computer() {
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
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::default(),
                        label: None,
                    },
                    None,
                )
                .await
                .map_err(|e| format!("request_device failed: {}", e))
                .unwrap();

            for cells_width in [64, 128] {
                let cells_height = cells_width;

                let creator = ComputerFactory::new(&device);
                let seed = 1;
                let initial_density = 50;
                let rule = &crate::rules::RULES[0];
                let mut computer = creator.create(
                    &device,
                    cells_width,
                    cells_height,
                    rule,
                    seed,
                    initial_density,
                    &queue,
                );
                instance.poll_all(true);

                let copy_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("copy_buffer"),
                    size: computer.cells_buffer_1.size(),
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                encoder.copy_buffer_to_buffer(
                    &computer.cells_buffer_0,
                    0,
                    &copy_buffer,
                    0,
                    copy_buffer.size(),
                );
                queue.submit(std::iter::once(encoder.finish()));
                let gpu_read_buffer_slice = copy_buffer.slice(..);
                gpu_read_buffer_slice.map_async(wgpu::MapMode::Read, Result::unwrap);
                instance.poll_all(true);
                let gpu_read_buffer_range = gpu_read_buffer_slice.get_mapped_range();
                let cells_data: &[u32] = bytemuck::cast_slice(&gpu_read_buffer_range);
                assert_eq!(
                    cells_width as usize * cells_height as usize,
                    cells_data.len()
                );
                let mut cpu_game_of_life = CpuBasedGameOfLife {
                    cells: cells_data.to_vec(),
                    width: cells_width as usize,
                    height: cells_height as usize,
                };
                drop(gpu_read_buffer_range);
                copy_buffer.unmap();

                assert!(computer.currently_computed_is_0);
                for iteration in 0..100 {
                    let mut encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                    computer.enqueue(&mut encoder);
                    assert!(computer.currently_computed_is_0 == (iteration % 2 == 1));

                    encoder.copy_buffer_to_buffer(
                        if computer.currently_computed_is_0 {
                            &computer.cells_buffer_0
                        } else {
                            &computer.cells_buffer_1
                        },
                        0,
                        &copy_buffer,
                        0,
                        copy_buffer.size(),
                    );

                    queue.submit(std::iter::once(encoder.finish()));
                    instance.poll_all(true);

                    let gpu_read_buffer_slice = copy_buffer.slice(..);
                    gpu_read_buffer_slice.map_async(wgpu::MapMode::Read, Result::unwrap);
                    instance.poll_all(true);
                    let gpu_read_buffer_range = gpu_read_buffer_slice.get_mapped_range();
                    let cells_data: &[u32] = bytemuck::cast_slice(&gpu_read_buffer_range);
                    cpu_game_of_life.next_generation(rule);
                    assert_eq!(
                        &cpu_game_of_life.cells[..],
                        cells_data,
                        "Iteration: {}",
                        iteration
                    );
                    drop(gpu_read_buffer_range);
                    copy_buffer.unmap();
                }
            }
        }
        pollster::block_on(async_test_computer());
    }
}
