use macroquad::color::Color;
use serde::Deserialize;

pub const VIRTUAL_PIXEL_FACTOR: f32 = 4.0;
pub const VIRTUAL_SCREEN_WIDTH: f32 = 160.0 * VIRTUAL_PIXEL_FACTOR;
pub const VIRTUAL_SCREEN_HEIGHT: f32 = 144.0 * VIRTUAL_PIXEL_FACTOR;

/* Colors */
pub const COLOR_1: Color = Color {
  r: 224.0 / 255.0,
  g: 248.0 / 255.0,
  b: 208.0 / 255.0,
  a: 1.0,
};
pub const COLOR_2: Color = Color {
  r: 136.0 / 255.0,
  g: 192.0 / 255.0,
  b: 112.0 / 255.0,
  a: 1.0,
};
pub const COLOR_3: Color = Color {
  r: 52.0 / 255.0,
  g: 104.0 / 255.0,
  b: 86.0 / 255.0,
  a: 1.0,
};
pub const COLOR_4: Color = Color {
  r: 8.0 / 255.0,
  g: 24.0 / 255.0,
  b: 32.0 / 255.0,
  a: 1.0,
};

#[derive(Clone)]
pub struct ColorPalette {
  pub color_1: Color,
  pub color_2: Color,
  pub color_3: Color,
  pub color_4: Color,
}

pub const BASE_COLORS: ColorPalette = ColorPalette {
  color_1: COLOR_1,
  color_2: COLOR_2,
  color_3: COLOR_3,
  color_4: COLOR_4,
};

pub const GRAYSCALE_COLORS: ColorPalette = ColorPalette {
  color_1: Color {
    r: 248.0 / 255.0,
    g: 248.0 / 255.0,
    b: 248.0 / 255.0,
    a: 1.0,
  },
  color_2: Color {
    r: 168.0 / 255.0,
    g: 168.0 / 255.0,
    b: 168.0 / 255.0,
    a: 1.0,
  },
  color_3: Color {
    r: 96.0 / 255.0,
    g: 96.0 / 255.0,
    b: 96.0 / 255.0,
    a: 1.0,
  },
  color_4: Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
  },
};

pub const SUPER_GREEN_COLORS: ColorPalette = ColorPalette {
  color_1: Color {
    r: 155.0 / 255.0,
    g: 188.0 / 255.0,
    b: 15.0 / 255.0,
    a: 1.0,
  },
  color_2: Color {
    r: 139.0 / 255.0,
    g: 172.0 / 255.0,
    b: 15.0 / 255.0,
    a: 1.0,
  },
  color_3: Color {
    r: 48.0 / 255.0,
    g: 98.0 / 255.0,
    b: 48.0 / 255.0,
    a: 1.0,
  },
  color_4: Color {
    r: 15.0 / 255.0,
    g: 56.0 / 255.0,
    b: 15.0 / 255.0,
    a: 1.0,
  },
};

pub const TEXT_FONT_FOREGROUND_COLOR: Color = Color {
  r: 224.0 / 255.0,
  g: 248.0 / 255.0,
  b: 207.0 / 255.0,
  a: 1.0,
};

pub const TEXT_FONT_BACKGROUND_COLOR: Color = Color {
  r: 0.0 / 255.0,
  g: 0.0 / 255.0,
  b: 0.0 / 255.0,
  a: 1.0,
};

#[derive(Deserialize, Clone, Copy)]
pub enum ColorPalettePresets {
  Default,
  Grayscale,
  SuperGreen,
}

impl ColorPalettePresets {
  pub fn to_color_palette(self) -> ColorPalette {
    match self {
      Self::Default => BASE_COLORS,
      Self::Grayscale => GRAYSCALE_COLORS,
      Self::SuperGreen => SUPER_GREEN_COLORS,
    }
  }
}
