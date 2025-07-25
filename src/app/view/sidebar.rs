use super::{ *, column };

pub struct Sidebar {
    library_pins: Vec<(u64, String)>,
    playlist_pins: Vec<(u64, String)>,
}

#[derive(Clone, Debug)]
pub enum SidebarMessage {
    LibraryAppend(u64, String),
    LibraryRemove(usize),
    LibrarySwap(usize, usize),
    PlaylistAppend(u64, String),
    PlaylistRemove(usize),
    PlaylistSwap(usize, usize),
}

impl Into<Message> for SidebarMessage {
    fn into(self) -> Message {
        Message::SidebarMessage(self)
    }
}

impl Sidebar {
    pub fn new(
        library_pins: Vec<(u64, String)>,
        playlist_pins: Vec<(u64, String)>,
    ) -> Self {
        Self { library_pins, playlist_pins }
    }

    pub fn update(&mut self, msg: SidebarMessage, config: &mut Config) {
        match msg {
            SidebarMessage::LibraryAppend(id, name) => {
                self.library_pins.push((id, name));
            }
            SidebarMessage::LibraryRemove(i) => {
                config.library.pins.remove(i);
                self.library_pins.remove(i);
            }
            SidebarMessage::LibrarySwap(i, j) => unsafe {
                if j < self.library_pins.len() {
                    config.library.pins.swap_unchecked(i, j);
                    self.library_pins.swap_unchecked(i, j);
                }
            }
            SidebarMessage::PlaylistAppend(id, name) => {
                self.playlist_pins.push((id, name));
            }
            SidebarMessage::PlaylistRemove(i) => {
                config.library.pins.remove(i);
                self.playlist_pins.remove(i);
            }
            SidebarMessage::PlaylistSwap(i, j) => unsafe {
                if j < self.playlist_pins.len() {
                    config.library.pins.swap_unchecked(i, j);
                    self.playlist_pins.swap_unchecked(i, j);
                }
            }
        }
    }

    pub fn view(&self) -> Element {
        let mut contents = vec![];
        contents.push(
            Self::section_btn(
                Icon::DiscAlbum,
                " Library",
                Message::ViewLibraryRoot
            )
        );
        self.library_pins.iter()
            .enumerate()
            .for_each(|(i, pin)| {
                contents.push(Self::item_btn(
                    &pin.1,
                    Message::ViewLibrary(pin.0),
                    SidebarMessage::LibrarySwap(i, i.saturating_sub(1)).into(),
                    SidebarMessage::LibrarySwap(i, i.saturating_add(1)).into(),
                    SidebarMessage::LibraryRemove(i).into(),
                ));
            });
        contents.push(
            Self::section_btn(
                Icon::FileMusic,
                " Playlists",
                Message::ViewPlaylist(None)
            )
        );
        self.playlist_pins.iter()
            .enumerate()
            .for_each(|(i, pin)| {
                contents.push(Self::item_btn(
                    &pin.1,
                    Message::ViewPlaylist(Some(pin.0)),
                    SidebarMessage::PlaylistSwap(i, i.saturating_sub(1)).into(),
                    SidebarMessage::PlaylistSwap(i, i.saturating_add(1)).into(),
                    SidebarMessage::PlaylistRemove(i).into(),
                ));
            });
        container(
            scrollable(
                column(contents)
            )
        )
            .style(style::bordered_container)
            .padding(1)
            .width(iced::Length::FillPortion(3))
            .height(iced::Length::Fill)
            .into()
    }

    fn section_btn(
        icon: Icon,
        txt: &'static str,
        msg: Message
    ) -> Element<'static> {
        button(
            row![
                text!("{}", char::from(icon))
                    .font(ICON_FONT)
                    .size(CONTROL_BUTTON_SIZE)
                    .align_y(iced::Alignment::Center),
                text(txt)
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
            .on_press(msg)
            .width(iced::Length::Fill)
            .height(48)
            .style(style::outlined_button)
            .into()
    }

    fn item_btn(
        txt: &str,
        open_msg: Message,
        up_msg: Message,
        down_msg: Message,
        remove_msg: Message,
    ) -> Element {
        iced::widget::hover(
            container(
                button(
                    text(txt)
                        .size(TEXT_SIZE)
                        .align_y(iced::Alignment::Center)
                        .height(iced::Length::Fill)
                )
                    .style(style::list_button)
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill)
                    .on_press(open_msg)
            )
                .width(iced::Length::Fill)
                .height(42),
            container(
                row![
                    column![
                        icon_button(Icon::ChevronUp, 12)
                            .on_press(up_msg)
                            .padding(1)
                            .style(style::plain_icon_button_with_colors(
                                iced::Color::parse("#242226").map(|c| c.into()),
                                None
                            )),
                        icon_button(Icon::ChevronDown, 12)
                            .on_press(down_msg)
                            .padding(1)
                            .style(style::plain_icon_button_with_colors(
                                iced::Color::parse("#242226").map(|c| c.into()),
                                None
                            )),
                    ],
                    control_button!(
                        icon: Icon::PinOff,
                        msg: remove_msg,
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
