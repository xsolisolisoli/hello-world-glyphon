use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use std::iter;
mod texture;
use std::sync::{Arc, Once};
use wgpu::{
    util::DeviceExt,
    CommandEncoderDescriptor, CompositeAlphaMode, DeviceDescriptor, Instance, InstanceDescriptor,
    LoadOp, MultisampleState, Operations, PresentMode, RenderPassColorAttachment,
    RenderPassDescriptor, RequestAdapterOptions, SurfaceConfiguration, TextureFormat,
    TextureUsages, TextureViewDescriptor,
};
use winit::{dpi::LogicalSize, event::{KeyEvent, WindowEvent}, event_loop::EventLoop, keyboard::{KeyCode, PhysicalKey}, window::Window};
use winit::event::{ElementState};
use log::info;
use env_logger::Env;
mod console;
mod cameracontroller;
use cameracontroller::CameraController;
static INIT: Once = Once::new();

fn main() {
    INIT.call_once(|| {
        env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    });
    let event_loop = EventLoop::new().unwrap();
    event_loop
        .run_app(&mut Application {window_state: None})
        .unwrap();
}


// unsafe impl bytemuck::Pod for Vertex {}
// unsafe impl bytemuck::Zeroable for Vertex {}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    // color: [f32; 3],
    tex_coords: [f32; 2], // NEW!
}


const VERTICES: &[Vertex] = &[
    // Changed
    Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614], }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354], }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397], }, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914], }, // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641], }, // E
];

const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}
impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}


impl CameraUniform {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}



#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);


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
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    window: Arc<Window>,
    camera: Camera,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: texture::Texture,
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2, // NEW!
                },
            ]
        }
    }
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

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        let num_indices = INDICES.len() as u32;
        let num_vertices = VERTICES.len() as u32;
        //Render Pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });



        let swapchain_format = TextureFormat::Bgra8UnormSrgb;
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
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

        //CAMERA
        let camera = Camera {
            eye:(0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: surface_config.width as f32 / surface_config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };
        let camera_controller = CameraController::new(0.2);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);
        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });


        //textures
        let diffuse_bytes = include_bytes!("./assets/cretin.png");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "cretin.png").unwrap(); // CHANGED!


        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view), // CHANGED!
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler), // CHANGED!
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );
    
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],            
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Vertex::desc(),
                ],
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
        

         
        
            

        // ///Possibly in wrong place ?TODO
        // queue.write_texture(
        //     wgpu::ImageCopyTexture {
        //         texture: &diffuse_texture,
        //         mip_level: 0,
        //         origin: wgpu::Origin3d::ZERO,
        //         aspect: wgpu::TextureAspect::All,
        //     },
        //     &diffuse_rgba,
        //     wgpu::ImageDataLayout {
        //         offset: 0,
        //         bytes_per_row: Some(4 * dimensions.0),
        //         rows_per_image: Some(dimensions.1),
        //     },
        //     texture_size,   
        // );
        
         
        
                 

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
            vertex_buffer,
            index_buffer,
            num_indices,
            window,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            diffuse_bind_group,
            diffuse_texture,
            }
        }


        
        fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
            if new_size.width > 0 && new_size.height > 0 {
                // self.size = new_size;
                self.surface_config.width = new_size.width;
                self.surface_config.height = new_size.height;
                self.surface.configure(&self.device, &self.surface_config);

                self.camera.aspect = self.surface_config.width as f32 / self.surface_config.height as f32;
            }
        }

        fn input(&mut self, event: &WindowEvent) -> bool {
            self.camera_controller.process_events(event)
        }
        fn update(&mut self) {
            self.camera_controller.update_camera(&mut self.camera);
            self.camera_uniform.update_view_proj(&self.camera);
            self.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[self.camera_uniform]),
            );
        }
        fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
            let output = self.surface.get_current_texture()?;
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
    
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
    
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
    
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
                render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
                &self.text_renderer.render(&self.atlas, &self.viewport, &mut render_pass).unwrap();

            }
            self.queue.submit(iter::once(encoder.finish()));
            output.present();
            Ok(())
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
            // state.input(&event);
            if !state.input(&event) {
                info!("input event not handled");
            }
            state.update();
            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if it's lost or outdated
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    let size = state.window.inner_size();
                    state.resize(size);
                },
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    log::error!("OutOfMemory");
                    // control_flow.exit();
                }

                // This happens when the a frame takes too long to present
                Err(wgpu::SurfaceError::Timeout) => {
                    log::warn!("Surface timeout")
                }
            }
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
                vertex_buffer,
                index_buffer,
                num_indices,
                diffuse_bind_group,
                diffuse_texture,
                camera,
                camera_controller,
                camera_uniform,
                camera_bind_group,
                camera_buffer,
                ..
            } = state;
            let chat_text = &mut state.chat_text;
            //Will be used for command mapping
            let mut key_table = vec![false; 255].into_boxed_slice();


            match event {
                //Todo refactor, this was for when enter was the only thing lol
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::KeyW),
                            ..
                        },
                    ..
                } => {
                    camera_controller.process_events(&event);
                    let inputMode = true;
                    info!("hi hi hi");
                    console::write_to_console(text_buffer, font_system, chat_text, "hi");
                    window.request_redraw();
                }
                //Any key pressed
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    camera_controller.process_events(&event);
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
                frame.present();

                atlas.trim();
            }
            WindowEvent::CloseRequested => event_loop.exit(), 
            _ => {}
        }
    }
}