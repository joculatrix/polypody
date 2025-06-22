use std::time::Duration;

use iced::Font;
use iced::widget::{
    button, column, container, row, text, Button, Column, Row,
    scrollable, slider, Space, image,
};

use crate::{ App, Message, PlayStatus, RepeatStatus };
use crate::internal::{ Directory, Metadata, Track };

mod library;
mod style;

pub const ICON_FONT_BYTES: &[u8] = include_bytes!("../../fonts/lucide.ttf");
const ICON_FONT: Font = Font::with_name("lucide");

const CONTROL_BUTTON_SIZE: u16 = 36;

enum Icon {
    ArrowCornerDR,
    ArrowCornerLU,
    Folder,
    Pause,
    Play,
    Repeat,
    Shuffle,
    SkipBack,
    SkipForward,
    Square,
    VolumeMute,
    VolumeLow,
    VolumeMid,
    VolumeHigh,
}

impl From<Icon> for char {
    fn from(value: Icon) -> char {
        match value {
            Icon::ArrowCornerDR =>  '\u{E0A6}',
            Icon::ArrowCornerLU =>  '\u{E0A8}',
            Icon::Folder =>         '\u{E0DB}',
            Icon::Pause =>          '\u{E132}',
            Icon::Play =>           '\u{E140}',
            Icon::Repeat =>         '\u{E14A}',
            Icon::Shuffle =>        '\u{E162}',
            Icon::SkipBack =>       '\u{E163}',
            Icon::SkipForward =>    '\u{E164}',
            Icon::Square =>         '\u{E16B}',
            Icon::VolumeMute =>     '\u{E1AC}',
            Icon::VolumeLow =>      '\u{E1A9}',
            Icon::VolumeMid =>      '\u{E1AA}',
            Icon::VolumeHigh =>     '\u{E1AB}',
        }
    }
}

fn icon_button(icon: Icon, text_size: u16) -> Button<'static, Message> {
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
        icon_button($icon, CONTROL_BUTTON_SIZE / 2)
            .width(CONTROL_BUTTON_SIZE)
            .height(CONTROL_BUTTON_SIZE)
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

macro_rules! list_item {
    ($item:expr) => {
        button($item)
            .style(style::list_item)
            .width(iced::Length::Fill)
            .height(48)
    }
}

pub(self) use list_item;

impl App {
    pub fn view (&self) -> iced::Element<Message> {
        let (current_time, total_duration, slider_pos) = match self.track_duration {
            Some((current, total)) =>
                (current, total, current.as_secs_f32() / total.as_secs_f32()),
            None =>
                (Duration::from_secs(0), Duration::from_secs(0), 0.5),
        };
        container(
            column![
                self.library_view(),
                text!("{}", self.playing.as_ref()
                    .map(|track| {
                        match &track.metadata.title {
                            Some(title) => title,
                            None => track.path
                                .file_name().unwrap()
                                .to_str().unwrap(),
                        }
                    }).unwrap_or("")
                )
                    .size(14)
                    .center(),
                text!("{}", self.playing.as_ref()
                    .map_or(
                        String::from(""),
                        |track| print_artists(&track.metadata.artists)
                    )
                )
                    .size(14)
                    .center(),
                row![
                    iced::widget::Space::with_width(iced::Length::Fill),
                    fill![text!("{}", print_duration(&current_time)).size(12).center()]
                        .align_x(iced::Alignment::Center)
                        .align_y(iced::Alignment::Center),
                    slider(0.0..=1.0, slider_pos, Message::PlayheadMoved)
                        .style(style::clean_slider)
                        .width(iced::Length::FillPortion(16)),
                    fill![
                        text!("{}", print_duration(&total_duration)).size(12).center()
                    ]
                        .align_x(iced::Alignment::Center)
                        .align_y(iced::Alignment::Center),
                    iced::widget::Space::with_width(iced::Length::Fill),
                ]
                    .align_y(iced::Alignment::Center),
                self.control_bar(),
            ]
                .align_x(iced::alignment::Horizontal::Center)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
        )
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .padding(10)
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();

                container::Style {
                    background: Some(palette.background.base.color.into()),
                    ..Default::default()
                }
            })
            .into()
    }

    fn control_bar(&self) -> iced::Element<Message> {
        let play_button = control_button!(
            icon: match self.play_status {
                PlayStatus::Pause | PlayStatus::Stopped => Icon::Play,
                PlayStatus::Play => Icon::Pause,
            },
            msg: Message::TogglePlay,
            style: style::play_button,
        );

        let back_button = control_button!(
            icon: Icon::SkipBack,
            msg: Message::SkipBack,
            style: style::plain_icon_button,
        );

        let forward_button = control_button!(
            icon: Icon::SkipForward,
            msg: Message::SkipForward,
            style: style::plain_icon_button,
        );

        let shuffle_button = control_button!(
            icon: Icon::Shuffle,
            msg: Message::Shuffle,
            style: style::plain_icon_button,
        );

        let stop_button = control_button!(
            icon: Icon::Square,
            msg: Message::Stop,
            style: style::plain_icon_button,
        );

        let repeat = self.repeat;

        let repeat_button = control_button!(
            icon: Icon::Repeat,
            msg: Message::ToggleRepeat,
            style: style::toggle_icon_button(self.repeat != RepeatStatus::None),
        );

        let repeat_text = text(match repeat {
            RepeatStatus::None => "Off",
            RepeatStatus::One => "Track",
            RepeatStatus::All => "All",
        })
            .size(14)
            .style(move |theme: &iced::Theme| {
                let palette = theme.extended_palette();

                text::Style {
                    color: Some(if repeat == RepeatStatus::None {
                        palette.background.base.text
                    } else {
                        palette.primary.base.color
                    })
                }
            });

        let volume_button = control_button!(
            icon: if self.mute || self.volume == 0.0 {
                Icon::VolumeMute
            } else if self.volume <= 0.3 {
                Icon::VolumeLow
            } else if self.volume <= 0.6 {
                Icon::VolumeMid
            } else {
                Icon::VolumeHigh
            },
            msg: Message::ToggleMute,
            style: style::plain_icon_button,
        );

        let volume_slider = slider(0.0..=1.0, self.volume, Message::VolumeChanged)
            .step(0.01)
            .width(iced::Length::Fill)
            .style(style::clean_slider);

        row![
            container(
                row![
                    fill![back_button],
                    fill![stop_button],
                    fill![play_button],
                    fill![shuffle_button],
                    fill![forward_button],
                ]
            )
                .width(iced::Length::FillPortion(3))
                .align_x(iced::Alignment::Center)
                .padding(5)
                .style(|theme: &iced::Theme| {
                    let palette = theme.extended_palette();

                    container::Style {
                        background: None,
                        border: iced::border::rounded(5)
                            .color(iced::Color {
                                a: 0.10,
                                ..palette.background.base.text
                            })
                            .width(2),
                        ..container::Style::default()
                    }
                }),
            container(
                row![
                    repeat_button,
                    repeat_text,
                ]
                    .align_y(iced::Alignment::Center)
            )
                .align_x(iced::Alignment::Start)
                .align_y(iced::Alignment::Center)
                .padding(5)
                .width(iced::Length::FillPortion(2)),
            Space::with_width(iced::Length::FillPortion(4)),
            container(
                row![
                    volume_button,
                    volume_slider,
                ]
                    .align_y(iced::Alignment::Center)
            )
                .align_y(iced::Alignment::Center)
                .padding(5)
                .width(iced::Length::FillPortion(2))
        ]
            .height(iced::Length::Shrink)
            .padding(5)
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
