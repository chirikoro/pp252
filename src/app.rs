use std::path::PathBuf;

use iced::widget::{button, column, container, horizontal_space, row, stack, text};
use iced::{window, Element, Length, Subscription, Task, Theme};

use crate::crop;
use crate::ui::canvas::{CanvasMessage, CropCanvas};
use crate::ui::theme;

#[derive(Debug, Clone)]
pub enum Message {
    OpenFile,
    FileSelected(Option<PathBuf>),
    ImageLoaded(Result<ImageInfo, String>),
    Canvas(CanvasMessage),
    CropAndSave,
    FileSaved(Result<PathBuf, String>),
    FileHovering(bool),
}

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub rgba_data: Vec<u8>,
    pub path: PathBuf,
}

pub enum Status {
    Idle,
    Loading,
    Loaded,
    Saving,
    Saved(PathBuf),
    Error(String),
}

pub struct App {
    pub canvas: CropCanvas,
    image: Option<image::DynamicImage>,
    image_dimensions: Option<(u32, u32)>,
    status: Status,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                canvas: CropCanvas::new(),
                image: None,
                image_dimensions: None,
                status: Status::Idle,
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        "PP252 - PNG Cropper".to_string()
    }

    pub fn theme(&self) -> Theme {
        theme::custom_theme()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::event::listen_with(|event, _status, _id| match event {
            iced::Event::Window(window::Event::FileDropped(path)) => {
                if path
                    .extension()
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("png"))
                {
                    Some(Message::FileSelected(Some(path)))
                } else {
                    Some(Message::ImageLoaded(Err(
                        "PNG ファイルのみ対応しています".to_string(),
                    )))
                }
            }
            iced::Event::Window(window::Event::FileHovered(_)) => Some(Message::FileHovering(true)),
            iced::Event::Window(window::Event::FilesHoveredLeft) => {
                Some(Message::FileHovering(false))
            }
            _ => None,
        })
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFile => {
                self.status = Status::Loading;
                Task::perform(
                    async {
                        let file = rfd::AsyncFileDialog::new()
                            .add_filter("PNG", &["png"])
                            .pick_file()
                            .await;
                        file.map(|f| f.path().to_path_buf())
                    },
                    Message::FileSelected,
                )
            }

            Message::FileSelected(path) => {
                self.canvas.is_file_hovering = false;
                match path {
                    Some(p) => {
                        self.status = Status::Loading;
                        let path = p.clone();
                        Task::perform(
                            async move {
                                let loaded = crop::load_image(&path)?;
                                Ok(ImageInfo {
                                    width: loaded.width,
                                    height: loaded.height,
                                    rgba_data: loaded.rgba_data,
                                    path,
                                })
                            },
                            Message::ImageLoaded,
                        )
                    }
                    None => {
                        self.status = Status::Idle;
                        Task::none()
                    }
                }
            }

            Message::ImageLoaded(result) => {
                match result {
                    Ok(info) => {
                        let handle = iced::widget::image::Handle::from_rgba(
                            info.width,
                            info.height,
                            info.rgba_data,
                        );
                        self.canvas.set_image(handle, info.width, info.height);
                        self.image_dimensions = Some((info.width, info.height));
                        self.status = Status::Loaded;

                        if let Ok(reader) = image::ImageReader::open(&info.path) {
                            if let Ok(img) = reader.decode() {
                                self.image = Some(img);
                            }
                        }
                        Task::none()
                    }
                    Err(e) => {
                        self.status = Status::Error(e);
                        Task::none()
                    }
                }
            }

            Message::Canvas(canvas_msg) => {
                match canvas_msg {
                    CanvasMessage::SelectionMoved { x, y } => {
                        self.canvas.selection_x = x;
                        self.canvas.selection_y = y;
                    }
                    CanvasMessage::ZoomChanged { zoom, pan_x, pan_y } => {
                        self.canvas.zoom = zoom;
                        self.canvas.pan_x = pan_x;
                        self.canvas.pan_y = pan_y;
                    }
                    CanvasMessage::Panned { dx, dy } => {
                        self.canvas.pan_x += dx;
                        self.canvas.pan_y += dy;
                    }
                }
                Task::none()
            }

            Message::CropAndSave => {
                if let Some(ref img) = self.image {
                    self.status = Status::Saving;
                    let x = self.canvas.selection_x as u32;
                    let y = self.canvas.selection_y as u32;

                    match crop::crop_and_encode(img, x, y) {
                        Ok(png_bytes) => Task::perform(
                            async move {
                                let file = rfd::AsyncFileDialog::new()
                                    .add_filter("PNG", &["png"])
                                    .set_file_name("cropped_252x252.png")
                                    .save_file()
                                    .await;
                                match file {
                                    Some(f) => {
                                        let path = f.path().to_path_buf();
                                        std::fs::write(&path, &png_bytes)
                                            .map(|_| path)
                                            .map_err(|e| e.to_string())
                                    }
                                    None => Err("キャンセルされました".to_string()),
                                }
                            },
                            Message::FileSaved,
                        ),
                        Err(e) => {
                            self.status = Status::Error(e);
                            Task::none()
                        }
                    }
                } else {
                    Task::none()
                }
            }

            Message::FileSaved(result) => {
                match result {
                    Ok(path) => self.status = Status::Saved(path),
                    Err(e) => self.status = Status::Error(e),
                }
                Task::none()
            }

            Message::FileHovering(hovering) => {
                self.canvas.is_file_hovering = hovering;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let title = theme::title_text("PP252");

        let canvas_content: Element<'_, Message> =
            if self.canvas.image_handle.is_some() {
                let image_layer = self.canvas.image_view().map(Message::Canvas);
                let overlay = self.canvas.overlay_view().map(Message::Canvas);

                stack![image_layer, overlay]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            } else {
                self.canvas.placeholder_view().map(Message::Canvas)
            };

        let canvas_area = container(canvas_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(2)
            .style(theme::card_container);

        // Info row
        let info = match self.image_dimensions {
            Some((w, h)) => {
                let sel_x = self.canvas.selection_x as u32;
                let sel_y = self.canvas.selection_y as u32;
                let zoom_pct = (self.canvas.zoom * 100.0) as u32;
                row![
                    theme::info_text(format!("画像サイズ: {w} x {h}")),
                    horizontal_space(),
                    theme::info_text(format!("選択位置: ({sel_x}, {sel_y})")),
                    horizontal_space(),
                    theme::info_text(format!("ズーム: {zoom_pct}%")),
                    horizontal_space(),
                    theme::info_text(format!("クロップサイズ: 252 x 252")),
                ]
                .spacing(10)
            }
            None => row![theme::info_text(
                "PNG ファイルをドラッグ＆ドロップ、またはボタンで読み込み"
            )],
        };

        // Buttons
        let open_btn = button(text("  PNG を開く  ").size(16))
            .style(theme::primary_button)
            .on_press(Message::OpenFile);

        let save_btn = button(text("  クロップして保存  ").size(16)).style(theme::save_button);

        let save_btn = if self.image.is_some() {
            save_btn.on_press(Message::CropAndSave)
        } else {
            save_btn
        };

        let buttons = row![open_btn, horizontal_space(), save_btn]
            .spacing(20)
            .width(Length::Fill);

        // Status bar
        let status = match &self.status {
            Status::Idle => theme::status_text("準備完了", false),
            Status::Loading => theme::status_text("読み込み中...", false),
            Status::Loaded => theme::status_text(
                "画像を読み込みました。選択枠をドラッグして範囲を調整してください。",
                false,
            ),
            Status::Saving => theme::status_text("保存中...", false),
            Status::Saved(path) => {
                theme::status_text(format!("保存しました: {}", path.display()), false)
            }
            Status::Error(e) => theme::status_text(e, true),
        };

        let content = column![title, canvas_area, info, buttons, status]
            .spacing(12)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::main_container)
            .into()
    }
}
