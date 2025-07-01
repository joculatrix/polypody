use std::path::PathBuf;

use crate::internal::library::Library;
use super::{ control_button, Icon };
use iced::{ widget::{ column, horizontal_space, row, text, text_input, vertical_space }, Task };

pub struct StartScreen {
    error: bool,
    pub lib: Option<Library>,
    path: String,
    scanning: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenDialog,
    PathChanged(String),
    Scan,
    Error,
    Selected(Option<rfd::FileHandle>),
    Lib(Library),
    Done,
}

impl StartScreen {
    pub fn new() -> Self {
        Self {
            error: false,
            lib: None,
            path: String::from(""),
            scanning: false,
        }
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::OpenDialog => {
                Task::perform(
                    rfd::AsyncFileDialog::new()
                        .set_directory("/")
                        .pick_folder(),
                    Message::Selected,
                )
            }
            Message::PathChanged(s) => {
                self.path = s;
                Task::none()
            }
            Message::Scan => {
                let path = PathBuf::from(&self.path);
                match path.try_exists() {
                    Ok(true) => {
                        self.scanning = true;
                        Task::done(Message::Lib(crate::internal::scan(path)))
                    }
                    Ok(false) => {
                        eprintln!("Given library path does not exist!");
                        Task::done(Message::Error)
                    }
                    Err(e) => {
                        eprintln!("Error verifying existence of path: {e}");
                        return Task::done(Message::Error);
                    }
                }
            }
            Message::Error => {
                self.error = true;
                Task::none()
            }
            Message::Selected(fh) => {
                let Some(fh) = fh else {
                    return Task::done(Message::Error);
                };
                self.path = fh.path().to_str().unwrap().to_owned();
                Task::none()
            }
            Message::Lib(lib) => {
                self.lib = Some(lib);
                Task::done(Message::Done)
            }
            Message::Done => Task::none(),
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        if self.scanning {
            text!("Scanning...")
                .size(super::TEXT_SIZE)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .center()
                .into()
        } else {
            column![
                vertical_space(),
                text!("Select a directory to scan for music.")
                    .size(super::TEXT_SIZE)
                    .width(iced::Length::Fill)
                    .center(),
                horizontal_space().height(8),
                text!("{}", if self.error {
                    "A problem occurred reading the given directory. Please retry."
                } else { 
                    ""
                })
                    .size(super::SMALL_TEXT_SIZE)
                    .width(iced::Length::Fill)
                    .color(iced::theme::Theme::Ferra.palette().danger)
                    .center(),
                horizontal_space().height(8),
                row![
                    horizontal_space(),
                    control_button!(
                        icon: Icon::Folder,
                        msg: Message::OpenDialog,
                        style: super::style::plain_icon_button,
                    ),
                    text_input("/", &self.path)
                        .on_input(Message::PathChanged),
                    control_button!(
                        icon: Icon::ArrowCornerDL,
                        msg: Message::Scan,
                        style: super::style::plain_icon_button,
                    ),
                    horizontal_space(),
                ],
                horizontal_space().height(20),
                row![
                    horizontal_space(),
                    text!("(If you've already set a library path and are still \
                        seeing this screen, then a problem may have occurred \
                        reading either your config file or the directory.)")
                        .size(super::SMALL_TEXT_SIZE)
                        .width(iced::Length::Fill)
                        .color(iced::theme::Theme::Ferra.palette().text.scale_alpha(0.6))
                        .center(),
                    horizontal_space(),
                ],
                vertical_space(),
            ]
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .into()
        }
    }
}
