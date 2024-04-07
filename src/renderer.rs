use crate::computer::Computer;
use wgpu::util::DeviceExt;

pub struct RendererFactory {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    shader: wgpu::ShaderModule,
    square_buffer: wgpu::Buffer,
}

impl RendererFactory {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
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
        }
    }

    pub(crate) fn create(
        &self,
        device: &wgpu::Device,
        computer: &Computer,
        size_buffer: &wgpu::Buffer,
        cells_width: u32,
        cells_height: u32,
        texture_format: wgpu::TextureFormat,
    ) -> Renderer {
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

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&self.pipeline_layout),
            vertex: wgpu::VertexState {
                buffers: &[cells_stride.clone(), square_stride.clone()],
                entry_point: "vertex_main",
                module: &self.shader,
                constants: &Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                entry_point: "fragment_main",
                module: &self.shader,
                targets: &[Some(texture_format.into())],
                constants: &Default::default(),
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
        });

        let size_bind_group = device.create_bind_group({
            &wgpu::BindGroupDescriptor {
                layout: &self.bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: size_buffer,
                        offset: 0,
                        size: None,
                    }),
                }],
                label: Some("size_bind_group"),
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
            render_bundle_encoder.draw(0..4, 0..(cells_width * cells_height));
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
    pub(crate) fn enqueue(
        &self,
        render_first_buffer: bool,
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
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        render_pass.execute_bundles(std::iter::once(if render_first_buffer {
            &self.render_bundle_0
        } else {
            &self.render_bundle_1
        }));
    }
}
