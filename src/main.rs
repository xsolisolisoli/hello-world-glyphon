mod texture;
mod console;
mod cameracontroller;
mod vertex;
mod camera;
mod window_state;

use crate::window_state::WindowState;
use std::sync::{Arc, Once};
use winit::{dpi::LogicalSize, event::{KeyEvent, WindowEvent}, event_loop::EventLoop, keyboard::{KeyCode, PhysicalKey}, window::Window};
use log::info;
use env_logger::Env;
use glyphon::{Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport};

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
            instances,
            instance_buffer,
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