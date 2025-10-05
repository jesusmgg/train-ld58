use std::collections::HashMap;

use macroquad::{
    audio::load_sound,
    camera::{set_camera, Camera2D},
    math::{f32, IVec2},
    shapes::draw_rectangle,
    text::{load_ttf_font, Font},
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
pub enum TrainState {
    Stopped,
    Running,
    Obstacle,
    BrokenRoute,
    Exiting,
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
    Rock1,
    House1,
    House2,

    // Garbage system
    GarbagePickupFull,
    GarbagePickupEmpty,
    GarbageDropoffEmpty,
    GarbageDropoffFull1,
    GarbageDropoffFull2,
    GarbageDropoffFull3,

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
    pub tile_highlight_pos: f32::Vec2, // Smoothly interpolated highlight position

    pub levels: Vec<Level>,
    pub level_active: Option<usize>,

    pub selected_tile: Option<TileType>,
    pub last_hovered_card: Option<usize>, // Track which UI card is being hovered over

    // Track piece inventory counts
    pub count_track_h: i32,
    pub count_track_v: i32,
    pub count_track_ul: i32,
    pub count_track_ur: i32,
    pub count_track_dl: i32,
    pub count_track_dr: i32,

    pub texture_background_01: Texture2D,
    pub texture_track_h: Texture2D,
    pub texture_track_v: Texture2D,
    pub texture_track_corner_ul: Texture2D,
    pub texture_track_corner_ur: Texture2D,
    pub texture_track_corner_dl: Texture2D,
    pub texture_track_corner_dr: Texture2D,
    pub texture_placeholder: Texture2D,

    // Obstacles
    pub texture_rock_1: Texture2D,
    pub texture_house_1: Texture2D,
    pub texture_house_2: Texture2D,

    // Garbage
    pub texture_garbage_full: Texture2D,
    pub texture_garbage_empty: Texture2D,
    pub texture_garbage_dropoff: Texture2D,
    pub texture_garbage_indicator_0: Texture2D,
    pub texture_garbage_indicator_1: Texture2D,
    pub texture_garbage_indicator_2: Texture2D,
    pub texture_garbage_indicator_3: Texture2D,

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

    // Tunnel holes
    pub texture_mountain_tunnel_hole_open_u: Texture2D,
    pub texture_mountain_tunnel_hole_open_d: Texture2D,
    pub texture_mountain_tunnel_hole_open_l: Texture2D,
    pub texture_mountain_tunnel_hole_open_r: Texture2D,
    pub texture_mountain_tunnel_hole_closed_u: Texture2D,
    pub texture_mountain_tunnel_hole_closed_d: Texture2D,
    pub texture_mountain_tunnel_hole_closed_l: Texture2D,
    pub texture_mountain_tunnel_hole_closed_r: Texture2D,

    // Train
    pub texture_train_l_001: Texture2D,
    pub texture_train_l_002: Texture2D,
    pub texture_train_r_001: Texture2D,
    pub texture_train_r_002: Texture2D,
    pub texture_train_u_001: Texture2D,
    pub texture_train_u_002: Texture2D,
    pub texture_train_d_001: Texture2D,
    pub texture_train_d_002: Texture2D,
    pub train_tile_pos: IVec2, // Logical grid position within current level
    pub train_pos_offset: f32::Vec2, // Smooth position offset from tile position (0.0 to 1.0)
    pub train_direction: TrainDirection,
    pub train_state: TrainState,
    pub train_anim_frame: u8,          // 0 or 1 for the two animation frames
    pub train_anim_timer: f32,         // Timer for animation
    pub garbage_held: i32,             // Amount of garbage currently on the train
    pub total_dropoffs_count: i32,     // Total number of dropoff sites across all levels
    pub dropoffs_full_count: i32,      // Number of dropoff sites at Full3 (3/3) state
    pub game_won: bool,                // True when all dropoffs are full
    pub message: Option<String>,       // Message to display in center of screen
    pub skip_level_requirements: bool, // Debug: skip level completion requirements
    pub visited_levels: Vec<bool>,     // Track which levels have been visited
    pub level_22_tunnel_timer: Option<f32>, // Timer for opening level 2-2 tunnels
    pub level_22_tunnels_opened: bool, // Whether level 2-2 tunnels have been opened
    pub win_message_shown: bool,       // Whether the win message has been shown
    pub help_message_shown: bool,      // Whether the help message has been shown

    // UI
    pub texture_ui_overlay: Texture2D,
    pub texture_ui_card_track_h: Texture2D,
    pub texture_ui_card_track_v: Texture2D,
    pub texture_ui_card_track_ul: Texture2D,
    pub texture_ui_card_track_ur: Texture2D,
    pub texture_ui_card_track_dl: Texture2D,
    pub texture_ui_card_track_dr: Texture2D,
    pub texture_ui_card_selection: Texture2D,

    // Font
    pub font: Font,

    // Music
    pub music_train_running_1: macroquad::audio::Sound,
    pub music_train_running_2: macroquad::audio::Sound,
    pub current_music_index: Option<usize>, // 0 or 1 for which track is playing
    pub music_volume: f32,                  // Current volume (0.0 to 1.0)
    pub music_target_volume: f32,           // Target volume for fading

    // Sound effects
    pub sfx_level_transition: macroquad::audio::Sound,
    pub sfx_ui_hover: macroquad::audio::Sound,
    pub sfx_ui_selection: macroquad::audio::Sound,
    pub sfx_ui_dialog_open: macroquad::audio::Sound,
    pub sfx_garbage_pickup: macroquad::audio::Sound,
    pub sfx_garbage_dispose_partial: macroquad::audio::Sound,
    pub sfx_garbage_dispose_full: macroquad::audio::Sound,
    pub sfx_track_place: macroquad::audio::Sound,
    pub sfx_track_remove: macroquad::audio::Sound,
    pub sfx_explosion: macroquad::audio::Sound,
}

impl GameState {
    pub async fn new() -> Self {
        let styles = Styles::new();

        // Load font first for loading screen
        let font = load_ttf_font("assets/fonts/KenneyPixel.ttf").await.unwrap();

        GameState::show_loading_screen(&styles, &font);

        let camera = Self::get_camera();
        let camera_target_pos = camera.target;

        let mouse_pos = f32::Vec2::ZERO;
        let tile_highlighted = None;
        let tile_highlighted_prev = None;
        let tile_highlight_pos = f32::Vec2::ZERO;

        let levels = GameState::create_levels();
        let level_active = Some(0);

        let selected_tile = None;
        let last_hovered_card = None;

        // Mark starting level as visited
        let mut visited_levels = vec![false; 9];
        if let Some(idx) = level_active {
            visited_levels[idx] = true;
        }

        // Initialize track piece counts
        let count_track_h = 10;
        let count_track_v = 10;
        let count_track_ul = 5;
        let count_track_ur = 5;
        let count_track_dl = 5;
        let count_track_dr = 5;

        // Initialize train position and direction based on first level's default start
        let train_tile_pos = levels[0].default_train_start;
        let train_direction = {
            let w = levels[0].grid_tiles.x;
            let h = levels[0].grid_tiles.y;
            let start = train_tile_pos;

            if start.x == -1 {
                TrainDirection::Right // Left tunnel, entering right
            } else if start.x == w {
                TrainDirection::Left // Right tunnel, entering left
            } else if start.y == -1 {
                TrainDirection::Down // Top tunnel, entering down
            } else if start.y == h {
                TrainDirection::Up // Bottom tunnel, entering up
            } else {
                TrainDirection::Right // Default
            }
        };
        let train_pos_offset = f32::Vec2::ZERO;
        let train_state = TrainState::Stopped;

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

        // Obstacles
        let texture_rock_1 = load_texture("assets/sprites/rock_001.png").await.unwrap();
        let texture_house_1 = load_texture("assets/sprites/house_001.png").await.unwrap();
        let texture_house_2 = load_texture("assets/sprites/house_002.png").await.unwrap();

        // Garbage
        let texture_garbage_full = load_texture("assets/sprites/garbage_full.png")
            .await
            .unwrap();
        let texture_garbage_empty = load_texture("assets/sprites/garbage_empty.png")
            .await
            .unwrap();
        let texture_garbage_dropoff = load_texture("assets/sprites/recyclying_center.png")
            .await
            .unwrap();
        let texture_garbage_indicator_0 = load_texture("assets/sprites/garbage_indicator_0.png")
            .await
            .unwrap();
        let texture_garbage_indicator_1 = load_texture("assets/sprites/garbage_indicator_1.png")
            .await
            .unwrap();
        let texture_garbage_indicator_2 = load_texture("assets/sprites/garbage_indicator_2.png")
            .await
            .unwrap();
        let texture_garbage_indicator_3 = load_texture("assets/sprites/garbage_indicator_3.png")
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
        let texture_mountain_border_corner_ul =
            load_texture("assets/sprites/mountain_corner_ul.png")
                .await
                .unwrap();
        let texture_mountain_border_corner_ur =
            load_texture("assets/sprites/mountain_corner_ur.png")
                .await
                .unwrap();
        let texture_mountain_border_corner_dl =
            load_texture("assets/sprites/mountain_corner_dl.png")
                .await
                .unwrap();
        let texture_mountain_border_corner_dr =
            load_texture("assets/sprites/mountain_corner_dr.png")
                .await
                .unwrap();

        // Mountain tunnels
        let texture_mountain_tunnel_u = load_texture("assets/sprites/mountain_tunnel_u.png")
            .await
            .unwrap();
        let texture_mountain_tunnel_d = load_texture("assets/sprites/mountain_tunnel_d.png")
            .await
            .unwrap();
        let texture_mountain_tunnel_l = load_texture("assets/sprites/mountain_tunnel_l.png")
            .await
            .unwrap();
        let texture_mountain_tunnel_r = load_texture("assets/sprites/mountain_tunnel_r.png")
            .await
            .unwrap();

        // Tunnel holes
        let texture_mountain_tunnel_hole_open_u =
            load_texture("assets/sprites/mountain_tunnel_hole_open_u.png")
                .await
                .unwrap();
        let texture_mountain_tunnel_hole_open_d =
            load_texture("assets/sprites/mountain_tunnel_hole_open_d.png")
                .await
                .unwrap();
        let texture_mountain_tunnel_hole_open_l =
            load_texture("assets/sprites/mountain_tunnel_hole_open_l.png")
                .await
                .unwrap();
        let texture_mountain_tunnel_hole_open_r =
            load_texture("assets/sprites/mountain_tunnel_hole_open_r.png")
                .await
                .unwrap();
        let texture_mountain_tunnel_hole_closed_u =
            load_texture("assets/sprites/mountain_tunnel_hole_closed_u.png")
                .await
                .unwrap();
        let texture_mountain_tunnel_hole_closed_d =
            load_texture("assets/sprites/mountain_tunnel_hole_closed_d.png")
                .await
                .unwrap();
        let texture_mountain_tunnel_hole_closed_l =
            load_texture("assets/sprites/mountain_tunnel_hole_closed_l.png")
                .await
                .unwrap();
        let texture_mountain_tunnel_hole_closed_r =
            load_texture("assets/sprites/mountain_tunnel_hole_closed_r.png")
                .await
                .unwrap();

        let texture_train_l_001 = load_texture("assets/sprites/train_front_l_001.png")
            .await
            .unwrap();
        let texture_train_l_002 = load_texture("assets/sprites/train_front_l_002.png")
            .await
            .unwrap();
        let texture_train_r_001 = load_texture("assets/sprites/train_front_r_001.png")
            .await
            .unwrap();
        let texture_train_r_002 = load_texture("assets/sprites/train_front_r_002.png")
            .await
            .unwrap();
        let texture_train_u_001 = load_texture("assets/sprites/train_front_u_001.png")
            .await
            .unwrap();
        let texture_train_u_002 = load_texture("assets/sprites/train_front_u_002.png")
            .await
            .unwrap();
        let texture_train_d_001 = load_texture("assets/sprites/train_front_d_001.png")
            .await
            .unwrap();
        let texture_train_d_002 = load_texture("assets/sprites/train_front_d_002.png")
            .await
            .unwrap();

        // UI
        let texture_ui_overlay = load_texture("assets/sprites/ui_overlay.png").await.unwrap();
        let texture_ui_card_track_h = load_texture("assets/sprites/ui_card_track_h.png")
            .await
            .unwrap();
        let texture_ui_card_track_v = load_texture("assets/sprites/ui_card_track_v.png")
            .await
            .unwrap();
        let texture_ui_card_track_ul = load_texture("assets/sprites/ui_card_track_ul.png")
            .await
            .unwrap();
        let texture_ui_card_track_ur = load_texture("assets/sprites/ui_card_track_ur.png")
            .await
            .unwrap();
        let texture_ui_card_track_dl = load_texture("assets/sprites/ui_card_track_dl.png")
            .await
            .unwrap();
        let texture_ui_card_track_dr = load_texture("assets/sprites/ui_card_track_dr.png")
            .await
            .unwrap();
        let texture_ui_card_selection = load_texture("assets/sprites/ui_card_selection.png")
            .await
            .unwrap();

        // Load sound effects
        let sfx_level_transition = load_sound("assets/sfx/level_transition.ogg").await.unwrap();
        let sfx_ui_hover = load_sound("assets/sfx/ui_hover.ogg").await.unwrap();
        let sfx_ui_selection = load_sound("assets/sfx/ui_selection.ogg").await.unwrap();
        let sfx_ui_dialog_open = load_sound("assets/sfx/ui_dialog_open.ogg").await.unwrap();
        let sfx_garbage_pickup = load_sound("assets/sfx/garbage_pickup.ogg").await.unwrap();
        let sfx_garbage_dispose_partial = load_sound("assets/sfx/garbage_dispose_partial.ogg").await.unwrap();
        let sfx_garbage_dispose_full = load_sound("assets/sfx/garbage_dispose_full.ogg").await.unwrap();
        let sfx_track_place = load_sound("assets/sfx/track_place.ogg").await.unwrap();
        let sfx_track_remove = load_sound("assets/sfx/track_remove.ogg").await.unwrap();
        let sfx_explosion = load_sound("assets/sfx/explosion_01.ogg").await.unwrap();

        // Load music tracks
        let music_train_running_1 = load_sound("assets/music/train_running_loop_01.ogg")
            .await
            .unwrap();
        let music_train_running_2 = load_sound("assets/music/train_running_loop_02.ogg")
            .await
            .unwrap();

        // Count total dropoffs across all levels
        let total_dropoffs_count = levels
            .iter()
            .flat_map(|level| level.tile_layout.values())
            .filter(|tile_type| {
                matches!(
                    tile_type,
                    TileType::GarbageDropoffEmpty
                        | TileType::GarbageDropoffFull1
                        | TileType::GarbageDropoffFull2
                        | TileType::GarbageDropoffFull3
                )
            })
            .count() as i32;

        Self {
            styles,

            camera,
            camera_target_pos,

            mouse_pos,
            tile_highlighted,
            tile_highlighted_prev,
            tile_highlight_pos,

            level_active,
            levels,

            selected_tile,
            last_hovered_card,

            count_track_h,
            count_track_v,
            count_track_ul,
            count_track_ur,
            count_track_dl,
            count_track_dr,

            texture_background_01,
            texture_track_h,
            texture_track_v,
            texture_track_corner_ul,
            texture_track_corner_ur,
            texture_track_corner_dl,
            texture_track_corner_dr,
            texture_placeholder,

            texture_rock_1,
            texture_house_1,
            texture_house_2,

            texture_garbage_full,
            texture_garbage_empty,
            texture_garbage_dropoff,
            texture_garbage_indicator_0,
            texture_garbage_indicator_1,
            texture_garbage_indicator_2,
            texture_garbage_indicator_3,

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

            texture_mountain_tunnel_hole_open_u,
            texture_mountain_tunnel_hole_open_d,
            texture_mountain_tunnel_hole_open_l,
            texture_mountain_tunnel_hole_open_r,
            texture_mountain_tunnel_hole_closed_u,
            texture_mountain_tunnel_hole_closed_d,
            texture_mountain_tunnel_hole_closed_l,
            texture_mountain_tunnel_hole_closed_r,

            texture_train_l_001,
            texture_train_l_002,
            texture_train_r_001,
            texture_train_r_002,
            texture_train_u_001,
            texture_train_u_002,
            texture_train_d_001,
            texture_train_d_002,
            train_tile_pos,
            train_pos_offset,
            train_direction,
            train_state,
            train_anim_frame: 0,
            train_anim_timer: 0.0,
            garbage_held: 0,
            total_dropoffs_count,
            dropoffs_full_count: 0,
            game_won: false,
            message: None,
            skip_level_requirements: false,
            visited_levels,
            level_22_tunnel_timer: None,
            level_22_tunnels_opened: false,
            win_message_shown: false,
            help_message_shown: false,

            texture_ui_overlay,
            texture_ui_card_track_h,
            texture_ui_card_track_v,
            texture_ui_card_track_ul,
            texture_ui_card_track_ur,
            texture_ui_card_track_dl,
            texture_ui_card_track_dr,
            texture_ui_card_selection,

            font,

            music_train_running_1,
            music_train_running_2,
            current_music_index: None,
            music_volume: 0.0,
            music_target_volume: 0.0,

            sfx_level_transition,
            sfx_ui_hover,
            sfx_ui_selection,
            sfx_ui_dialog_open,
            sfx_garbage_pickup,
            sfx_garbage_dispose_partial,
            sfx_garbage_dispose_full,
            sfx_track_place,
            sfx_track_remove,
            sfx_explosion,
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

            TileType::Rock1 => &self.texture_rock_1,
            TileType::House1 => &self.texture_house_1,
            TileType::House2 => &self.texture_house_2,

            TileType::GarbagePickupFull => &self.texture_garbage_full,
            TileType::GarbagePickupEmpty => &self.texture_garbage_empty,
            TileType::GarbageDropoffEmpty => &self.texture_garbage_dropoff,
            TileType::GarbageDropoffFull1 => &self.texture_garbage_dropoff,
            TileType::GarbageDropoffFull2 => &self.texture_garbage_dropoff,
            TileType::GarbageDropoffFull3 => &self.texture_garbage_dropoff,

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
                | TileType::Rock1
                | TileType::House1
                | TileType::House2
                | TileType::GarbagePickupFull
                | TileType::GarbagePickupEmpty
                | TileType::GarbageDropoffEmpty
                | TileType::GarbageDropoffFull1
                | TileType::GarbageDropoffFull2
                | TileType::GarbageDropoffFull3
        )
    }

    pub fn get_track_count(&self, tile_type: TileType) -> i32 {
        match tile_type {
            TileType::TrackHorizontal => self.count_track_h,
            TileType::TrackVertical => self.count_track_v,
            TileType::TrackCornerUL => self.count_track_ul,
            TileType::TrackCornerUR => self.count_track_ur,
            TileType::TrackCornerDL => self.count_track_dl,
            TileType::TrackCornerDR => self.count_track_dr,
            _ => 0,
        }
    }

    pub fn decrement_track_count(&mut self, tile_type: TileType) {
        match tile_type {
            TileType::TrackHorizontal => {
                if self.count_track_h > 0 {
                    self.count_track_h -= 1;
                }
            }
            TileType::TrackVertical => {
                if self.count_track_v > 0 {
                    self.count_track_v -= 1;
                }
            }
            TileType::TrackCornerUL => {
                if self.count_track_ul > 0 {
                    self.count_track_ul -= 1;
                }
            }
            TileType::TrackCornerUR => {
                if self.count_track_ur > 0 {
                    self.count_track_ur -= 1;
                }
            }
            TileType::TrackCornerDL => {
                if self.count_track_dl > 0 {
                    self.count_track_dl -= 1;
                }
            }
            TileType::TrackCornerDR => {
                if self.count_track_dr > 0 {
                    self.count_track_dr -= 1;
                }
            }
            _ => {}
        }
    }

    pub fn increment_track_count(&mut self, tile_type: TileType) {
        match tile_type {
            TileType::TrackHorizontal => self.count_track_h += 1,
            TileType::TrackVertical => self.count_track_v += 1,
            TileType::TrackCornerUL => self.count_track_ul += 1,
            TileType::TrackCornerUR => self.count_track_ur += 1,
            TileType::TrackCornerDL => self.count_track_dl += 1,
            TileType::TrackCornerDR => self.count_track_dr += 1,
            _ => {}
        }
    }

    pub fn reset_level(&mut self) {
        // Reset all garbage tiles in the current level
        // Only adjust garbage_held for pickups/dropoffs in this level
        if let Some(level_idx) = self.level_active {
            let level = &mut self.levels[level_idx];
            for y in 0..level.grid_tiles.y {
                for x in 0..level.grid_tiles.x {
                    let tile_pos = IVec2::new(x, y);
                    if let Some(tile_type) = level.tile_layout.get_mut(&tile_pos) {
                        match tile_type {
                            TileType::GarbagePickupEmpty => {
                                // This garbage was picked up from this level, return it
                                *tile_type = TileType::GarbagePickupFull;
                                self.garbage_held -= 1;
                            }
                            TileType::GarbageDropoffFull1 => {
                                // Return 1 garbage to player
                                *tile_type = TileType::GarbageDropoffEmpty;
                                self.garbage_held += 1;
                            }
                            TileType::GarbageDropoffFull2 => {
                                // Return 2 garbage to player
                                *tile_type = TileType::GarbageDropoffEmpty;
                                self.garbage_held += 2;
                            }
                            TileType::GarbageDropoffFull3 => {
                                // Return 3 garbage to player
                                *tile_type = TileType::GarbageDropoffEmpty;
                                self.garbage_held += 3;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Update dropoff counts
        self.update_dropoff_counts();
    }

    pub fn update_dropoff_counts(&mut self) {
        let mut total = 0;
        let mut full = 0;

        // Count across all levels
        for level in &self.levels {
            for tile_type in level.tile_layout.values() {
                match tile_type {
                    TileType::GarbageDropoffEmpty
                    | TileType::GarbageDropoffFull1
                    | TileType::GarbageDropoffFull2 => {
                        total += 1;
                    }
                    TileType::GarbageDropoffFull3 => {
                        total += 1;
                        full += 1;
                    }
                    _ => {}
                }
            }
        }

        self.total_dropoffs_count = total;
        self.dropoffs_full_count = full;
        self.game_won = full > 0 && full == total;
    }

    fn show_loading_screen(styles: &Styles, font: &Font) {
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
            font,
        );
    }

    pub fn create_levels() -> Vec<Level> {
        let mut levels = Vec::with_capacity(9);
        let grid_size = IVec2::new(10, 7);
        let w = grid_size.x;
        let h = grid_size.y;

        // Level 1-1 (grid 0,0 - has neighbors: right 1-2, down 2-1)
        // Default start: right tunnel (first one at h/3)
        let mut level11 = Level::new("1-1", grid_size, f32::vec2(0.0, 0.0), IVec2::new(w, h / 3));
        level11
            .tile_layout
            .insert(IVec2::new(-1, -1), TileType::MountainBorderCornerDL);
        level11
            .tile_layout
            .insert(IVec2::new(w, -1), TileType::MountainBorderCornerDR);
        level11
            .tile_layout
            .insert(IVec2::new(-1, h), TileType::MountainBorderCornerUL);
        level11
            .tile_layout
            .insert(IVec2::new(w, h), TileType::MountainBorderCornerUR);
        for x in 0..w {
            level11
                .tile_layout
                .insert(IVec2::new(x, -1), TileType::MountainBorderDown);
        }
        for x in 0..w {
            if x == w / 3 {
                level11
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownOpen);
            } else if x == 2 * w / 3 {
                level11
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownClosed);
            } else {
                level11
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::MountainBorderUp);
            }
        }
        for y in 0..h {
            level11
                .tile_layout
                .insert(IVec2::new(-1, y), TileType::MountainBorderLeft);
        }
        for y in 0..h {
            if y == h / 3 {
                level11
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightOpen);
            } else if y == 2 * h / 3 {
                level11
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightClosed);
            } else {
                level11
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::MountainBorderRight);
            }
        }
        // Add obstacles
        level11
            .tile_layout
            .insert(IVec2::new(5, 6), TileType::Rock1);
        level11
            .tile_layout
            .insert(IVec2::new(5, 5), TileType::Rock1);
        level11
            .tile_layout
            .insert(IVec2::new(6, 3), TileType::Rock1);
        level11
            .tile_layout
            .insert(IVec2::new(8, 3), TileType::House1);
        level11
            .tile_layout
            .insert(IVec2::new(5, 3), TileType::House2);
        level11
            .tile_layout
            .insert(IVec2::new(3, 4), TileType::House1);
        level11
            .tile_layout
            .insert(IVec2::new(9, 3), TileType::Rock1);
        level11
            .tile_layout
            .insert(IVec2::new(7, 3), TileType::GarbagePickupFull);
        level11
            .tile_layout
            .insert(IVec2::new(4, 3), TileType::GarbagePickupFull);
        level11
            .tile_layout
            .insert(IVec2::new(2, 4), TileType::GarbagePickupFull);
        level11
            .tile_layout
            .insert(IVec2::new(9, 6), TileType::GarbagePickupFull);
        // Add recycling center (dropoff)
        level11
            .tile_layout
            .insert(IVec2::new(0, 0), TileType::GarbageDropoffEmpty);
        levels.push(level11);

        // Level 1-2 (grid 1,0 - has neighbors: left 1-1, right 1-3, down 2-2)
        // Default start: right tunnel at (w, 2)
        let mut level12 = Level::new("1-2", grid_size, f32::vec2(SCREEN_W, 0.0), IVec2::new(w, 2));
        level12
            .tile_layout
            .insert(IVec2::new(-1, -1), TileType::MountainBorderCornerDL);
        level12
            .tile_layout
            .insert(IVec2::new(w, -1), TileType::MountainBorderCornerDR);
        level12
            .tile_layout
            .insert(IVec2::new(-1, h), TileType::MountainBorderCornerUL);
        level12
            .tile_layout
            .insert(IVec2::new(w, h), TileType::MountainBorderCornerUR);
        for x in 0..w {
            level12
                .tile_layout
                .insert(IVec2::new(x, -1), TileType::MountainBorderDown);
        }
        for x in 0..w {
            if x == w / 3 {
                level12
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownOpen);
            } else if x == 2 * w / 3 {
                level12
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownOpen);
            } else {
                level12
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::MountainBorderUp);
            }
        }
        for y in 0..h {
            if y == h / 3 {
                level12
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftOpen);
            } else if y == 2 * h / 3 {
                level12
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftClosed);
            } else {
                level12
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::MountainBorderLeft);
            }
        }
        for y in 0..h {
            if y == h / 3 {
                level12
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightOpen);
            } else if y == 2 * h / 3 {
                level12
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightClosed);
            } else {
                level12
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::MountainBorderRight);
            }
        }
        // Add rocks
        level12
            .tile_layout
            .insert(IVec2::new(0, 2), TileType::Rock1);
        level12
            .tile_layout
            .insert(IVec2::new(0, 1), TileType::Rock1);
        level12
            .tile_layout
            .insert(IVec2::new(0, 3), TileType::Rock1);
        // Add garbage pickups - full row 0 except 0,0
        level12
            .tile_layout
            .insert(IVec2::new(9, 6), TileType::GarbagePickupFull);
        level12
            .tile_layout
            .insert(IVec2::new(9, 5), TileType::GarbagePickupFull);
        level12
            .tile_layout
            .insert(IVec2::new(0, 6), TileType::GarbagePickupFull);
        level12
            .tile_layout
            .insert(IVec2::new(1, 0), TileType::GarbagePickupFull);
        level12
            .tile_layout
            .insert(IVec2::new(2, 0), TileType::GarbagePickupFull);
        level12
            .tile_layout
            .insert(IVec2::new(3, 0), TileType::GarbagePickupFull);
        level12
            .tile_layout
            .insert(IVec2::new(4, 0), TileType::GarbagePickupFull);
        level12
            .tile_layout
            .insert(IVec2::new(5, 0), TileType::GarbagePickupFull);
        level12
            .tile_layout
            .insert(IVec2::new(6, 0), TileType::GarbagePickupFull);
        level12
            .tile_layout
            .insert(IVec2::new(7, 0), TileType::GarbagePickupFull);
        level12
            .tile_layout
            .insert(IVec2::new(8, 0), TileType::GarbagePickupFull);
        level12
            .tile_layout
            .insert(IVec2::new(9, 0), TileType::GarbagePickupFull);
        // Add garbage pickups at row 4
        level12
            .tile_layout
            .insert(IVec2::new(3, 4), TileType::GarbagePickupFull);
        level12
            .tile_layout
            .insert(IVec2::new(4, 4), TileType::GarbagePickupFull);
        level12
            .tile_layout
            .insert(IVec2::new(5, 4), TileType::GarbagePickupFull);
        level12
            .tile_layout
            .insert(IVec2::new(6, 4), TileType::GarbagePickupFull);
        levels.push(level12);

        // Level 1-3 (grid 2,0 - has neighbors: left 1-2, down 2-3)
        // Default start: bottom tunnel at (3, h)
        let mut level13 = Level::new(
            "1-3",
            grid_size,
            f32::vec2(SCREEN_W * 2.0, 0.0),
            IVec2::new(3, h),
        );
        level13
            .tile_layout
            .insert(IVec2::new(-1, -1), TileType::MountainBorderCornerDL);
        level13
            .tile_layout
            .insert(IVec2::new(w, -1), TileType::MountainBorderCornerDR);
        level13
            .tile_layout
            .insert(IVec2::new(-1, h), TileType::MountainBorderCornerUL);
        level13
            .tile_layout
            .insert(IVec2::new(w, h), TileType::MountainBorderCornerUR);
        for x in 0..w {
            level13
                .tile_layout
                .insert(IVec2::new(x, -1), TileType::MountainBorderDown);
        }
        for x in 0..w {
            if x == w / 3 {
                level13
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownOpen);
            } else if x == 2 * w / 3 {
                level13
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownClosed);
            } else {
                level13
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::MountainBorderUp);
            }
        }
        for y in 0..h {
            if y == h / 3 {
                level13
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftOpen);
            } else if y == 2 * h / 3 {
                level13
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftClosed);
            } else {
                level13
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::MountainBorderLeft);
            }
        }
        for y in 0..h {
            level13
                .tile_layout
                .insert(IVec2::new(w, y), TileType::MountainBorderRight);
        }
        // Add houses
        level13
            .tile_layout
            .insert(IVec2::new(9, 6), TileType::House1);
        level13
            .tile_layout
            .insert(IVec2::new(9, 5), TileType::House2);
        level13
            .tile_layout
            .insert(IVec2::new(9, 4), TileType::House1);
        level13
            .tile_layout
            .insert(IVec2::new(9, 3), TileType::House2);
        level13
            .tile_layout
            .insert(IVec2::new(4, 6), TileType::House1);
        level13
            .tile_layout
            .insert(IVec2::new(5, 6), TileType::House2);
        level13
            .tile_layout
            .insert(IVec2::new(4, 5), TileType::House1);
        level13
            .tile_layout
            .insert(IVec2::new(5, 5), TileType::House2);
        level13
            .tile_layout
            .insert(IVec2::new(4, 3), TileType::House1);
        level13
            .tile_layout
            .insert(IVec2::new(4, 2), TileType::House1);
        level13
            .tile_layout
            .insert(IVec2::new(1, 5), TileType::House2);
        level13
            .tile_layout
            .insert(IVec2::new(8, 6), TileType::House1);
        level13
            .tile_layout
            .insert(IVec2::new(8, 5), TileType::House2);
        // Add rocks
        level13
            .tile_layout
            .insert(IVec2::new(0, 0), TileType::Rock1);
        level13
            .tile_layout
            .insert(IVec2::new(0, 3), TileType::Rock1);
        level13
            .tile_layout
            .insert(IVec2::new(1, 3), TileType::Rock1);
        level13
            .tile_layout
            .insert(IVec2::new(2, 1), TileType::Rock1);
        level13
            .tile_layout
            .insert(IVec2::new(2, 2), TileType::Rock1);
        // Add garbage pickups
        level13
            .tile_layout
            .insert(IVec2::new(3, 5), TileType::GarbagePickupFull);
        level13
            .tile_layout
            .insert(IVec2::new(9, 1), TileType::GarbagePickupFull);
        level13
            .tile_layout
            .insert(IVec2::new(9, 2), TileType::GarbagePickupFull);
        level13
            .tile_layout
            .insert(IVec2::new(9, 0), TileType::GarbagePickupFull);
        level13
            .tile_layout
            .insert(IVec2::new(5, 3), TileType::GarbagePickupFull);
        level13
            .tile_layout
            .insert(IVec2::new(5, 2), TileType::GarbagePickupFull);
        level13
            .tile_layout
            .insert(IVec2::new(1, 2), TileType::GarbagePickupFull);
        // Add recycling centers (dropoffs)
        level13
            .tile_layout
            .insert(IVec2::new(6, 0), TileType::GarbageDropoffEmpty);
        level13
            .tile_layout
            .insert(IVec2::new(7, 3), TileType::GarbageDropoffEmpty);
        level13
            .tile_layout
            .insert(IVec2::new(2, 3), TileType::GarbageDropoffEmpty);
        levels.push(level13);

        // Level 2-1 (grid 0,1 - has neighbors: up 1-1, right 2-2, down 3-1)
        // Default start: top tunnel (first one at w/3)
        let mut level21 = Level::new(
            "2-1",
            grid_size,
            f32::vec2(0.0, SCREEN_H),
            IVec2::new(w / 3, -1),
        );
        level21
            .tile_layout
            .insert(IVec2::new(-1, -1), TileType::MountainBorderCornerDL);
        level21
            .tile_layout
            .insert(IVec2::new(w, -1), TileType::MountainBorderCornerDR);
        level21
            .tile_layout
            .insert(IVec2::new(-1, h), TileType::MountainBorderCornerUL);
        level21
            .tile_layout
            .insert(IVec2::new(w, h), TileType::MountainBorderCornerUR);
        for x in 0..w {
            if x == w / 3 {
                level21
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpOpen);
            } else if x == 2 * w / 3 {
                level21
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpClosed);
            } else {
                level21
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::MountainBorderDown);
            }
        }
        for x in 0..w {
            if x == w / 3 {
                level21
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownOpen);
            } else if x == 2 * w / 3 {
                level21
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownClosed);
            } else {
                level21
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::MountainBorderUp);
            }
        }
        for y in 0..h {
            level21
                .tile_layout
                .insert(IVec2::new(-1, y), TileType::MountainBorderLeft);
        }
        for y in 0..h {
            if y == 2 * h / 3 {
                level21
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightClosed);
            } else {
                level21
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::MountainBorderRight);
            }
        }
        // Add houses
        level21
            .tile_layout
            .insert(IVec2::new(2, 2), TileType::House1);
        level21
            .tile_layout
            .insert(IVec2::new(5, 3), TileType::House2);
        level21
            .tile_layout
            .insert(IVec2::new(9, 1), TileType::House1);
        // Add rocks
        level21
            .tile_layout
            .insert(IVec2::new(6, 5), TileType::Rock1);
        level21
            .tile_layout
            .insert(IVec2::new(7, 4), TileType::Rock1);
        level21
            .tile_layout
            .insert(IVec2::new(8, 3), TileType::Rock1);
        level21
            .tile_layout
            .insert(IVec2::new(3, 1), TileType::Rock1);
        // Add garbage pickups
        level21
            .tile_layout
            .insert(IVec2::new(1, 2), TileType::GarbagePickupFull);
        level21
            .tile_layout
            .insert(IVec2::new(6, 3), TileType::GarbagePickupFull);
        level21
            .tile_layout
            .insert(IVec2::new(8, 1), TileType::GarbagePickupFull);
        // Add recycling centers (dropoffs)
        level21
            .tile_layout
            .insert(IVec2::new(1, 5), TileType::GarbageDropoffEmpty);
        level21
            .tile_layout
            .insert(IVec2::new(9, 6), TileType::GarbageDropoffEmpty);
        levels.push(level21);

        // Level 2-2 (grid 1,1 - has neighbors: up 1-2, left 2-1, right 2-3, down 3-2)
        // Default start: top tunnel (first one at w/3)
        let mut level22 = Level::new(
            "2-2",
            grid_size,
            f32::vec2(SCREEN_W, SCREEN_H),
            IVec2::new(w / 3, -1),
        );
        level22
            .tile_layout
            .insert(IVec2::new(-1, -1), TileType::MountainBorderCornerDL);
        level22
            .tile_layout
            .insert(IVec2::new(w, -1), TileType::MountainBorderCornerDR);
        level22
            .tile_layout
            .insert(IVec2::new(-1, h), TileType::MountainBorderCornerUL);
        level22
            .tile_layout
            .insert(IVec2::new(w, h), TileType::MountainBorderCornerUR);
        for x in 0..w {
            if x == w / 3 {
                level22
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpOpen);
            } else if x == 2 * w / 3 {
                level22
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpOpen);
            } else {
                level22
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::MountainBorderDown);
            }
        }
        for x in 0..w {
            if x == 2 * w / 3 {
                level22
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownClosed);
            } else {
                level22
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::MountainBorderUp);
            }
        }
        for y in 0..h {
            if y == 2 * h / 3 {
                level22
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftClosed);
            } else {
                level22
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::MountainBorderLeft);
            }
        }
        for y in 0..h {
            if y == h / 3 {
                level22
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightClosed);
            } else if y == 2 * h / 3 {
                level22
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightClosed);
            } else {
                level22
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::MountainBorderRight);
            }
        }
        level22
            .tile_layout
            .insert(IVec2::new(4, 4), TileType::GarbagePickupFull);
        level22
            .tile_layout
            .insert(IVec2::new(5, 4), TileType::GarbagePickupFull);
        // Add houses
        level22
            .tile_layout
            .insert(IVec2::new(4, 3), TileType::House1);
        level22
            .tile_layout
            .insert(IVec2::new(5, 3), TileType::House2);
        // Add rocks
        level22
            .tile_layout
            .insert(IVec2::new(3, 3), TileType::Rock1);
        level22
            .tile_layout
            .insert(IVec2::new(6, 3), TileType::Rock1);
        // Add recycling centers (dropoffs) at 4 corners
        level22
            .tile_layout
            .insert(IVec2::new(0, 0), TileType::GarbageDropoffEmpty);
        level22
            .tile_layout
            .insert(IVec2::new(9, 0), TileType::GarbageDropoffEmpty);
        level22
            .tile_layout
            .insert(IVec2::new(0, 6), TileType::GarbageDropoffEmpty);
        level22
            .tile_layout
            .insert(IVec2::new(9, 6), TileType::GarbageDropoffEmpty);
        levels.push(level22);

        // Level 2-3 (grid 2,1 - has neighbors: up 1-3, left 2-2, down 3-3)
        // Default start: bottom tunnel at (3, h)
        let mut level23 = Level::new(
            "2-3",
            grid_size,
            f32::vec2(SCREEN_W * 2.0, SCREEN_H),
            IVec2::new(3, h),
        );
        level23
            .tile_layout
            .insert(IVec2::new(-1, -1), TileType::MountainBorderCornerDL);
        level23
            .tile_layout
            .insert(IVec2::new(w, -1), TileType::MountainBorderCornerDR);
        level23
            .tile_layout
            .insert(IVec2::new(-1, h), TileType::MountainBorderCornerUL);
        level23
            .tile_layout
            .insert(IVec2::new(w, h), TileType::MountainBorderCornerUR);
        for x in 0..w {
            if x == w / 3 {
                level23
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpOpen);
            } else if x == 2 * w / 3 {
                level23
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpClosed);
            } else {
                level23
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::MountainBorderDown);
            }
        }
        for x in 0..w {
            if x == w / 3 {
                level23
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownOpen);
            } else if x == 2 * w / 3 {
                level23
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::TunnelDownClosed);
            } else {
                level23
                    .tile_layout
                    .insert(IVec2::new(x, h), TileType::MountainBorderUp);
            }
        }
        for y in 0..h {
            if y == h / 3 {
                level23
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftClosed);
            } else if y == 2 * h / 3 {
                level23
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftClosed);
            } else {
                level23
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::MountainBorderLeft);
            }
        }
        for y in 0..h {
            level23
                .tile_layout
                .insert(IVec2::new(w, y), TileType::MountainBorderRight);
        }
        // Add houses
        level23
            .tile_layout
            .insert(IVec2::new(1, 5), TileType::House1);
        level23
            .tile_layout
            .insert(IVec2::new(3, 5), TileType::House2);
        level23
            .tile_layout
            .insert(IVec2::new(5, 5), TileType::House1);
        level23
            .tile_layout
            .insert(IVec2::new(7, 5), TileType::House2);
        level23
            .tile_layout
            .insert(IVec2::new(9, 5), TileType::House1);
        level23
            .tile_layout
            .insert(IVec2::new(1, 3), TileType::House2);
        level23
            .tile_layout
            .insert(IVec2::new(2, 3), TileType::House1);
        level23
            .tile_layout
            .insert(IVec2::new(3, 3), TileType::House2);
        level23
            .tile_layout
            .insert(IVec2::new(4, 3), TileType::House1);
        level23
            .tile_layout
            .insert(IVec2::new(6, 3), TileType::House2);
        level23
            .tile_layout
            .insert(IVec2::new(8, 3), TileType::House1);
        level23
            .tile_layout
            .insert(IVec2::new(1, 1), TileType::House2);
        level23
            .tile_layout
            .insert(IVec2::new(3, 1), TileType::House1);
        level23
            .tile_layout
            .insert(IVec2::new(4, 1), TileType::House2);
        level23
            .tile_layout
            .insert(IVec2::new(6, 1), TileType::House1);
        level23
            .tile_layout
            .insert(IVec2::new(7, 1), TileType::House2);
        level23
            .tile_layout
            .insert(IVec2::new(9, 1), TileType::House1);
        // Add garbage pickups
        level23
            .tile_layout
            .insert(IVec2::new(0, 5), TileType::GarbagePickupFull);
        level23
            .tile_layout
            .insert(IVec2::new(0, 1), TileType::GarbagePickupFull);
        level23
            .tile_layout
            .insert(IVec2::new(9, 2), TileType::GarbagePickupFull);
        level23
            .tile_layout
            .insert(IVec2::new(9, 6), TileType::GarbagePickupFull);
        // Add recycling center (dropoff)
        level23
            .tile_layout
            .insert(IVec2::new(9, 0), TileType::GarbageDropoffEmpty);
        levels.push(level23);

        // Level 3-1 (grid 0,2 - has neighbors: up 2-1, right 3-2)
        // Default start: top tunnel (first one at w/3)
        let mut level31 = Level::new(
            "3-1",
            grid_size,
            f32::vec2(0.0, SCREEN_H * 2.0),
            IVec2::new(w / 3, -1),
        );
        level31
            .tile_layout
            .insert(IVec2::new(-1, -1), TileType::MountainBorderCornerDL);
        level31
            .tile_layout
            .insert(IVec2::new(w, -1), TileType::MountainBorderCornerDR);
        level31
            .tile_layout
            .insert(IVec2::new(-1, h), TileType::MountainBorderCornerUL);
        level31
            .tile_layout
            .insert(IVec2::new(w, h), TileType::MountainBorderCornerUR);
        for x in 0..w {
            if x == w / 3 {
                level31
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpOpen);
            } else if x == 2 * w / 3 {
                level31
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpClosed);
            } else {
                level31
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::MountainBorderDown);
            }
        }
        for x in 0..w {
            level31
                .tile_layout
                .insert(IVec2::new(x, h), TileType::MountainBorderUp);
        }
        for y in 0..h {
            level31
                .tile_layout
                .insert(IVec2::new(-1, y), TileType::MountainBorderLeft);
        }
        for y in 0..h {
            if y == h / 3 {
                level31
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightOpen);
            } else if y == 2 * h / 3 {
                level31
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightClosed);
            } else {
                level31
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::MountainBorderRight);
            }
        }
        // Add houses
        level31
            .tile_layout
            .insert(IVec2::new(2, 2), TileType::House1);
        level31
            .tile_layout
            .insert(IVec2::new(1, 6), TileType::House2);
        level31
            .tile_layout
            .insert(IVec2::new(6, 4), TileType::House1);
        // Add rocks
        level31
            .tile_layout
            .insert(IVec2::new(4, 0), TileType::Rock1);
        level31
            .tile_layout
            .insert(IVec2::new(4, 1), TileType::Rock1);
        level31
            .tile_layout
            .insert(IVec2::new(4, 4), TileType::Rock1);
        level31
            .tile_layout
            .insert(IVec2::new(4, 5), TileType::Rock1);
        level31
            .tile_layout
            .insert(IVec2::new(4, 6), TileType::Rock1);
        // Add garbage pickups
        level31
            .tile_layout
            .insert(IVec2::new(3, 2), TileType::GarbagePickupFull);
        level31
            .tile_layout
            .insert(IVec2::new(0, 6), TileType::GarbagePickupFull);
        level31
            .tile_layout
            .insert(IVec2::new(5, 4), TileType::GarbagePickupFull);
        // Add recycling centers (dropoffs)
        level31
            .tile_layout
            .insert(IVec2::new(2, 6), TileType::GarbageDropoffEmpty);
        level31
            .tile_layout
            .insert(IVec2::new(8, 3), TileType::GarbageDropoffEmpty);
        levels.push(level31);

        // Level 3-2 (grid 1,2 - has neighbors: up 2-2, left 3-1, right 3-3)
        // Default start: left tunnel (first one at h/3)
        let mut level32 = Level::new(
            "3-2",
            grid_size,
            f32::vec2(SCREEN_W, SCREEN_H * 2.0),
            IVec2::new(-1, h / 3),
        );
        level32
            .tile_layout
            .insert(IVec2::new(-1, -1), TileType::MountainBorderCornerDL);
        level32
            .tile_layout
            .insert(IVec2::new(w, -1), TileType::MountainBorderCornerDR);
        level32
            .tile_layout
            .insert(IVec2::new(-1, h), TileType::MountainBorderCornerUL);
        level32
            .tile_layout
            .insert(IVec2::new(w, h), TileType::MountainBorderCornerUR);
        for x in 0..w {
            if x == 2 * w / 3 {
                level32
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpClosed);
            } else {
                level32
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::MountainBorderDown);
            }
        }
        for x in 0..w {
            level32
                .tile_layout
                .insert(IVec2::new(x, h), TileType::MountainBorderUp);
        }
        for y in 0..h {
            if y == h / 3 {
                level32
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftOpen);
            } else if y == 2 * h / 3 {
                level32
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftClosed);
            } else {
                level32
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::MountainBorderLeft);
            }
        }
        for y in 0..h {
            if y == h / 3 {
                level32
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightOpen);
            } else if y == 2 * h / 3 {
                level32
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::TunnelRightClosed);
            } else {
                level32
                    .tile_layout
                    .insert(IVec2::new(w, y), TileType::MountainBorderRight);
            }
        }
        // Add houses
        level32
            .tile_layout
            .insert(IVec2::new(1, 0), TileType::House1);
        level32
            .tile_layout
            .insert(IVec2::new(9, 1), TileType::House2);
        // Add rocks
        level32
            .tile_layout
            .insert(IVec2::new(4, 4), TileType::Rock1);
        level32
            .tile_layout
            .insert(IVec2::new(5, 4), TileType::Rock1);
        level32
            .tile_layout
            .insert(IVec2::new(6, 4), TileType::Rock1);
        level32
            .tile_layout
            .insert(IVec2::new(8, 2), TileType::Rock1);
        // Add garbage pickups
        level32
            .tile_layout
            .insert(IVec2::new(0, 0), TileType::GarbagePickupFull);
        level32
            .tile_layout
            .insert(IVec2::new(2, 0), TileType::GarbagePickupFull);
        level32
            .tile_layout
            .insert(IVec2::new(9, 0), TileType::GarbagePickupFull);
        // Add recycling center (dropoff)
        level32
            .tile_layout
            .insert(IVec2::new(5, 6), TileType::GarbageDropoffEmpty);
        levels.push(level32);

        // Level 3-3 (grid 2,2 - has neighbors: up 2-3, left 3-2)
        // Default start: left tunnel at y=2
        let mut level33 = Level::new(
            "3-3",
            grid_size,
            f32::vec2(SCREEN_W * 2.0, SCREEN_H * 2.0),
            IVec2::new(-1, 2),
        );
        level33
            .tile_layout
            .insert(IVec2::new(-1, -1), TileType::MountainBorderCornerDL);
        level33
            .tile_layout
            .insert(IVec2::new(w, -1), TileType::MountainBorderCornerDR);
        level33
            .tile_layout
            .insert(IVec2::new(-1, h), TileType::MountainBorderCornerUL);
        level33
            .tile_layout
            .insert(IVec2::new(w, h), TileType::MountainBorderCornerUR);
        for x in 0..w {
            if x == w / 3 {
                level33
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpOpen);
            } else if x == 2 * w / 3 {
                level33
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::TunnelUpClosed);
            } else {
                level33
                    .tile_layout
                    .insert(IVec2::new(x, -1), TileType::MountainBorderDown);
            }
        }
        for x in 0..w {
            level33
                .tile_layout
                .insert(IVec2::new(x, h), TileType::MountainBorderUp);
        }
        for y in 0..h {
            if y == h / 3 {
                level33
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftOpen);
            } else if y == 2 * h / 3 {
                level33
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::TunnelLeftClosed);
            } else {
                level33
                    .tile_layout
                    .insert(IVec2::new(-1, y), TileType::MountainBorderLeft);
            }
        }
        for y in 0..h {
            level33
                .tile_layout
                .insert(IVec2::new(w, y), TileType::MountainBorderRight);
        }
        // Add houses
        level33
            .tile_layout
            .insert(IVec2::new(0, 6), TileType::House1);
        level33
            .tile_layout
            .insert(IVec2::new(1, 6), TileType::House2);
        level33
            .tile_layout
            .insert(IVec2::new(2, 6), TileType::House1);
        level33
            .tile_layout
            .insert(IVec2::new(8, 2), TileType::House2);
        // Add rocks
        level33
            .tile_layout
            .insert(IVec2::new(9, 4), TileType::Rock1);
        level33
            .tile_layout
            .insert(IVec2::new(5, 0), TileType::Rock1);
        level33
            .tile_layout
            .insert(IVec2::new(5, 1), TileType::Rock1);
        // Add garbage pickups
        level33
            .tile_layout
            .insert(IVec2::new(0, 5), TileType::GarbagePickupFull);
        level33
            .tile_layout
            .insert(IVec2::new(1, 5), TileType::GarbagePickupFull);
        level33
            .tile_layout
            .insert(IVec2::new(3, 6), TileType::GarbagePickupFull);
        level33
            .tile_layout
            .insert(IVec2::new(7, 2), TileType::GarbagePickupFull);
        level33
            .tile_layout
            .insert(IVec2::new(8, 3), TileType::GarbagePickupFull);
        level33
            .tile_layout
            .insert(IVec2::new(9, 2), TileType::GarbagePickupFull);
        // Add recycling centers (dropoffs)
        level33
            .tile_layout
            .insert(IVec2::new(5, 3), TileType::GarbageDropoffEmpty);
        level33
            .tile_layout
            .insert(IVec2::new(9, 6), TileType::GarbageDropoffEmpty);
        levels.push(level33);

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
        let tile_layout = HashMap::new();

        Self {
            name,
            grid_tiles,
            pos_world,

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
}
