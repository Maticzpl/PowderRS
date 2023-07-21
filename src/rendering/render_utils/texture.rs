use std::rc::Rc;

pub struct Texture {
	pub texture: wgpu::Texture,
	view:        wgpu::TextureView,
	sampler:     wgpu::Sampler,

	pub bind_group:        Rc<wgpu::BindGroup>,
	pub bind_group_layout: Rc<wgpu::BindGroupLayout>,
	pub size:              wgpu::Extent3d
}

impl Texture {
	pub fn new(
		device: &wgpu::Device,
		size: wgpu::Extent3d,
		format: wgpu::TextureFormat,
		usage: wgpu::TextureUsages,
		visibility: wgpu::ShaderStages,
		name: &str
	) -> Self {
		let texture = device.create_texture(&wgpu::TextureDescriptor {
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format,
			usage,
			label: Some(&*format!("{} texture ", name)),
			view_formats: &[]
		});
		let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

		let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::Repeat,
			address_mode_v: wgpu::AddressMode::Repeat,
			address_mode_w: wgpu::AddressMode::Repeat,
			mag_filter: wgpu::FilterMode::Nearest, // This is a pixel game ok?
			min_filter: wgpu::FilterMode::Nearest,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		});

		let texture_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility,
						ty: wgpu::BindingType::Texture {
							multisampled:   false,
							view_dimension: wgpu::TextureViewDimension::D2,
							sample_type:    wgpu::TextureSampleType::Float { filterable: false }
						},
						count: None
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility,
						ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
						count: None
					}
				],
				label:   Some(&*format!("{} texture bind group layout", name))
			});

		let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout:  &texture_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding:  0,
					resource: wgpu::BindingResource::TextureView(&texture_view)
				},
				wgpu::BindGroupEntry {
					binding:  1,
					resource: wgpu::BindingResource::Sampler(&sampler)
				}
			],
			label:   Some(&*format!("{} texture bind group", name))
		});

		Self {
			texture,
			view: texture_view,
			sampler,
			size,
			bind_group: Rc::new(texture_bind_group),
			bind_group_layout: Rc::new(texture_bind_group_layout)
		}
	}
}
