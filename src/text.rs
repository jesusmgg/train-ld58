use macroquad::{
    color::Color,
    text::{camera_font_scale, draw_text_ex, TextParams},
};

pub fn get_text_params(font_size: f32, color: &Color) -> TextParams {
    let (font_size, font_scale, font_aspect) = camera_font_scale(font_size);
    let text_params = TextParams {
        font_size,
        font_scale,
        font_scale_aspect: font_aspect,
        color: *color,
        ..Default::default()
    };

    text_params
}

pub fn draw_scaled_text(text: &str, x: f32, y: f32, font_size: f32, color: &Color) {
    let text_params = get_text_params(font_size, color);
    draw_text_ex(text, x, y, text_params);
}
