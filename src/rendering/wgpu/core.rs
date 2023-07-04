use wgpu::{CommandEncoder, Device, DeviceType, PresentMode, Queue, Surface, SurfaceConfiguration, TextureFormat};
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub struct Core {
    pub instance: wgpu::Instance,
    pub window: Window,
    pub window_size: PhysicalSize<u32>,
    pub surface: Surface,
    pub surface_format: TextureFormat,
    pub device: Device,
    pub config: SurfaceConfiguration,
    pub queue: Queue,
}

impl Core {
    pub async fn new(title: &str, window_size: PhysicalSize<u32>, event_loop: &EventLoop<()>) -> Self {
        cfg_if::cfg_if! {
			if #[cfg(target_arch = "wasm32")] {
				std::panic::set_hook(Box::new(console_error_panic_hook::hook));
				console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
			} else {
				env_logger::init();
			}
		}

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let window = WindowBuilder::new().build(event_loop).unwrap();
        window.set_inner_size(window_size);
        window.set_title(title);

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("powderrs")?;
                    let canvas = web_sys::Element::from(window.canvas());
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let device_type = match adapter.get_info().device_type {
            DeviceType::Other => {"Other"}
            DeviceType::IntegratedGpu => {"Integrated GPU"}
            DeviceType::DiscreteGpu => {"Discrete GPU"}
            DeviceType::VirtualGpu => {"Virtual GPU"}
            DeviceType::Cpu => {"CPU"}
        };

        println!("Device: {}, Driver: {} {}, Type: {}", adapter.get_info().name, adapter.get_info().driver, adapter.get_info().driver_info, device_type);

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None,
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let mut present_mode = PresentMode::Immediate;

        #[cfg(target_arch = "wasm32")]
        {
            present_mode = PresentMode::Fifo; // Bad performance ):
        }

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        Self {
            instance,
            window,
            window_size,
            surface,
            surface_format,
            device,
            queue,
            config,
        }
    }
}