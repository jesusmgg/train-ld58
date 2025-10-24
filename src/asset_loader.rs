use std::collections::HashMap;

use macroquad::{
    audio::load_sound,
    experimental::coroutines::start_coroutine,
    prelude::*,
    texture::{load_texture, Texture2D},
    window::next_frame,
};

use crate::{constants::*, styles::Styles, text::draw_scaled_text};

/// Progress tracking for asset loading
pub struct LoadingProgress {
    pub progress: f32, // 0.0 to 1.0
    pub text: String,
}

/// Render loading screen with progress bar
pub fn render_loading_screen(
    progress: &LoadingProgress,
    styles: &Styles,
    font: &macroquad::text::Font,
) {
    set_default_camera();
    clear_background(styles.colors.green_4);

    // Calculate integer zoom factor for pixel perfect rendering (same as camera)
    let zoom = ((screen_width() as i32 / SCREEN_W as i32)
        .min(screen_height() as i32 / SCREEN_H as i32)) as i32;

    let zoomed_w = (SCREEN_W as i32) * zoom;
    let zoomed_h = (SCREEN_H as i32) * zoom;

    // Center on screen
    let x_offset = ((screen_width() as i32 - zoomed_w) / 2) as f32;
    let y_offset = ((screen_height() as i32 - zoomed_h) / 2) as f32;

    // Message box dimensions (in virtual coordinates)
    let font_size = 16.0;
    let box_width = 148.0;
    let box_height = 32.0;
    let box_x = (SCREEN_W - box_width) / 2.0;
    let box_y = (SCREEN_H - box_height) / 2.0;

    let screen_box_x = x_offset + (box_x * zoom as f32);
    let screen_box_y = y_offset + (box_y * zoom as f32);

    // Border
    draw_rectangle(
        screen_box_x - 2.0 * zoom as f32,
        screen_box_y - 2.0 * zoom as f32,
        (box_width + 4.0) * zoom as f32,
        (box_height + 4.0) * zoom as f32,
        styles.colors.brown_3,
    );

    // Background
    draw_rectangle(
        screen_box_x,
        screen_box_y,
        box_width * zoom as f32,
        box_height * zoom as f32,
        styles.colors.yellow_1,
    );

    // Text
    let text_dims = measure_text(&progress.text, Some(font), font_size as u16, 1.0);
    let text_x = box_x + (box_width - text_dims.width) / 2.0;
    let text_y = box_y + 6.0 + text_dims.offset_y; // Padding from top
    let screen_text_x = x_offset + (text_x * zoom as f32);
    let screen_text_y = y_offset + (text_y * zoom as f32);

    draw_scaled_text(
        &progress.text,
        screen_text_x,
        screen_text_y,
        font_size * zoom as f32,
        &styles.colors.brown_3,
        font,
    );

    // Progress bar
    let bar_width = box_width - 8.0;
    let bar_height = 8.0;
    let bar_x = box_x + 4.0;
    let bar_y = box_y + box_height - bar_height - 4.0;

    let screen_bar_x = x_offset + (bar_x * zoom as f32);
    let screen_bar_y = y_offset + (bar_y * zoom as f32);

    // Progress bar background
    draw_rectangle(
        screen_bar_x,
        screen_bar_y,
        bar_width * zoom as f32,
        bar_height * zoom as f32,
        styles.colors.brown_3,
    );

    // Progress bar fill
    let fill_width = bar_width * progress.progress;
    draw_rectangle(
        screen_bar_x,
        screen_bar_y,
        fill_width * zoom as f32,
        bar_height * zoom as f32,
        styles.colors.green_1,
    );
}

/// Load multiple textures in parallel using coroutines
pub async fn load_textures_parallel(
    paths: Vec<String>,
    progress: &mut LoadingProgress,
    styles: &Styles,
    font: &macroquad::text::Font,
) -> HashMap<String, Texture2D> {
    let total = paths.len();
    progress.text = "Loading graphics...".to_string();

    // Spawn coroutines for each texture load
    let mut loaders = Vec::new();
    for path in paths {
        let handle = start_coroutine(async move {
            let asset = load_texture(&path).await.unwrap();
            (path, asset)
        });
        loaders.push(handle);
    }

    // Wait for all coroutines to complete, updating progress
    loop {
        let completed = loaders.iter().filter(|h| h.is_done()).count();
        progress.progress = completed as f32 / total as f32;

        render_loading_screen(progress, styles, font);

        let all_done = loaders.iter().all(|h| h.is_done());
        if all_done {
            break;
        }
        next_frame().await;
    }

    // Collect results into HashMap
    loaders.into_iter().map(|h| h.retrieve().unwrap()).collect()
}

/// Load multiple audio files in parallel using coroutines
pub async fn load_audio_parallel(
    paths: Vec<String>,
    progress: &mut LoadingProgress,
    styles: &Styles,
    font: &macroquad::text::Font,
) -> HashMap<String, macroquad::audio::Sound> {
    let total = paths.len();
    progress.text = "Loading audio...".to_string();

    // Spawn coroutines for each sound load
    let mut loaders = Vec::new();
    for path in paths {
        let handle = start_coroutine(async move {
            let asset = load_sound(&path).await.unwrap();
            (path, asset)
        });
        loaders.push(handle);
    }

    // Wait for all coroutines to complete, updating progress
    loop {
        let completed = loaders.iter().filter(|h| h.is_done()).count();
        progress.progress = completed as f32 / total as f32;

        render_loading_screen(progress, styles, font);

        let all_done = loaders.iter().all(|h| h.is_done());
        if all_done {
            break;
        }
        next_frame().await;
    }

    // Collect results into HashMap
    loaders.into_iter().map(|h| h.retrieve().unwrap()).collect()
}
