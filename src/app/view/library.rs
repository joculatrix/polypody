use iced::widget::vertical_space;

use super::{ column, * };

impl App {
    fn library_subdir_view(dir: &Directory) -> Element {
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
                    match &dir.img {
                        Some(img_path) => {
                            let img = image(img_path)
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
                    .align_y(iced::Alignment::Center),
                text!("{}", &dir.path.file_name().unwrap().to_str().unwrap())
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
            .on_press(Message::ViewLibrary(crate::internal::library::directory_hash(dir)))
            .into()
    }

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

    fn track_buttons(index: usize) -> Element<'static> {
        row![
            control_button!(
                icon: Icon::Play,
                msg: Message::PlayTrack(index),
                style: style::plain_icon_button_with_colors(
                    iced::Color::parse("#242226").map(|c| c.into()),
                    None
                ),
            )
                .width(iced::Length::FillPortion(1)),
            control_button!(
                icon: Icon::Plus,
                msg: Message::AppendTrack(index),
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

    fn library_track_view(track: &Track, num: usize) -> Element {
        let internal_track_index = unsafe {
            num.unchecked_sub(1)
        };
        iced_aw::ContextMenu::new(
            iced::widget::hover(
                Self::track_list_item(track, num),
                Self::track_buttons(internal_track_index),
            ),
            move || {
                container(
                    column(vec![
                        button("Play")
                            .on_press(Message::PlayTrack(internal_track_index))
                            .width(iced::Length::Fill)
                            .style(style::context_menu_button)
                            .into(),
                        button("Add to queue")
                            .on_press(Message::AppendTrack(internal_track_index))
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

    fn library_header_view(dir: &Directory) -> Element {
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
                                        .size(TEXT_SIZE),
                                    text!(" back")
                                        .size(TEXT_SIZE),
                                ]
                            )
                                .style(style::plain_icon_button)
                                .on_press(Message::ViewLibrary(dir.parent))
                                .into()
                        )
                        .unwrap_or_else(||
                            text!("").size(TEXT_SIZE).into(),
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

    fn tracks_header(draw: bool) -> Element<'static> {
        container(
            if draw {
                row![
                    text!("#    ")
                        .size(TEXT_SIZE)
                        .width(iced::Length::FillPortion(2))
                        .align_x(iced::Alignment::End),
                    text!("Title")
                        .size(TEXT_SIZE)
                        .width(iced::Length::FillPortion(10))
                        .align_x(iced::Alignment::Start),
                    text!("Album")
                        .size(TEXT_SIZE)
                        .width(iced::Length::FillPortion(10))
                        .align_x(iced::Alignment::Start),
                    text!("Duration")
                        .size(TEXT_SIZE)
                        .width(iced::Length::FillPortion(4))
                        .align_x(iced::Alignment::Start),
                ]
                    .into()
            } else {
                <Space as Into<iced::Element<Message>>>
                    ::into(Space::new(iced::Length::Fill, iced::Length::Shrink))
            }
        )
            .width(iced::Length::Fill)
            .style(style::tracks_header)
            .into()
    }

    pub(super) fn library_view(&self) -> Element {   
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

        container(
            column![
                Self::library_header_view(dir),
                scrollable(
                    column(
                        dir_items
                            .chain(std::iter::once(
                                Self::tracks_header(!track_items.is_empty())))
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
            .style(style::track_list_container)
            .padding(2)
            .width(iced::Length::FillPortion(9))
            .height(iced::Length::Fill)
            .into()
    }
}
