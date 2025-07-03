use super::{ *, column };

impl App {
    pub(super) fn sidebar_view(&self) -> Element {
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
                contents.push(Self::sidebar_item_view(
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
                contents.push(Self::sidebar_item_view(
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

    fn sidebar_item_view(
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
}
