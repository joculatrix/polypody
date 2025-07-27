use super::{column, *};

#[derive(Clone, Debug)]
pub enum QueueMessage {
    Append(u64),
    PlayFolder,
    PlayList,
    PlayTrack(usize),
    Remove(usize),
    Shuffle,
    ShuffleFolder,
    ShuffleList,
    SkipBack,
    SkipForward,
    Stop,
    Swap(usize, usize),
}

impl Into<Message> for QueueMessage {
    fn into(self) -> Message {
        Message::Queue(self)
    }
}

impl App {
    fn queue_item(num: usize, track: &Track) -> Element {
        iced::widget::hover(
            container(column![
                text!(
                    "{}",
                    track.metadata.title.as_deref().unwrap_or(
                        track.path.file_name().unwrap().to_str().unwrap()
                    )
                )
                .size(TEXT_SIZE),
                text!("{}", print_artists(&track.metadata.artists))
                    .size(TEXT_SIZE)
                    .style(|theme: &iced::Theme| {
                        text::Style {
                            color: Some(
                                theme
                                    .extended_palette()
                                    .background
                                    .base
                                    .text
                                    .scale_alpha(0.5),
                            ),
                        }
                    }),
            ])
            .padding(3)
            .width(iced::Length::Fill)
            .height(48),
            container(
                row![
                    column![
                        icon_button(Icon::ChevronUp, 16)
                            .on_press(
                                QueueMessage::Swap(num, num.saturating_sub(1))
                                    .into()
                            )
                            .padding(1)
                            .style(style::plain_icon_button_with_colors(
                                iced::Color::parse("#242226").map(|c| c.into()),
                                None
                            )),
                        icon_button(Icon::ChevronDown, 16)
                            .on_press(
                                QueueMessage::Swap(num, num.saturating_add(1))
                                    .into()
                            )
                            .padding(1)
                            .style(style::plain_icon_button_with_colors(
                                iced::Color::parse("#242226").map(|c| c.into()),
                                None
                            )),
                    ],
                    control_button!(
                        icon: Icon::Trash,
                        msg: QueueMessage::Remove(num).into(),
                        style: style::plain_icon_button,
                    )
                ]
                .height(iced::Length::Fill)
                .align_y(iced::Alignment::Center),
            )
            .align_x(iced::Alignment::End)
            .padding(iced::Padding {
                right: 5.0,
                ..iced::Padding::default()
            })
            .width(iced::Length::Fill)
            .height(iced::Length::Fill),
        )
    }

    pub(super) fn view_queue(&self) -> Element {
        let mut contents = vec![];

        let queue = self
            .queue
            .iter()
            .enumerate()
            .map(|(i, track)| {
                let track = self.library.get_track(*track).unwrap();
                Self::queue_item(i, track)
            })
            .collect::<Vec<_>>();
        contents.push(
            container(scrollable(column(queue)))
                .style(style::track_list_container)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .into(),
        );

        column(contents)
            .width(iced::Length::FillPortion(3))
            .height(iced::Length::Fill)
            .into()
    }

    pub fn update_queue(&mut self, msg: QueueMessage) -> Task<Message> {
        fn get_tracks_from_playlist(app: &App, playlist: u64) -> Vec<u64> {
            app.playlists
                .get_playlist(playlist)
                .unwrap()
                .tracks
                .iter()
                .filter_map(|track| match track {
                    PlaylistTrack::Track(id, _) => Some(*id),
                    _ => None,
                })
                .collect::<Vec<_>>()
        }

        fn shuffle_into_queue(queue: &mut Vec<u64>, tracks: &[u64]) {
            use rand::{rng, seq::SliceRandom};

            queue.resize(tracks.len(), 0);
            let mut shuffle = (0..tracks.len()).collect::<Vec<usize>>();
            shuffle.shuffle(&mut rng());
            for (i, j) in shuffle.into_iter().enumerate() {
                queue[i] = tracks[j];
            }
        }

        match msg {
            QueueMessage::Append(id) => {
                self.queue.push(id);
                Task::none()
            }
            QueueMessage::PlayFolder => {
                let tracks = &self.library.current_directory().tracks;
                self.queue.resize(tracks.len(), 0);
                self.queue.copy_from_slice(&tracks);
                Task::done(Message::PlayNext)
            }
            QueueMessage::PlayList => {
                let Viewing::Playlist(Some(list)) = self.viewing else {
                    return Task::none();
                };
                let tracks = get_tracks_from_playlist(&self, list);
                self.queue.resize(tracks.len(), 0);
                self.queue.copy_from_slice(&tracks);
                Task::done(Message::PlayNext)
            }
            QueueMessage::PlayTrack(i) => {
                self.queue.clear();
                match self.viewing {
                    Viewing::Library => {
                        self.library.current_directory().tracks[i..]
                            .iter()
                            .for_each(|track| self.queue.push(*track));
                        if self.repeat == RepeatStatus::All {
                            self.library.current_directory().tracks[..i]
                                .iter()
                                .for_each(|track| self.queue.push(*track));
                        }
                    }
                    Viewing::Playlist(pl) => unsafe {
                        let id = pl.unwrap_unchecked();
                        let pl =
                            self.playlists.get_playlist(id).unwrap_unchecked();
                        pl.tracks[i..]
                            .iter()
                            .filter_map(|pt| match pt {
                                PlaylistTrack::Unresolved(_) => None,
                                PlaylistTrack::Track(id, _) => Some(*id),
                            })
                            .for_each(|track| self.queue.push(track));
                        if self.repeat == RepeatStatus::All {
                            pl.tracks[..i]
                                .iter()
                                .filter_map(|pt| match pt {
                                    PlaylistTrack::Unresolved(_) => None,
                                    PlaylistTrack::Track(id, _) => Some(*id),
                                })
                                .for_each(|track| self.queue.push(track));
                        }
                    },
                };
                Task::done(Message::PlayNext)
            }
            QueueMessage::Remove(i) => {
                self.queue.remove(i);
                Task::none()
            }
            QueueMessage::Shuffle => {
                use rand::{rng, seq::SliceRandom};

                self.queue.shuffle(&mut rng());
                Task::none()
            }
            QueueMessage::ShuffleFolder => {
                let tracks = &self.library.current_directory().tracks;
                shuffle_into_queue(&mut self.queue, tracks);
                Task::done(Message::PlayNext)
            }
            QueueMessage::ShuffleList => {
                let Viewing::Playlist(Some(list)) = self.viewing else {
                    return Task::none();
                };
                let tracks = get_tracks_from_playlist(&self, list);
                shuffle_into_queue(&mut self.queue, &tracks);
                Task::done(Message::PlayNext)
            }
            QueueMessage::SkipBack => {
                let Some(playing) = &self.playing else {
                    return Task::none();
                };
                match self.repeat {
                    RepeatStatus::None | RepeatStatus::One => {
                        self.sink.try_seek(Duration::from_secs(0));
                        Task::none()
                    }
                    RepeatStatus::All => {
                        let Some((current, _)) = &self.track_duration else {
                            return Task::none();
                        };
                        if current.as_secs() <= 1 && !self.queue.is_empty() {
                            let last =
                                unsafe { self.queue.pop().unwrap_unchecked() };
                            self.queue.insert(0, last);
                            self.queue.insert(1, track_hash(playing));
                            Task::done(Message::PlayNext)
                        } else {
                            self.sink.try_seek(Duration::from_secs(0));
                            Task::none()
                        }
                    }
                }
            }
            QueueMessage::SkipForward => {
                let Some(playing) = &self.playing else {
                    return Task::none();
                };
                if self.repeat == RepeatStatus::All {
                    self.queue.push(track_hash(playing));
                }
                Task::done(Message::PlayNext)
            }
            QueueMessage::Stop => {
                self.play_status = PlayStatus::Stopped;
                self.queue.clear();
                self.stop();
                Task::none()
            }
            QueueMessage::Swap(i, j) => unsafe {
                if j < self.queue.len() {
                    self.queue.swap_unchecked(i, j);
                }
                Task::none()
            },
        }
    }
}
