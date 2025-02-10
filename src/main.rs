mod texture;
mod console;
mod cameracontroller;
mod vertex;
mod camera;
mod window_state;
use std::{f32::consts::FRAC_PI_2, time::Instant};
mod model;
mod rendering;
mod light;
mod resources;

mod common {
    pub mod utils;
}

use crate::window_state::WindowState;
use wgpu_sandbox::run;
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
use anyhow::*;
use std::env;

// static INIT: Once = Once::new();

fn main() {
    pollster::block_on(run());
}

// struct Application {
//     window_state: Option<WindowState>,
// }

// impl winit::application::ApplicationHandler for Application {
//     fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) { 
//         if self.window_state.is_some() {
//             return;
//         }
        
//         // let window = Arc::new(
//         //     event_loop
//         //         .create_window(
//         //             Window::default_attributes()
//         //                 .with_inner_size(LogicalSize::new(800.0, 600.0))
//         //                 .with_title("glyphon hello world")
//         //         )
//         //         .unwrap()
//         // );
        
//         self.window_state = Some(pollster::block_on(WindowState::new(window)));
//     }

//     fn device_event(
//         &mut self,
//         _event_loop: &winit::event_loop::ActiveEventLoop,
//         _device_id: winit::event::DeviceId,
//         _event: winit::event::DeviceEvent,
//     ) {
//         match _event {
//             winit::event::DeviceEvent::MouseMotion { delta } => {
//                 if let Some(state) = &mut self.window_state {
//                     state.camera_controller.process_mouse(delta.0, delta.1);
//                 }
//             }
//             _ => {}
//         }
//     }
//     fn window_event(
//         &mut self,
//         event_loop: &winit::event_loop::ActiveEventLoop,
//         _window_id: winit::window::WindowId,
//         event: WindowEvent,
//     ) {
//         let mut last_render_time = Instant::now();

//         let Some(state) = &mut self.window_state else {
//             return;
//         };
//         // Handle input and updates first
//         // let handled = state.input(&event);
//         // if !handled {
//         //     info!("input event not handled");
//         // }
//         let is_keyboard_event = matches!(event, WindowEvent::KeyboardInput { .. });
//         let handled = state.input(&event);

//         if is_keyboard_event && !handled {
//             info!("keyboard event not handled");
//         }


//             info!("input event not handled");
//             match event {
//                 WindowEvent::Resized(physical_size) => {
//                     state.resize(physical_size);
//                     state.window.request_redraw();
//                 }
//                 WindowEvent::KeyboardInput { event: KeyEvent { state: ElementState::Pressed, physical_key, .. }, .. } => {
//                     match physical_key {
//                         PhysicalKey::Code(KeyCode::KeyW) => {
//                             console::write_to_console(
//                                 &mut state.text_buffer,
//                                 &mut state.font_system,
//                                 &mut state.chat_text,
//                                 "hi",
//                             );
//                             state.window.request_redraw();
//                         }
//                         _ => {}
//                     }
//                 }
//                 WindowEvent::RedrawRequested => {
//                     state.window.request_redraw();
//                     let now = Instant::now();
//                     let dt = now - last_render_time;
//                     last_render_time = now;
//                     state.update(dt);

//                     // Update text rendering viewport
//                     state.viewport.update(
//                         &state.queue,
//                         Resolution {
//                             width: state.surface_config.width,
//                             height: state.surface_config.height,
//                         },
//                     );

//                     // Prepare text rendering
//                     let text_area = TextArea {
//                         buffer: &state.text_buffer,
//                         left: 10.0,
//                         top: 10.0,
//                         scale: 1.0,
//                         bounds: TextBounds {
//                             left: 0,
//                             top: 0,
//                             right: 600,
//                             bottom: 160
//                         },
//                         default_color: Color::rgb(255, 255, 255),
//                         custom_glyphs: &[],
//                     };
    
//                     state.text_renderer.prepare(
//                         &state.device,
//                         &state.queue,
//                         &mut state.font_system,
//                         &mut state.atlas,
//                         &state.viewport,
//                         [text_area],
//                         &mut state.swash_cache,
//                     ).unwrap();
    
//                     // Handle main rendering
//                     if let Err(e) = state.render() {
//                         match e {
//                             wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated => {
//                                 state.resize(state.window.inner_size());
//                             }
//                             wgpu::SurfaceError::OutOfMemory => event_loop.exit(),
//                             wgpu::SurfaceError::Timeout => log::warn!("Surface timeout"),
//                         }
//                     }
    
//                     state.atlas.trim();
//                 }
//                 WindowEvent::CloseRequested => event_loop.exit(),
//                 _ => {}
//             }
//         // Handle window events

//     }
// }