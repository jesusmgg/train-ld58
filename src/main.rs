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
        // Input
        game_state.mouse_pos = game_state
            .camera
            .screen_to_world(f32::Vec2::from(mouse_position()));

        // Game logic update
        update_current_level(&mut game_state);
        update_sim(&mut game_state);
        update_camera(&mut game_state);

        // Render
        set_camera(&game_state.camera);
        // render_grid(&game_state);
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

fn update_current_level(game_state: &mut GameState) {
    let active_idx = match game_state.level_active {
        Some(idx) => idx,
        None => return,
    };

    let mut grid_x = (active_idx % 3) as i32;
    let mut grid_y = (active_idx / 3) as i32;

    // Navigate between levels with WASD
    if is_key_pressed(KeyCode::S) {
        grid_y = (grid_y - 1).max(0);
    }
    if is_key_pressed(KeyCode::W) {
        grid_y = (grid_y + 1).min(2);
    }
    if is_key_pressed(KeyCode::A) {
        grid_x = (grid_x - 1).max(0);
    }
    if is_key_pressed(KeyCode::D) {
        grid_x = (grid_x + 1).min(2);
    }

    let new_idx = (grid_y * 3 + grid_x) as usize;

    if new_idx != active_idx {
        game_state.level_active = Some(new_idx);
        let new_level = &game_state.levels[new_idx];

        // Set camera target to new level center
        game_state.camera_target_pos = f32::vec2(
            new_level.pos_world.x + SCREEN_W / 2.0,
            new_level.pos_world.y + SCREEN_H / 2.0,
        );
    }
}

fn render_background(game_state: &GameState) {
    clear_background(game_state.styles.colors.green_4);

    let mut color = WHITE;
    color.a = 1.0;

    // Get current level's grid position
    if let Some(active_idx) = game_state.level_active {
        let grid_x = active_idx % 3;
        let grid_y = active_idx / 3;

        // Draw 3x3 block centered on current level
        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = grid_x as i32 + dx;
                let ny = grid_y as i32 + dy;

                if nx >= 0 && nx < 3 && ny >= 0 && ny < 3 {
                    let neighbor_idx = (ny * 3 + nx) as usize;
                    let level = &game_state.levels[neighbor_idx];

                    draw_texture(
                        &game_state.texture_background_01,
                        level.pos_world.x,
                        level.pos_world.y,
                        color,
                    );
                }
            }
        }
    }
}

fn render_diagnostics(game_state: &GameState) {
    let font_size = 32.0;
    let color = Color::from_hex(0x151515);
    let x = 16.0;
    let mut y = 32.0;

    let current_level_name = match &game_state.current_level() {
        Some(level) => level.name,
        None => "-",
    };

    let current_level_idx = match game_state.level_active {
        Some(idx) => idx,
        None => return,
    };

    draw_scaled_text(
        format!(
            "Level: {} (index {})",
            &current_level_name, &current_level_idx
        )
        .as_str(),
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

fn update_camera(game_state: &mut GameState) {
    // Lerp camera towards target position with easing
    let diff = game_state.camera_target_pos - game_state.camera.target;
    let distance = diff.length();

    if distance > 0.1 {
        // Apply smoothstep easing (ease-in-out)
        let t = CAMERA_TRANSITION_SPEED;
        let eased_t = t * t * (3.0 - 2.0 * t);

        game_state.camera.target = game_state.camera.target.lerp(game_state.camera_target_pos, eased_t);
    } else {
        // Snap to target when close enough
        game_state.camera.target = game_state.camera_target_pos;
    }
}

fn configure() {
    set_default_filter_mode(FilterMode::Nearest);
}
