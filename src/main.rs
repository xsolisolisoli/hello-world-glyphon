mod texture;
mod console;
mod cameracontroller;
mod vertex;
mod camera;
mod window_state;

use crate::window_state::WindowState;
use std::sync::{Arc, Once};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};
use log::info;
use env_logger::Env;
use glyphon::{Attrs, Buffer, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea, TextBounds};

static INIT: Once = Once::new();

fn main() {
    INIT.call_once(|| {
        env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    });
    let event_loop = EventLoop::new().unwrap();
    event_loop
        .run_app(&mut Application { window_state: None })
        .unwrap();
}

struct Application {
    window_state: Option<WindowState>,
}

impl winit::application::ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) { 
        if self.window_state.is_some() {
            return;
        }
        
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_inner_size(LogicalSize::new(800.0, 600.0))
                        .with_title("glyphon hello world")
                )
                .unwrap()
        );
        
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

        // Handle input and updates first
        let handled = state.input(&event);
        if !handled {
            info!("input event not handled");
        }
        state.update();

        // Handle window events
        match event {
            WindowEvent::Resized(physical_size) => {
                state.resize(physical_size);
                state.window.request_redraw();
            }
            WindowEvent::KeyboardInput { event: KeyEvent { state: ElementState::Pressed, physical_key, .. }, .. } => {
                match physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => {
                        console::write_to_console(
                            &mut state.text_buffer,
                            &mut state.font_system,
                            &mut state.chat_text,
                            "hi",
                        );
                        state.window.request_redraw();
                    }
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => {
                // Update text rendering viewport
                state.viewport.update(
                    &state.queue,
                    Resolution {
                        width: state.surface_config.width,
                        height: state.surface_config.height,
                    },
                );

                // Prepare text rendering
                let text_area = TextArea {
                    buffer: &state.text_buffer,
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
                };

                state.text_renderer.prepare(
                    &state.device,
                    &state.queue,
                    &mut state.font_system,
                    &mut state.atlas,
                    &state.viewport,
                    [text_area],
                    &mut state.swash_cache,
                ).unwrap();

                // Handle main rendering
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.window.inner_size());
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }

                state.atlas.trim();
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
    }
}