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
mod style;

pub const ICON_FONT_BYTES: &[u8] = include_bytes!("../../../fonts/lucide.ttf");
const ICON_FONT: Font = Font::with_name("lucide");

const CONTROL_BUTTON_SIZE: u16 = 36;
const TEXT_SIZE: u16 = 14;
const SMALL_TEXT_SIZE: u16 = 12;

type Element<'a> = iced::Element<'a, Message>;

enum Icon {
    ArrowCornerDR,
    ArrowCornerLU,
    Folder,
    Pause,
    Play,
    Plus,
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
            Icon::Plus =>           '\u{E141}',
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
    pub fn view (&self) -> Element {
        container(
            column![
                self.library_view(),
                self.current_track(),
                self.progress_bar(),
                self.control_bar(),
            ]
                .align_x(iced::alignment::Horizontal::Center)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
        )
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .padding(10)
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
