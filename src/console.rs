use glyphon::{Attrs, Buffer, Family, FontSystem, Shaping};
use log::info;
use winit::event::WindowEvent;

pub fn process_events(event: WindowEvent, text_buffer: &mut Buffer, font_system: &mut FontSystem, chat_text: &mut String, new_text: &str) {
    
}

pub fn write_to_console(text_buffer: &mut Buffer, font_system: &mut FontSystem, chat_text: &mut String, new_text: &str) {
    info!("Writing to console: {}", new_text);
    *chat_text = format!("{}\n{}", new_text, chat_text);
    text_buffer.set_text(
        font_system,
        chat_text,
        Attrs::new().family(Family::SansSerif),
        Shaping::Advanced,
    );
    text_buffer.shape_until_scroll(font_system, false);
}

pub fn handle_user_input(text_buffer: &mut Buffer, font_system: &mut FontSystem, chat_text: &mut String, new_line: &mut String, new_char: char) {
    info!("Handling user input: {}", new_char);
    new_line.push(new_char);
    text_buffer.set_text(
        font_system,
        &format!("{}\n{}", new_line, chat_text),
        Attrs::new().family(Family::SansSerif),
        Shaping::Advanced,
    );
}

pub fn enter_new_line(text_buffer: &mut Buffer, font_system: &mut FontSystem, chat_text: &mut String, new_line: &mut String) {
    info!("Entering new line: {}", new_line);
    *chat_text = format!("{}\n{}", new_line, chat_text);
    text_buffer.set_text(
        font_system,
        chat_text,
        Attrs::new().family(Family::SansSerif),
        Shaping::Advanced,
    );
    text_buffer.shape_until_scroll(font_system, false);
    new_line.clear();
}