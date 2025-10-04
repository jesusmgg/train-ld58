mod constants;
mod game_state;
mod styles;
mod text;

use constants::*;
use game_state::GameState;
use macroquad::{
    audio::{play_sound, play_sound_once, stop_sound, PlaySoundParams},
    prelude::*,
};
use text::draw_scaled_text;

#[macroquad::main("ld-58")]
async fn main() {
    configure();

    let mut game_state = GameState::new().await;

    loop {
        // Game logic update
        if is_key_down(KeyCode::W) {
            game_state.camera.target.y = (game_state.camera.target.y + 1.0).round();
        }
        if is_key_down(KeyCode::A) {
            game_state.camera.target.x = (game_state.camera.target.x - 1.0).round();
        }
        if is_key_down(KeyCode::S) {
            game_state.camera.target.y = (game_state.camera.target.y - 1.0).round();
        }
        if is_key_down(KeyCode::D) {
            game_state.camera.target.x = (game_state.camera.target.x + 1.0).round();
        }

        game_state.mouse_pos = game_state
            .camera
            .screen_to_world(f32::Vec2::from(mouse_position()));
        update_sim(&mut game_state);

        // Render
        set_camera(&game_state.camera);
        render_background(&game_state);

        // UI
        set_default_camera();
        #[cfg(debug_assertions)]
        render_diagnostics(&game_state);

        // Late game logic update
        update_win_condition(&mut game_state);

        next_frame().await
    }
}

fn render_background(game_state: &GameState) {
    clear_background(game_state.styles.colors.green_4);

    let mut color = WHITE;
    color.a = 1.0;
    draw_texture(&game_state.texture_background_01, 0.0, 0.0, color);
}

fn render_diagnostics(game_state: &GameState) {
    let font_size = 32.0;
    let color = Color::from_hex(0x151515);
    let x = 16.0;
    let mut y = 32.0;

    draw_scaled_text(
        format!("Level: {}", "none").as_str(),
        x,
        y,
        font_size,
        &color,
    );
    y += 24.0;
    draw_scaled_text(
        format!(
            "Camera tgt: {}, {}",
            &game_state.camera.target.x, &game_state.camera.target.y
        )
        .as_str(),
        x,
        y,
        font_size,
        &color,
    );
}

/// Returns `true` if level was setup this frame
fn setup_level(game_state: &mut GameState) -> bool {
    let level = match game_state.current_level_mut() {
        None => return false,
        Some(level) => level,
    };

    if level.is_setup {
        return false;
    }

    level.is_setup = true;

    true
}

fn update_sim(game_state: &mut GameState) {}

fn update_win_condition(game_state: &mut GameState) {}

fn configure() {
    set_default_filter_mode(FilterMode::Nearest);
}
