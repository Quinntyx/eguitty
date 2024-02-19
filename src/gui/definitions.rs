pub mod color {
    use egui::Color32;

    pub const BG_PRIMARY: Color32 = Color32::from_rgb(49, 46, 63);
    pub const BG_LIGHT: Color32 = Color32::from_rgb(63, 58, 83);
    pub const BG_DARK: Color32 = Color32::from_rgb(41, 38, 55);

    pub const ACCENT_RED: Color32 = Color32::from_rgb(246, 65, 96);
    pub const ACCENT_GREEN: Color32 = Color32::from_rgb(17, 168, 122);
    pub const ACCENT_BLUE: Color32 = Color32::from_rgb(72, 103, 232);
    pub const ACCENT_YELLOW: Color32 = Color32::from_rgb(229, 127, 69);
    
    pub const FG_DARK: Color32 = Color32::from_rgb(109, 106, 120);
    pub const FG_PRIMARY: Color32 = Color32::from_rgb(171, 168, 217);
}

pub mod font {
    use egui::{FontFamily, FontId, RichText};
    use super::color;

    pub const FG_MONO: FontId = FontId::new(16., FontFamily::Monospace);

    pub fn fg_mono_rt (val: &str) -> RichText {
        RichText::new(val).font(FG_MONO).color(color::FG_PRIMARY)
    }
}