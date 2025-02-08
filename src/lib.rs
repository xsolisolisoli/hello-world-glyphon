use std::sync::Arc;
use crate::window_state::WindowState;

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
    let mut state = WindowState::new(&window).await;
    let mut last_render_time = instant::Instant::now();
    
}