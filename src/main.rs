mod app;
mod crop;
mod ui;

fn main() -> iced::Result {
    iced::application(app::App::title, app::App::update, app::App::view)
        .theme(app::App::theme)
        .window_size(iced::Size::new(900.0, 700.0))
        .run_with(app::App::new)
}
