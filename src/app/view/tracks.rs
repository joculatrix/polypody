use iced::widget::vertical_space;

use super::{column, *};

impl App {
    pub(super) fn add_to_playlist_menu(&self) -> Element {
        let content = self
            .playlists
            .playlists()
            .map(|(id, pl)| {
                button(text!("{}", pl.title).size(TEXT_SIZE))
                    .width(iced::Length::Fill)
                    .on_press(Message::PlaylistSelected(*id))
                    .style(style::plain_icon_button_with_colors(
                        iced::Color::parse("#242226").map(|c| c.into()),
                        None,
                    ))
                    .into()
            })
            .collect::<Vec<_>>();

        container(column![
            row![
                container(text("Add to playlist").size(20)),
                container(control_button!(
                    icon: Icon::X,
                    msg: Message::CloseAddToPlaylist,
                    style: style::plain_icon_button_with_colors(
                        iced::Color::parse("#242226").map(|c| c.into()),
                        None
                    )
                ))
                .width(iced::Length::Fill)
                .align_x(iced::Alignment::End),
            ]
            .width(iced::Length::Fill),
            scrollable(column(content)).spacing(0)
        ])
        .padding(10)
        .style(|theme: &iced::Theme| container::Style {
            shadow: iced::Shadow {
                offset: iced::Vector::new(2.0, 2.0),
                ..iced::Shadow::default()
            },
            ..style::track_list_container(theme)
        })
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
    }

    fn track_list_item(track: &Track, num: usize) -> Element {
        container(
            row![
                text!("{}    ", num)
                    .style(|theme: &iced::Theme| {
                        let palette = theme.extended_palette();

                        text::Style {
                            color: Some(
                                palette.background.base.text.scale_alpha(0.75),
                            ),
                        }
                    })
                    .size(TEXT_SIZE)
                    .align_x(iced::Alignment::End)
                    .width(iced::Length::FillPortion(2)),
                column![
                    match &track.metadata.title {
                        Some(title) => text!("{}", title),
                        None => text!(
                            "{}",
                            track.path.file_name().unwrap().to_str().unwrap()
                        ),
                    }
                    .size(TEXT_SIZE)
                    .height(iced::Length::Fill)
                    .align_x(iced::Alignment::Start)
                    .align_y(iced::Alignment::Center),
                    if track.metadata.artists.is_empty() {
                        text!("")
                    } else {
                        text!("{}", print_artists(&track.metadata.artists))
                    }
                    .size(TEXT_SIZE)
                    .height(iced::Length::Fill)
                    .align_x(iced::Alignment::Start)
                    .align_y(iced::Alignment::Center),
                ]
                .align_x(iced::Alignment::Start)
                .width(iced::Length::FillPortion(10)),
                match &track.metadata.album {
                    Some(album) => text!("{}", album),
                    None => text!(""),
                }
                .size(TEXT_SIZE)
                .align_x(iced::Alignment::Start)
                .align_y(iced::Alignment::Center)
                .width(iced::Length::FillPortion(10)),
                match &track.metadata.duration {
                    Some(duration) => text!("{}", print_duration(duration)),
                    None => text!(""),
                }
                .size(TEXT_SIZE)
                .align_x(iced::Alignment::Start)
                .align_y(iced::Alignment::Center)
                .width(iced::Length::FillPortion(4)),
            ]
            .height(iced::Length::Fill)
            .align_y(iced::Alignment::Center),
        )
        .height(48)
        .into()
    }

    fn track_buttons(id: u64, num: usize, playlist: bool) -> Element<'static> {
        let right_hand: Element = if playlist {
            container(row![
                column![
                    icon_button(Icon::ChevronUp, 16)
                        .on_press(Message::PlaylistSwap(
                            num,
                            num.saturating_sub(1)
                        ))
                        .padding(1)
                        .style(style::plain_icon_button_with_colors(
                            iced::Color::parse("#242226").map(|c| c.into()),
                            None
                        )),
                    icon_button(Icon::ChevronDown, 16)
                        .on_press(Message::PlaylistSwap(
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
                    icon: Icon::Trash,
                    msg: Message::PlaylistRemove(num),
                    style: style::plain_icon_button,
                )
            ])
            .align_x(iced::Alignment::End)
            .width(iced::Length::FillPortion(24))
            .height(iced::Length::Fill)
            .into()
        } else {
            vertical_space().width(iced::Length::FillPortion(24)).into()
        };
        row![
            control_button!(
                icon: Icon::Play,
                msg: queue::QueueMessage::PlayTrack(num).into(),
                style: style::plain_icon_button_with_colors(
                    iced::Color::parse("#242226").map(|c| c.into()),
                    None
                ),
            )
            .width(iced::Length::FillPortion(1)),
            control_button!(
                icon: Icon::Plus,
                msg: queue::QueueMessage::Append(id).into(),
                style: style::plain_icon_button_with_colors(
                    iced::Color::parse("#242226").map(|c| c.into()),
                    None
                ),
            )
            .width(iced::Length::FillPortion(1)),
            right_hand,
        ]
        .align_y(iced::Alignment::Center)
        .into()
    }

    pub(super) fn track_view(
        track: &Track,
        id: u64,
        num: usize,
        playlist: bool,
    ) -> Element {
        iced_aw::ContextMenu::new(
            iced::widget::hover(
                Self::track_list_item(track, num),
                Self::track_buttons(id, num - 1, playlist),
            ),
            move || {
                container(column(vec![
                    button("Play")
                        .on_press(queue::QueueMessage::PlayTrack(num).into())
                        .width(iced::Length::Fill)
                        .style(style::list_button)
                        .into(),
                    button("Add to queue")
                        .on_press(queue::QueueMessage::Append(id).into())
                        .width(iced::Length::Fill)
                        .style(style::list_button)
                        .into(),
                    button("Add to playlist...")
                        .on_press(Message::SelectPlaylist(id))
                        .width(iced::Length::Fill)
                        .style(style::list_button)
                        .into(),
                ]))
                .padding(2)
                .width(144)
                .style(style::context_menu)
                .into()
            },
        )
        .into()
    }
}
