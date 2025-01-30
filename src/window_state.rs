use crate::camera::{Camera, CameraUniform};
use crate::texture;
use crate::vertex::{Vertex, Instanced, InstanceRaw};
use cgmath::{InnerSpace, Rotation3, Zero};
use glyphon::{Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport};
use wgpu::util::RenderEncoder;
use winit::event::WindowEvent;
use std::collections::btree_map::Range;
use std::sync::Arc;
use wgpu::{util::DeviceExt, CommandEncoderDescriptor, CompositeAlphaMode, DeviceDescriptor, Instance, InstanceDescriptor, LoadOp, MultisampleState, Operations, PresentMode, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor};
use winit::window::Window;
use crate::cameracontroller::CameraController;
use std::iter;
//Skybox
// const VERTICES: &[Vertex] = &[
//     // Front face
//     Vertex { position: [-0.5, -0.5,  0.5], tex_coords: [0.0, 0.0], },
//     Vertex { position: [ 0.5, -0.5,  0.5], tex_coords: [1.0, 0.0], },
//     Vertex { position: [ 0.5,  0.5,  0.5], tex_coords: [1.0, 1.0], },
//     Vertex { position: [-0.5,  0.5,  0.5], tex_coords: [0.0, 1.0], },
//     // Back face
//     Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [1.0, 0.0], },
//     Vertex { position: [ 0.5, -0.5, -0.5], tex_coords: [0.0, 0.0], },
//     Vertex { position: [ 0.5,  0.5, -0.5], tex_coords: [0.0, 1.0], },
//     Vertex { position: [-0.5,  0.5, -0.5], tex_coords: [1.0, 1.0], },
//     // Top face
//     Vertex { position: [-0.5,  0.5, -0.5], tex_coords: [0.0, 0.0], },
//     Vertex { position: [ 0.5,  0.5, -0.5], tex_coords: [1.0, 0.0], },
//     Vertex { position: [ 0.5,  0.5,  0.5], tex_coords: [1.0, 1.0], },
//     Vertex { position: [-0.5,  0.5,  0.5], tex_coords: [0.0, 1.0], },
//     // Bottom face
//     Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [1.0, 1.0], },
//     Vertex { position: [ 0.5, -0.5, -0.5], tex_coords: [0.0, 1.0], },
//     Vertex { position: [ 0.5, -0.5,  0.5], tex_coords: [0.0, 0.0], },
//     Vertex { position: [-0.5, -0.5,  0.5], tex_coords: [1.0, 0.0], },
//     // Right face
//     Vertex { position: [ 0.5, -0.5, -0.5], tex_coords: [1.0, 0.0], },
//     Vertex { position: [ 0.5,  0.5, -0.5], tex_coords: [1.0, 1.0], },
//     Vertex { position: [ 0.5,  0.5,  0.5], tex_coords: [0.0, 1.0], },
//     Vertex { position: [ 0.5, -0.5,  0.5], tex_coords: [0.0, 0.0], },
//     // Left face
//     Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 0.0], },
//     Vertex { position: [-0.5,  0.5, -0.5], tex_coords: [0.0, 1.0], },
//     Vertex { position: [-0.5,  0.5,  0.5], tex_coords: [1.0, 1.0], },
//     Vertex { position: [-0.5, -0.5,  0.5], tex_coords: [1.0, 0.0], },
// ];
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        tex_coords: [0.4131759, 0.00759614],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        tex_coords: [0.0048659444, 0.43041354],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        tex_coords: [0.28081453, 0.949397],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        tex_coords: [0.85967, 0.84732914],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        tex_coords: [0.9414737, 0.2652641],
    }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4, /* padding */ 0];

const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

pub struct WindowState {
    pub instances: Vec<Instanced>,
    pub instance_buffer: wgpu::Buffer,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: SurfaceConfiguration,
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub viewport: Viewport,
    pub atlas: TextAtlas,
    pub text_renderer: TextRenderer,
    pub text_buffer: Buffer,
    pub chat_text: String,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub window: Arc<Window>,
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub diffuse_bind_group: wgpu::BindGroup,
    pub diffuse_texture: crate::texture::Texture,
    depth_texture: texture::Texture,
}
impl WindowState {
    pub async fn new(window: Arc<Window>) -> Self {
        let physical_size = window.inner_size();
        let scale_factor = window.scale_factor();


        //Instance code
        let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let position = cgmath::Vector3 { x: x as f32, y: 0.0, z: z as f32 } - INSTANCE_DISPLACEMENT;

                let rotation = if position.is_zero() {
                    // this is needed so an object at (0, 0, 0) won't get scaled to zero
                    // as Quaternions can affect scale if they're not created correctly
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };

                Instanced {
                    position, rotation,
                }
            })
        }).collect::<Vec<_>>();

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

        let instance_data = instances.iter().map(Instanced::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        



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

        let depth_texture = texture::Texture::create_depth_texture(&device, &surface_config, "depth_texture");


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
                    InstanceRaw::desc(),
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                //Todo test other comparefunction values
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
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
        let text_renderer = TextRenderer::new(
            &mut atlas, &device, wgpu::MultisampleState::default(),
            Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            })
        );

        // let text_renderer = 
        // TextRenderer::new(&mut atlas, &device, MultisampleState::default(), None);
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
            instances,
            depth_texture,
            instance_buffer
            }
        }

        pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
            if new_size.width > 0 && new_size.height > 0 {
                // Update surface configuration
                self.surface_config.width = new_size.width;
                self.surface_config.height = new_size.height;
                self.surface.configure(&self.device, &self.surface_config);
        
                // Recreate depth texture with the new size
                self.depth_texture.resize(
                    &self.device,
                    self.surface_config.width,
                    self.surface_config.height,
                );
        
                // Update camera aspect ratio
                self.camera.aspect = self.surface_config.width as f32 / self.surface_config.height as f32;
            }
        }

        pub fn input(&mut self, event: &WindowEvent) -> bool {
            self.camera_controller.process_events(event)
        }
        pub fn update(&mut self) {
            self.camera_controller.update_camera(&mut self.camera);
            self.camera_uniform.update_view_proj(&self.camera);
            self.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[self.camera_uniform]),
            );
        }
        pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
    
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
                render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

                render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

                // render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as _);
                &self.text_renderer.render(&self.atlas, &self.viewport, &mut render_pass).unwrap();

            }
            self.queue.submit(iter::once(encoder.finish()));
            output.present();
            Ok(())
        }
    }