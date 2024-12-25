use colorgrad::Gradient;
use eframe::{egui::Color32, epaint::Hsva};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use super::PolesPosition;

macro_rules! gradients_generator {
    {$(const $n:ident/$g:ident: $t:ident = &[ $($c:expr,)* ];)*} => {
        $(
            pub const $n: &[Color32] = &[ $($c,)* ];
            pub static $g: Lazy<colorgrad::$t> = Lazy::new(|| {
                let colors = $n.iter().map(|c| colorgrad::Color::from_rgba8(c.r(), c.g(), c.b(), c.a())).collect::<Vec<colorgrad::Color>>();
                let gradient = colorgrad::GradientBuilder::new()
                    .colors(&colors)
                    .build::<colorgrad::$t>()
                    .unwrap();
                gradient
            });
        )*
    };
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
pub enum ColorTheme {
    #[default]
    Purple,
    Rainbow,
    Sites,
    BadApple,
    Specky,
    Bob,
    Peter,
    Eric,
}

impl ColorTheme {
    pub fn to_color(self, disk_number: usize, disks_count: usize) -> Color32 {
        let modulo = |theme: &[Color32]| theme[(disk_number - 1) % theme.len()];
        let spread = |theme: &[Color32]| theme[(disk_number - 1) * theme.len() / disks_count];
        fn gradient(gradient: &impl Gradient, disk_number: usize, disks_count: usize) -> Color32 {
            let color = &gradient.colors(disks_count)[disk_number - 1];
            let [ r, g, b, _ ] = color.to_rgba8();
            Color32::from_rgb(r, g, b)
        }
        match self {
            ColorTheme::Rainbow => {
                let hsv = Hsva::new(disk_number as f32 / disks_count as f32, 1.0, 1.0, 1.0);
                let [ r, g, b ] = hsv.to_srgb();
                Color32::from_rgb(r, g, b)
            },
            ColorTheme::Purple => modulo(THEME_PURPLE_COLORS),
            ColorTheme::Sites => modulo(THEME_SITES_COLORS),
            ColorTheme::BadApple => modulo(THEME_BAD_APPLE_COLORS),
            ColorTheme::Specky => gradient(&*THEME_SPECKY_GRADIENT, disk_number, disks_count),
            ColorTheme::Bob => spread(THEME_BOB_COLORS),
            ColorTheme::Peter => spread(THEME_PETER_COLORS),
            ColorTheme::Eric => spread(THEME_ERIC_COLORS),
        }
    }
    pub fn to_emojis(self) -> (char, char, char) {
        match self {
            ColorTheme::Purple => ('🟪', '⬜', '🟪'),
            ColorTheme::Rainbow => ('🟩', '🟦', '🟥'),
            ColorTheme::Sites => ('🟩', '🟦', '🟧'),
            ColorTheme::BadApple => ('⬜', '🟫', '⬜'), // brown is so ugly :/
            ColorTheme::Specky => ('🟪', '⬜', '🟦'),
            ColorTheme::Bob => ('🟨', '⬜', '🟫'),
            ColorTheme::Peter => ('🟫', '⬜', '🟩'),
            ColorTheme::Eric => ('🟦', '⬜', '🟥'),
        }
    }
}

pub const THEME_PURPLE_COLORS: &[Color32] = &[
    Color32::from_rgb(212, 156, 234),
    Color32::from_rgb(134, 88, 154),
];

pub const THEME_SITES_COLORS: &[Color32] = &[
    Color32::from_rgb(170, 229, 164),   // rule
    Color32::from_rgb(1, 46, 87),       // e
    Color32::from_rgb(30, 30, 44),      // dan
    Color32::from_rgb(247, 152, 23),    // hub
];

pub const THEME_BAD_APPLE_COLORS: &[Color32] = &[
    Color32::from_rgb(255, 255, 255),
    Color32::from_rgb(0, 0, 0),
];

pub const THEME_BOB_COLORS: &[Color32] = &[
    Color32::from_rgb(255, 243, 96),
    Color32::from_rgb(255, 243, 96),
    Color32::from_rgb(255, 243, 96),
    Color32::from_rgb(255, 255, 255),
    Color32::from_rgb(180,122,61),
    
    // // tried to represent everything, looks bad
    // Color32::from_rgb(255, 243, 96),
    // Color32::from_rgb(126, 181, 232),
    // Color32::from_rgb(255, 243, 96),
    // Color32::from_rgb(255, 255, 255),
    // Color32::from_rgb(180, 122, 61),
    // Color32::from_rgb(255, 255, 255),
    // Color32::from_rgb(0, 0, 0),
];

pub const THEME_PETER_COLORS: &[Color32] = &[
    Color32::from_rgb(124, 51, 14),
    Color32::from_rgb(253, 184, 164),
    Color32::from_rgb(255, 255, 255),
    Color32::from_rgb(35, 98, 19),
    Color32::from_rgb(70, 29, 4),
];

pub const THEME_ERIC_COLORS: &[Color32] = &[
    Color32::from_rgb(255, 225, 29),
    Color32::from_rgb(0, 184, 196),
    Color32::from_rgb(255, 238, 195),
    Color32::from_rgb(238, 50, 83),
    Color32::from_rgb(132, 77, 56),
    Color32::from_rgb(48, 46, 60),
];

gradients_generator!{
    const THEME_SPECKY_COLORS/THEME_SPECKY_GRADIENT: CatmullRomGradient = &[
        Color32::from_rgb(255, 43, 254),
        Color32::from_rgb(254, 254, 254),
        Color32::from_rgb(114, 253, 255),
    ];
}

pub fn draw_share_tower(color_theme: ColorTheme, poles_position: PolesPosition) -> String {
    let b0 = '⬛';
    let (b1, b2, b3) = color_theme.to_emojis();

    let mut lines = [
        format!("{b0}{b0}{b1}{b0}{b0}"),
        format!("{b0}{b2}{b2}{b2}{b0}"),
        format!("{b3}{b3}{b3}{b3}{b3}"),
    ];

    if poles_position == PolesPosition::Top {
        lines.reverse();
    }

    lines.join("\n")
}
