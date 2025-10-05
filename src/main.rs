mod constants;
mod game_state;
mod styles;
mod text;

use constants::*;
use game_state::{GameState, TileType, TrainDirection, TrainState};
use macroquad::{math::Rect, prelude::*};
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
        update_train_input(&mut game_state);

        // Game logic update
        update_current_level(&mut game_state);
        update_tile_highlight(&mut game_state);
        update_ui_card_selection(&mut game_state);
        update_tile_placement(&mut game_state);
        update_tile_removal(&mut game_state);
        update_train_movement(&mut game_state);
        check_garbage_pickup(&mut game_state);
        check_garbage_dropoff(&mut game_state);
        update_train_animation(&mut game_state);
        update_sim(&mut game_state);
        update_camera(&mut game_state);

        // Render
        set_camera(&game_state.camera);
        render_background(&game_state);
        render_grid(&game_state);
        render_placed_tiles(&game_state);
        render_garbage_indicators(&game_state);
        render_tunnel_layer_2(&game_state);
        render_tunnel_layer_3(&game_state);
        render_tile_highlight(&game_state);
        render_selected_tile_preview(&game_state);
        render_train(&game_state);
        render_tunnel_frames(&game_state);

        // UI
        set_default_camera();
        render_ui_overlay(&game_state);
        render_garbage_counters(&game_state);
        #[cfg(debug_assertions)]
        render_diagnostics(&game_state);

        // Late game logic update
        update_win_condition(&mut game_state);

        next_frame().await
    }
}

fn update_train_input(game_state: &mut GameState) {
    // Space bar to start/stop train
    if is_key_pressed(KeyCode::Space) {
        game_state.train_state = match game_state.train_state {
            TrainState::Stopped => TrainState::Running,
            TrainState::Running => TrainState::Stopped,
            TrainState::Obstacle => TrainState::Stopped,
            TrainState::BrokenRoute => TrainState::Running,
            TrainState::Exiting => TrainState::Stopped,
        };
    }

    // R to reset train to starting position
    if is_key_pressed(KeyCode::R) {
        if let Some(level) = game_state.current_level() {
            // Copy values before modifying state
            let w = level.grid_tiles.x;
            let h = level.grid_tiles.y;
            let start = level.default_train_start;

            game_state.train_tile_pos = start;
            game_state.train_pos_offset = f32::Vec2::ZERO;
            game_state.train_direction = if start.x == -1 {
                TrainDirection::Right
            } else if start.x == w {
                TrainDirection::Left
            } else if start.y == -1 {
                TrainDirection::Down
            } else if start.y == h {
                TrainDirection::Up
            } else {
                TrainDirection::Right
            };
            game_state.train_state = TrainState::Stopped;

            // Reset level
            game_state.reset_level();
        }
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

        // Update train position to new level's default start
        game_state.train_tile_pos = new_level.default_train_start;

        // Update train direction based on tunnel position
        let w = new_level.grid_tiles.x;
        let h = new_level.grid_tiles.y;
        let start = new_level.default_train_start;

        game_state.train_direction = if start.x == -1 {
            TrainDirection::Right // Left tunnel, entering right
        } else if start.x == w {
            TrainDirection::Left // Right tunnel, entering left
        } else if start.y == -1 {
            TrainDirection::Down // Top tunnel, entering down
        } else if start.y == h {
            TrainDirection::Up // Bottom tunnel, entering up
        } else {
            TrainDirection::Right // Default
        };

        game_state.train_pos_offset = f32::Vec2::ZERO;
        game_state.train_state = TrainState::Stopped;
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

/// Renders grid for current and surrounding levels
fn render_grid(game_state: &GameState) {
    // Subtle checkboard colors with low alpha
    let mut color1 = game_state.styles.colors.green_1;
    color1.a = 0.1;
    let mut color2 = game_state.styles.colors.green_2;
    color2.a = 0.1;

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

                    // Calculate grid position (centered in level)
                    let grid_offset = level.grid_offset();
                    let grid_origin = level.pos_world + grid_offset;

                    // Draw checkboard pattern
                    for ty in 0..level.grid_tiles.y {
                        for tx in 0..level.grid_tiles.x {
                            let x = grid_origin.x + (tx as f32 * TILE_SIZE_X);
                            let y = grid_origin.y + (ty as f32 * TILE_SIZE_Y);

                            // Alternate colors for checkboard
                            let color = if (tx + ty) % 2 == 0 { color1 } else { color2 };

                            draw_rectangle(x, y, TILE_SIZE_X, TILE_SIZE_Y, color);
                        }
                    }
                }
            }
        }
    }
}

fn update_tile_highlight(game_state: &mut GameState) {
    game_state.tile_highlighted_prev = game_state.tile_highlighted;

    let mouse_pos = &game_state.mouse_pos;

    // Check only current level
    if let Some(level) = game_state.current_level() {
        // Calculate grid position (centered in level)
        let grid_offset = level.grid_offset();
        let grid_origin = level.pos_world + grid_offset;
        let grid_size = level.grid_size_px();

        // Check if mouse is within this level's grid
        if mouse_pos.x >= grid_origin.x
            && mouse_pos.x < grid_origin.x + grid_size.x
            && mouse_pos.y >= grid_origin.y
            && mouse_pos.y < grid_origin.y + grid_size.y
        {
            // Calculate tile coordinates
            let tile_x = ((mouse_pos.x - grid_origin.x) / TILE_SIZE_X) as i32;
            let tile_y = ((mouse_pos.y - grid_origin.y) / TILE_SIZE_Y) as i32;

            game_state.tile_highlighted = Some(IVec2::new(tile_x, tile_y));
        } else {
            game_state.tile_highlighted = None;
        }
    } else {
        game_state.tile_highlighted = None;
    }
}

fn render_tile_highlight(game_state: &GameState) {
    if let Some(tile) = game_state.tile_highlighted {
        if let Some(level) = game_state.current_level() {
            // Highlight color
            let mut highlight_color = game_state.styles.colors.yellow_1;
            highlight_color.a = 0.4;

            let grid_offset = level.grid_offset();
            let grid_origin = level.pos_world + grid_offset;

            let x = grid_origin.x + (tile.x as f32 * TILE_SIZE_X);
            let y = grid_origin.y + (tile.y as f32 * TILE_SIZE_Y);

            draw_rectangle(x, y, TILE_SIZE_X, TILE_SIZE_Y, highlight_color);
        }
    }
}

fn render_ui_overlay(game_state: &GameState) {
    // Calculate integer zoom factor for pixel perfect rendering (same as camera)
    let zoom = ((screen_width() as i32 / SCREEN_W as i32)
        .min(screen_height() as i32 / SCREEN_H as i32)) as i32;

    let zoomed_w = (SCREEN_W as i32) * zoom;
    let zoomed_h = (SCREEN_H as i32) * zoom;

    // Center on screen
    let x_offset = ((screen_width() as i32 - zoomed_w) / 2) as f32;
    let y_offset = ((screen_height() as i32 - zoomed_h) / 2) as f32;

    // Draw overlay
    draw_texture_ex(
        &game_state.texture_ui_overlay,
        x_offset,
        y_offset,
        WHITE,
        DrawTextureParams {
            dest_size: Some(Vec2::new(zoomed_w as f32, zoomed_h as f32)),
            ..Default::default()
        },
    );

    let card_x = 14.0;

    // Draw track cards on left panel (single column)
    let card_positions = [
        (
            card_x,
            14.0,
            TileType::TrackHorizontal,
            &game_state.texture_ui_card_track_h,
            game_state.count_track_h,
        ),
        (
            card_x,
            54.0,
            TileType::TrackVertical,
            &game_state.texture_ui_card_track_v,
            game_state.count_track_v,
        ),
        (
            card_x,
            94.0,
            TileType::TrackCornerUL,
            &game_state.texture_ui_card_track_ul,
            game_state.count_track_ul,
        ),
        (
            card_x,
            134.0,
            TileType::TrackCornerUR,
            &game_state.texture_ui_card_track_ur,
            game_state.count_track_ur,
        ),
        (
            card_x,
            174.0,
            TileType::TrackCornerDL,
            &game_state.texture_ui_card_track_dl,
            game_state.count_track_dl,
        ),
        (
            card_x,
            214.0,
            TileType::TrackCornerDR,
            &game_state.texture_ui_card_track_dr,
            game_state.count_track_dr,
        ),
    ];

    for (card_x, card_y, tile_type, texture, count) in &card_positions {
        let screen_x = x_offset + (card_x * zoom as f32);
        let screen_y = y_offset + (card_y * zoom as f32);

        draw_texture_ex(
            texture,
            screen_x,
            screen_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(36.0 * zoom as f32, 36.0 * zoom as f32)),
                ..Default::default()
            },
        );

        // Draw selection indicator on selected card
        if let Some(selected) = game_state.selected_tile {
            if selected == *tile_type {
                draw_texture_ex(
                    &game_state.texture_ui_card_selection,
                    screen_x - 6.0,
                    screen_y - 6.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(40.0 * zoom as f32, 40.0 * zoom as f32)),
                        ..Default::default()
                    },
                );
            }
        }

        // Draw count overlay on bottom-left corner of the card
        let count_x = screen_x + (2.0 * zoom as f32);
        let count_y = screen_y + (32.0 * zoom as f32);
        draw_scaled_text(
            &count.to_string(),
            count_x,
            count_y,
            16.0 * zoom as f32,
            &WHITE,
            &game_state.font,
        );
    }
}

fn render_garbage_counters(game_state: &GameState) {
    // Calculate integer zoom factor for pixel perfect rendering (same as camera)
    let zoom = ((screen_width() as i32 / SCREEN_W as i32)
        .min(screen_height() as i32 / SCREEN_H as i32)) as i32;

    let zoomed_w = (SCREEN_W as i32) * zoom;
    let zoomed_h = (SCREEN_H as i32) * zoom;

    // Center on screen
    let x_offset = ((screen_width() as i32 - zoomed_w) / 2) as f32;
    let y_offset = ((screen_height() as i32 - zoomed_h) / 2) as f32;

    // Position on right side of screen
    let text = format!(
        "{}/{}",
        game_state.dropoffs_full_count, game_state.total_dropoffs_count
    );
    let text_x = SCREEN_W - 40.0;
    let text_y = 98.0;
    let font_size = 18.0;

    let screen_x = x_offset + (text_x * zoom as f32);
    let screen_y = y_offset + (text_y * zoom as f32);

    draw_scaled_text(
        &text,
        screen_x,
        screen_y,
        font_size * zoom as f32,
        &WHITE,
        &game_state.font,
    );

    // Garbage held count below
    let garbage_x = SCREEN_W - 36.0;
    let garbage_text = format!("{}", game_state.garbage_held);
    let garbage_y = 170.0;
    let garbage_screen_x = x_offset + (garbage_x * zoom as f32);
    let garbage_screen_y = y_offset + (garbage_y * zoom as f32);

    draw_scaled_text(
        &garbage_text,
        garbage_screen_x,
        garbage_screen_y,
        font_size * zoom as f32,
        &WHITE,
        &game_state.font,
    );
}

fn render_diagnostics(game_state: &GameState) {
    let font_size = 32.0;
    let color = WHITE;
    let x = 680.0;
    let mut y = 32.0;

    draw_scaled_text(
        format!("FPS: {}", get_fps()).as_str(),
        x,
        y,
        font_size,
        &color,
        &game_state.font,
    );
    y += 24.0;

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
        &game_state.font,
    );
    y += 24.0;
    draw_scaled_text(
        format!("Train state: {:?}", &game_state.train_state).as_str(),
        x,
        y,
        font_size,
        &color,
        &game_state.font,
    );
    y += 24.0;
    draw_scaled_text(
        format!("Game won: {}", &game_state.game_won).as_str(),
        x,
        y,
        font_size,
        &color,
        &game_state.font,
    );
}

fn update_train_movement(game_state: &mut GameState) {
    if game_state.train_state != TrainState::Running {
        return;
    }

    // Calculate movement delta based on direction and speed
    let delta = get_frame_time() * TRAIN_SPEED;

    let movement = match game_state.train_direction {
        TrainDirection::Up => f32::Vec2::new(0.0, -delta),
        TrainDirection::Down => f32::Vec2::new(0.0, delta),
        TrainDirection::Left => f32::Vec2::new(-delta, 0.0),
        TrainDirection::Right => f32::Vec2::new(delta, 0.0),
    };

    // Check if we're about to cross into next tile
    let new_offset = game_state.train_pos_offset + movement;
    let will_cross = match game_state.train_direction {
        TrainDirection::Up => new_offset.y <= -1.0,
        TrainDirection::Down => new_offset.y >= 1.0,
        TrainDirection::Left => new_offset.x <= -1.0,
        TrainDirection::Right => new_offset.x >= 1.0,
    };

    // If we're about to cross, validate the next tile FIRST
    if will_cross {
        let level = match game_state.current_level() {
            Some(l) => l,
            None => return,
        };

        let next_pos = match game_state.train_direction {
            TrainDirection::Up => game_state.train_tile_pos + IVec2::new(0, -1),
            TrainDirection::Down => game_state.train_tile_pos + IVec2::new(0, 1),
            TrainDirection::Left => game_state.train_tile_pos + IVec2::new(-1, 0),
            TrainDirection::Right => game_state.train_tile_pos + IVec2::new(1, 0),
        };

        // Check if next position is a tunnel (level connection)
        let w = level.grid_tiles.x;
        let h = level.grid_tiles.y;
        let is_tunnel = next_pos.x < 0 || next_pos.x >= w || next_pos.y < 0 || next_pos.y >= h;

        if is_tunnel {
            // Check if there's actually a tunnel at this position
            if let Some(tile) = level.tile_layout.get(&next_pos) {
                if matches!(
                    tile,
                    TileType::TunnelUpOpen
                        | TileType::TunnelDownOpen
                        | TileType::TunnelLeftOpen
                        | TileType::TunnelRightOpen
                ) {
                    // Check if train is exiting (direction matches tunnel direction)
                    let is_exiting = matches!(
                        (game_state.train_direction, tile),
                        (TrainDirection::Up, TileType::TunnelUpOpen)
                            | (TrainDirection::Down, TileType::TunnelDownOpen)
                            | (TrainDirection::Left, TileType::TunnelLeftOpen)
                            | (TrainDirection::Right, TileType::TunnelRightOpen)
                    );

                    if is_exiting {
                        // Calculate which level to transition to
                        let current_idx = game_state.level_active.unwrap();
                        let grid_x = current_idx % 3;
                        let grid_y = current_idx / 3;

                        let next_level_idx = match game_state.train_direction {
                            TrainDirection::Right if grid_x < 2 => Some(current_idx + 1),
                            TrainDirection::Left if grid_x > 0 => Some(current_idx - 1),
                            TrainDirection::Down if grid_y < 2 => Some(current_idx + 3),
                            TrainDirection::Up if grid_y > 0 => Some(current_idx - 3),
                            _ => None,
                        };

                        if let Some(next_idx) = next_level_idx {
                            // Transition to next level
                            game_state.level_active = Some(next_idx);
                            let next_level = &game_state.levels[next_idx];

                            // Set camera target to new level
                            game_state.camera_target_pos = f32::vec2(
                                next_level.pos_world.x + SCREEN_W / 2.0,
                                next_level.pos_world.y + SCREEN_H / 2.0,
                            );

                            // Calculate arrival tunnel position based on exit position
                            let new_w = next_level.grid_tiles.x;
                            let new_h = next_level.grid_tiles.y;
                            let current_pos = game_state.train_tile_pos;

                            let arrival_pos = match game_state.train_direction {
                                // Exiting right -> arriving at left
                                TrainDirection::Right => IVec2::new(-1, current_pos.y),
                                // Exiting left -> arriving at right
                                TrainDirection::Left => IVec2::new(new_w, current_pos.y),
                                // Exiting down -> arriving at top
                                TrainDirection::Down => IVec2::new(current_pos.x, -1),
                                // Exiting up -> arriving at bottom
                                TrainDirection::Up => IVec2::new(current_pos.x, new_h),
                            };

                            // Position train at arrival tunnel with offset zero
                            game_state.train_tile_pos = arrival_pos;
                            game_state.train_pos_offset = f32::Vec2::ZERO;

                            // Keep direction (train continues in same direction)
                            // Train state remains Running
                            return;
                        }
                    } else {
                        // Train is entering - allow crossing and stop
                        match game_state.train_direction {
                            TrainDirection::Up => game_state.train_pos_offset.y += 1.0,
                            TrainDirection::Down => game_state.train_pos_offset.y -= 1.0,
                            TrainDirection::Left => game_state.train_pos_offset.x += 1.0,
                            TrainDirection::Right => game_state.train_pos_offset.x -= 1.0,
                        }
                        game_state.train_tile_pos = next_pos;
                        game_state.train_state = TrainState::Stopped;
                        return;
                    }
                }
            }
            // No tunnel or closed tunnel - broken route, clamp position and stop
            match game_state.train_direction {
                TrainDirection::Up => game_state.train_pos_offset.y = -0.9,
                TrainDirection::Down => game_state.train_pos_offset.y = 0.9,
                TrainDirection::Left => game_state.train_pos_offset.x = -0.9,
                TrainDirection::Right => game_state.train_pos_offset.x = 0.9,
            }
            game_state.train_state = TrainState::BrokenRoute;
            return;
        }

        // Check if next position has a valid track
        if let Some(tile) = level.tile_layout.get(&next_pos) {
            // Check if it's a track tile
            let is_track = matches!(
                tile,
                TileType::TrackHorizontal
                    | TileType::TrackVertical
                    | TileType::TrackCornerUL
                    | TileType::TrackCornerUR
                    | TileType::TrackCornerDL
                    | TileType::TrackCornerDR
            );

            if !is_track {
                // Hit an obstacle - clamp position and stop
                match game_state.train_direction {
                    TrainDirection::Up => game_state.train_pos_offset.y = -0.9,
                    TrainDirection::Down => game_state.train_pos_offset.y = 0.9,
                    TrainDirection::Left => game_state.train_pos_offset.x = -0.9,
                    TrainDirection::Right => game_state.train_pos_offset.x = 0.9,
                }
                game_state.train_state = TrainState::Obstacle;
                return;
            }

            // Validate track connection and update direction
            let valid_and_new_direction = match (game_state.train_direction, tile) {
                // Horizontal track
                (TrainDirection::Left, TileType::TrackHorizontal) => Some(TrainDirection::Left),
                (TrainDirection::Right, TileType::TrackHorizontal) => Some(TrainDirection::Right),

                // Vertical track
                (TrainDirection::Up, TileType::TrackVertical) => Some(TrainDirection::Up),
                (TrainDirection::Down, TileType::TrackVertical) => Some(TrainDirection::Down),

                // Corner UL (upper-left position, connects down and right)
                (TrainDirection::Down, TileType::TrackCornerUL) => Some(TrainDirection::Right),
                (TrainDirection::Left, TileType::TrackCornerUL) => Some(TrainDirection::Up),

                // Corner UR (upper-right position, connects down and left)
                (TrainDirection::Down, TileType::TrackCornerUR) => Some(TrainDirection::Left),
                (TrainDirection::Right, TileType::TrackCornerUR) => Some(TrainDirection::Up),

                // Corner DL (lower-left position, connects up and right)
                (TrainDirection::Up, TileType::TrackCornerDL) => Some(TrainDirection::Right),
                (TrainDirection::Left, TileType::TrackCornerDL) => Some(TrainDirection::Down),

                // Corner DR (lower-right position, connects up and left)
                (TrainDirection::Up, TileType::TrackCornerDR) => Some(TrainDirection::Left),
                (TrainDirection::Right, TileType::TrackCornerDR) => Some(TrainDirection::Down),

                _ => None,
            };

            if let Some(new_direction) = valid_and_new_direction {
                // Valid track - but check if there's a valid continuation after this tile
                let next_next_pos = match new_direction {
                    TrainDirection::Up => next_pos + IVec2::new(0, -1),
                    TrainDirection::Down => next_pos + IVec2::new(0, 1),
                    TrainDirection::Left => next_pos + IVec2::new(-1, 0),
                    TrainDirection::Right => next_pos + IVec2::new(1, 0),
                };

                // Check if the tile after next is a tunnel or valid track
                let is_next_tunnel = next_next_pos.x < 0
                    || next_next_pos.x >= w
                    || next_next_pos.y < 0
                    || next_next_pos.y >= h;
                let has_valid_continuation = if is_next_tunnel {
                    // Check if there's an open tunnel
                    if let Some(tile) = level.tile_layout.get(&next_next_pos) {
                        matches!(
                            tile,
                            TileType::TunnelUpOpen
                                | TileType::TunnelDownOpen
                                | TileType::TunnelLeftOpen
                                | TileType::TunnelRightOpen
                        )
                    } else {
                        false
                    }
                } else {
                    // Check if there's a valid track tile
                    if let Some(tile) = level.tile_layout.get(&next_next_pos) {
                        matches!(
                            tile,
                            TileType::TrackHorizontal
                                | TileType::TrackVertical
                                | TileType::TrackCornerUL
                                | TileType::TrackCornerUR
                                | TileType::TrackCornerDL
                                | TileType::TrackCornerDR
                        )
                    } else {
                        false
                    }
                };

                if has_valid_continuation {
                    // Valid continuation exists - allow crossing
                    game_state.train_pos_offset = match game_state.train_direction {
                        TrainDirection::Up => {
                            game_state.train_pos_offset.y += 1.0;
                            game_state.train_pos_offset
                        }
                        TrainDirection::Down => {
                            game_state.train_pos_offset.y -= 1.0;
                            game_state.train_pos_offset
                        }
                        TrainDirection::Left => {
                            game_state.train_pos_offset.x += 1.0;
                            game_state.train_pos_offset
                        }
                        TrainDirection::Right => {
                            game_state.train_pos_offset.x -= 1.0;
                            game_state.train_pos_offset
                        }
                    };
                    game_state.train_tile_pos = next_pos;
                    game_state.train_direction = new_direction;
                } else {
                    // No valid continuation - don't enter this tile
                    match game_state.train_direction {
                        TrainDirection::Up => game_state.train_pos_offset.y = -0.9,
                        TrainDirection::Down => game_state.train_pos_offset.y = 0.9,
                        TrainDirection::Left => game_state.train_pos_offset.x = -0.9,
                        TrainDirection::Right => game_state.train_pos_offset.x = 0.9,
                    }
                    game_state.train_state = TrainState::BrokenRoute;
                }
            } else {
                // Invalid track connection - clamp position and stop
                match game_state.train_direction {
                    TrainDirection::Up => game_state.train_pos_offset.y = -0.9,
                    TrainDirection::Down => game_state.train_pos_offset.y = 0.9,
                    TrainDirection::Left => game_state.train_pos_offset.x = -0.9,
                    TrainDirection::Right => game_state.train_pos_offset.x = 0.9,
                }
                game_state.train_state = TrainState::BrokenRoute;
            }
        } else {
            // No tile at next position - clamp position and stop
            match game_state.train_direction {
                TrainDirection::Up => game_state.train_pos_offset.y = -0.9,
                TrainDirection::Down => game_state.train_pos_offset.y = 0.9,
                TrainDirection::Left => game_state.train_pos_offset.x = -0.9,
                TrainDirection::Right => game_state.train_pos_offset.x = 0.9,
            }
            game_state.train_state = TrainState::BrokenRoute;
        }
    } else {
        // Not crossing yet, just update offset
        game_state.train_pos_offset = new_offset;
    }
}

fn check_garbage_pickup(game_state: &mut GameState) {
    if game_state.train_state != TrainState::Running {
        return;
    }

    let train_pos = game_state.train_tile_pos;

    // Check all 4 adjacent tiles for garbage pickup
    let adjacent_positions = [
        train_pos + IVec2::new(0, -1), // Up
        train_pos + IVec2::new(0, 1),  // Down
        train_pos + IVec2::new(-1, 0), // Left
        train_pos + IVec2::new(1, 0),  // Right
    ];

    // Check which tiles have garbage to pick up
    let garbage_positions: Vec<IVec2> = if let Some(level) = game_state.current_level() {
        adjacent_positions
            .iter()
            .filter(|pos| {
                if let Some(tile) = level.tile_layout.get(pos) {
                    matches!(tile, TileType::GarbagePickupFull)
                } else {
                    false
                }
            })
            .copied()
            .collect()
    } else {
        Vec::new()
    };

    // Pick up garbage and mark as empty
    for pos in garbage_positions {
        if let Some(level) = game_state.current_level_mut() {
            level.tile_layout.insert(pos, TileType::GarbagePickupEmpty);
            game_state.garbage_held += 1;
        }
    }
}

fn check_garbage_dropoff(game_state: &mut GameState) {
    if game_state.train_state != TrainState::Running {
        return;
    }

    if game_state.garbage_held <= 0 {
        return;
    }

    let train_pos = game_state.train_tile_pos;

    // Check all 4 adjacent tiles for garbage dropoff sites
    let adjacent_positions = [
        train_pos + IVec2::new(0, -1), // Up
        train_pos + IVec2::new(0, 1),  // Down
        train_pos + IVec2::new(-1, 0), // Left
        train_pos + IVec2::new(1, 0),  // Right
    ];

    // Find dropoff sites that aren't full
    let dropoff_positions: Vec<(IVec2, TileType)> = if let Some(level) = game_state.current_level()
    {
        adjacent_positions
            .iter()
            .filter_map(|pos| {
                if let Some(tile) = level.tile_layout.get(pos) {
                    match tile {
                        TileType::GarbageDropoffEmpty
                        | TileType::GarbageDropoffFull1
                        | TileType::GarbageDropoffFull2 => Some((*pos, *tile)),
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    // Drop off garbage at each available site
    for (pos, current_state) in dropoff_positions {
        if game_state.garbage_held <= 0 {
            break;
        }

        // Calculate current fullness and remaining capacity
        let current_fullness = match current_state {
            TileType::GarbageDropoffEmpty => 0,
            TileType::GarbageDropoffFull1 => 1,
            TileType::GarbageDropoffFull2 => 2,
            _ => continue,
        };

        let remaining_capacity = 3 - current_fullness;
        let amount_to_drop = game_state.garbage_held.min(remaining_capacity);

        if amount_to_drop <= 0 {
            continue;
        }

        // Calculate new fullness level
        let new_fullness = current_fullness + amount_to_drop;
        let new_state = match new_fullness {
            1 => TileType::GarbageDropoffFull1,
            2 => TileType::GarbageDropoffFull2,
            3 => TileType::GarbageDropoffFull3,
            _ => continue,
        };

        if let Some(level) = game_state.current_level_mut() {
            level.tile_layout.insert(pos, new_state);
            game_state.garbage_held -= amount_to_drop;
        }
    }

    // Update dropoff counts after any changes
    game_state.update_dropoff_counts();
}

fn update_train_animation(game_state: &mut GameState) {
    if game_state.train_state != TrainState::Running {
        return;
    }

    // Update animation timer
    game_state.train_anim_timer += get_frame_time();

    // Switch frames
    if game_state.train_anim_timer >= TRAIN_ANIM_SPEED {
        game_state.train_anim_timer = 0.0;
        game_state.train_anim_frame = if game_state.train_anim_frame == 0 {
            1
        } else {
            0
        };
    }
}

fn update_sim(game_state: &mut GameState) {}

fn update_win_condition(game_state: &mut GameState) {}

fn update_camera(game_state: &mut GameState) {
    // Recalculate viewport for current window size
    let zoom = ((screen_width() as i32 / SCREEN_W as i32)
        .min(screen_height() as i32 / SCREEN_H as i32)) as i32;

    let zoomed_w = (SCREEN_W as i32) * zoom;
    let zoomed_h = (SCREEN_H as i32) * zoom;

    // Center viewport on screen
    let x_offset = (screen_width() as i32 - zoomed_w) / 2;
    let y_offset = (screen_height() as i32 - zoomed_h) / 2;

    game_state.camera.viewport = Some((x_offset, y_offset, zoomed_w, zoomed_h));

    // Lerp camera towards target position with easing
    let diff = game_state.camera_target_pos - game_state.camera.target;
    let distance = diff.length();

    if distance > 0.1 {
        // Apply smoothstep easing (ease-in-out)
        let t = CAMERA_TRANSITION_SPEED;
        let eased_t = t * t * (3.0 - 2.0 * t);

        game_state.camera.target = game_state
            .camera
            .target
            .lerp(game_state.camera_target_pos, eased_t);
    } else {
        // Snap to target when close enough
        game_state.camera.target = game_state.camera_target_pos;
    }
}

fn update_ui_card_selection(game_state: &mut GameState) {
    if !is_mouse_button_pressed(MouseButton::Left) {
        return;
    }

    // Get screen-space mouse position
    let mouse_screen = mouse_position();

    // Calculate UI overlay position (same as render_ui_overlay)
    let zoom = ((screen_width() as i32 / SCREEN_W as i32)
        .min(screen_height() as i32 / SCREEN_H as i32)) as i32;

    let zoomed_w = (SCREEN_W as i32) * zoom;
    let zoomed_h = (SCREEN_H as i32) * zoom;

    let x_offset = ((screen_width() as i32 - zoomed_w) / 2) as f32;
    let y_offset = ((screen_height() as i32 - zoomed_h) / 2) as f32;

    let card_x = 14.0;

    // Card positions (same as render_ui_overlay)
    let card_positions = [
        (card_x, 14.0, TileType::TrackHorizontal),
        (card_x, 54.0, TileType::TrackVertical),
        (card_x, 94.0, TileType::TrackCornerUL),
        (card_x, 134.0, TileType::TrackCornerUR),
        (card_x, 174.0, TileType::TrackCornerDL),
        (card_x, 214.0, TileType::TrackCornerDR),
    ];

    let card_size = 36.0 * zoom as f32;

    // Check if mouse is over any card
    for (card_x, card_y, tile_type) in &card_positions {
        let screen_x = x_offset + (card_x * zoom as f32);
        let screen_y = y_offset + (card_y * zoom as f32);

        if mouse_screen.0 >= screen_x
            && mouse_screen.0 < screen_x + card_size
            && mouse_screen.1 >= screen_y
            && mouse_screen.1 < screen_y + card_size
        {
            // Check if we have pieces available
            let count = game_state.get_track_count(*tile_type);
            if count <= 0 {
                return;
            }

            // Toggle selection: deselect if already selected, otherwise select
            if game_state.selected_tile == Some(*tile_type) {
                game_state.selected_tile = None;
            } else {
                game_state.selected_tile = Some(*tile_type);
            }
            return;
        }
    }
}

fn update_tile_placement(game_state: &mut GameState) {
    // Only allow placement if tile is selected and highlighted
    if game_state.selected_tile.is_none() || game_state.tile_highlighted.is_none() {
        return;
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        // Copy values before mutable borrow
        let tile_pos = game_state.tile_highlighted.unwrap();
        let tile_type = game_state.selected_tile.unwrap();

        // Check if we have pieces available
        let count = game_state.get_track_count(tile_type);
        if count <= 0 {
            return;
        }

        // Check if placement is allowed and get existing tile info
        let (can_place, existing_tile) = if let Some(level) = game_state.current_level() {
            if let Some(existing) = level.tile_layout.get(&tile_pos) {
                (!game_state.is_tile_permanent(*existing), Some(*existing))
            } else {
                (true, None)
            }
        } else {
            (false, None)
        };

        if can_place {
            // Return old piece to pool if replacing
            if let Some(old_tile) = existing_tile {
                game_state.increment_track_count(old_tile);
            }

            // Place new piece
            if let Some(level) = game_state.current_level_mut() {
                level.tile_layout.insert(tile_pos, tile_type);
            }
            game_state.decrement_track_count(tile_type);

            // Deselect if we just placed the last piece
            if game_state.get_track_count(tile_type) <= 0 {
                game_state.selected_tile = None;
            }
        }
    }
}

fn update_tile_removal(game_state: &mut GameState) {
    // Right-click to remove placed track pieces
    if !is_mouse_button_pressed(MouseButton::Right) {
        return;
    }

    if game_state.tile_highlighted.is_none() {
        return;
    }

    let tile_pos = game_state.tile_highlighted.unwrap();

    // Check if there's a removable tile at this position
    let tile_to_remove = if let Some(level) = game_state.current_level() {
        if let Some(tile) = level.tile_layout.get(&tile_pos) {
            if !game_state.is_tile_permanent(*tile) {
                Some(*tile)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Remove the tile and return it to the pool
    if let Some(tile_type) = tile_to_remove {
        if let Some(level) = game_state.current_level_mut() {
            level.tile_layout.remove(&tile_pos);
        }
        game_state.increment_track_count(tile_type);
        // Select the removed piece type
        game_state.selected_tile = Some(tile_type);
    }
}

fn render_selected_tile_preview(game_state: &GameState) {
    // Show selected tile at cursor with low alpha
    if let Some(tile_type) = game_state.selected_tile {
        if let Some(tile_pos) = game_state.tile_highlighted {
            if let Some(level) = game_state.current_level() {
                let grid_offset = level.grid_offset();
                let grid_origin = level.pos_world + grid_offset;

                let x = grid_origin.x + (tile_pos.x as f32 * TILE_SIZE_X);
                let y = grid_origin.y + (tile_pos.y as f32 * TILE_SIZE_Y);

                let texture = game_state.get_texture_for_tile(tile_type);
                let mut color = WHITE;
                color.a = 0.5;

                draw_texture_ex(
                    texture,
                    x,
                    y,
                    color,
                    DrawTextureParams {
                        flip_y: true,
                        ..Default::default()
                    },
                );
            }
        }
    }
}

fn render_placed_tiles(game_state: &GameState) {
    // Render tiles for current level and neighbors
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

                    let grid_offset = level.grid_offset();
                    let grid_origin = level.pos_world + grid_offset;

                    // Draw all placed tiles in this level (skip tunnels, they're rendered separately)
                    for (tile_pos, tile_type) in &level.tile_layout {
                        // Skip tunnel tiles - they will be rendered in layers
                        if matches!(
                            tile_type,
                            TileType::TunnelUpOpen
                                | TileType::TunnelUpClosed
                                | TileType::TunnelDownOpen
                                | TileType::TunnelDownClosed
                                | TileType::TunnelLeftOpen
                                | TileType::TunnelLeftClosed
                                | TileType::TunnelRightOpen
                                | TileType::TunnelRightClosed
                        ) {
                            continue;
                        }

                        let x = grid_origin.x + (tile_pos.x as f32 * TILE_SIZE_X);
                        let y = grid_origin.y + (tile_pos.y as f32 * TILE_SIZE_Y);

                        let texture = game_state.get_texture_for_tile(*tile_type);
                        draw_texture_ex(
                            texture,
                            x,
                            y,
                            WHITE,
                            DrawTextureParams {
                                flip_y: true,
                                ..Default::default()
                            },
                        );
                    }
                }
            }
        }
    }
}

fn render_garbage_indicators(game_state: &GameState) {
    // Render fullness indicators for garbage dropoff sites
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

                    let grid_offset = level.grid_offset();
                    let grid_origin = level.pos_world + grid_offset;

                    // Draw indicators for dropoff sites
                    for (tile_pos, tile_type) in &level.tile_layout {
                        let indicator_texture = match tile_type {
                            TileType::GarbageDropoffEmpty => {
                                Some(&game_state.texture_garbage_indicator_0)
                            }
                            TileType::GarbageDropoffFull1 => {
                                Some(&game_state.texture_garbage_indicator_1)
                            }
                            TileType::GarbageDropoffFull2 => {
                                Some(&game_state.texture_garbage_indicator_2)
                            }
                            TileType::GarbageDropoffFull3 => {
                                Some(&game_state.texture_garbage_indicator_3)
                            }
                            _ => None,
                        };

                        if let Some(texture) = indicator_texture {
                            let x = grid_origin.x + (tile_pos.x as f32 * TILE_SIZE_X);
                            let y = grid_origin.y + (tile_pos.y as f32 * TILE_SIZE_Y);

                            draw_texture_ex(
                                texture,
                                x,
                                y,
                                WHITE,
                                DrawTextureParams {
                                    flip_y: true,
                                    ..Default::default()
                                },
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Render tunnel layer 2: holes for open tunnels, half-tracks for closed tunnels
fn render_tunnel_layer_2(game_state: &GameState) {
    if let Some(active_idx) = game_state.level_active {
        let grid_x = active_idx % 3;
        let grid_y = active_idx / 3;

        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = grid_x as i32 + dx;
                let ny = grid_y as i32 + dy;

                if nx >= 0 && nx < 3 && ny >= 0 && ny < 3 {
                    let neighbor_idx = (ny * 3 + nx) as usize;
                    let level = &game_state.levels[neighbor_idx];

                    let grid_offset = level.grid_offset();
                    let grid_origin = level.pos_world + grid_offset;

                    for (tile_pos, tile_type) in &level.tile_layout {
                        let x = grid_origin.x + (tile_pos.x as f32 * TILE_SIZE_X);
                        let y = grid_origin.y + (tile_pos.y as f32 * TILE_SIZE_Y);

                        match tile_type {
                            TileType::TunnelUpOpen => {
                                draw_texture(
                                    &game_state.texture_mountain_tunnel_hole_open_u,
                                    x,
                                    y,
                                    WHITE,
                                );
                            }
                            TileType::TunnelDownOpen => {
                                draw_texture(
                                    &game_state.texture_mountain_tunnel_hole_open_d,
                                    x,
                                    y,
                                    WHITE,
                                );
                            }
                            TileType::TunnelLeftOpen => {
                                draw_texture(
                                    &game_state.texture_mountain_tunnel_hole_open_l,
                                    x,
                                    y,
                                    WHITE,
                                );
                            }
                            TileType::TunnelRightOpen => {
                                draw_texture(
                                    &game_state.texture_mountain_tunnel_hole_open_r,
                                    x,
                                    y,
                                    WHITE,
                                );
                            }
                            TileType::TunnelUpClosed => {
                                // Show bottom half of vertical track (positioned at bottom of tile)
                                draw_texture_ex(
                                    &game_state.texture_track_v,
                                    x,
                                    y + TILE_SIZE_Y / 2.0,
                                    WHITE,
                                    DrawTextureParams {
                                        source: Some(Rect::new(0.0, 16.0, 32.0, 16.0)),
                                        dest_size: Some(Vec2::new(TILE_SIZE_X, TILE_SIZE_Y / 2.0)),
                                        flip_y: true,
                                        ..Default::default()
                                    },
                                );
                            }
                            TileType::TunnelDownClosed => {
                                // Show top half of vertical track (positioned at top of tile)
                                draw_texture_ex(
                                    &game_state.texture_track_v,
                                    x,
                                    y,
                                    WHITE,
                                    DrawTextureParams {
                                        source: Some(Rect::new(0.0, 0.0, 32.0, 16.0)),
                                        dest_size: Some(Vec2::new(TILE_SIZE_X, TILE_SIZE_Y / 2.0)),
                                        flip_y: true,
                                        ..Default::default()
                                    },
                                );
                            }
                            TileType::TunnelLeftClosed => {
                                // Show right half of horizontal track (positioned at right of tile)
                                draw_texture_ex(
                                    &game_state.texture_track_h,
                                    x + TILE_SIZE_X / 2.0,
                                    y,
                                    WHITE,
                                    DrawTextureParams {
                                        source: Some(Rect::new(16.0, 0.0, 16.0, 32.0)),
                                        dest_size: Some(Vec2::new(TILE_SIZE_X / 2.0, TILE_SIZE_Y)),
                                        flip_y: true,
                                        ..Default::default()
                                    },
                                );
                            }
                            TileType::TunnelRightClosed => {
                                // Show left half of horizontal track (positioned at left of tile)
                                draw_texture_ex(
                                    &game_state.texture_track_h,
                                    x,
                                    y,
                                    WHITE,
                                    DrawTextureParams {
                                        source: Some(Rect::new(0.0, 0.0, 16.0, 32.0)),
                                        dest_size: Some(Vec2::new(TILE_SIZE_X / 2.0, TILE_SIZE_Y)),
                                        flip_y: true,
                                        ..Default::default()
                                    },
                                );
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

/// Render tunnel layer 3: half-tracks for open tunnels, holes for closed tunnels
fn render_tunnel_layer_3(game_state: &GameState) {
    if let Some(active_idx) = game_state.level_active {
        let grid_x = active_idx % 3;
        let grid_y = active_idx / 3;

        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = grid_x as i32 + dx;
                let ny = grid_y as i32 + dy;

                if nx >= 0 && nx < 3 && ny >= 0 && ny < 3 {
                    let neighbor_idx = (ny * 3 + nx) as usize;
                    let level = &game_state.levels[neighbor_idx];

                    let grid_offset = level.grid_offset();
                    let grid_origin = level.pos_world + grid_offset;

                    for (tile_pos, tile_type) in &level.tile_layout {
                        let x = grid_origin.x + (tile_pos.x as f32 * TILE_SIZE_X);
                        let y = grid_origin.y + (tile_pos.y as f32 * TILE_SIZE_Y);

                        match tile_type {
                            TileType::TunnelUpOpen => {
                                // Show bottom half of vertical track (positioned at bottom of tile)
                                draw_texture_ex(
                                    &game_state.texture_track_v,
                                    x,
                                    y + TILE_SIZE_Y / 2.0,
                                    WHITE,
                                    DrawTextureParams {
                                        source: Some(Rect::new(0.0, 16.0, 32.0, 16.0)),
                                        dest_size: Some(Vec2::new(TILE_SIZE_X, TILE_SIZE_Y / 2.0)),
                                        flip_y: true,
                                        ..Default::default()
                                    },
                                );
                            }
                            TileType::TunnelDownOpen => {
                                // Show top half of vertical track (positioned at top of tile)
                                draw_texture_ex(
                                    &game_state.texture_track_v,
                                    x,
                                    y,
                                    WHITE,
                                    DrawTextureParams {
                                        source: Some(Rect::new(0.0, 0.0, 32.0, 16.0)),
                                        dest_size: Some(Vec2::new(TILE_SIZE_X, TILE_SIZE_Y / 2.0)),
                                        flip_y: true,
                                        ..Default::default()
                                    },
                                );
                            }
                            TileType::TunnelLeftOpen => {
                                // Show right half of horizontal track (positioned at right of tile)
                                draw_texture_ex(
                                    &game_state.texture_track_h,
                                    x + TILE_SIZE_X / 2.0,
                                    y,
                                    WHITE,
                                    DrawTextureParams {
                                        source: Some(Rect::new(16.0, 0.0, 16.0, 32.0)),
                                        dest_size: Some(Vec2::new(TILE_SIZE_X / 2.0, TILE_SIZE_Y)),
                                        flip_y: true,
                                        ..Default::default()
                                    },
                                );
                            }
                            TileType::TunnelRightOpen => {
                                // Show left half of horizontal track (positioned at left of tile)
                                draw_texture_ex(
                                    &game_state.texture_track_h,
                                    x,
                                    y,
                                    WHITE,
                                    DrawTextureParams {
                                        source: Some(Rect::new(0.0, 0.0, 16.0, 32.0)),
                                        dest_size: Some(Vec2::new(TILE_SIZE_X / 2.0, TILE_SIZE_Y)),
                                        flip_y: true,
                                        ..Default::default()
                                    },
                                );
                            }
                            TileType::TunnelUpClosed => {
                                draw_texture(
                                    &game_state.texture_mountain_tunnel_hole_closed_u,
                                    x,
                                    y,
                                    WHITE,
                                );
                            }
                            TileType::TunnelDownClosed => {
                                draw_texture(
                                    &game_state.texture_mountain_tunnel_hole_closed_d,
                                    x,
                                    y,
                                    WHITE,
                                );
                            }
                            TileType::TunnelLeftClosed => {
                                draw_texture(
                                    &game_state.texture_mountain_tunnel_hole_closed_l,
                                    x,
                                    y,
                                    WHITE,
                                );
                            }
                            TileType::TunnelRightClosed => {
                                draw_texture(
                                    &game_state.texture_mountain_tunnel_hole_closed_r,
                                    x,
                                    y,
                                    WHITE,
                                );
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

/// Render tunnel layer 5: mountain tunnel frames
fn render_tunnel_frames(game_state: &GameState) {
    if let Some(active_idx) = game_state.level_active {
        let grid_x = active_idx % 3;
        let grid_y = active_idx / 3;

        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = grid_x as i32 + dx;
                let ny = grid_y as i32 + dy;

                if nx >= 0 && nx < 3 && ny >= 0 && ny < 3 {
                    let neighbor_idx = (ny * 3 + nx) as usize;
                    let level = &game_state.levels[neighbor_idx];

                    let grid_offset = level.grid_offset();
                    let grid_origin = level.pos_world + grid_offset;

                    for (tile_pos, tile_type) in &level.tile_layout {
                        let x = grid_origin.x + (tile_pos.x as f32 * TILE_SIZE_X);
                        let y = grid_origin.y + (tile_pos.y as f32 * TILE_SIZE_Y);

                        let texture = match tile_type {
                            TileType::TunnelUpOpen | TileType::TunnelUpClosed => {
                                Some(&game_state.texture_mountain_tunnel_u)
                            }
                            TileType::TunnelDownOpen | TileType::TunnelDownClosed => {
                                Some(&game_state.texture_mountain_tunnel_d)
                            }
                            TileType::TunnelLeftOpen | TileType::TunnelLeftClosed => {
                                Some(&game_state.texture_mountain_tunnel_l)
                            }
                            TileType::TunnelRightOpen | TileType::TunnelRightClosed => {
                                Some(&game_state.texture_mountain_tunnel_r)
                            }
                            _ => None,
                        };

                        if let Some(tex) = texture {
                            draw_texture(tex, x, y, WHITE);
                        }
                    }
                }
            }
        }
    }
}

fn render_train(game_state: &GameState) {
    // Calculate train world position from current level + train_tile_pos + offset
    if let Some(level) = game_state.current_level() {
        let grid_offset = level.grid_offset();
        let grid_origin = level.pos_world + grid_offset;

        // Base tile position
        let base_x = grid_origin.x + (game_state.train_tile_pos.x as f32 * TILE_SIZE_X);
        let base_y = grid_origin.y + (game_state.train_tile_pos.y as f32 * TILE_SIZE_Y);

        // Add smooth offset
        let train_world_x = base_x + (game_state.train_pos_offset.x * TILE_SIZE_X);
        let train_world_y = base_y + (game_state.train_pos_offset.y * TILE_SIZE_Y);

        // Select texture based on direction and animation frame
        let texture = match (game_state.train_direction, game_state.train_anim_frame) {
            (TrainDirection::Left, 0) => &game_state.texture_train_l_001,
            (TrainDirection::Left, _) => &game_state.texture_train_l_002,
            (TrainDirection::Right, 0) => &game_state.texture_train_r_001,
            (TrainDirection::Right, _) => &game_state.texture_train_r_002,
            (TrainDirection::Up, 0) => &game_state.texture_train_d_001,
            (TrainDirection::Up, _) => &game_state.texture_train_d_002,
            (TrainDirection::Down, 0) => &game_state.texture_train_u_001,
            (TrainDirection::Down, _) => &game_state.texture_train_u_002,
        };

        draw_texture_ex(
            texture,
            train_world_x,
            train_world_y,
            WHITE,
            DrawTextureParams {
                flip_y: true,
                ..Default::default()
            },
        );
    }
}

fn configure() {
    set_default_filter_mode(FilterMode::Nearest);
}
