use super::{ *, column };

impl App {
    fn queue_item(num: usize, track: &Track) -> Element {
        iced::widget::hover(
            container(
                column![
                    text!(
                        "{}",
                        track.metadata.title
                            .as_deref()
                            .unwrap_or(
                                track.path
                                    .file_name().unwrap()
                                    .to_str().unwrap()
                            )
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

    pub(super) fn queue(&self) -> Element {
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
