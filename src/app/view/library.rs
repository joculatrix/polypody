use super::{ column, * };

impl App {
    fn library_subdir_view(dir: &Directory) -> iced::Element<Message> {
        list_item!(
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
                    match &dir.img {
                        Some(img_path) => {
                            let img = image(img_path)
                                .content_fit(iced::ContentFit::Cover)
                                .width(CONTROL_BUTTON_SIZE)
                                .height(CONTROL_BUTTON_SIZE);
                            <image::Image as Into<iced::Element<Message>>>::into(img)
                        }
                        _ => {
                            text!("{}", char::from(Icon::Folder))
                                .font(ICON_FONT)
                                .size(CONTROL_BUTTON_SIZE / 2)
                                .center()
                                .into()
                        }
                    }
                )
                    .width(CONTROL_BUTTON_SIZE + 10)
                    .align_x(iced::Alignment::Start)
                    .center_y(CONTROL_BUTTON_SIZE),
                text!("{}", &dir.path.file_name().unwrap().to_str().unwrap())
                    .size(14)
                    .align_x(iced::Alignment::Start)
                    .align_y(iced::Alignment::Center)
                    .width(iced::Length::FillPortion(24)),
            ]
                .height(iced::Length::Fill)
                .align_y(iced::Alignment::Center)
        )
            .on_press(Message::ViewLibrary(crate::internal::library::directory_hash(dir)))
            .into()
    }

    fn library_track_view(track: &Track, num: usize) -> iced::Element<Message> {
        list_item!(
            row![
                text!("{}", num)
                    .style(|theme: &iced::Theme| {
                        let palette = theme.extended_palette();

                        text::Style {
                            color: Some(palette
                                .background.base.text
                                .scale_alpha(0.75)),
                        }
                    })
                    .size(14)
                    .align_x(iced::Alignment::Center)
                    .width(iced::Length::FillPortion(1)),
                column![
                    match &track.metadata.title {
                        Some(title) => text!("{}", title),
                        None => text!(
                            "{}",
                            track.path.file_name().unwrap().to_str().unwrap()
                        ),
                    }
                        .size(14)
                        .align_x(iced::Alignment::Start)
                        .align_y(iced::Alignment::Center),
                    if track.metadata.artists.is_empty() {
                        text!("")
                    } else {
                        text!("{}", print_artists(&track.metadata.artists))
                    }
                        .size(14)
                        .align_x(iced::Alignment::Start)
                        .align_y(iced::Alignment::Center),
                ]
                    .align_x(iced::Alignment::Start)
                    .width(iced::Length::FillPortion(8)),
                match &track.metadata.album {
                    Some(album) => text!("{}", album),
                    None => text!(""),
                }
                    .size(14)
                    .align_x(iced::Alignment::Start)
                    .align_y(iced::Alignment::Center)
                    .width(iced::Length::FillPortion(8)),
                match &track.metadata.duration {
                    Some(duration) => text!("{}", print_duration(duration)),
                    None => text!(""),
                }
                    .size(14)
                    .align_x(iced::Alignment::Start)
                    .align_y(iced::Alignment::Center)
                    .width(iced::Length::FillPortion(3)),
            ]
                .height(iced::Length::Fill)
                .align_y(iced::Alignment::Center)
        )
            .on_press(Message::Dummy)
            .into()
    }

    fn library_header_view(dir: &Directory) -> iced::Element<Message> {
        container(
            row![
                container(
                    match &dir.img {
                        Some(img_path) => {
                            let img = image(img_path)
                                .content_fit(iced::ContentFit::Cover)
                                .width(128)
                                .width(128);
                            <image::Image as Into<iced::Element<Message>>>::into(img)
                        }
                        None => text!("{}", char::from(Icon::Folder))
                            .font(ICON_FONT)
                            .size(64)
                            .center()
                            .into(),
                    }
                )
                    .center(138)
                    .padding(5),
                column![
                    dir.parent.ne(&0)
                        .then::<iced::Element<Message>, _>(||
                            button(
                                row![
                                    text!("{}", char::from(Icon::ArrowCornerLU))
                                        .font(ICON_FONT)
                                        .size(14),
                                    text!(" back")
                                        .size(14),
                                ]
                            )
                                .style(style::plain_icon_button)
                                .on_press(Message::ViewLibrary(dir.parent))
                                .into()
                        )
                        .unwrap_or_else(||
                            text!("").size(14).into(),
                        ),
                    text!("{}", dir.path.file_name().unwrap().to_str().unwrap())
                        .size(20)
                        .align_x(iced::Alignment::Start),
                    row![
                        control_button!(
                            icon: Icon::Play,
                            msg: Message::PlayFolder,
                            style: style::plain_icon_button,
                        ),
                        control_button!(
                            icon: Icon::Shuffle,
                            msg: Message::ShuffleFolder,
                            style: style::plain_icon_button,
                        ),
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

    pub(super) fn library_view(&self) -> iced::Element<Message> {   
        let dir = self.library.current_directory();

        let mut subdirs = dir
            .subdirs
            .iter()
            .map(|id| unsafe {
                self.library.get_directory(*id).unwrap_unchecked()
            })
            .collect::<Vec<_>>();

        subdirs.sort_by_key(|dir|
            dir.path.file_name().unwrap()
        );

        let dir_items = subdirs
            .into_iter()
            .map(|dir| Self::library_subdir_view(dir));

        let tracks = dir
            .tracks
            .iter()
            .map(|id| unsafe {
                self.library.get_track(*id).unwrap_unchecked()
            })
            .collect::<Vec<_>>();    

        let track_items = tracks
            .into_iter()
            .enumerate()
            .map(|(i, track)| Self::library_track_view(track, i + 1));

        let tracks_header = container(
            row![
                text!("#")
                    .size(14)
                    .width(iced::Length::FillPortion(1))
                    .align_x(iced::Alignment::Center),
                text!("Title")
                    .size(14)
                    .width(iced::Length::FillPortion(8))
                    .align_x(iced::Alignment::Start),
                text!("Album")
                    .size(14)
                    .width(iced::Length::FillPortion(8))
                    .align_x(iced::Alignment::Start),
                text!("Duration")
                    .size(14)
                    .width(iced::Length::FillPortion(3))
                    .align_x(iced::Alignment::Start),
            ]
        )
            .width(iced::Length::Fill)
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();

                container::Style {
                    text_color: Some(palette.background.base.text.scale_alpha(0.6)),
                    background: Some(palette.background.base.color.into()),
                    ..container::Style::default()
                }
            })
            .into();


        container(
            column![
                Self::library_header_view(dir),
                scrollable(
                    column(
                        dir_items
                            .chain(std::iter::once(tracks_header))
                            .chain(track_items)
                    )
                )
                    .direction(scrollable::Direction::Vertical(
                        scrollable::Scrollbar::default()))
                    .spacing(0)
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill)
            ]
        )
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();

                container::Style {
                    background: iced::Color::parse("#242226").map(|c| c.into()),
                    border: iced::Border {
                        color: palette.background.base.text.scale_alpha(0.2),
                        width: 1.0,
                        radius: (2.0).into(),
                    },
                    ..container::Style::default()
                }
            })
            .padding(2)
            .height(iced::Length::Fill)
            .into()
    }
}
