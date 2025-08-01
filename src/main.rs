#![feature(exact_size_is_empty)]
#![feature(new_range_api)]
#![feature(seek_stream_len)]
#![feature(slice_as_array)]
#![feature(slice_swap_unchecked)]
#![feature(stmt_expr_attributes)]
#![deny(unused_imports)]
#![deny(unused_import_braces)]

use std::{path::PathBuf, time::Duration};

use app::App;
use iced::advanced::graphics::image::image_rs::ImageFormat;
use internal::{
    Track,
    library::{Library, track_hash},
};
use symphonia::core::{codecs::CodecRegistry, probe::Probe};

pub mod app;
pub mod internal;

const APP_ICON: &[u8] = include_bytes!("../res/img/icon.png");

fn main() -> iced::Result {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    iced::application("polypody", App::update, App::view)
        .font(app::ICON_FONT_BYTES)
        .theme(theme)
        .subscription(App::progress_subscription)
        .window(iced::window::Settings {
            min_size: Some([1200.0, 760.0].into()),
            icon: iced::window::icon::from_file_data(
                APP_ICON,
                Some(ImageFormat::Png),
            )
            .ok(),
            ..Default::default()
        })
        .run_with(|| (App::new(stream_handle), iced::Task::none()))
}

fn exe_path() -> std::io::Result<PathBuf> {
    let mut dir = std::env::current_exe()?;
    dir.pop();
    Ok(dir)
}

fn theme(_state: &App) -> iced::Theme {
    iced::Theme::Ferra
}
