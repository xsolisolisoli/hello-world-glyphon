use crate::camera::{self, Camera, CameraUniform};
use crate::{cameracontroller, light, model, rendering, resources, texture};
use crate::vertex::{Instanced, InstanceRaw};
use crate::model::{DrawLight, DrawModel, Model, ModelVertex, Vertex};
use crate::common::utils::IsNullOrEmpty;
use cgmath::{InnerSpace, Rotation3, Zero};
use glyphon::{Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport};
use wgpu::util::RenderEncoder;
use instant::Duration;
use winit::event::MouseButton;

use std::collections::btree_map::Range;
use std::sync::Arc;
use std::time::Instant;
use wgpu::{util::DeviceExt, CommandEncoderDescriptor, CompositeAlphaMode, DeviceDescriptor, Instance, InstanceDescriptor, LoadOp, MultisampleState, Operations, PresentMode, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor};
use winit::{
    dpi::LogicalSize,
    event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};
use crate::cameracontroller::CameraController;

use std::iter;

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4, /* padding */ 0];
const NUM_INSTANCES_PER_ROW: u32 = 5;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

pub struct WindowState<'a> {
    pub window: &'a Arc<Window>,
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
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub diffuse_bind_group: wgpu::BindGroup,
    pub diffuse_texture: crate::texture::Texture,
    depth_texture: texture::Texture,
    light_buffer: wgpu::Buffer,
    light_uniform: light::LightUniform,
    light_bind_group_layout: wgpu::BindGroupLayout,
    light_bind_group: wgpu::BindGroup,
    light_render_pipeline: wgpu::RenderPipeline,
    pub obj_model: Model,
    pub mouse_pressed: bool,
    projection: camera::Projection,    
}

const VERTICES: &[ModelVertex] = &[
    // Front face
    ModelVertex { position: [-0.5, -0.5,  0.5], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 1.0] },
    ModelVertex { position: [ 0.5, -0.5,  0.5], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 1.0] },
    ModelVertex { position: [ 0.5,  0.5,  0.5], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 1.0] },
    ModelVertex { position: [-0.5,  0.5,  0.5], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 1.0] },
    // Back face
    ModelVertex { position: [-0.5, -0.5, -0.5], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, -1.0] },
    ModelVertex { position: [ 0.5, -0.5, -0.5], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, -1.0] },
    ModelVertex { position: [ 0.5,  0.5, -0.5], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, -1.0] },
    ModelVertex { position: [-0.5,  0.5, -0.5], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, -1.0] },
    // Top face
    ModelVertex { position: [-0.5,  0.5, -0.5], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
    ModelVertex { position: [ 0.5,  0.5, -0.5], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
    ModelVertex { position: [ 0.5,  0.5,  0.5], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
    ModelVertex { position: [-0.5,  0.5,  0.5], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
    // Bottom face
    ModelVertex { position: [-0.5, -0.5, -0.5], tex_coords: [1.0, 1.0], normal: [0.0, -1.0, 0.0] },
    ModelVertex { position: [ 0.5, -0.5, -0.5], tex_coords: [0.0, 1.0], normal: [0.0, -1.0, 0.0] },
    ModelVertex { position: [ 0.5, -0.5,  0.5], tex_coords: [0.0, 0.0], normal: [0.0, -1.0, 0.0] },
    ModelVertex { position: [-0.5, -0.5,  0.5], tex_coords: [1.0, 0.0], normal: [0.0, -1.0, 0.0] },
    // Right face
    ModelVertex { position: [ 0.5, -0.5, -0.5], tex_coords: [1.0, 0.0], normal: [1.0, 0.0, 0.0] },
    ModelVertex { position: [ 0.5,  0.5, -0.5], tex_coords: [1.0, 1.0], normal: [1.0, 0.0, 0.0] },
    ModelVertex { position: [ 0.5,  0.5,  0.5], tex_coords: [0.0, 1.0], normal: [1.0, 0.0, 0.0] },
    ModelVertex { position: [ 0.5, -0.5,  0.5], tex_coords: [0.0, 0.0], normal: [1.0, 0.0, 0.0] },
    // Left face
    ModelVertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 0.0], normal: [-1.0, 0.0, 0.0] },
    ModelVertex { position: [-0.5,  0.5, -0.5], tex_coords: [0.0, 1.0], normal: [-1.0, 0.0, 0.0] },
    ModelVertex { position: [-0.5,  0.5,  0.5], tex_coords: [1.0, 1.0], normal: [-1.0, 0.0, 0.0] },
    ModelVertex { position: [-0.5, -0.5,  0.5], tex_coords: [1.0, 0.0], normal: [-1.0, 0.0, 0.0] },
];

impl<'a> WindowState<'a> {
    pub async fn new(window: &'a Arc<Window>) -> Self {
        let physical_size = window.inner_size();
        let scale_factor = window.scale_factor();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).expect("Create Surface");
        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await.unwrap();

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default(), None).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: physical_size.width,
            height: physical_size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let camera = camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection = camera::Projection::new(surface_config.width, surface_config.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = cameracontroller::CameraController::new(4.0, 0.4);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        const SPACE_BETWEEN: f32 = 2.0;
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let position = cgmath::Vector3 { x, y: 0.0, z };

                    let rotation = cgmath::Quaternion::from_axis_angle(
                        cgmath::Vector3::unit_z(),
                        cgmath::Deg(0.0));
                    Instanced { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instanced::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let obj_model = resources::load_model("cube.obj", &device, &queue, &texture_bind_group_layout)
            .await.unwrap();

        let light_uniform = light::LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Light Bind Group Layout"),
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
                },
            ],
        });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = INDICES.len() as u32;

        let diffuse_bytes = include_bytes!("./assets/cretin.png");
        let diffuse_texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "cretin.png").unwrap();

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                }
            ],
            label: Some("diffuse_bind_group"),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &camera_bind_group_layout,
                &light_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
            };
            rendering::create_render_pipeline(
                &device,
                &render_pipeline_layout,
                surface_config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), InstanceRaw::desc()],
                shader,
            )
        };

        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/light.wgsl").into()),
            };
            rendering::create_render_pipeline(
                &device,
                &layout,
                surface_config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc()],
                shader,
            )
        };

        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let viewport = Viewport::new(&device, &cache);
        let mut atlas = TextAtlas::new(&device, &queue, &cache, surface_config.format);
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

        let depth_texture = texture::Texture::create_depth_texture(&device, &surface_config, "depth_texture");

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
            obj_model,
            light_buffer,
            light_uniform,
            light_bind_group,
            light_bind_group_layout,
            light_render_pipeline,
            depth_texture,
            instance_buffer,
            projection,
            mouse_pressed: false,
        }
    }

    // Remaining methods unchanged for brevity
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) { /* ... */ }
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        ..
                    },
                ..
            } => self.camera_controller.process_keyboard(*key, *state),
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }
    pub fn update(&mut self, dt: instant::Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform.update_view_proj(&self.camera, &self.projection);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        let old_position: cgmath::Vector3<_> = self.light_uniform.position.into();
        self.light_uniform.position =
        (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(60.0 * dt.as_secs_f32()))
        * old_position).into(); // UPDATED!
    
        self.queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light_uniform]));


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

            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_pipeline(&self.light_render_pipeline);
            render_pass.draw_light_model(
                &self.obj_model,
                &self.camera_bind_group,
                &self.light_bind_group,
            );

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]); // Ensure this line is present
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]); // Ensure this line is present
            render_pass.set_bind_group(2, &self.light_bind_group, &[]); // Ensure this line is present
            render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as u32); 

            &self.text_renderer.render(&self.atlas, &self.viewport, &mut render_pass).unwrap();
        }
        self.queue.submit(iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}