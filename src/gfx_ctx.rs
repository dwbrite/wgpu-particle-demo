use futures::executor::block_on;
use wgpu::{PresentMode, RequestAdapterOptions, SurfaceConfiguration, TextureUsages, TextureView};
use winit::window::Window;

pub struct GraphicsContext {
    pub window: winit::window::Window,
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub msaa_framebuffer: TextureView,
    pub sample_count: u32,
}

impl GraphicsContext {
    pub(crate) fn new(window: Window, sample_count: u32) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);

        let surface = unsafe { instance.create_surface(&window) };

        // TODO: hey asshole, fix this later - we need to know if a given adapter will be supported.
        let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap();

        println!("{:?}", adapter.get_info().backend);

        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits {
                    max_storage_buffer_binding_size: 256 << 20,
                    ..wgpu::Limits::default()
                }, // so we can run on webgl
                label: None,
            },
            None, // Trace path
        ))
        .unwrap();

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            // just for performance testing
            present_mode: PresentMode::Immediate,
        };

        surface.configure(&device, &config);

        let msaa_framebuffer = Self::create_msaa_framebuffer(&config, &device, sample_count);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            msaa_framebuffer,
            sample_count,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }

        self.msaa_framebuffer =
            Self::create_msaa_framebuffer(&self.config, &self.device, self.sample_count);
    }

    fn create_msaa_framebuffer(
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        sample_count: u32,
    ) -> TextureView {
        let multisampled_texture_extent = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
            size: multisampled_texture_extent,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
        };

        device
            .create_texture(multisampled_frame_descriptor)
            .create_view(&wgpu::TextureViewDescriptor::default())
    }
}
