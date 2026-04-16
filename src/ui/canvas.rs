use iced::mouse;
use iced::widget::canvas::{self, event, Canvas, Event, Frame, Geometry, Image as CanvasImage, Stroke, Text};
use iced::{alignment, Color, Element, Font, Length, Point, Rectangle, Renderer, Size, Theme};

use crate::JP_FONT_NAME;

use crate::ui::theme;

const CROP_SIZE: f32 = 252.0;
const MIN_ZOOM: f32 = 0.5;
const MAX_ZOOM: f32 = 10.0;

#[derive(Debug, Clone)]
pub enum CanvasMessage {
    SelectionMoved { x: f32, y: f32 },
    ZoomChanged { zoom: f32, pan_x: f32, pan_y: f32 },
    Panned { dx: f32, dy: f32 },
}

#[derive(Default)]
pub struct CanvasState {
    is_dragging: bool,
    drag_offset_x: f32,
    drag_offset_y: f32,
    is_panning: bool,
    pan_start_x: f32,
    pan_start_y: f32,
}

pub struct CropCanvas {
    pub image_handle: Option<iced::widget::image::Handle>,
    pub image_width: u32,
    pub image_height: u32,
    pub selection_x: f32,
    pub selection_y: f32,
    pub is_file_hovering: bool,
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
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
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
        }
    }

    pub fn set_image(&mut self, handle: iced::widget::image::Handle, width: u32, height: u32) {
        self.image_handle = Some(handle);
        self.image_width = width;
        self.image_height = height;
        self.selection_x = ((width as f32 - CROP_SIZE) / 2.0).max(0.0);
        self.selection_y = ((height as f32 - CROP_SIZE) / 2.0).max(0.0);
        self.zoom = 1.0;
        self.pan_x = 0.0;
        self.pan_y = 0.0;
    }

    pub fn image_view(&self) -> Element<'_, CanvasMessage> {
        Canvas::new(ImageLayer { canvas: self })
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn overlay_view(&self) -> Element<'_, CanvasMessage> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn placeholder_view(&self) -> Element<'_, CanvasMessage> {
        Canvas::new(PlaceholderLayer {
            is_file_hovering: self.is_file_hovering,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub fn display_transform(&self, canvas_size: Size) -> (f32, Point) {
        if self.image_width == 0 || self.image_height == 0 {
            return (1.0, Point::ORIGIN);
        }
        let scale_x = canvas_size.width / self.image_width as f32;
        let scale_y = canvas_size.height / self.image_height as f32;
        let fit_scale = scale_x.min(scale_y);
        let scale = fit_scale * self.zoom;

        let display_w = self.image_width as f32 * scale;
        let display_h = self.image_height as f32 * scale;
        let offset_x = (canvas_size.width - display_w) / 2.0 + self.pan_x;
        let offset_y = (canvas_size.height - display_h) / 2.0 + self.pan_y;

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

// --- Image layer: draws only the image (bottom of stack) ---

struct ImageLayer<'a> {
    canvas: &'a CropCanvas,
}

impl canvas::Program<CanvasMessage> for ImageLayer<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), theme::CANVAS_BG);

        if let Some(ref handle) = self.canvas.image_handle {
            let (scale, offset) = self.canvas.display_transform(bounds.size());
            let display_w = self.canvas.image_width as f32 * scale;
            let display_h = self.canvas.image_height as f32 * scale;
            let img_rect = Rectangle::new(offset, Size::new(display_w, display_h));
            frame.draw_image(img_rect, CanvasImage::new(handle.clone()));
        }

        vec![frame.into_geometry()]
    }
}

// --- Placeholder layer: shown when no image is loaded ---

struct PlaceholderLayer {
    is_file_hovering: bool,
}

impl canvas::Program<CanvasMessage> for PlaceholderLayer {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
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
            content: "PNG ファイルをドラッグ＆ドロップ\nまたは「PNG を開く」ボタンで読み込み"
                .to_string(),
            position: center,
            color: theme::TEXT_MUTED,
            size: 18.0.into(),
            font: Font::with_name(JP_FONT_NAME),
            horizontal_alignment: alignment::Horizontal::Center,
            vertical_alignment: alignment::Vertical::Center,
            ..Default::default()
        });

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
}

// --- Overlay layer: selection UI + event handling (top of stack) ---

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

        // Handle mouse release even when cursor is outside bounds
        match &event {
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.is_dragging => {
                state.is_dragging = false;
                return (event::Status::Captured, None);
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right)) if state.is_panning => {
                state.is_panning = false;
                return (event::Status::Captured, None);
            }
            _ => {}
        }

        let Some(cursor_pos) = cursor.position_in(bounds) else {
            return (event::Status::Ignored, None);
        };

        let (scale, offset) = self.display_transform(bounds.size());
        let sel_rect = self.selection_display_rect(scale, offset);

        match event {
            // Scroll wheel: zoom
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let scroll_y = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => y,
                    mouse::ScrollDelta::Pixels { y, .. } => y / 100.0,
                };
                let factor = 1.0 + scroll_y * 0.15;
                let new_zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);

                // Zoom towards cursor: adjust pan so cursor stays over the same image point
                let img_x = (cursor_pos.x - offset.x) / scale;
                let img_y = (cursor_pos.y - offset.y) / scale;

                let fit_scale_x = bounds.width / self.image_width as f32;
                let fit_scale_y = bounds.height / self.image_height as f32;
                let fit_scale = fit_scale_x.min(fit_scale_y);
                let new_scale = fit_scale * new_zoom;

                let new_display_w = self.image_width as f32 * new_scale;
                let new_display_h = self.image_height as f32 * new_scale;
                let new_center_x = (bounds.width - new_display_w) / 2.0;
                let new_center_y = (bounds.height - new_display_h) / 2.0;

                let new_pan_x = cursor_pos.x - img_x * new_scale - new_center_x;
                let new_pan_y = cursor_pos.y - img_y * new_scale - new_center_y;

                (
                    event::Status::Captured,
                    Some(CanvasMessage::ZoomChanged {
                        zoom: new_zoom,
                        pan_x: new_pan_x,
                        pan_y: new_pan_y,
                    }),
                )
            }

            // Right-click: start panning
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                state.is_panning = true;
                state.pan_start_x = cursor_pos.x;
                state.pan_start_y = cursor_pos.y;
                (event::Status::Captured, None)
            }

            // Mouse move while panning
            Event::Mouse(mouse::Event::CursorMoved { .. }) if state.is_panning => {
                let dx = cursor_pos.x - state.pan_start_x;
                let dy = cursor_pos.y - state.pan_start_y;
                state.pan_start_x = cursor_pos.x;
                state.pan_start_y = cursor_pos.y;
                (
                    event::Status::Captured,
                    Some(CanvasMessage::Panned { dx, dy }),
                )
            }

            // Left-click: selection drag
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
                let img_x = ((cursor_pos.x - state.drag_offset_x - offset.x) / scale).max(0.0);
                let img_y = ((cursor_pos.y - state.drag_offset_y - offset.y) / scale).max(0.0);
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
            let (scale, offset) = self.display_transform(bounds.size());
            let display_w = self.image_width as f32 * scale;
            let display_h = self.image_height as f32 * scale;

            // Clamp image rect to visible canvas area for dimming
            let vis_x = offset.x.max(0.0);
            let vis_y = offset.y.max(0.0);
            let vis_right = (offset.x + display_w).min(bounds.width);
            let vis_bottom = (offset.y + display_h).min(bounds.height);
            let img_rect = Rectangle::new(
                Point::new(vis_x, vis_y),
                Size::new((vis_right - vis_x).max(0.0), (vis_bottom - vis_y).max(0.0)),
            );

            let sel_rect = self.selection_display_rect(scale, offset);
            let dim = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.35 };

            // Clamp selection rect to visible area for dimming strips
            let sel_x = sel_rect.x.max(vis_x);
            let sel_y_top = sel_rect.y.max(vis_y);
            let sel_right = (sel_rect.x + sel_rect.width).min(vis_right);
            let sel_bottom = (sel_rect.y + sel_rect.height).min(vis_bottom);

            // Top strip
            if sel_y_top > vis_y {
                frame.fill_rectangle(
                    Point::new(vis_x, vis_y),
                    Size::new(img_rect.width, sel_y_top - vis_y),
                    dim,
                );
            }
            // Bottom strip
            if sel_bottom < vis_bottom {
                frame.fill_rectangle(
                    Point::new(vis_x, sel_bottom),
                    Size::new(img_rect.width, vis_bottom - sel_bottom),
                    dim,
                );
            }
            // Left strip
            if sel_x > vis_x {
                frame.fill_rectangle(
                    Point::new(vis_x, sel_y_top),
                    Size::new(sel_x - vis_x, (sel_bottom - sel_y_top).max(0.0)),
                    dim,
                );
            }
            // Right strip
            if sel_right < vis_right {
                frame.fill_rectangle(
                    Point::new(sel_right, sel_y_top),
                    Size::new(vis_right - sel_right, (sel_bottom - sel_y_top).max(0.0)),
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
                Point::new(
                    sel_rect.x + sel_rect.width - hs,
                    sel_rect.y + sel_rect.height - hs,
                ),
            ];
            for corner in &corners {
                frame.fill_rectangle(*corner, Size::new(hs, hs), theme::SELECTION);
            }
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

        if state.is_panning {
            return mouse::Interaction::Grabbing;
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
