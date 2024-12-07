use three_d::core::prelude::Srgba as Color;

pub trait ColorConvert {
    fn to_color(&self) -> Color;
}

impl ColorConvert for battleground_construct::display::Color {
    fn to_color(&self) -> Color {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
}
