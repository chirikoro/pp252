use iced::widget::{button, container};
use iced::{border, color, Color, Theme};

// Pastel color palette
pub const BG: Color = color!(0xFFF5F0);
pub const CARD_BG: Color = color!(0xFFFFFF);
pub const ACCENT: Color = color!(0xFF9A76);
pub const ACCENT_PINK: Color = color!(0xFFACE4);
pub const SELECTION: Color = color!(0xFF6B6B);
pub const TEXT_COLOR: Color = color!(0x4A4A4A);
pub const TEXT_MUTED: Color = color!(0x999999);
pub const BORDER: Color = color!(0xFFD3B6);
pub const SUCCESS: Color = color!(0x4EC56B);
pub const ERROR: Color = color!(0xFF6B6B);
pub const CANVAS_BG: Color = color!(0xFAF0EB);

pub fn custom_theme() -> Theme {
    Theme::custom(
        "Pastel".to_string(),
        iced::theme::Palette {
            background: BG,
            text: TEXT_COLOR,
            primary: ACCENT,
            success: SUCCESS,
            danger: ERROR,
        },
    )
}

pub fn card_container(theme: &Theme) -> container::Style {
    let _ = theme;
    container::Style {
        background: Some(CARD_BG.into()),
        border: border::rounded(12).color(BORDER).width(1),
        ..Default::default()
    }
}

pub fn main_container(theme: &Theme) -> container::Style {
    let _ = theme;
    container::Style {
        background: Some(BG.into()),
        ..Default::default()
    }
}

pub fn primary_button(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    match status {
        button::Status::Active => button::Style {
            background: Some(ACCENT.into()),
            text_color: Color::WHITE,
            border: border::rounded(8),
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(ACCENT_PINK.into()),
            text_color: Color::WHITE,
            border: border::rounded(8),
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(SELECTION.into()),
            text_color: Color::WHITE,
            border: border::rounded(8),
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(Color { a: 0.3, ..ACCENT }.into()),
            text_color: Color { a: 0.5, ..Color::WHITE },
            border: border::rounded(8),
            ..Default::default()
        },
    }
}

pub fn save_button(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    match status {
        button::Status::Active => button::Style {
            background: Some(ACCENT_PINK.into()),
            text_color: TEXT_COLOR,
            border: border::rounded(8),
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(ACCENT.into()),
            text_color: Color::WHITE,
            border: border::rounded(8),
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(SELECTION.into()),
            text_color: Color::WHITE,
            border: border::rounded(8),
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(Color { a: 0.3, ..ACCENT_PINK }.into()),
            text_color: Color { a: 0.5, ..TEXT_COLOR },
            border: border::rounded(8),
            ..Default::default()
        },
    }
}

pub fn title_text<'a>(content: impl ToString) -> iced::widget::Text<'a> {
    iced::widget::text(content.to_string())
        .size(28)
        .color(ACCENT)
}

pub fn info_text<'a>(content: impl ToString) -> iced::widget::Text<'a> {
    iced::widget::text(content.to_string())
        .color(TEXT_MUTED)
        .size(14)
}

pub fn status_text<'a>(content: impl ToString, is_error: bool) -> iced::widget::Text<'a> {
    let c = if is_error { ERROR } else { TEXT_COLOR };
    iced::widget::text(content.to_string()).color(c).size(14)
}
