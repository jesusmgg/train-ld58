use macroquad::color::Color;

#[derive(Clone)]
pub struct Styles {
    pub colors: Colors,
}

impl Styles {
    pub fn new() -> Self {
        let colors = Colors {
            // Backgrounds - Light & Clean
            bg_light: Color::from_hex(0xf8f9fa),
            bg_cream: Color::from_hex(0xfff9db),

            // Greens - Recycling/Eco theme
            green_1: Color::from_hex(0xd3f9d8),
            green_2: Color::from_hex(0x8ce99a),
            green_3: Color::from_hex(0x51cf66),
            green_4: Color::from_hex(0x2f9e44),

            // Blues - Clean/Sky
            blue_1: Color::from_hex(0xd0ebff),
            blue_2: Color::from_hex(0x74c0fc),
            blue_3: Color::from_hex(0x339af0),

            // Yellows - Garbage/Warning
            yellow_1: Color::from_hex(0xfff3bf),
            yellow_2: Color::from_hex(0xffe066),
            yellow_3: Color::from_hex(0xfcc419),

            // Oranges - Garbage/Alerts
            orange_1: Color::from_hex(0xffe8cc),
            orange_2: Color::from_hex(0xffa94d),
            orange_3: Color::from_hex(0xfd7e14),

            // Browns - Train/Tracks/Dirt
            brown_1: Color::from_hex(0xe7d7c1),
            brown_2: Color::from_hex(0xa47551),
            brown_3: Color::from_hex(0x7a5438),

            // Grays - Tracks/Metal/UI
            gray_1: Color::from_hex(0xe9ecef),
            gray_2: Color::from_hex(0xadb5bd),
            gray_3: Color::from_hex(0x495057),

            // Accents
            purple: Color::from_hex(0xcc5de8),
            red: Color::from_hex(0xff6b6b),
            white: Color::from_hex(0xffffff),
        };

        Self { colors }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Colors {
    // Backgrounds - Light & Clean
    pub bg_light: Color,
    pub bg_cream: Color,

    // Greens - Recycling/Eco theme
    pub green_1: Color,
    pub green_2: Color,
    pub green_3: Color,
    pub green_4: Color,

    // Blues - Clean/Sky
    pub blue_1: Color,
    pub blue_2: Color,
    pub blue_3: Color,

    // Yellows - Garbage/Warning
    pub yellow_1: Color,
    pub yellow_2: Color,
    pub yellow_3: Color,

    // Oranges - Garbage/Alerts
    pub orange_1: Color,
    pub orange_2: Color,
    pub orange_3: Color,

    // Browns - Train/Tracks/Dirt
    pub brown_1: Color,
    pub brown_2: Color,
    pub brown_3: Color,

    // Grays - Tracks/Metal/UI
    pub gray_1: Color,
    pub gray_2: Color,
    pub gray_3: Color,

    // Accents
    pub purple: Color,
    pub red: Color,
    pub white: Color,
}
