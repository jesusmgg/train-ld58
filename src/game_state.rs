use std::collections::HashMap;

use macroquad::{
    audio::{load_sound, Sound},
    camera::{set_camera, Camera2D},
    math::{f32, IVec2, Rect},
    shapes::draw_rectangle,
    texture::{load_texture, Texture2D},
    window::{clear_background, screen_height, screen_width},
};

use crate::constants::*;
use crate::{styles::Styles, text::draw_scaled_text};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrainDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileType {
    // Track pieces
    TrackHorizontal,
    TrackVertical,
    TrackCornerUL,
    TrackCornerUR,
    TrackCornerDL,
    TrackCornerDR,

    // Obstacles
    Rock,
    Water,

    // Garbage system
    GarbagePickup,
    GarbageDropoff,

    // Mountain borders
    MountainBorderUp,
    MountainBorderDown,
    MountainBorderLeft,
    MountainBorderRight,
    MountainBorderCornerUL,
    MountainBorderCornerUR,
    MountainBorderCornerDL,
    MountainBorderCornerDR,

    // Tunnels (level connections with state)
    TunnelUpOpen,
    TunnelUpClosed,
    TunnelDownOpen,
    TunnelDownClosed,
    TunnelLeftOpen,
    TunnelLeftClosed,
    TunnelRightOpen,
    TunnelRightClosed,
}

pub struct GameState {
    pub styles: Styles,

    pub camera: Camera2D,
    pub camera_target_pos: f32::Vec2,

    pub mouse_pos: f32::Vec2,
    pub tile_highlighted: Option<IVec2>,
    pub tile_highlighted_prev: Option<IVec2>,

    pub levels: Vec<Level>,
    pub level_active: Option<usize>,

    pub selected_tile: Option<TileType>,

    pub texture_background_01: Texture2D,
    pub texture_track_h: Texture2D,
    pub texture_track_v: Texture2D,
    pub texture_track_corner_ul: Texture2D,
    pub texture_track_corner_ur: Texture2D,
    pub texture_track_corner_dl: Texture2D,
    pub texture_track_corner_dr: Texture2D,
    pub texture_placeholder: Texture2D,

    // Mountain borders
    pub texture_mountain_border_u: Texture2D,
    pub texture_mountain_border_d: Texture2D,
    pub texture_mountain_border_l: Texture2D,
    pub texture_mountain_border_r: Texture2D,
    pub texture_mountain_border_corner_ul: Texture2D,
    pub texture_mountain_border_corner_ur: Texture2D,
    pub texture_mountain_border_corner_dl: Texture2D,
    pub texture_mountain_border_corner_dr: Texture2D,

    // Mountain tunnels
    pub texture_mountain_tunnel_u: Texture2D,
    pub texture_mountain_tunnel_d: Texture2D,
    pub texture_mountain_tunnel_l: Texture2D,
    pub texture_mountain_tunnel_r: Texture2D,

    // Train
    pub texture_train_l_001: Texture2D,
    pub texture_train_r_001: Texture2D,
    pub train_tile_pos: IVec2, // Grid position within current level
    pub train_direction: TrainDirection,
}

impl GameState {
    pub async fn new() -> Self {
        let styles = Styles::new();

        GameState::show_loading_screen(&styles);

        let camera = Self::get_camera();
        let camera_target_pos = camera.target;

        let mouse_pos = f32::Vec2::ZERO;
        let tile_highlighted = None;
        let tile_highlighted_prev = None;

        let levels = GameState::create_levels();
        let level_active = Some(0);
        // let level_active = Some(levels.len() - 1);

        let selected_tile = None;

        // Initialize train position and direction based on first level's default start
        let train_tile_pos = levels[0].default_train_start;
        let train_direction = TrainDirection::Left; // Start facing left (entering from right)

        let texture_background_01 = load_texture("assets/sprites/background.png").await.unwrap();
        let texture_track_h = load_texture("assets/sprites/track_h.png").await.unwrap();
        let texture_track_v = load_texture("assets/sprites/track_v.png").await.unwrap();
        let texture_track_corner_ul = load_texture("assets/sprites/track_corner_ul.png")
            .await
            .unwrap();
        let texture_track_corner_ur = load_texture("assets/sprites/track_corner_ur.png")
            .await
            .unwrap();
        let texture_track_corner_dl = load_texture("assets/sprites/track_corner_dl.png")
            .await
            .unwrap();
        let texture_track_corner_dr = load_texture("assets/sprites/track_corner_dr.png")
            .await
            .unwrap();
        let texture_placeholder = load_texture("assets/sprites/placeholder.png")
            .await
            .unwrap();

        // Mountain borders
        let texture_mountain_border_u = load_texture("assets/sprites/mountain_border_u.png")
            .await
            .unwrap();
        let texture_mountain_border_d = load_texture("assets/sprites/mountain_border_d.png")
            .await
            .unwrap();
        let texture_mountain_border_l = load_texture("assets/sprites/mountain_border_l.png")
            .await
            .unwrap();
        let texture_mountain_border_r = load_texture("assets/sprites/mountain_border_r.png")
            .await
            .unwrap();

        // Corners use placeholder for now
        let texture_mountain_border_corner_ul = texture_placeholder.clone();
        let texture_mountain_border_corner_ur = texture_placeholder.clone();
        let texture_mountain_border_corner_dl = texture_placeholder.clone();
        let texture_mountain_border_corner_dr = texture_placeholder.clone();

        // Mountain tunnels
        let texture_mountain_tunnel_u = load_texture("assets/sprites/mountain_tunnel_u.png")
            .await
            .unwrap();
        let texture_mountain_tunnel_d = load_texture("assets/sprites/mountain_tunnel_d.png")
            .await
            .unwrap();
        let texture_mountain_tunnel_l = load_texture("assets/sprites/mountain_tunnel_L.png")
            .await
            .unwrap();
        let texture_mountain_tunnel_r = load_texture("assets/sprites/mountain_tunnel_r.png")
            .await
            .unwrap();

        let texture_train_l_001 = load_texture("assets/sprites/train_front_l_001.png")
            .await
            .unwrap();
        let texture_train_r_001 = load_texture("assets/sprites/train_front_r_001.png")
            .await
            .unwrap();

        let sfx_hover_01 = load_sound("assets/sfx/hover_02.ogg").await.unwrap();
        let sfx_explosion_01 = load_sound("assets/sfx/explosion_01.ogg").await.unwrap();
        let sfx_level_start_01 = load_sound("assets/sfx/level_start_01.ogg").await.unwrap();

        Self {
            styles,

            camera,
            camera_target_pos,

            mouse_pos,
            tile_highlighted,
            tile_highlighted_prev,

            level_active,
            levels,

            selected_tile,

            texture_background_01,
            texture_track_h,
            texture_track_v,
            texture_track_corner_ul,
            texture_track_corner_ur,
            texture_track_corner_dl,
            texture_track_corner_dr,
            texture_placeholder,

            texture_mountain_border_u,
            texture_mountain_border_d,
            texture_mountain_border_l,
            texture_mountain_border_r,
            texture_mountain_border_corner_ul,
            texture_mountain_border_corner_ur,
            texture_mountain_border_corner_dl,
            texture_mountain_border_corner_dr,

            texture_mountain_tunnel_u,
            texture_mountain_tunnel_d,
            texture_mountain_tunnel_l,
            texture_mountain_tunnel_r,

            texture_train_l_001,
            texture_train_r_001,
            train_tile_pos,
            train_direction,
        }
    }

    pub fn current_level_mut(&mut self) -> Option<&mut Level> {
        match self.level_active {
            None => return None,
            Some(i) => return Some(&mut self.levels[i]),
        }
    }

    pub fn current_level(&self) -> Option<&Level> {
        match self.level_active {
            None => return None,
            Some(i) => return Some(&self.levels[i]),
        }
    }

    pub fn get_texture_for_tile(&self, tile_type: TileType) -> &Texture2D {
        match tile_type {
            TileType::TrackHorizontal => &self.texture_track_h,
            TileType::TrackVertical => &self.texture_track_v,
            TileType::TrackCornerUL => &self.texture_track_corner_ul,
            TileType::TrackCornerUR => &self.texture_track_corner_ur,
            TileType::TrackCornerDL => &self.texture_track_corner_dl,
            TileType::TrackCornerDR => &self.texture_track_corner_dr,

            TileType::MountainBorderUp => &self.texture_mountain_border_u,
            TileType::MountainBorderDown => &self.texture_mountain_border_d,
            TileType::MountainBorderLeft => &self.texture_mountain_border_l,
            TileType::MountainBorderRight => &self.texture_mountain_border_r,
            TileType::MountainBorderCornerUL => &self.texture_mountain_border_corner_ul,
            TileType::MountainBorderCornerUR => &self.texture_mountain_border_corner_ur,
            TileType::MountainBorderCornerDL => &self.texture_mountain_border_corner_dl,
            TileType::MountainBorderCornerDR => &self.texture_mountain_border_corner_dr,

            TileType::TunnelUpOpen | TileType::TunnelUpClosed => &self.texture_mountain_tunnel_u,
            TileType::TunnelDownOpen | TileType::TunnelDownClosed => {
                &self.texture_mountain_tunnel_d
            }
            TileType::TunnelLeftOpen | TileType::TunnelLeftClosed => {
                &self.texture_mountain_tunnel_l
            }
            TileType::TunnelRightOpen | TileType::TunnelRightClosed => {
                &self.texture_mountain_tunnel_r
            }

            _ => &self.texture_placeholder,
        }
    }

    pub fn is_tile_permanent(&self, tile_type: TileType) -> bool {
        matches!(
            tile_type,
            TileType::MountainBorderUp
                | TileType::MountainBorderDown
                | TileType::MountainBorderLeft
                | TileType::MountainBorderRight
                | TileType::MountainBorderCornerUL
                | TileType::MountainBorderCornerUR
                | TileType::MountainBorderCornerDL
                | TileType::MountainBorderCornerDR
                | TileType::TunnelUpOpen
                | TileType::TunnelUpClosed
                | TileType::TunnelDownOpen
                | TileType::TunnelDownClosed
                | TileType::TunnelLeftOpen
                | TileType::TunnelLeftClosed
                | TileType::TunnelRightOpen
                | TileType::TunnelRightClosed
        )
    }

    fn show_loading_screen(styles: &Styles) {
        clear_background(styles.colors.bg_light);
        let font_size = 16.0;
        let message_size = 148.0;
        let pos_message_x = SCREEN_W / 2.0 - message_size / 2.0;
        let pos_message_y = (SCREEN_H / 2.0) - font_size;
        draw_rectangle(
            pos_message_x - 2.0,
            pos_message_y - 2.0,
            message_size + 4.0,
            16.0 + 4.0,
            styles.colors.orange_2,
        );
        draw_rectangle(
            pos_message_x,
            pos_message_y,
            message_size,
            16.0,
            styles.colors.yellow_1,
        );
        draw_scaled_text(
            "LOADING...",
            pos_message_x,
            pos_message_y + font_size / 1.333,
            font_size,
            &styles.colors.brown_3,
        );
    }

    pub fn create_levels() -> Vec<Level> {
        let mut levels = Vec::with_capacity(9);
        let grid_size = IVec2::new(12, 7);
        let w = grid_size.x;
        let h = grid_size.y;

        // Level 1-1 (grid 0,0 - has neighbors: right 1-2, down 2-1)
        // Default start: right tunnel (first one at w/3)
        let mut level = Level::new(
            "1-1",
            grid_size,
            f32::vec2(0.0, 0.0),
            IVec2::new(w - 1, h / 3),
        );
        level
            .tile_layout
            .insert(IVec2::new(-1, -1), TileType::MountainBorderCornerUL);
        level
            .tile_layout
            .insert(IVec2::new(w, -1), TileType::MountainBorderCornerUR);
        level
            .tile_layout
            .insert(IVec2::new(-1, h), TileType::MountainBorderCornerDL);
        level
            .tile_layout
            .insert(IVec2::new(w, h), TileType::MountainBorderCornerDR);
        for x in 0..w {
            level
                .tile_layout
                .insert(IVec2::new(x, -1), TileType::MountainBorderUp);
        }
        for x in 0..w {
            if x == w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownOpen);
            } else if x == 2 * w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::MountainBorderDown);
            }
        }
        for y in 0..h {
            level
                .tile_layout
                .insert(IVec2::new(-1, y), TileType::MountainBorderLeft);
        }
        for y in 0..h {
            if y == h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightOpen);
            } else if y == 2 * h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::MountainBorderRight);
            }
        }
        levels.push(level);

        // Level 1-2 (grid 1,0 - has neighbors: left 1-1, right 1-3, down 2-2)
        // Default start: left tunnel (first one at h/3)
        let mut level = Level::new(
            "1-2",
            grid_size,
            f32::vec2(SCREEN_W, 0.0),
            IVec2::new(0, h / 3),
        );
        level
            .tile_layout
            .insert(IVec2::new(-1, -1), TileType::MountainBorderCornerUL);
        level
            .tile_layout
            .insert(IVec2::new(w, -1), TileType::MountainBorderCornerUR);
        level
            .tile_layout
            .insert(IVec2::new(-1, h), TileType::MountainBorderCornerDL);
        level
            .tile_layout
            .insert(IVec2::new(w, h), TileType::MountainBorderCornerDR);
        for x in 0..w {
            level
                .tile_layout
                .insert(IVec2::new(x, -1), TileType::MountainBorderUp);
        }
        for x in 0..w {
            if x == w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownOpen);
            } else if x == 2 * w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::MountainBorderDown);
            }
        }
        for y in 0..h {
            if y == h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftOpen);
            } else if y == 2 * h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::MountainBorderLeft);
            }
        }
        for y in 0..h {
            if y == h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightOpen);
            } else if y == 2 * h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::MountainBorderRight);
            }
        }
        levels.push(level);

        // Level 1-3 (grid 2,0 - has neighbors: left 1-2, down 2-3)
        // Default start: left tunnel (first one at h/3)
        let mut level = Level::new(
            "1-3",
            grid_size,
            f32::vec2(SCREEN_W * 2.0, 0.0),
            IVec2::new(0, h / 3),
        );
        level
            .tile_layout
            .insert(IVec2::new(-1, -1), TileType::MountainBorderCornerUL);
        level
            .tile_layout
            .insert(IVec2::new(w, -1), TileType::MountainBorderCornerUR);
        level
            .tile_layout
            .insert(IVec2::new(-1, h), TileType::MountainBorderCornerDL);
        level
            .tile_layout
            .insert(IVec2::new(w, h), TileType::MountainBorderCornerDR);
        for x in 0..w {
            level
                .tile_layout
                .insert(IVec2::new(x, -1), TileType::MountainBorderUp);
        }
        for x in 0..w {
            if x == w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownOpen);
            } else if x == 2 * w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::MountainBorderDown);
            }
        }
        for y in 0..h {
            if y == h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftOpen);
            } else if y == 2 * h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::MountainBorderLeft);
            }
        }
        for y in 0..h {
            level
                .tile_layout
                .insert(IVec2::new(w, y), TileType::MountainBorderRight);
        }
        levels.push(level);

        // Level 2-1 (grid 0,1 - has neighbors: up 1-1, right 2-2, down 3-1)
        // Default start: top tunnel (first one at w/3)
        let mut level = Level::new(
            "2-1",
            grid_size,
            f32::vec2(0.0, SCREEN_H),
            IVec2::new(w / 3, 0),
        );
        level
            .tile_layout
            .insert(IVec2::new(-1, -1), TileType::MountainBorderCornerUL);
        level
            .tile_layout
            .insert(IVec2::new(w, -1), TileType::MountainBorderCornerUR);
        level
            .tile_layout
            .insert(IVec2::new(-1, h), TileType::MountainBorderCornerDL);
        level
            .tile_layout
            .insert(IVec2::new(w, h), TileType::MountainBorderCornerDR);
        for x in 0..w {
            if x == w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpOpen);
            } else if x == 2 * w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::MountainBorderUp);
            }
        }
        for x in 0..w {
            if x == w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownOpen);
            } else if x == 2 * w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::MountainBorderDown);
            }
        }
        for y in 0..h {
            level
                .tile_layout
                .insert(IVec2::new(-1, y), TileType::MountainBorderLeft);
        }
        for y in 0..h {
            if y == h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightOpen);
            } else if y == 2 * h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::MountainBorderRight);
            }
        }
        levels.push(level);

        // Level 2-2 (grid 1,1 - has neighbors: up 1-2, left 2-1, right 2-3, down 3-2)
        // Default start: top tunnel (first one at w/3)
        let mut level = Level::new(
            "2-2",
            grid_size,
            f32::vec2(SCREEN_W, SCREEN_H),
            IVec2::new(w / 3, 0),
        );
        level
            .tile_layout
            .insert(IVec2::new(-1, -1), TileType::MountainBorderCornerUL);
        level
            .tile_layout
            .insert(IVec2::new(w, -1), TileType::MountainBorderCornerUR);
        level
            .tile_layout
            .insert(IVec2::new(-1, h), TileType::MountainBorderCornerDL);
        level
            .tile_layout
            .insert(IVec2::new(w, h), TileType::MountainBorderCornerDR);
        for x in 0..w {
            if x == w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpOpen);
            } else if x == 2 * w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::MountainBorderUp);
            }
        }
        for x in 0..w {
            if x == w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownOpen);
            } else if x == 2 * w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::MountainBorderDown);
            }
        }
        for y in 0..h {
            if y == h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftOpen);
            } else if y == 2 * h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::MountainBorderLeft);
            }
        }
        for y in 0..h {
            if y == h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightOpen);
            } else if y == 2 * h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::MountainBorderRight);
            }
        }
        levels.push(level);

        // Level 2-3 (grid 2,1 - has neighbors: up 1-3, left 2-2, down 3-3)
        // Default start: top tunnel (first one at w/3)
        let mut level = Level::new(
            "2-3",
            grid_size,
            f32::vec2(SCREEN_W * 2.0, SCREEN_H),
            IVec2::new(w / 3, 0),
        );
        level
            .tile_layout
            .insert(IVec2::new(-1, -1), TileType::MountainBorderCornerUL);
        level
            .tile_layout
            .insert(IVec2::new(w, -1), TileType::MountainBorderCornerUR);
        level
            .tile_layout
            .insert(IVec2::new(-1, h), TileType::MountainBorderCornerDL);
        level
            .tile_layout
            .insert(IVec2::new(w, h), TileType::MountainBorderCornerDR);
        for x in 0..w {
            if x == w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpOpen);
            } else if x == 2 * w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::MountainBorderUp);
            }
        }
        for x in 0..w {
            if x == w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownOpen);
            } else if x == 2 * w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::MountainBorderDown);
            }
        }
        for y in 0..h {
            if y == h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftOpen);
            } else if y == 2 * h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::MountainBorderLeft);
            }
        }
        for y in 0..h {
            level
                .tile_layout
                .insert(IVec2::new(w, y), TileType::MountainBorderRight);
        }
        levels.push(level);

        // Level 3-1 (grid 0,2 - has neighbors: up 2-1, right 3-2)
        // Default start: top tunnel (first one at w/3)
        let mut level = Level::new(
            "3-1",
            grid_size,
            f32::vec2(0.0, SCREEN_H * 2.0),
            IVec2::new(w / 3, 0),
        );
        level
            .tile_layout
            .insert(IVec2::new(-1, -1), TileType::MountainBorderCornerUL);
        level
            .tile_layout
            .insert(IVec2::new(w, -1), TileType::MountainBorderCornerUR);
        level
            .tile_layout
            .insert(IVec2::new(-1, h), TileType::MountainBorderCornerDL);
        level
            .tile_layout
            .insert(IVec2::new(w, h), TileType::MountainBorderCornerDR);
        for x in 0..w {
            if x == w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpOpen);
            } else if x == 2 * w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::MountainBorderUp);
            }
        }
        for x in 0..w {
            level
                .tile_layout
                .insert(IVec2::new(x, h), TileType::MountainBorderDown);
        }
        for y in 0..h {
            level
                .tile_layout
                .insert(IVec2::new(-1, y), TileType::MountainBorderLeft);
        }
        for y in 0..h {
            if y == h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightOpen);
            } else if y == 2 * h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::MountainBorderRight);
            }
        }
        levels.push(level);

        // Level 3-2 (grid 1,2 - has neighbors: up 2-2, left 3-1, right 3-3)
        // Default start: top tunnel (first one at w/3)
        let mut level = Level::new(
            "3-2",
            grid_size,
            f32::vec2(SCREEN_W, SCREEN_H * 2.0),
            IVec2::new(w / 3, 0),
        );
        level
            .tile_layout
            .insert(IVec2::new(-1, -1), TileType::MountainBorderCornerUL);
        level
            .tile_layout
            .insert(IVec2::new(w, -1), TileType::MountainBorderCornerUR);
        level
            .tile_layout
            .insert(IVec2::new(-1, h), TileType::MountainBorderCornerDL);
        level
            .tile_layout
            .insert(IVec2::new(w, h), TileType::MountainBorderCornerDR);
        for x in 0..w {
            if x == w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpOpen);
            } else if x == 2 * w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::MountainBorderUp);
            }
        }
        for x in 0..w {
            level
                .tile_layout
                .insert(IVec2::new(x, h), TileType::MountainBorderDown);
        }
        for y in 0..h {
            if y == h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftOpen);
            } else if y == 2 * h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::MountainBorderLeft);
            }
        }
        for y in 0..h {
            if y == h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightOpen);
            } else if y == 2 * h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::MountainBorderRight);
            }
        }
        levels.push(level);

        // Level 3-3 (grid 2,2 - has neighbors: up 2-3, left 3-2)
        // Default start: top tunnel (first one at w/3)
        let mut level = Level::new(
            "3-3",
            grid_size,
            f32::vec2(SCREEN_W * 2.0, SCREEN_H * 2.0),
            IVec2::new(w / 3, 0),
        );
        level
            .tile_layout
            .insert(IVec2::new(-1, -1), TileType::MountainBorderCornerUL);
        level
            .tile_layout
            .insert(IVec2::new(w, -1), TileType::MountainBorderCornerUR);
        level
            .tile_layout
            .insert(IVec2::new(-1, h), TileType::MountainBorderCornerDL);
        level
            .tile_layout
            .insert(IVec2::new(w, h), TileType::MountainBorderCornerDR);
        for x in 0..w {
            if x == w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpOpen);
            } else if x == 2 * w / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::MountainBorderUp);
            }
        }
        for x in 0..w {
            level
                .tile_layout
                .insert(IVec2::new(x, h), TileType::MountainBorderDown);
        }
        for y in 0..h {
            if y == h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftOpen);
            } else if y == 2 * h / 3 {
                level
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftClosed);
            } else {
                level
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::MountainBorderLeft);
            }
        }
        for y in 0..h {
            level
                .tile_layout
                .insert(IVec2::new(w, y), TileType::MountainBorderRight);
        }
        levels.push(level);

        levels
    }

    fn get_camera() -> Camera2D {
        // Calculate integer zoom factor for pixel perfect rendering
        let zoom = ((screen_width() as i32 / SCREEN_W as i32)
            .min(screen_height() as i32 / SCREEN_H as i32)) as i32;

        let zoomed_w = (SCREEN_W as i32) * zoom;
        let zoomed_h = (SCREEN_H as i32) * zoom;

        // Center viewport on screen
        let x_offset = (screen_width() as i32 - zoomed_w) / 2;
        let y_offset = (screen_height() as i32 - zoomed_h) / 2;

        let camera = Camera2D {
            target: f32::vec2(SCREEN_W / 2.0, SCREEN_H / 2.0),
            zoom: f32::vec2(2.0 / SCREEN_W, -2.0 / SCREEN_H),
            offset: f32::Vec2::ZERO,
            rotation: 0.0,
            render_target: None,
            viewport: Some((x_offset, y_offset, zoomed_w, zoomed_h)),
        };

        set_camera(&camera);

        camera
    }
}

#[derive(Clone)]
pub struct Level {
    pub name: &'static str,
    pub grid_tiles: IVec2,
    pub pos_world: f32::Vec2,

    pub is_setup: bool,
    pub tile_layout: HashMap<IVec2, TileType>,
    pub default_train_start: IVec2, // Grid tile position where train starts by default
}

impl Level {
    pub fn new(
        name: &'static str,
        grid_tiles: IVec2,
        pos_world: f32::Vec2,
        default_train_start: IVec2,
    ) -> Self {
        let is_setup = false;
        let tile_layout = HashMap::new();

        Self {
            name,
            grid_tiles,
            pos_world,

            is_setup,
            tile_layout,
            default_train_start,
        }
    }

    pub fn grid_size_px(&self) -> f32::Vec2 {
        f32::Vec2::new(
            TILE_SIZE_X * self.grid_tiles.x as f32,
            TILE_SIZE_Y * self.grid_tiles.y as f32,
        )
    }

    pub fn grid_offset(&self) -> f32::Vec2 {
        let grid_size_px = self.grid_size_px();

        f32::Vec2::new(
            (SCREEN_W - grid_size_px.x) / 2.0,
            (SCREEN_H - grid_size_px.y) / 2.0,
        )
    }

    pub fn reset(&mut self) {
        self.is_setup = false;
    }
}
