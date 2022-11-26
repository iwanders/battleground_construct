pub trait ColorConvert {
    fn to_color(&self) -> three_d::Color;
}

impl ColorConvert for battleground_construct::display::Color {
    fn to_color(&self) -> three_d::Color {
        three_d::Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
}
