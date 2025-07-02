use crate::app::{playlist::Playlist, App};
use super::*;

use iced::widget::{ button, container, column, row, scrollable };

impl App {
    fn playlist_list_item(playlist: &Playlist) -> Element {
        button(
            row![
                container(
                    text!("{}", char::from(Icon::ArrowCornerDR))
                        .font(ICON_FONT)
                        .size(CONTROL_BUTTON_SIZE / 2)
                        .center()
                )
                    .center_x(iced::Length::FillPortion(1))
                    .center_y(iced::Length::Fill)
                    .padding(2),
                container(
                    match &playlist.img {
                        Some(img_path) => {
                            let img = image(img_path)
                                .width(CONTROL_BUTTON_SIZE)
                                .height(CONTROL_BUTTON_SIZE);
                            <image::Image as Into<iced::Element<Message>>>::into(img)
                        }
                        _ => {
                            text!("{}", char::from(Icon::FileMusic))
                                .font(ICON_FONT)
                                .size(CONTROL_BUTTON_SIZE / 2)
                                .center()
                                .into()
                        }
                    }
                )
                    .width(CONTROL_BUTTON_SIZE + 10)
                    .align_x(iced::Alignment::Start)
                    .align_y(iced::Alignment::Center),
                text!("{}", playlist.title)
                    .size(TEXT_SIZE)
                    .align_x(iced::Alignment::Start)
                    .align_y(iced::Alignment::Center)
                    .width(iced::Length::FillPortion(24)),
            ]
                .height(iced::Length::Fill)
                .align_y(iced::Alignment::Center)
        )
            .width(iced::Length::Fill)
            .height(48)
            .style(style::dir_list_item)
            .on_press(Message::ViewPlaylist(
                Some(
                    xxhash_rust::xxh3::xxh3_64(playlist.filename.as_bytes())
                )))
            .into()
    }

    pub fn view_playlists(&self) -> Element {
        let mut playlists = self.playlists.playlists()
            .map(|(_, pl)| pl)
            .collect::<Vec<_>>();
        playlists.sort_unstable_by_key(|pl| &pl.title);
        let contents = playlists
            .into_iter()
            .map(|pl| Self::playlist_list_item(pl))
            .collect::<Vec<_>>();

        container(
            scrollable(column(contents))
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::default()))
                .spacing(0)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
        )
            .style(style::track_list_container)
            .padding(2)
            .width(iced::Length::FillPortion(10))
            .height(iced::Length::Fill)
            .into()
    }
}
