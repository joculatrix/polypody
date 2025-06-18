use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;

use ringbuf::traits::{Observer, Producer};
use ringbuf::{ HeapCons, HeapProd, HeapRb, };
use rodio::Source;

use symphonia::core::audio::{ AudioBuffer, SampleBuffer, Signal, SignalSpec };
use symphonia::core::codecs::{ CodecRegistry, Decoder, DecoderOptions };
use symphonia::core::conv::ConvertibleSample;
use symphonia::core::formats::{ FormatOptions, FormatReader };
use symphonia::core::io::{ MediaSourceStream, MediaSourceStreamOptions };
use symphonia::core::meta::{ Limit, MetadataOptions };
use symphonia::core::probe::{ Hint, Probe };
use symphonia::core::sample::Sample;

use tokio::sync::mpsc::{ Receiver, Sender };

use super::WRITE_THRESHOLD;

enum StreamMessage {
    VacantLen(usize),
}

pub struct TrackStream {
    decoder: Box<dyn Decoder + 'static>,
    format: Box<dyn FormatReader + 'static>,
}

impl TrackStream {
    pub fn new(
        path: &PathBuf,
        codec_registry: &CodecRegistry,
        probe: &Probe,
    ) -> Self {
        let format = probe.format(
            Hint::new()
                .with_extension(path.extension().unwrap().to_str().unwrap()),
            MediaSourceStream::new(
                Box::new(File::open(path).unwrap()),
                MediaSourceStreamOptions::default(),
            ),
            &FormatOptions {
                seek_index_fill_rate: 5,
                ..FormatOptions::default()
            },
            &MetadataOptions {
                limit_metadata_bytes: Limit::Maximum(0), // we already have metadata
                limit_visual_bytes: Limit::Maximum(0), // visuals currently aren't used
            },
        )
            .unwrap()
            .format;
        let decoder = codec_registry.make(
            &format.default_track().unwrap().codec_params,
            &DecoderOptions::default()
        )
            .unwrap();

        Self { decoder, format }
    }
}

async fn decode_packet<S>(
    convert: fn(S) -> f32,
    mut decoder: Box<dyn Decoder + 'static>,
    mut format: Box<dyn FormatReader + 'static>,
    mut rcx: Receiver<StreamMessage>,
    mut ring_buf: HeapProd<f32>,
)
where
    S: ConvertibleSample
{
    let packet = format.next_packet().unwrap();
    let packet = decoder.decode(&packet).unwrap();
    let spec = packet.spec();
    let mut samples = SampleBuffer::<S>::new(
        packet.frames() as u64,
        *spec,
    );
    samples.copy_interleaved_ref(packet);
    let mut samples = &mut samples.samples()
        .into_iter()
        .map(|s| convert(*s))
        .collect::<Vec<f32>>()[..];

    let mut samples_left = samples.len();
    while samples_left != 0 {
        let vacant = ring_buf.vacant_len();
        if vacant >= samples_left || vacant >= WRITE_THRESHOLD {
            let written = ring_buf.push_slice(samples);
            if written < samples_left {
                samples = &mut samples[written..];
            }
            samples_left = samples.len();
        }
        tokio::select! {
            msg = rcx.recv() => match msg {
                Some(StreamMessage::VacantLen(usize)) => {
                    continue;
                }
                None => {
                    return;
                }
            }
        }
    }
}

impl Iterator for TrackStream {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {

    }
}
