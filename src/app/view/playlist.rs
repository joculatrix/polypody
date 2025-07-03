use crate::app::{playlist::Playlist, App};
use super::*;

use iced::widget::{ button, column, container, horizontal_space, row, scrollable, stack, text_input, vertical_space };

impl App {
    fn new_playlist_menu(&self) -> Element {
        container(
            column![
                text("New Playlist")
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..iced::Font::default()
                    })
                    .size(TEXT_SIZE)
                    .center(),
                horizontal_space().height(5),
                text_input("Title", &self.new_playlist_title)
                    .on_input(Message::PlaylistTitleChanged)
                    .size(TEXT_SIZE)
                    .width(iced::Length::Fill),
                horizontal_space().height(5),
                text_input(
                    "Filename (without path or extension)",
                    &self.new_playlist_path
                )
                    .on_input(Message::PlaylistPathChanged)
                    .size(TEXT_SIZE)
                    .width(iced::Length::Fill),
                horizontal_space().height(5),
                row![
                    text_input(
                        "Cover image path",
                        &self.new_playlist_img
                    )
                        .on_input(Message::ImgPathChanged)
                        .size(TEXT_SIZE)
                        .width(iced::Length::Fill),
                    control_button!(
                        icon: Icon::Folder,
                        msg: Message::OpenImgDialog,
                        style: style::plain_icon_button,
                    )
                        .width(iced::Length::Shrink),
                ],
                horizontal_space().height(5),
                row![
                    button(text("Done").size(TEXT_SIZE))
                        .on_press(Message::CreatePlaylist)
                        .style(style::outlined_button)
                        .width(iced::Length::Fill),
                    vertical_space().width(5),
                    button(text("Cancel").size(TEXT_SIZE))
                        .on_press(Message::CancelCreatePlaylist)
                        .style(style::outlined_button)
                        .width(iced::Length::Fill),
                ]
            ]
                .align_x(iced::Alignment::Center)
        )
            .padding(10)
            .width(iced::Length::FillPortion(6))
            .style(|theme: &iced::Theme| container::Style {
                shadow: iced::Shadow {
                    offset: iced::Vector::new(2.0, 2.0),
                    ..iced::Shadow::default()
                },
                ..style::track_list_container(theme)
            })
            .into()
    }

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
                horizontal_space().height(5),
                button(row![
                    text!("{}", char::from(Icon::Plus))
                        .font(ICON_FONT)
                        .size(TEXT_SIZE),
                    text(" New")
                        .size(TEXT_SIZE),
                ])
                    .on_press(Message::OpenNewPlaylist)
                    .style(style::outlined_button)
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

        let main_elem = container(
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
            .into();

        if self.new_playlist_menu {
            stack!(
                main_elem,
                container(
                    column![
                        horizontal_space().height(iced::Length::FillPortion(3)),
                        row![
                            vertical_space().width(iced::Length::FillPortion(2)),
                            self.new_playlist_menu(),
                            vertical_space().width(iced::Length::FillPortion(2)),
                        ]
                            .height(iced::Length::FillPortion(3)),
                        horizontal_space().height(iced::Length::FillPortion(3)),
                    ]
                )
                    .center(iced::Length::Fill)
            )
                .into()
        } else {
            main_elem
        }
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
            .map(|(num, (id, track))| Self::track_view(track, id, num + 1, true))
            .collect::<Vec<_>>();
        contents.insert(0, Self::tracks_header(!contents.is_empty()));

        let main_elem = container(
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
            .into();

        if self.selecting_playlist.is_some() {
            stack!(
                main_elem,
                column![
                    horizontal_space().height(iced::Length::FillPortion(1)),
                    row![
                        vertical_space().width(iced::Length::FillPortion(3)),
                        container(self.add_to_playlist_menu())
                            .width(iced::Length::FillPortion(6))
                            .height(iced::Length::Fill),
                        vertical_space().width(iced::Length::FillPortion(3)),
                    ]
                        .width(iced::Length::Fill)
                        .height(iced::Length::FillPortion(8)),
                    horizontal_space().height(iced::Length::FillPortion(1)),
                ]
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill)
            )
                .into()
        } else {
            main_elem
        }
    }
}
