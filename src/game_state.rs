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

pub struct GameState {
    pub styles: Styles,

    pub camera: Camera2D,

    pub mouse_pos: f32::Vec2,

    pub levels: Vec<Level>,
    pub level_active: Option<usize>,

    pub texture_background_01: Texture2D,
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

        let texture_background_01 = load_texture("assets/background.png").await.unwrap();

        let sfx_hover_01 = load_sound("assets/sfx/hover_02.ogg").await.unwrap();
        let sfx_explosion_01 = load_sound("assets/sfx/explosion_01.ogg").await.unwrap();
        let sfx_level_start_01 = load_sound("assets/sfx/level_start_01.ogg").await.unwrap();

        Self {
            styles,

            camera,

            mouse_pos,

            level_active,
            levels,

            texture_background_01,
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
        let mut levels = Vec::with_capacity(9);

        // 3x3 grid of levels, each at full screen size intervals
        levels.push(Level::new("1-1", IVec2::ZERO, f32::vec2(0.0, 0.0)));
        levels.push(Level::new("1-2", IVec2::ZERO, f32::vec2(SCREEN_W, 0.0)));
        levels.push(Level::new(
            "1-3",
            IVec2::ZERO,
            f32::vec2(SCREEN_W * 2.0, 0.0),
        ));
        levels.push(Level::new("2-1", IVec2::ZERO, f32::vec2(0.0, SCREEN_H)));
        levels.push(Level::new(
            "2-2",
            IVec2::ZERO,
            f32::vec2(SCREEN_W, SCREEN_H),
        ));
        levels.push(Level::new(
            "2-3",
            IVec2::ZERO,
            f32::vec2(SCREEN_W * 2.0, SCREEN_H),
        ));
        levels.push(Level::new(
            "3-1",
            IVec2::ZERO,
            f32::vec2(0.0, SCREEN_H * 2.0),
        ));
        levels.push(Level::new(
            "3-2",
            IVec2::ZERO,
            f32::vec2(SCREEN_W, SCREEN_H * 2.0),
        ));
        levels.push(Level::new(
            "3-3",
            IVec2::ZERO,
            f32::vec2(SCREEN_W * 2.0, SCREEN_H * 2.0),
        ));

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
}

impl Level {
    pub fn new(name: &'static str, grid_tiles: IVec2, pos_world: f32::Vec2) -> Self {
        let is_setup = false;

        Self {
            name,
            grid_tiles,
            pos_world,

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
        self.is_setup = false;
    }
}
