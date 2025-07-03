use crate::app::{playlist::Playlist, App};
use super::*;

use iced::widget::{ button, column, container, row, scrollable };

impl App {
    fn playlist_list_item_view(playlist: &Playlist) -> Element {
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

    fn playlist_list_header_view() -> Element<'static> {
        container(
            column![
                text("Playlists")
                    .size(20),
            ]
        )
            .width(iced::Length::Fill)
            .height(148)
            .padding(20)
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();

                container::Style {
                    text_color: Some(palette.background.base.text),
                    background: Some(palette.background.base.color.into()),
                    ..container::Style::default()
                }
            })
            .into()
    }

    pub fn playlist_list_view(&self) -> Element {
        let mut playlists = self.playlists.playlists()
            .map(|(_, pl)| pl)
            .collect::<Vec<_>>();
        playlists.sort_unstable_by_key(|pl| &pl.title);
        let mut contents = playlists
            .into_iter()
            .map(|pl| Self::playlist_list_item_view(pl))
            .collect::<Vec<_>>();
        contents.insert(0, Self::playlist_list_header_view());

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

    fn playlist_header_view(pl: &Playlist) -> Element {
        container(
            row![
                container(
                    match &pl.img {
                        Some(img_path) => {
                            let img = image(img_path)
                                .content_fit(iced::ContentFit::Cover)
                                .width(128)
                                .width(128);
                            <image::Image as Into<iced::Element<Message>>>::into(img)
                        }
                        None => text!("{}", char::from(Icon::FileMusic))
                            .font(ICON_FONT)
                            .size(64)
                            .center()
                            .into(),
                    }
                )
                    .center(138)
                    .padding(5),
                column![
                    text!("{}", pl.title)
                        .size(20)
                        .align_x(iced::Alignment::Start),
                    text!("({})", pl.filename)
                        .size(SMALL_TEXT_SIZE)
                        .align_x(iced::Alignment::Start),
                    row![
                        control_button!(
                            icon: Icon::Play,
                            msg: Message::PlayList,
                            style: style::plain_icon_button,
                        ),
                        control_button!(
                            icon: Icon::Shuffle,
                            msg: Message::ShuffleList,
                            style: style::plain_icon_button,
                        ),
                        control_button!(
                            icon: Icon::Pin,
                            msg: Message::PinAdd(
                                PinKind::Playlist,
                                pl.filename.clone().into()
                            ),
                            style: style::plain_icon_button,
                        )
                    ]
                ]
                    .padding(5)
            ]
        )
            .width(iced::Length::Fill)
            .height(148)
            .padding(5)
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();

                container::Style {
                    text_color: Some(palette.background.base.text),
                    background: Some(palette.background.base.color.into()),
                    ..container::Style::default()
                }
            })
            .into()
    }

    pub(super) fn playlist_view(&self, id: u64) -> Element {
        let pl = self.playlists.get_playlist(id).unwrap();

        let mut contents = pl.tracks
            .iter()
            .map(|t| match t {
                PlaylistTrack::Track(id, _) => {
                    match self.library.get_track(*id) {
                        Some(track) => Some((*id, track)),
                        None => None,
                    }
                }
                PlaylistTrack::Unresolved(_) => None,
            })
            .flatten()
            .enumerate()
            .map(|(num, (id, track))| Self::track_view(track, id, num + 1))
            .collect::<Vec<_>>();
        contents.insert(0, Self::tracks_header(!contents.is_empty()));

        container(
            column![
                Self::playlist_header_view(pl),
                scrollable(
                    column(contents)
                )
                    .direction(scrollable::Direction::Vertical(
                        scrollable::Scrollbar::default()))
                    .spacing(0)
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill)
            ]
        )
            .style(style::track_list_container)
            .padding(2)
            .width(iced::Length::FillPortion(10))
            .height(iced::Length::Fill)
            .into()
    }
}
