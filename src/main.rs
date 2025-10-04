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
        if is_key_down(KeyCode::W) {
            game_state.camera.target.x += 1.0;
        }

        game_state.mouse_pos = game_state
            .camera
            .screen_to_world(f32::Vec2::from(mouse_position()));

        update_next_level(&mut game_state);
        if setup_level(&mut game_state) {
            next_frame().await;
            continue;
        }

        update_sim(&mut game_state);
        update_score(&mut game_state);

        render_background(&game_state);
        render_grid(&mut game_state);
        render_level_name(&game_state);
        render_level_failed(&game_state);
        render_help(&game_state);
        render_score(&game_state);

        update_win_condition(&mut game_state);

        draw_scaled_text(
            format!("{}", &game_state.camera.target.x).as_str(),
            100.0,
            100.0,
            16.0,
            &Color::from_hex(0x151515),
        );

        next_frame().await
    }
}

fn render_background(game_state: &GameState) {
    clear_background(game_state.styles.colors.green_4);

    let mut color = WHITE;
    color.a = 1.0;
    draw_texture(&game_state.texture_background_01, 0.0, 0.0, color);
}

fn render_level_name(game_state: &GameState) {
    let level = match game_state.current_level() {
        None => return,
        Some(level) => level,
    };

    let font_size = 16.0;
    let message_size = 148.0;
    let pos_message_x = SCREEN_W / 2.0 - message_size / 2.0;
    let pos_message_y = 4.0;
    draw_rectangle(
        pos_message_x - 2.0,
        pos_message_y - 2.0,
        message_size + 4.0,
        16.0 + 4.0,
        game_state.styles.colors.orange_2,
    );
    draw_rectangle(
        pos_message_x,
        pos_message_y,
        message_size,
        16.0,
        game_state.styles.colors.yellow_1,
    );
    draw_scaled_text(
        format!("{}", level.name).as_str(),
        pos_message_x,
        pos_message_y + font_size / 1.333,
        font_size,
        &game_state.styles.colors.brown_3,
    );
}

fn render_score(game_state: &GameState) {
    let font_size = 12.0;
    let pos_message_x = 8.0;
    let pos_message_y = SCREEN_H - font_size * 1.666;
    draw_scaled_text(
        format!("Score: {}", game_state.score).as_str(),
        pos_message_x,
        pos_message_y,
        font_size,
        &game_state.styles.colors.gray_3,
    );
}

fn render_help(game_state: &GameState) {
    let font_size = 12.0;
    let pos_message_x = 8.0;
    let pos_message_y = SCREEN_H - font_size * 0.666;
    draw_scaled_text(
        "<R> to retry level",
        pos_message_x,
        pos_message_y,
        font_size,
        &game_state.styles.colors.gray_2,
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

    game_state.planet_current_index = 0;
    game_state.sim_step = 0;
    game_state.sim_step_computed = 0;

    play_sound_once(&game_state.sfx_level_start_01);

    true
}

fn update_next_level(game_state: &mut GameState) {
    let level_count = game_state.levels.len();
    let mut level_index: isize = match game_state.level_active {
        Some(i) => i as isize,
        None => -1,
    };

    // Restart level
    if is_key_pressed(KeyCode::R) {
        match game_state.current_level_mut() {
            None => {}
            Some(level) => level.reset(),
        }
    }
    // Change level
    else if is_key_pressed(KeyCode::F1) {
        match game_state.current_level_mut() {
            None => {}
            Some(level) => level.reset(),
        }
        level_index -= 1;
        level_index = clamp(level_index, 0, level_count as isize - 1);
        game_state.level_active = Some(level_index as usize);
        match game_state.current_level_mut() {
            None => {}
            Some(level) => level.reset(),
        }
    } else if is_key_pressed(KeyCode::F2) {
        match game_state.current_level_mut() {
            None => {}
            Some(level) => level.reset(),
        }
        level_index += 1;
        level_index = clamp(level_index, 0, level_count as isize - 1);
        game_state.level_active = Some(level_index as usize);
        match game_state.current_level_mut() {
            None => {}
            Some(level) => level.reset(),
        }
    } else if is_key_pressed(KeyCode::F3) {
        // TODO(Jesus): Remove before release.
        match game_state.current_level_mut() {
            None => {}
            Some(level) => level.reset(),
        }
        game_state.level_active = Some(0);
        match game_state.current_level_mut() {
            None => {}
            Some(level) => level.reset(),
        }
    } else if is_key_pressed(KeyCode::F4) {
        // TODO(Jesus): Remove before release.
        match game_state.current_level_mut() {
            None => {}
            Some(level) => level.reset(),
        }
        game_state.level_active = Some(level_count - 1);
        match game_state.current_level_mut() {
            None => {}
            Some(level) => level.reset(),
        }
    }

    let level = match game_state.current_level_mut() {
        None => return,
        Some(level) => level,
    };

    if level.is_stable {
        if is_mouse_button_pressed(MouseButton::Left) || is_mouse_button_down(MouseButton::Right) {
            let current_level_i = match game_state.level_active {
                Some(i) => i,
                None => return,
            };

            if current_level_i + 1 >= game_state.levels.len() {
            } else {
                // Load next level
                game_state.level_active = Some(current_level_i + 1);
            }
        }
    }
}

fn update_sim(game_state: &mut GameState) {
    // Simulation advances 1 step when a planet is placed or removed
    if game_state.sim_step_computed >= game_state.sim_step {
        return;
    }

    let level = match game_state.current_level_mut() {
        None => return,
        Some(level) => level,
    };

    game_state.sim_step_computed += 1;
}

fn update_win_condition(game_state: &mut GameState) {
    let colors = game_state.styles.colors.clone();

    let level_count = game_state.levels.len();
    let is_last_level = match game_state.level_active {
        None => false,
        Some(i) => i >= level_count - 1,
    };

    let level = match game_state.current_level_mut() {
        None => return,
        Some(level) => level,
    };

    if level.is_stable {
        let font_size = 16.0;
        let message_size = 132.0;
        let pos_message_x = SCREEN_W / 2.0 - message_size / 2.0;
        let mut pos_message_y = (SCREEN_H * 0.333) - font_size;
        draw_rectangle(
            pos_message_x - 2.0,
            pos_message_y - 2.0,
            message_size + 4.0,
            32.0 + 4.0,
            colors.orange_2,
        );
        draw_rectangle(
            pos_message_x,
            pos_message_y,
            message_size,
            32.0,
            colors.yellow_2,
        );
        draw_scaled_text(
            "Stable system!",
            pos_message_x,
            pos_message_y + font_size / 1.333,
            font_size,
            &colors.brown_3,
        );

        pos_message_y += font_size;
        let message = if is_last_level {
            "Thanks for playing!"
        } else {
            "Click to continue"
        };
        draw_scaled_text(
            message,
            pos_message_x,
            pos_message_y + font_size / 1.333,
            font_size,
            &colors.brown_3,
        );
    }

    let mut play_sound_stable = false;
    if level.is_stable && !level.was_stable {
        level.was_stable = true;
        play_sound_stable = true;
    }

    let mut play_sound_failed = false;
    if level.is_failed && !level.was_failed {
        level.was_failed = true;
        play_sound_failed = true;
    }

    if play_sound_stable {
        stop_sound(&game_state.music_level_end_01);
        play_sound(
            &game_state.music_level_end_01,
            PlaySoundParams {
                looped: false,
                volume: 0.8,
            },
        );
    }

    if play_sound_failed {
        play_sound_once(&game_state.sfx_explosion_01);
    }
}

fn update_score(game_state: &mut GameState) {
    // Total score
    let mut score = 0;
    for level in &game_state.levels {
        if level.is_stable {
            score += 100 + level.score;
        }
    }

    game_state.score = score;
}

fn render_level_failed(game_state: &GameState) {
    let level = match game_state.current_level() {
        None => return,
        Some(level) => level,
    };

    if level.is_failed {
        let font_size = 16.0;
        let message_size = 162.0;
        let pos_message_x = SCREEN_W / 2.0 - message_size / 2.0;
        let pos_message_y = (SCREEN_H * 0.333) - font_size;
        draw_rectangle(
            pos_message_x - 2.0,
            pos_message_y - 2.0,
            message_size + 4.0,
            16.0 + 4.0,
            game_state.styles.colors.red,
        );
        draw_rectangle(
            pos_message_x,
            pos_message_y,
            message_size,
            16.0,
            game_state.styles.colors.orange_3,
        );
        draw_scaled_text(
            "Collision! <R> to retry",
            pos_message_x,
            pos_message_y + font_size / 1.333,
            font_size,
            &game_state.styles.colors.brown_3,
        );
    }
}

fn render_grid(game_state: &mut GameState) {
    let styles = &game_state.styles;
    let mouse_pos = &game_state.mouse_pos;

    let cell_w = TILE_SIZE_X;
    let cell_h = TILE_SIZE_Y;

    let grid_size_px: f32::Vec2;
    let grid_offset: f32::Vec2;
    let grid_tiles: IVec2;

    let is_stable: bool;
    let is_failed: bool;

    match game_state.current_level() {
        Some(level) => {
            grid_size_px = level.grid_size_px();
            grid_offset = level.grid_offset();
            grid_tiles = level.grid_tiles;
            is_stable = level.is_stable;
            is_failed = level.is_failed;
        }
        None => {
            grid_size_px = f32::Vec2::ZERO;
            grid_offset = f32::Vec2::ZERO;
            grid_tiles = IVec2::ZERO;
            is_stable = false;
            is_failed = false;
        }
    }

    let color_lines = styles.colors.gray_3;
    let color_dark = styles.colors.bg_light;
    let color_light = styles.colors.bg_cream;

    // Draw alternating colored cells for a chess board effect
    game_state.is_mouse_in_grid = false;
    for j in 0..grid_tiles.y {
        for i in 0..grid_tiles.x {
            let x = i as f32 * cell_w + grid_offset.x;
            let y = j as f32 * cell_h + grid_offset.y;

            let is_dark = (i + j) % 2 == 0;

            let mut color = if is_dark { color_dark } else { color_light };
            color.a = 0.7;

            draw_rectangle(x, y, cell_w, cell_h, color);

            if mouse_pos.x >= x
                && mouse_pos.x < x + cell_w
                && mouse_pos.y >= y
                && mouse_pos.y < y + cell_h
            {
                game_state.tile_highlighted.x = i;
                game_state.tile_highlighted.y = j;
                game_state.is_mouse_in_grid = true;

                if game_state.tile_highlighted_prev != game_state.tile_highlighted {
                    game_state.tile_highlighted_prev = game_state.tile_highlighted;
                    if !is_stable && !is_failed {
                        play_sound(
                            &game_state.sfx_hover_01,
                            PlaySoundParams {
                                looped: false,
                                volume: 0.1,
                            },
                        );
                    }
                }

                color = styles.colors.gray_1;
                color.a = 0.5;
                draw_rectangle(x, y, cell_w, cell_h, color);
            }
        }
    }

    // Draw vertical grid lines
    for i in 0..=grid_tiles.x {
        let x = i as f32 * cell_w;
        draw_line(
            x + grid_offset.x,
            grid_offset.y,
            x + grid_offset.x,
            grid_size_px.y + grid_offset.y,
            GRID_THICKNESS,
            color_lines,
        )
    }

    // Draw horizontal grid lines
    for j in 0..=grid_tiles.y {
        let y = j as f32 * cell_h;
        draw_line(
            grid_offset.x,
            y + grid_offset.y,
            grid_size_px.x + grid_offset.x,
            y + grid_offset.y,
            GRID_THICKNESS,
            color_lines,
        );
    }
}

fn configure() {
    set_default_filter_mode(FilterMode::Nearest);
}
