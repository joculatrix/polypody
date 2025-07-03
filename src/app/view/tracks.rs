use super::{ *, column };
use iced::widget::vertical_space;

impl App {
    fn track_list_item(track: &Track, num: usize) -> Element {
        container(
            row![
                text!("{}    ", num)
                    .style(|theme: &iced::Theme| {
                        let palette = theme.extended_palette();

                        text::Style {
                            color: Some(palette
                                .background.base.text
                                .scale_alpha(0.75)),
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
                            track.path.file_name().unwrap()
                                .to_str().unwrap()
                        ),
                    }
                        .size(TEXT_SIZE)
                        .align_x(iced::Alignment::Start)
                        .align_y(iced::Alignment::Center),
                    if track.metadata.artists.is_empty() {
                        text!("")
                    } else {
                        text!("{}", print_artists(&track.metadata.artists))
                    }
                        .size(TEXT_SIZE)
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
                .align_y(iced::Alignment::Center)
        )
            .height(48)
            .into()
    }

    fn track_buttons(id: u64) -> Element<'static> {
        row![
            control_button!(
                icon: Icon::Play,
                msg: Message::PlayTrack(id),
                style: style::plain_icon_button_with_colors(
                    iced::Color::parse("#242226").map(|c| c.into()),
                    None
                ),
            )
                .width(iced::Length::FillPortion(1)),
            control_button!(
                icon: Icon::Plus,
                msg: Message::AppendTrack(id),
                style: style::plain_icon_button_with_colors(
                    iced::Color::parse("#242226").map(|c| c.into()),
                    None
                ),
            )
                .width(iced::Length::FillPortion(1)),
            vertical_space().width(iced::Length::FillPortion(24)),
        ]
            .align_y(iced::Alignment::Center)
            .into()
    }

    pub(super) fn track_view(track: &Track, id: u64, num: usize) -> Element {
        iced_aw::ContextMenu::new(
            iced::widget::hover(
                Self::track_list_item(track, num),
                Self::track_buttons(id),
            ),
            move || {
                container(
                    column(vec![
                        button("Play")
                            .on_press(Message::PlayTrack(id))
                            .width(iced::Length::Fill)
                            .style(style::context_menu_button)
                            .into(),
                        button("Add to queue")
                            .on_press(Message::AppendTrack(id))
                            .width(iced::Length::Fill)
                            .style(style::context_menu_button)
                            .into(),
                    ])
                )
                    .padding(2)
                    .width(128)
                    .style(style::context_menu)
                    .into()
            }
        )
            .into()
    }
}
