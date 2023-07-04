use wgpu::{Buffer, ColorTargetState, PipelineLayout, ShaderModule, TextureFormat, VertexBufferLayout};
use wgpu::util::DeviceExt;

pub enum ShaderType<'a> {
    Vertex(&'a [VertexBufferLayout<'static>]),
    Fragment
}

pub struct Shader<'a> {
    pub module: &'a wgpu::ShaderModule,
    pub entry: &'a str,
    pub shader_type: ShaderType<'a>,
}

/// Rendering pipeline
pub struct Pipeline {
    pipeline_layout: wgpu::PipelineLayout,
    pub pipeline: wgpu::RenderPipeline,
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub vert_buffer: wgpu::Buffer,
    pub ind_buffer: wgpu::Buffer,
}

impl Pipeline {
    pub(crate) fn new<T: bytemuck::Pod>(device: &wgpu::Device, name: &str, shaders: Vec<Shader>, format: TextureFormat, uniform_defaults: T, vert_buffer: Buffer, ind_buffer: Buffer) -> Result<Self, String> { // TODO Maybe use an error enum in the future
        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&*format!("{} Uniform Buffer", name)),
                contents: bytemuck::cast_slice(&[uniform_defaults]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some(&*format!("{} Uniform Bind Group Layout", name)),
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }
            ],
            label: Some(&*format!("{} Uniform Bind Group", name)),
        });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&*format!("{} Pipeline Layout", name)),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let mut vert: Option<Shader> = None;
        let mut frag: Option<Shader> = None;
        let mut vert_buffers: &[VertexBufferLayout] = &[];

        for shader in shaders {
            match shader.shader_type {
                ShaderType::Fragment => {
                    if frag.is_some() {
                        return Err(format!("{} Pipeline: Too many fragment shaders", name));
                    }

                    frag = Some(shader);
                }
                ShaderType::Vertex(buffers) => {
                    if vert.is_some() {
                        return Err(format!("{} Pipeline: Too many vertex shaders", name));
                    }
                    vert_buffers = buffers;
                    vert = Some(shader);
                }
            }
        }

        if let Some(vert) = vert {
            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&*format!("{} Pipeline", name)),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: vert.module,
                    entry_point: vert.entry,
                    buffers: vert_buffers,
                },
                fragment: match frag {
                    Some(shader) =>
                        Some(wgpu::FragmentState {
                            module: shader.module,
                            entry_point: shader.entry,
                            targets: &[],
                        }),
                    None => None
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

            Ok(Self {
                pipeline_layout,
                pipeline,

                uniform_buffer,
                uniform_bind_group,

                vert_buffer,
                ind_buffer
            })
        }
        else {
            return Err(format!("{} Pipeline: Too many vertex shaders", name));
        }
    }
}