use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use std::sync::{Arc, Once};
use wgpu::{
    CommandEncoderDescriptor, CompositeAlphaMode, DeviceDescriptor, Instance, InstanceDescriptor,
    LoadOp, MultisampleState, Operations, PresentMode, RenderPassColorAttachment,
    RenderPassDescriptor, RequestAdapterOptions, SurfaceConfiguration, TextureFormat,
    TextureUsages, TextureViewDescriptor,
};
use winit::{dpi::LogicalSize, event::{KeyEvent, WindowEvent}, event_loop::EventLoop, keyboard::{KeyCode, PhysicalKey}, window::Window};
use winit::event::{ElementState};
use log::info;
mod console;
static INIT: Once = Once::new();

fn main() {
    INIT.call_once(|| {
        env_logger::init();
    });
    let event_loop = EventLoop::new().unwrap();
    event_loop
        .run_app(&mut Application {window_state: None})
        .unwrap();
}

struct WindowState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: SurfaceConfiguration,
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: glyphon::Viewport,
    atlas: glyphon::TextAtlas,
    text_renderer: glyphon::TextRenderer,
    text_buffer: glyphon::Buffer,
    chat_text: String,
    render_pipeline: wgpu::RenderPipeline,

    window: Arc<Window>,
}

impl WindowState {
    async fn new(window: Arc<Window>) -> Self {
        let physical_size = window.inner_size();
        let scale_factor = window.scale_factor();

        let instance = Instance::new(InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await.unwrap();
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default(), None).await.unwrap();
        let surface = instance.create_surface(window.clone()).expect("Create Surface");

        //Render Pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let render_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let swapchain_format = TextureFormat::Bgra8UnormSrgb;

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: swapchain_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
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
            cache: None,
        });
    

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: physical_size.width,
            height: physical_size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let viewport = Viewport::new(&device, &cache);
        let mut atlas = TextAtlas::new(&device, &queue, &cache, swapchain_format);
        let text_renderer = 
        TextRenderer::new(&mut atlas, &device, MultisampleState::default(), None);
        let mut text_buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 42.0));

        let physical_width = (physical_size.width as f64 * scale_factor) as u32;
        let physical_height = (physical_size.height as f64 * scale_factor) as u32;

        text_buffer.set_size(
            &mut font_system,
            Some(physical_width as f32),
            Some(physical_height as f32)
        );

        let mut chat_text = "Hello world! 👋\nThis is rendered with 🦅 glyphon 🦁\nThe text below should be partially clipped.\na b c d e f g h i j k l m n o p q r s t u v w x y z".to_string();
        text_buffer.set_text(&mut font_system, &chat_text, Attrs::new().family(Family::SansSerif), Shaping::Advanced); 
        text_buffer.shape_until_scroll(&mut font_system, false);

        Self {
            device,
            queue,
            surface,
            surface_config,
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
            text_buffer,
            chat_text,
            render_pipeline,

            window,
            }
        }
    }

    struct Application {
        window_state: Option<WindowState>,
    }

    impl winit::application::ApplicationHandler for Application {
        fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop){ 
            if self.window_state.is_some() {
                return;
            }
            let (width, height) = (800, 600);
            let window_attributes = Window::default_attributes()
                .with_inner_size(LogicalSize::new(width as f64, height as f64))
                .with_title("glyphon hello world");
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());        
        self.window_state = Some(pollster::block_on(WindowState::new(window)));
        }
        fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            _window_id: winit::window::WindowId,
            event: WindowEvent,
        ) {
            let Some(state) = &mut self.window_state else {
                return;
            };
            let WindowState {
                window,
                device,
                queue,
                surface,
                surface_config,
                font_system,
                swash_cache,
                viewport,
                atlas,
                text_renderer,
                text_buffer,
                chat_text,
                render_pipeline,
                ..
            } = state;
            let chat_text = &mut state.chat_text;

            match event {
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Enter),
                            ..
                        },
                    ..
                } => {
                    let inputMode = true;
                    info!("hi hi hi");
                    console::write_to_console(text_buffer, font_system, chat_text, "hi");
                    window.request_redraw();
                }
                WindowEvent::Resized(size) => {
                    surface_config.width = size.width;
                    surface_config.height = size.height;
                    surface.configure(&device, surface_config);
                    window.request_redraw();
                }
                WindowEvent::RedrawRequested => {
                    viewport.update(
                        &queue,
                        Resolution {
                            width: surface_config.width,
                            height: surface_config.height,
                        },
                    );
                text_renderer
                    .prepare(
                        device, queue, font_system, atlas, viewport, 
                        [TextArea {
                            buffer: text_buffer,
                            left: 10.0,
                            top: 10.0,
                            scale: 1.0,
                            bounds: TextBounds {
                                left: 0,
                                top: 0,
                                right: 600,
                                bottom: 160
                            },
                            default_color: Color::rgb(255, 255, 255),
                            custom_glyphs: &[],
                        }],
                        swash_cache,
                    ).unwrap();
                    
                    let frame = surface.get_current_texture().unwrap();
                    let view = frame.texture.create_view(&TextureViewDescriptor::default());
                    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
                    {
                        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                            label: Some("Render pass"),
                            color_attachments: &[Some(RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: Operations {
                                    load: LoadOp::Clear(wgpu::Color::RED),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                    });
                    text_renderer.render(&atlas, &viewport, &mut pass).unwrap();
                    pass.set_pipeline(&render_pipeline);
                    pass.draw(0..3,0..1);
                }
                queue.submit(Some(encoder.finish()));
                frame.present();

                atlas.trim();
            }
            WindowEvent::CloseRequested => event_loop.exit(), 
            _ => {}
        }
    }
}