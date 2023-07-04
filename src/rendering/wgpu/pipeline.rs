use std::cell::Cell;
use std::mem;
use wgpu::{BindGroup, BindGroupLayout, BlendState, Buffer, ColorTargetState, ColorWrites, CommandEncoder, PipelineLayout, RenderPass, ShaderModule, SurfaceError, SurfaceTexture, TextureFormat, TextureView, VertexBufferLayout};
use wgpu::util::DeviceExt;
use crate::rendering::wgpu::core::Core;

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
    pub vert_num: usize,
    pub ind_buffer: wgpu::Buffer,
    bindings: Vec<BindGroup>,
    // Used in render loop
    output: Cell<Option<SurfaceTexture>>,
}

pub struct PipelineDescriptor<'a, T: bytemuck::Pod> {
    pub device: &'a wgpu::Device,
    pub name: &'a str,
    pub shaders: Vec<Shader<'a>>,
    pub uniform_defaults: T,
    pub vert_buffer: Buffer,
    pub vert_num: usize,
    pub ind_buffer: Buffer,
    pub bindings: Vec<BindGroup>,
    pub bindings_layout: Vec<BindGroupLayout>,
    pub format: TextureFormat
}

impl Pipeline {
    pub fn new<T: bytemuck::Pod>(mut descriptor: PipelineDescriptor<T>) -> Result<Self, String> { // TODO Maybe use an error enum in the future
        let uniform_buffer = descriptor.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&*format!("{} Uniform Buffer", descriptor.name)),
                contents: bytemuck::cast_slice(&[descriptor.uniform_defaults]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let uniform_bind_group_layout = descriptor.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some(&*format!("{} Uniform Bind Group Layout", descriptor.name)),
        });

        let uniform_bind_group = descriptor.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }
            ],
            label: Some(&*format!("{} Uniform Bind Group", descriptor.name)),
        });

        descriptor.bindings_layout.insert(0, uniform_bind_group_layout);
        let mut bindings_ref: Vec<&BindGroupLayout> = vec![];
        for i in 0..descriptor.bindings_layout.len() {
            bindings_ref.push(&descriptor.bindings_layout[i]);
        }

        let pipeline_layout =
            descriptor.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&*format!("{} Pipeline Layout", descriptor.name)),
                bind_group_layouts: bindings_ref.as_slice(),
                push_constant_ranges: &[],
            });

        let mut vert: Option<Shader> = None;
        let mut frag: Option<Shader> = None;
        let mut vert_buffers: &[VertexBufferLayout] = &[];

        for shader in descriptor.shaders {
            match shader.shader_type {
                ShaderType::Fragment => {
                    if frag.is_some() {
                        return Err(format!("{} Pipeline: Too many fragment shaders", descriptor.name));
                    }

                    frag = Some(shader);
                }
                ShaderType::Vertex(buffers) => {
                    if vert.is_some() {
                        return Err(format!("{} Pipeline: Too many vertex shaders", descriptor.name));
                    }
                    vert_buffers = buffers;
                    vert = Some(shader);
                }
            }
        }

        let targets = &[Some(ColorTargetState {
            format: descriptor.format,
            blend: Some(BlendState::REPLACE),
            write_mask: ColorWrites::ALL
        })];

        if let Some(vert) = vert {
            let pipeline = descriptor.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&*format!("{} Pipeline", descriptor.name)),
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
                            targets: targets,
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

                vert_buffer: descriptor.vert_buffer,
                ind_buffer: descriptor.ind_buffer,
                bindings: descriptor.bindings,
                vert_num: descriptor.vert_num,

                output: Cell::new(None),
            })
        }
        else {
            return Err(format!("{} Pipeline: Too many vertex shaders", descriptor.name));
        }
    }

    pub fn create_window_view(&self, rendering_core: &Core) -> Result<TextureView, SurfaceError> {
        let output = rendering_core.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.output.set(Some(output));
        Ok(view)
    }

    pub fn begin_render_pass<'a>(&'a self, view: &'a TextureView, encoder: &'a mut CommandEncoder) -> Result<wgpu::RenderPass, SurfaceError> {
        // I really mean this abstraction layer is simple
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0, // if window is transparent remember to make this 0
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        Ok(render_pass)
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        for i in 0..self.bindings.len() {
            render_pass.set_bind_group((i + 1) as u32, &self.bindings[i], &[]);
        }
        render_pass.set_vertex_buffer(0, self.vert_buffer.slice(..));
        render_pass.set_index_buffer(self.ind_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..(self.vert_num as u32), 0, 0..1);
    }

    pub fn submit_frame(&self, rendering_core: &mut Core, encoder: CommandEncoder) {
        rendering_core.queue.submit(std::iter::once(encoder.finish()));
        self.output.take().unwrap().present();
        self.output.set(None);
    }

}