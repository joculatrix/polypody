use iced::widget::{horizontal_space, stack, vertical_space};

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
                                        .size(SMALL_TEXT_SIZE),
                                    text!(" back")
                                        .size(SMALL_TEXT_SIZE),
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
                        control_button!(
                            icon: Icon::Pin,
                            msg: Message::PinAdd(PinKind::Library, dir.path.clone()),
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

    pub(super) fn tracks_header(draw: bool) -> Element<'static> {
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
                (*id, self.library.get_track(*id).unwrap_unchecked())
            })
            .collect::<Vec<_>>();    

        let track_items = tracks
            .into_iter()
            .enumerate()
            .map(|(i, (id, track))| Self::track_view(track, id, i + 1, false));

        let main_elem = container(
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
