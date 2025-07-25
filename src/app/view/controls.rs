use super::{ column, * };

impl App {
    pub(super) fn current_track(&self) -> Element {
        column![
            self.current_title(),
            self.current_artist(),
        ]
            .align_x(iced::Alignment::Center)
            .padding(3)
            .into()
    }

    fn current_title(&self) -> Element {
        text!("{}", self.playing.as_ref()
            .map_or(
                "",
                |track| {
                    match &track.metadata.title {
                        Some(title) => title,
                        None => track.path.file_name().unwrap()
                            .to_str().unwrap(),
                    }
                }
            )
        )
            .size(TEXT_SIZE)
            .center()
            .into()
    }

    fn current_artist(&self) -> Element {
        text!("{}", self.playing.as_ref()
            .map_or(
                String::from(""),
                |track| print_artists(&track.metadata.artists)
            )
        )
            .size(TEXT_SIZE)
            .center()
            .into()
    }

    pub(super) fn progress_bar(&self) -> Element {
        let (current_time, total_duration) = match self.track_duration {
            Some((current, total)) => (current, total),
            None =>
                (Duration::from_secs(0), Duration::from_secs(0)),
        };
        row![
            iced::widget::Space::with_width(iced::Length::Fill),
            fill![text!("{}", print_duration(&current_time)).size(SMALL_TEXT_SIZE).center()]
                .align_x(iced::Alignment::Center)
                .align_y(iced::Alignment::Center),
            slider(0.0..=1.0, self.playhead_position, Message::PlayheadMoved)
                .on_release(Message::PlayheadReleased)
                .step(0.01)
                .style(style::clean_slider)
                .width(iced::Length::FillPortion(16)),
            fill![
                text!("{}", print_duration(&total_duration)).size(SMALL_TEXT_SIZE).center()
            ]
                .align_x(iced::Alignment::Center)
                .align_y(iced::Alignment::Center),
            iced::widget::Space::with_width(iced::Length::Fill),
        ]
            .align_y(iced::Alignment::Center)
            .into()
    }

    fn play_button(play_status: &PlayStatus) -> Element {
        control_button!(
            icon: match play_status {
                PlayStatus::Pause | PlayStatus::Stopped => Icon::Play,
                PlayStatus::Play => Icon::Pause,
            },
            msg: Message::TogglePlay,
            style: style::play_button,
        )
            .into()
    }

    fn back_button() -> Element<'static> {
        control_button!(
            icon: Icon::SkipBack,
            msg: queue::QueueMessage::SkipBack.into(),
            style: style::plain_icon_button,
        )
            .into()
    }

    fn forward_button() -> Element<'static> {
        control_button!(
            icon: Icon::SkipForward,
            msg: queue::QueueMessage::SkipForward.into(),
            style: style::plain_icon_button,
        )
            .into()
    }

    fn shuffle_button() -> Element<'static> {
        control_button!(
            icon: Icon::Shuffle,
            msg: queue::QueueMessage::Shuffle.into(),
            style: style::plain_icon_button,
        )
            .into()
    }

    fn stop_button() -> Element<'static> {
        control_button!(
            icon: Icon::Square,
            msg: queue::QueueMessage::Stop.into(),
            style: style::plain_icon_button,
        )
            .into()
    }

    fn repeat_button(repeat: &RepeatStatus) -> Element {
        control_button!(
            icon: Icon::Repeat,
            msg: Message::ToggleRepeat,
            style: style::toggle_icon_button(*repeat != RepeatStatus::None),
        )
            .into()
    }

    fn repeat_text(repeat: &RepeatStatus) -> Element {
        text(match repeat {
            RepeatStatus::None => "Off",
            RepeatStatus::One => "Track",
            RepeatStatus::All => "All",
        })
            .size(SMALL_TEXT_SIZE)
            .style(style::toggle_text(*repeat != RepeatStatus::None))
            .into()
    }

    fn volume_button(mute: bool, volume: f32) -> Element<'static> {
        control_button!(
            icon: if mute || volume == 0.0 {
                Icon::VolumeMute
            } else if volume <= 0.3 {
                Icon::VolumeLow
            } else if volume <= 0.6 {
                Icon::VolumeMid
            } else {
                Icon::VolumeHigh
            },
            msg: Message::ToggleMute,
            style: style::plain_icon_button,
        )
            .into()
    }

    fn volume_slider(volume: f32) -> Element<'static> {
        slider(0.0..=1.0, volume, Message::VolumeChanged)
            .step(0.01)
            .width(iced::Length::Fill)
            .style(style::clean_slider)
            .into()
    }

    pub(super) fn control_bar(&self) -> Element {
        row![
            container(
                row![
                    fill![Self::back_button()],
                    fill![Self::stop_button()],
                    fill![Self::play_button(&self.play_status)],
                    fill![Self::shuffle_button()],
                    fill![Self::forward_button()],
                ]
            )
                .width(iced::Length::FillPortion(3))
                .align_x(iced::Alignment::Center)
                .padding(5)
                .style(style::control_panel_box),
            container(
                row![
                    Self::repeat_button(&self.repeat),
                    Self::repeat_text(&self.repeat),
                ]
                    .align_y(iced::Alignment::Center)
            )
                .align_x(iced::Alignment::Start)
                .align_y(iced::Alignment::Center)
                .padding(5)
                .width(iced::Length::FillPortion(2)),
            Space::with_width(iced::Length::FillPortion(4)),
            container(
                row![
                    Self::volume_button(self.mute, self.volume),
                    Self::volume_slider(self.volume),
                ]
                    .align_y(iced::Alignment::Center)
            )
                .align_y(iced::Alignment::Center)
                .padding(5)
                .width(iced::Length::FillPortion(2))
        ]
            .height(iced::Length::Shrink)
            .padding(5)
            .into()
    }
}
