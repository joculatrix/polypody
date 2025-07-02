use super::*;
use crate::internal::{ Directory, Track };

use iced::Font;
use iced::widget::{
    button, column, container, row, text, Button,
    scrollable, slider, Space, image,
};

use std::time::Duration;

mod controls;
mod library;
mod playlist;
pub mod start_screen;
mod style;

pub const ICON_FONT_BYTES: &[u8] = include_bytes!("../../../fonts/lucide.ttf");
const ICON_FONT: Font = Font::with_name("lucide");

const CONTROL_BUTTON_SIZE: u16 = 36;
const TEXT_SIZE: u16 = 14;
const SMALL_TEXT_SIZE: u16 = 12;

type Element<'a> = iced::Element<'a, Message>;

enum Icon {
    ArrowCornerDL,
    ArrowCornerDR,
    ArrowCornerLU,
    ChevronDown,
    ChevronUp,
    DiscAlbum,
    FileMusic,
    Folder,
    Pause,
    Pin,
    PinOff,
    Play,
    Plus,
    Queue,
    Repeat,
    Shuffle,
    SkipBack,
    SkipForward,
    Square,
    Trash,
    VolumeMute,
    VolumeLow,
    VolumeMid,
    VolumeHigh,
}

impl From<Icon> for char {
    fn from(value: Icon) -> char {
        match value {
            Icon::ArrowCornerDL =>  '\u{E0A5}',
            Icon::ArrowCornerDR =>  '\u{E0A6}',
            Icon::ArrowCornerLU =>  '\u{E0A8}',
            Icon::ChevronDown =>    '\u{E071}',
            Icon::ChevronUp =>      '\u{E074}',
            Icon::DiscAlbum =>      '\u{E561}',
            Icon::FileMusic =>      '\u{E563}',
            Icon::Folder =>         '\u{E0DB}',
            Icon::Pause =>          '\u{E132}',
            Icon::Pin =>            '\u{E259}',
            Icon::PinOff =>         '\u{E2B6}',
            Icon::Play =>           '\u{E140}',
            Icon::Plus =>           '\u{E141}',
            Icon::Queue =>          '\u{E2E0}',
            Icon::Repeat =>         '\u{E14A}',
            Icon::Shuffle =>        '\u{E162}',
            Icon::SkipBack =>       '\u{E163}',
            Icon::SkipForward =>    '\u{E164}',
            Icon::Square =>         '\u{E16B}',
            Icon::Trash =>          '\u{E18E}',
            Icon::VolumeMute =>     '\u{E1AC}',
            Icon::VolumeLow =>      '\u{E1A9}',
            Icon::VolumeMid =>      '\u{E1AA}',
            Icon::VolumeHigh =>     '\u{E1AB}',
        }
    }
}

fn icon_button<M>(icon: Icon, text_size: u16) -> Button<'static, M> {
    button(
        text!("{}", char::from(icon))
            .font(ICON_FONT)
            .size(text_size)
            .center()
    )
}

macro_rules! control_button {
    // optional argument labels for clarity:
    (icon: $icon:expr, msg: $msg:expr, style: $style:expr $(,)?) => {
        control_button!($icon, $msg, $style)
    };
    ($icon:expr, $msg:expr, $style:expr) => {
        crate::app::view::icon_button($icon, crate::app::view::CONTROL_BUTTON_SIZE / 2)
            .width(crate::app::view::CONTROL_BUTTON_SIZE)
            .height(crate::app::view::CONTROL_BUTTON_SIZE)
            .style($style)
            .on_press($msg)
    }
}

pub(self) use control_button;

macro_rules! fill {
    ($item:expr) => {
        container($item).width(iced::Length::Fill)
    }
}

pub(self) use fill;

impl App {
    pub fn view (&self) -> Element {
        if let Some(start) = &self.start_screen {
            start.view().map(|s_msg| Message::StartScreen(s_msg))
        } else {
            self.main_screen()
        }
    }

    fn main_screen(&self) -> Element {
        container(
            column![
                row![
                    self.sidebar(),
                    iced::widget::vertical_space().width(5),
                    match self.viewing {
                        Viewing::Library => self.library_view(),
                        Viewing::Playlist(None) => self.view_playlists(),
                        Viewing::Playlist(Some(id)) => self.playlist_view(id),
                    },
                    iced::widget::vertical_space().width(5),                    
                    self.queue_view(),
                ],
                self.current_track(),
                self.progress_bar(),
                self.control_bar(),
            ]
                .padding(3)
                .align_x(iced::alignment::Horizontal::Center)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
        )
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .padding(10)
            .into()
    }

    fn sidebar(&self) -> Element {
        let library_btn = button(
            row![
                text!("{}", char::from(Icon::DiscAlbum))
                    .font(ICON_FONT)
                    .size(CONTROL_BUTTON_SIZE / 2)
                    .align_y(iced::Alignment::Center),
                text(" Library")
                    .font(Font {
                        weight: iced::font::Weight::Bold,
                        ..Font::default()
                    })
                    .size(TEXT_SIZE)
                    .align_y(iced::Alignment::Center),
            ]
                .height(iced::Length::Fill)
                .align_y(iced::Alignment::Center),
        )
            .on_press(Message::ViewLibrary(self.library.root_dir))
            .width(iced::Length::Fill)
            .height(48)
            .style(style::outlined_button)
            .into();

        let playlists_btn = button(
            row![
                text!("{}", char::from(Icon::FileMusic))
                    .font(ICON_FONT)
                    .size(CONTROL_BUTTON_SIZE / 2)
                    .align_y(iced::Alignment::Center),
                text(" Playlists")
                    .font(Font {
                        weight: iced::font::Weight::Bold,
                        ..Font::default()
                    })
                    .size(TEXT_SIZE)
                    .align_y(iced::Alignment::Center),
            ]
                .height(iced::Length::Fill)
                .align_y(iced::Alignment::Center),
        )
            .on_press(Message::ViewPlaylist(None))
            .width(iced::Length::Fill)
            .height(48)
            .style(style::outlined_button)
            .into();

        let mut contents = vec![];
        contents.push(library_btn);
        self.config.library.pins
            .iter()
            .enumerate()
            .for_each(|(i, dir)| {
                let name = dir.file_name().unwrap().to_str().unwrap().to_owned();
                contents.push(Self::sidebar_item(
                    name,
                    Message::ViewLibrary(crate::internal::library::path_hash(dir)),
                    PinKind::Library,
                    i,
                ));
            });
        contents.push(playlists_btn);
        self.config.playlists.pins
            .iter()
            .enumerate()
            .for_each(|(i, pl)| {
                let id = crate::internal::library::path_hash(pl);
                let pl = self.playlists.get_playlist(id).unwrap();
                contents.push(Self::sidebar_item(
                    pl.title.clone(),
                    Message::ViewPlaylist(Some(id)),
                    PinKind::Playlist,
                    i,
                ));
            });

        container(
            scrollable(
                column(contents)
            )
        )
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();

                container::Style {
                    border: iced::Border {
                        color: palette.background.base.text.scale_alpha(0.2),
                        width: 1.0,
                        radius: (2.0).into(),
                    },
                    ..container::Style::default()
                }
            })
            .padding(1)
            .width(iced::Length::FillPortion(3))
            .height(iced::Length::Fill)
            .into()
    }

    fn sidebar_item(
        txt: String,
        msg: Message,
        pin_kind: PinKind,
        num: usize
    ) -> Element<'static> {
        iced::widget::hover(
            container(
                button(
                    text(txt)
                        .size(TEXT_SIZE)
                        .align_y(iced::Alignment::Center)
                        .height(iced::Length::Fill)
                )
                    .style(|theme: &iced::Theme, status: button::Status| {
                        let palette = theme.extended_palette();
                        button::Style {
                            text_color: palette.background.base.text,
                            background: match status {
                                button::Status::Hovered | button::Status::Pressed =>
                                    iced::Color::parse("#242226").map(|c| c.into()),
                                _ => Some(palette.background.base.color.into()),
                            },
                            ..button::Style::default()
                        }
                    })
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill)
                    .on_press(msg)
            )
                .width(iced::Length::Fill)
                .height(42),
            container(
                row![
                    column![
                        icon_button(Icon::ChevronUp, 12)
                            .on_press(Message::PinSwap(
                                pin_kind,
                                num,
                                num.saturating_sub(1)
                            ))
                            .padding(1)
                            .style(style::plain_icon_button_with_colors(
                                iced::Color::parse("#242226").map(|c| c.into()),
                                None
                            )),
                        icon_button(Icon::ChevronDown, 12)
                            .on_press(Message::PinSwap(
                                pin_kind,
                                num,
                                num.saturating_add(1)
                            ))
                            .padding(1)
                            .style(style::plain_icon_button_with_colors(
                                iced::Color::parse("#242226").map(|c| c.into()),
                                None
                            )),
                    ],
                    control_button!(
                        icon: Icon::PinOff,
                        msg: Message::PinRemove(pin_kind, num),
                        style: style::plain_icon_button,
                    )
                ]
                    .height(iced::Length::Fill)
                    .align_y(iced::Alignment::Center)
            )
                .align_x(iced::Alignment::End)
                .padding(iced::Padding { right: 5.0, ..iced::Padding::default() })
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
        ) 
    }

    fn queue_item(num: usize, track: &Track) -> Element {
        iced::widget::hover(
            container(
                column![
                    text!(
                        "{}",
                        track.metadata.title
                            .as_deref()
                            .unwrap_or(track.path.file_name().unwrap()
                                .to_str().unwrap())
                    )
                        .size(TEXT_SIZE),
                    text!("{}", print_artists(&track.metadata.artists))
                        .size(TEXT_SIZE)
                        .style(|theme: &iced::Theme| {
                            text::Style {
                                color: Some(
                                    theme.extended_palette()
                                        .background.base.text
                                        .scale_alpha(0.5)
                                )
                            }
                        }),
                ]
            )
                .padding(3)
                .width(iced::Length::Fill)
                .height(48),
            container(
                row![
                    column![
                        icon_button(Icon::ChevronUp, 16)
                            .on_press(Message::QueueSwap(num, num.saturating_sub(1)))
                            .padding(1)
                            .style(style::plain_icon_button_with_colors(
                                iced::Color::parse("#242226").map(|c| c.into()),
                                None
                            )),
                        icon_button(Icon::ChevronDown, 16)
                            .on_press(Message::QueueSwap(num, num.saturating_add(1)))
                            .padding(1)
                            .style(style::plain_icon_button_with_colors(
                                iced::Color::parse("#242226").map(|c| c.into()),
                                None
                            )),
                    ],
                    control_button!(
                        icon: Icon::Trash,
                        msg: Message::QueueRemove(num),
                        style: style::plain_icon_button,
                    )
                ]
                    .height(iced::Length::Fill)
                    .align_y(iced::Alignment::Center)
            )
                .align_x(iced::Alignment::End)
                .padding(iced::Padding { right: 5.0, ..iced::Padding::default() })
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
        ) 
    }

    fn queue_view(&self) -> Element {
        let mut contents = vec![];

        let queue = self.queue.iter()
            .enumerate()
            .map(|(i, track)| {
                let track = self.library.get_track(*track).unwrap();
                Self::queue_item(i, track)
            })
            .collect::<Vec<_>>();
        contents.push(
            container(
                scrollable(
                    column(queue)
                )
            )
                .style(style::track_list_container)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .into()
        );

        column(contents)
            .width(iced::Length::FillPortion(3))
            .height(iced::Length::Fill)
            .into()
    }
}

fn print_artists(artists: &Vec<String>) -> String {
    let mut txt = String::new();

    for artist in artists {
        txt.push_str(&format!("{}, ", artist));
    }

    txt.pop();
    txt.pop();

    txt
}

fn print_duration(duration: &std::time::Duration) -> String {
    let secs = duration.as_secs() % 60;
    let mins = duration.as_secs() / 60;
    if mins >= 100 {
        let hrs = mins / 60;
        let mins = mins % 60;
        format!("{}:{:#02}:{:#02}", hrs, mins, secs)
    } else {
        format!("{:#02}:{:#02}", mins, secs)
    }
}
