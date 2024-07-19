use std::sync::Arc;
use wgpu::{CompositeAlphaMode, DeviceDescriptor};
use winit::window::Window;

pub struct WgpuCtx<'window> {
    surface: wgpu::Surface<'window>,
    surface_config: wgpu::SurfaceConfiguration,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl<'window> WgpuCtx<'window> {
    pub async fn new_async(window: Arc<Window>) -> WgpuCtx<'window> {
        let instance_desc = wgpu::InstanceDescriptor {
            flags: wgpu::InstanceFlags::empty(),
            backends: wgpu::Backends::GL, // DX12, GL or VULKAN does not matter nothing works
            ..Default::default()
        };
        let instance = wgpu::Instance::new(instance_desc);

        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("Failed to find an appropriate adapter");
        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    ..DeviceDescriptor::default()
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let mut size = window.inner_size();

        let width = size.width.max(1);
        let height = size.height.max(1);
        let surface_capabilities = surface.get_capabilities(&adapter);
        println!("alpha_modes:{:?}", surface_capabilities.alpha_modes);
        println!("formats:{:?}", surface_capabilities.formats);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_capabilities.formats[1],
            width,
            height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: CompositeAlphaMode::Auto,
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);
        WgpuCtx {
            surface,
            surface_config,
            adapter,
            device,
            queue,
        }
    }

    pub fn new(window: Arc<Window>) -> WgpuCtx<'window> {
        pollster::block_on(WgpuCtx::new_async(window))
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        let (width, height) = new_size;
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn draw(&mut self) {
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let _r_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }
        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }
}
