mod app;
mod crop;
mod ui;

// Bundled Japanese font (IPAGothic). Ensures Japanese text renders correctly
// on macOS, Windows, and Linux without relying on system fonts.
const JP_FONT_BYTES: &[u8] = include_bytes!("../assets/ipag.ttf");
pub const JP_FONT_NAME: &str = "IPAGothic";

fn main() -> iced::Result {
    iced::application(app::App::title, app::App::update, app::App::view)
        .theme(app::App::theme)
        .window_size(iced::Size::new(900.0, 700.0))
        .font(JP_FONT_BYTES)
        .default_font(iced::Font::with_name(JP_FONT_NAME))
        .run_with(app::App::new)
}
