#![feature(exact_size_is_empty)]
#![feature(new_range_api)]
#![feature(seek_stream_len)]
#![feature(slice_as_array)]

use app::App;
use internal::Track;
use internal::library::{track_hash, Library};
use std::time::Duration;
use symphonia::core::codecs::CodecRegistry;
use symphonia::core::probe::Probe;

pub mod app;
pub mod internal;


#[tokio::main]
async fn main() -> iced::Result {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    iced::application("Music player", App::update, App::view)
        .font(app::ICON_FONT_BYTES)
        .theme(theme)
        .subscription(App::progress_subscription)
        .window(iced::window::Settings {
            min_size: Some([1200.0, 760.0].into()),
            ..Default::default()
        })
        .run_with(|| {
            (App::new(stream_handle), iced::Task::none())
        })
}

fn theme(_state: &App) -> iced::Theme {
    iced::Theme::Ferra
}
