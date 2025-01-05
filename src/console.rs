use glyphon::{Attrs, Buffer, Family, FontSystem, Shaping};

pub fn write_to_console(text_buffer: &mut Buffer, font_system: &mut FontSystem, chat_text: &mut String, new_text: &str) {
    *chat_text = format!("{}\n{}", new_text, chat_text);
    text_buffer.set_text(
        font_system,
        chat_text,
        Attrs::new().family(Family::SansSerif),
        Shaping::Advanced,
    );
    text_buffer.shape_until_scroll(font_system, false);
}