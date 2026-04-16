use iced::mouse;
use iced::widget::canvas::{self, event, Canvas, Event, Frame, Geometry, Stroke, Text};
use iced::{alignment, Color, Element, Font, Length, Point, Rectangle, Renderer, Size, Theme};

use crate::JP_FONT_NAME;

use crate::ui::theme;

const CROP_SIZE: f32 = 252.0;

#[derive(Debug, Clone)]
pub enum CanvasMessage {
    SelectionMoved { x: f32, y: f32 },
}

#[derive(Default)]
pub struct CanvasState {
    is_dragging: bool,
    drag_offset_x: f32,
    drag_offset_y: f32,
}

pub struct CropCanvas {
    pub image_handle: Option<iced::widget::image::Handle>,
    pub image_width: u32,
    pub image_height: u32,
    pub selection_x: f32,
    pub selection_y: f32,
    pub is_file_hovering: bool,
}

impl CropCanvas {
    pub fn new() -> Self {
        Self {
            image_handle: None,
            image_width: 0,
            image_height: 0,
            selection_x: 0.0,
            selection_y: 0.0,
            is_file_hovering: false,
        }
    }

    pub fn set_image(&mut self, handle: iced::widget::image::Handle, width: u32, height: u32) {
        self.image_handle = Some(handle);
        self.image_width = width;
        self.image_height = height;
        self.selection_x = ((width as f32 - CROP_SIZE) / 2.0).max(0.0);
        self.selection_y = ((height as f32 - CROP_SIZE) / 2.0).max(0.0);
    }

    pub fn overlay_view(&self) -> Element<'_, CanvasMessage> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn placeholder_view(&self) -> Element<'_, CanvasMessage> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Calculate scale and offset to fit image within bounds (same as ContentFit::Contain)
    pub fn display_transform(&self, canvas_size: Size) -> (f32, Point) {
        if self.image_width == 0 || self.image_height == 0 {
            return (1.0, Point::ORIGIN);
        }
        let scale_x = canvas_size.width / self.image_width as f32;
        let scale_y = canvas_size.height / self.image_height as f32;
        let scale = scale_x.min(scale_y);

        let display_w = self.image_width as f32 * scale;
        let display_h = self.image_height as f32 * scale;
        let offset_x = (canvas_size.width - display_w) / 2.0;
        let offset_y = (canvas_size.height - display_h) / 2.0;

        (scale, Point::new(offset_x, offset_y))
    }

    fn selection_display_rect(&self, scale: f32, offset: Point) -> Rectangle {
        Rectangle::new(
            Point::new(
                offset.x + self.selection_x * scale,
                offset.y + self.selection_y * scale,
            ),
            Size::new(CROP_SIZE * scale, CROP_SIZE * scale),
        )
    }
}

impl canvas::Program<CanvasMessage> for CropCanvas {
    type State = CanvasState;

    fn update(
        &self,
        state: &mut CanvasState,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (event::Status, Option<CanvasMessage>) {
        if self.image_handle.is_none() {
            return (event::Status::Ignored, None);
        }

        let Some(cursor_pos) = cursor.position_in(bounds) else {
            if state.is_dragging {
                if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) = event {
                    state.is_dragging = false;
                }
            }
            return (event::Status::Ignored, None);
        };

        let (scale, offset) = self.display_transform(bounds.size());
        let sel_rect = self.selection_display_rect(scale, offset);

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if sel_rect.contains(cursor_pos) {
                    state.is_dragging = true;
                    state.drag_offset_x = cursor_pos.x - sel_rect.x;
                    state.drag_offset_y = cursor_pos.y - sel_rect.y;
                    (event::Status::Captured, None)
                } else {
                    let img_x = ((cursor_pos.x - offset.x) / scale - CROP_SIZE / 2.0).max(0.0);
                    let img_y = ((cursor_pos.y - offset.y) / scale - CROP_SIZE / 2.0).max(0.0);
                    let max_x = (self.image_width as f32 - CROP_SIZE).max(0.0);
                    let max_y = (self.image_height as f32 - CROP_SIZE).max(0.0);

                    state.is_dragging = true;
                    state.drag_offset_x = CROP_SIZE * scale / 2.0;
                    state.drag_offset_y = CROP_SIZE * scale / 2.0;

                    (
                        event::Status::Captured,
                        Some(CanvasMessage::SelectionMoved {
                            x: img_x.min(max_x),
                            y: img_y.min(max_y),
                        }),
                    )
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if state.is_dragging => {
                let img_x =
                    ((cursor_pos.x - state.drag_offset_x - offset.x) / scale).max(0.0);
                let img_y =
                    ((cursor_pos.y - state.drag_offset_y - offset.y) / scale).max(0.0);
                let max_x = (self.image_width as f32 - CROP_SIZE).max(0.0);
                let max_y = (self.image_height as f32 - CROP_SIZE).max(0.0);

                (
                    event::Status::Captured,
                    Some(CanvasMessage::SelectionMoved {
                        x: img_x.min(max_x),
                        y: img_y.min(max_y),
                    }),
                )
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.is_dragging = false;
                (event::Status::Captured, None)
            }
            _ => (event::Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        _state: &CanvasState,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        if self.image_handle.is_some() {
            // Overlay mode: transparent background, draw only selection UI
            let (scale, offset) = self.display_transform(bounds.size());
            let display_w = self.image_width as f32 * scale;
            let display_h = self.image_height as f32 * scale;
            let img_rect = Rectangle::new(offset, Size::new(display_w, display_h));

            let sel_rect = self.selection_display_rect(scale, offset);
            let dim = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.35 };

            // Dimming overlay (4 strips around selection)
            if sel_rect.y > img_rect.y {
                frame.fill_rectangle(
                    img_rect.position(),
                    Size::new(img_rect.width, sel_rect.y - img_rect.y),
                    dim,
                );
            }
            let sel_bottom = sel_rect.y + sel_rect.height;
            let img_bottom = img_rect.y + img_rect.height;
            if sel_bottom < img_bottom {
                frame.fill_rectangle(
                    Point::new(img_rect.x, sel_bottom),
                    Size::new(img_rect.width, img_bottom - sel_bottom),
                    dim,
                );
            }
            if sel_rect.x > img_rect.x {
                frame.fill_rectangle(
                    Point::new(img_rect.x, sel_rect.y),
                    Size::new(sel_rect.x - img_rect.x, sel_rect.height),
                    dim,
                );
            }
            let sel_right = sel_rect.x + sel_rect.width;
            let img_right = img_rect.x + img_rect.width;
            if sel_right < img_right {
                frame.fill_rectangle(
                    Point::new(sel_right, sel_rect.y),
                    Size::new(img_right - sel_right, sel_rect.height),
                    dim,
                );
            }

            // Selection border
            frame.stroke_rectangle(
                sel_rect.position(),
                sel_rect.size(),
                Stroke::default()
                    .with_color(theme::SELECTION)
                    .with_width(2.5),
            );

            // Corner handles
            let hs = 8.0;
            let corners = [
                sel_rect.position(),
                Point::new(sel_rect.x + sel_rect.width - hs, sel_rect.y),
                Point::new(sel_rect.x, sel_rect.y + sel_rect.height - hs),
                Point::new(sel_rect.x + sel_rect.width - hs, sel_rect.y + sel_rect.height - hs),
            ];
            for corner in &corners {
                frame.fill_rectangle(*corner, Size::new(hs, hs), theme::SELECTION);
            }
        } else {
            // Placeholder mode
            frame.fill_rectangle(Point::ORIGIN, bounds.size(), theme::CANVAS_BG);

            frame.stroke_rectangle(
                Point::new(20.0, 20.0),
                Size::new(bounds.width - 40.0, bounds.height - 40.0),
                Stroke::default()
                    .with_color(theme::BORDER)
                    .with_width(2.0),
            );

            let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
            frame.fill_text(Text {
                content: "PNG ファイルをドラッグ＆ドロップ\nまたは「PNG を開く」ボタンで読み込み".to_string(),
                position: center,
                color: theme::TEXT_MUTED,
                size: 18.0.into(),
                font: Font::with_name(JP_FONT_NAME),
                horizontal_alignment: alignment::Horizontal::Center,
                vertical_alignment: alignment::Vertical::Center,
                ..Default::default()
            });
        }

        // File drag-over feedback
        if self.is_file_hovering {
            let accent_transparent = Color { a: 0.15, ..theme::ACCENT };
            frame.fill_rectangle(Point::ORIGIN, bounds.size(), accent_transparent);
            frame.stroke_rectangle(
                Point::new(4.0, 4.0),
                Size::new(bounds.width - 8.0, bounds.height - 8.0),
                Stroke::default()
                    .with_color(theme::ACCENT)
                    .with_width(3.0),
            );
            frame.fill_text(Text {
                content: "ここにドロップ".to_string(),
                position: Point::new(bounds.width / 2.0, bounds.height - 40.0),
                color: theme::ACCENT,
                size: 22.0.into(),
                font: Font::with_name(JP_FONT_NAME),
                horizontal_alignment: alignment::Horizontal::Center,
                vertical_alignment: alignment::Vertical::Center,
                ..Default::default()
            });
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &CanvasState,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if self.image_handle.is_none() {
            return mouse::Interaction::default();
        }

        if state.is_dragging {
            return mouse::Interaction::Grabbing;
        }

        if let Some(cursor_pos) = cursor.position_in(bounds) {
            let (scale, offset) = self.display_transform(bounds.size());
            let sel_rect = self.selection_display_rect(scale, offset);
            if sel_rect.contains(cursor_pos) {
                return mouse::Interaction::Grab;
            }
        }

        mouse::Interaction::Crosshair
    }
}
