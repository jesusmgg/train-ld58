use macroquad::{
    audio::{load_sound, Sound},
    camera::{set_camera, Camera2D},
    math::{f32, IVec2, Rect},
    shapes::draw_rectangle,
    texture::{load_texture, Texture2D},
    window::clear_background,
};

use crate::constants::*;
use crate::{styles::Styles, text::draw_scaled_text};

pub struct GameState {
    pub styles: Styles,

    pub camera: Camera2D,

    pub mouse_pos: f32::Vec2,
    pub is_mouse_in_grid: bool,
    pub tile_highlighted_prev: IVec2,
    pub tile_highlighted: IVec2,

    pub levels: Vec<Level>,
    pub level_active: Option<usize>,
    pub planet_current_index: usize,

    pub score: i32,

    pub sim_step: usize,
    pub sim_step_computed: usize,

    pub texture_background_01: Texture2D,

    pub sfx_hover_01: Sound,
    pub sfx_explosion_01: Sound,
    pub sfx_level_start_01: Sound,

    pub music_level_end_01: Sound,
}

impl GameState {
    pub async fn new() -> Self {
        let styles = Styles::new();

        GameState::show_loading_screen(&styles);

        let camera = Self::get_camera();

        let mouse_pos = f32::Vec2::ZERO;
        let is_mouse_in_grid = false;
        let tile_highlighted_prev = IVec2::splat(-1);
        let tile_highlighted = IVec2::ZERO;

        let levels = GameState::create_levels();
        let level_active = Some(0);
        // let level_active = Some(levels.len() - 1);
        let planet_current_index = 0;

        let score = 0;

        let sim_step = 0;
        let sim_step_computed = 0;

        let texture_background_01 = load_texture("assets/background.png").await.unwrap();

        let sfx_hover_01 = load_sound("assets/sfx/hover_02.ogg").await.unwrap();
        let sfx_explosion_01 = load_sound("assets/sfx/explosion_01.ogg").await.unwrap();
        let sfx_level_start_01 = load_sound("assets/sfx/level_start_01.ogg").await.unwrap();

        let music_level_end_01 = load_sound("assets/music/planet_001_short.ogg")
            .await
            .unwrap();

        Self {
            styles,

            camera,

            mouse_pos,
            is_mouse_in_grid,
            tile_highlighted_prev,
            tile_highlighted,

            level_active,
            levels,
            planet_current_index,

            score,

            sim_step,
            sim_step_computed,

            texture_background_01,

            sfx_hover_01,
            sfx_explosion_01,
            sfx_level_start_01,

            music_level_end_01,
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
        Vec::new()
    }

    fn get_camera() -> Camera2D {
        let camera_rect = Rect {
            x: 0.0,
            y: 0.0,
            w: SCREEN_W,
            h: SCREEN_H,
        };

        let camera_target = f32::vec2(
            camera_rect.x + camera_rect.w / 2.,
            camera_rect.y + camera_rect.h / 2.,
        );
        let camera_zoom = f32::vec2(1. / camera_rect.w * 2., 1. / camera_rect.h * 2.);

        let camera = Camera2D {
            target: camera_target,
            zoom: camera_zoom,
            offset: f32::Vec2::ZERO,
            rotation: 0.,

            render_target: None,
            viewport: None,
        };

        set_camera(&camera);

        camera
    }
}

#[derive(Clone)]
pub struct Level {
    pub name: &'static str,
    pub grid_tiles: IVec2,

    pub score: i32,

    pub was_failed: bool,
    pub was_stable: bool,

    pub is_failed: bool,
    pub is_stable: bool,

    pub is_setup: bool,
}

impl Level {
    pub fn new(name: &'static str, grid_tiles: IVec2) -> Self {
        let was_failed = false;
        let was_stable = false;

        let is_failed = false;
        let is_stable = false;

        let is_setup = false;

        let score = 0;

        Self {
            name,
            grid_tiles,

            score,

            was_failed,
            was_stable,

            is_failed,
            is_stable,

            is_setup,
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
        self.was_failed = false;
        self.was_stable = false;

        self.is_failed = false;
        self.is_stable = false;

        self.is_setup = false;

        self.score = 0;
    }
}
