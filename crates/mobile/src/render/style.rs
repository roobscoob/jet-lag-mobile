use jet_lag_core::shape::types::Centimeters;

pub enum Size {
    WorldSpace(Centimeters),
    ScreenSpace { pixels: f32 },
}

pub enum Pattern {
    SolidColor(palette::Srgba<f32>),
    Stripes {
        color1: palette::Srgba<f32>,
        color2: palette::Srgba<f32>,
        stripe_width_1: Size,
        stripe_width_2: Size,
        rotation_degrees: f32,
    },
}

pub struct Style {
    borderColor: palette::Srgba<f32>,
    borderWidth: f32,
    fill: Option<Pattern>,
}
