use glyphon::{Attrs, Buffer, Cache, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport};
use log::info;
use wgpu::{Device, Queue, TextureFormat};
use winit::{event::ElementState, keyboard::KeyCode};

pub struct Console {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub viewport: Viewport,
    pub atlas: TextAtlas,
    pub text_renderer: TextRenderer,
    pub text_buffer: Buffer,
    pub chat_text: String,
    pub current_line: String,   
    pub input_mode: bool,
}

impl Console {
    pub fn new(device: &Device, queue: &Queue, swapchain_format: TextureFormat, physical_width: u32, physical_height: u32) -> Self {
        let input_mode = false;
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, swapchain_format);
        let text_renderer = TextRenderer::new(
            &mut atlas, device, wgpu::MultisampleState::default(),
            Some(wgpu::DepthStencilState {
                format: crate::texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            })
        );

        let mut text_buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 42.0));

        text_buffer.set_size(
            &mut font_system,
            Some(physical_width as f32),
            Some(physical_height as f32)
        );

        let mut chat_text = "WIG Engine v0.1 \n".to_string();
        let current_line = String::new();
        text_buffer.set_text(&mut font_system, &chat_text, Attrs::new().family(Family::SansSerif), Shaping::Advanced); 
        text_buffer.shape_until_scroll(&mut font_system, false);

        Self {
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
            text_buffer,
            chat_text,
            current_line,
            input_mode
        }
    }
    pub fn process_input(&mut self, key: KeyCode, state: ElementState) {
        info!("Processing input: {:?}", key);
        match key {  
            KeyCode::Enter => {
                if state == ElementState::Pressed {
                    if !self.input_mode {
                        self.console_newline();
                    }
                    else {

                    }

                    //If input mode off && message not null/whitespace, add new line, then invert input mode
                    self.input_mode = !self.input_mode;
                }
            }
            
            _ =>  {
                if self.input_mode {
                    match key {
                        KeyCode::Backspace => {
                            if state == ElementState::Pressed {
                                self.chat_text.pop();
                            }
                        }
                        _ => {}
                    }
                }
            },
        }
    }

    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass) {
        self.text_renderer.render(&self.atlas, &self.viewport, render_pass).unwrap();
    }
    fn console_newline(&mut self) {
        self.current_line.push('\n');
        self.current_line.push('e');
        self.text_buffer.set_text(
            &mut self.font_system,
            &(self.chat_text.clone() + &self.current_line),
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );
        self.text_buffer.shape_until_scroll(&mut self.font_system, false);

    }
}



// use glyphon::{Attrs, Buffer, Family, FontSystem, Shaping};
// use log::info;
// use winit::event::WindowEvent;

// pub fn process_events(event: WindowEvent, text_buffer: &mut Buffer, font_system: &mut FontSystem, chat_text: &mut String, new_text: &str) {
    
// }

// pub fn write_to_console(text_buffer: &mut Buffer, font_system: &mut FontSystem, chat_text: &mut String, new_text: &str) {
//     info!("Writing to console: {}", new_text);
//     *chat_text = format!("{}\n{}", new_text, chat_text);
//     text_buffer.set_text(
//         font_system,
//         chat_text,
//         Attrs::new().family(Family::SansSerif),
//         Shaping::Advanced,
//     );
//     text_buffer.shape_until_scroll(font_system, false);
// }

// pub fn handle_user_input(text_buffer: &mut Buffer, font_system: &mut FontSystem, chat_text: &mut String, new_line: &mut String, new_char: char) {
//     info!("Handling user input: {}", new_char);
//     new_line.push(new_char);
//     text_buffer.set_text(
//         font_system,
//         &format!("{}\n{}", new_line, chat_text),
//         Attrs::new().family(Family::SansSerif),
//         Shaping::Advanced,
//     );
// }

// pub fn enter_new_line(text_buffer: &mut Buffer, font_system: &mut FontSystem, chat_text: &mut String, new_line: &mut String) {
//     info!("Entering new line: {}", new_line);
//     *chat_text = format!("{}\n{}", new_line, chat_text);
//     text_buffer.set_text(
//         font_system,
//         chat_text,
//         Attrs::new().family(Family::SansSerif),
//         Shaping::Advanced,
//     );
//     text_buffer.shape_until_scroll(font_system, false);
//     new_line.clear();
// }

// pub fn create_console() {
    
// }

