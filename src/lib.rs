use std::sync::Arc;
mod window_state;
use window_state::WindowState;
use winit::{
    dpi::LogicalSize,
    event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

//TODO refactor to host event loop.
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::{Window, WindowBuilder}};

pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).expect("Could't initialize logger");
        } else {
            env_logger::init();
        }
    }
    let event_loop = EventLoop::new();
    let title = "WIG 1.0";
    let window = Arc::new(
        WindowBuilder::new()
            .with_inner_size(LogicalSize::new(800.0, 600.0))
            .with_title("glyphon hello world")
            .build(&event_loop)
            .unwrap()
    );
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
    pub async fn run() {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(log::Level::Info).expect("Could't initialize logger");
            } else {
                env_logger::init();
            }
        }
    
        let event_loop = EventLoop::new().unwrap();
        let title = env!("CARGO_PKG_NAME");
        let window = WindowBuilder::new()
            .with_title(title)
            .build(&event_loop)
            .unwrap();
    
        #[cfg(target_arch = "wasm32")]
        {
            // Winit prevents sizing with CSS, so we have to set
            // the size manually when on web.
            use winit::dpi::PhysicalSize;
            let _ = window.request_inner_size(PhysicalSize::new(450, 400));
    
            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wasm-example")?;
                    let canvas = web_sys::Element::from(window.canvas()?);
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }
    
        let mut state = WindowState::new(&window).await; // NEW!
        let mut last_render_time = instant::Instant::now();
        event_loop.run(move |event, control_flow| {
            match event {
                // NEW!
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion{ delta, },
                    .. // We're not using device_id currently
                } => if state.mouse_pressed {
                    state.camera_controller.process_mouse(delta.0, delta.1)
                }
                // UPDATED!
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window().id() && !state.input(event) => {
                    match event {
                        #[cfg(not(target_arch="wasm32"))]
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    ..
                                },
                            ..
                        } => control_flow.exit(),
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        // UPDATED!
                        WindowEvent::RedrawRequested => {
                            window().request_redraw();
                            let now = instant::Instant::now();
                            let dt = now - last_render_time;
                            last_render_time = now;
                            state.update(dt);
                            match state.render() {
                                Ok(_) => {}
                                // Reconfigure the surface if it's lost or outdated
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => state.resize(state.size),
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                                // We're ignoring timeouts
                                Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }).unwrap();
    }
}