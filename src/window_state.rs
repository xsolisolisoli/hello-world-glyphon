use crate::camera::{Camera, CameraUniform};
use crate::vertex::{Vertex, Instanced, InstanceRaw};
use glyphon::{Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport};
use std::sync::Arc;
use wgpu::{util::DeviceExt, CommandEncoderDescriptor, CompositeAlphaMode, DeviceDescriptor, Instance, InstanceDescriptor, LoadOp, MultisampleState, Operations, PresentMode, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor};
use winit::window::Window;
use crate::cameracontroller::CameraController;

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
}

impl WindowState {
    // ...existing code...
}
