use super::*;

use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;

use ringbuf::traits::*;
use ringbuf::{ HeapCons, HeapProd, HeapRb };
use rodio::Source;

use symphonia::core::audio::{ Layout, SampleBuffer };
use symphonia::core::codecs::{ CodecRegistry, Decoder, DecoderOptions };
use symphonia::core::conv::ConvertibleSample;
use symphonia::core::formats::{ FormatOptions, FormatReader, SeekMode, SeekTo };
use symphonia::core::io::{ MediaSourceStream, MediaSourceStreamOptions };
use symphonia::core::meta::{ Limit, MetadataOptions };
use symphonia::core::probe::{ Hint, Probe };
use symphonia::core::sample::SampleFormat;

use tokio::sync::mpsc::{ self, Receiver, Sender };
use tokio::task::JoinHandle;

enum StreamMessage {
    Seek(Duration),
    Vacancy,
}

pub struct AudioStream {
    channels: u16,
    handle: JoinHandle<()>,
    read_counter: usize,
    ring_buf_reader: HeapCons<f32>,
    sample_rate: u32,
    total_duration: Duration,
    tcx: Sender<StreamMessage>,
}

impl AudioStream {
    pub fn new(
        path: &PathBuf,
        codec_registry: &CodecRegistry,
        probe: &Probe,
        duration: Duration,
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
        
        let codec_params = &format.tracks()[0].codec_params;
        let channels = codec_params.channels.unwrap().count() as u16; 
        let sample_rate = codec_params.sample_rate.unwrap();
        
        let (prod, cons) = HeapRb::<f32>::new(RINGBUF_CAPACITY).split();
        let (tcx, rcx) = mpsc::channel(16);

        let sample_format = codec_params.sample_format
            .unwrap_or_else(|| match codec_params.bits_per_sample.unwrap() {
                8  => SampleFormat::S8,
                16 => SampleFormat::S16,
                24 => SampleFormat::S24,
                32 => SampleFormat::S32,
                64 => SampleFormat::F64,
                _  => panic!(),
            });

        let handle = tokio::spawn(async move {
            match sample_format {
                SampleFormat::U8 => {
                    let mut decoder =
                        SymphoniaDecoder::new(decoder, format, rcx, prod,
                            pcm_u8_to_ieee);
                    decoder.run().await;
                }
                SampleFormat::U16 => {
                    let mut decoder =
                        SymphoniaDecoder::new(decoder, format, rcx, prod,
                            pcm_u16_to_ieee);
                    decoder.run().await;
                }
                SampleFormat::U24 => {
                    let mut decoder =
                        SymphoniaDecoder::new(decoder, format, rcx, prod,
                            pcm_u24_to_ieee);
                    decoder.run().await;
                }
                SampleFormat::U32 => {
                    let mut decoder =
                        SymphoniaDecoder::new(decoder, format, rcx, prod,
                            pcm_u32_to_ieee);
                    decoder.run().await;
                }
                SampleFormat::S8 => {
                    let mut decoder =
                        SymphoniaDecoder::new(decoder, format, rcx, prod,
                            pcm_s8_to_ieee);
                    decoder.run().await;
                }
                SampleFormat::S16 => {
                    let mut decoder =
                        SymphoniaDecoder::new(decoder, format, rcx, prod,
                            pcm_s16_to_ieee);
                    decoder.run().await;
                }
                SampleFormat::S24 => {
                    let mut decoder =
                        SymphoniaDecoder::new(decoder, format, rcx, prod,
                            pcm_s24_to_ieee);
                    decoder.run().await;
                }
                SampleFormat::S32 => {
                    let mut decoder =
                        SymphoniaDecoder::new(decoder, format, rcx, prod,
                            pcm_s32_to_ieee);
                    decoder.run().await;
                }
                SampleFormat::F32 => {
                    let mut decoder =
                        SymphoniaDecoder::new(decoder, format, rcx, prod,
                            |x| x);
                    decoder.run().await;
                }
                SampleFormat::F64 => {
                    let mut decoder =
                        SymphoniaDecoder::new(decoder, format, rcx, prod,
                            |x: f64| { x as f32 });
                    decoder.run().await;
                }
            }
        });

        Self {
            channels,
            handle,
            read_counter: 0,
            ring_buf_reader: cons,
            sample_rate,
            total_duration: duration,
            tcx
        }
    }
}

impl Iterator for AudioStream {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_counter += 1;
        if self.read_counter >= (WRITE_THRESHOLD - 1) {
            self.tcx.blocking_send(StreamMessage::Vacancy);
            self.read_counter = 0;
        }
        let res = self.ring_buf_reader.try_pop();
        if res.is_none() {
            (!self.handle.is_finished()).then(|| 0_f32)
        } else {
            res
        }
    }
}

impl Source for AudioStream {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(self.total_duration)
    }

    fn try_seek(&mut self, pos: Duration)
        -> Result<(), rodio::source::SeekError>
    {
        self.tcx.blocking_send(StreamMessage::Seek(pos));
        Ok(())
    }
}

impl Drop for AudioStream {
    fn drop(&mut self) {
        self.handle.abort();
    }
}


struct SymphoniaDecoder<S>
where
    S: ConvertibleSample
{
    convert: fn(S) -> f32,
    inner: Box<dyn Decoder + 'static>,
    format: Box<dyn FormatReader + 'static>,
    rcx: Receiver<StreamMessage>,
    rcx_buf: Vec<StreamMessage>,
    ring_buf_writer: HeapProd<f32>,
}

enum WaitErr {
    ChannelClosed,
    SeekRequested,
}

impl<S: ConvertibleSample> SymphoniaDecoder<S> {
    fn new(
        decoder: Box<dyn Decoder + 'static>,
        format: Box<dyn FormatReader + 'static>,
        rcx: Receiver<StreamMessage>,
        ring_buf_writer: HeapProd<f32>,
        convert: fn(S) -> f32,
    ) -> Self {
        let rcx_buf = Vec::with_capacity(16);
        Self { convert, inner: decoder, format, rcx, rcx_buf, ring_buf_writer }
    }

    async fn run(&mut self) {
        loop {
            self.rcx_buf.clear();
            self.rcx.recv_many(&mut self.rcx_buf, 16).await;
            self.rcx_buf.iter()
                .filter_map(|msg|
                    if let StreamMessage::Seek(target) = msg {
                        Some(*target)
                    } else {
                        None
                    }
                )
                .last()
                .inspect(|target| {
                    self.seek(*target);
                });
            let Some(sample_buf) = self.get_packet() else {
                break;
            };
            let packet_len = sample_buf.len();
            match self.wait_for_vacancy(packet_len).await {
                Ok(_) => {
                    let mut samples = sample_buf
                        .samples()
                        .into_iter()
                        .map(|s| (self.convert)(*s));
                    let written = self.ring_buf_writer
                        .push_iter(samples.by_ref());
                    debug_assert_eq!(written, packet_len);
                }
                Err(WaitErr::SeekRequested) => { continue; }
                Err(WaitErr::ChannelClosed) => { break; }
            }
        }    
    }

    fn get_packet(&mut self) -> Option<SampleBuffer<S>> {
        let packet = self.inner
            .decode(&self.format.next_packet().ok()?)
            .unwrap();
        let spec = packet.spec();
        let mut samples = SampleBuffer::<S>::new(
            packet.frames() as u64,
            *spec,
        );
        samples.copy_interleaved_ref(packet);
        Some(samples)
    }

    fn seek(&mut self, target: Duration) {
        self.format.seek(
            SeekMode::Accurate,
            SeekTo::Time { time: target.into(), track_id: None },
        );
    }

    /// Returns `true` if the decoder received and handled a request to seek.
    async fn wait_for_vacancy(&mut self, required_size: usize)
        -> Result<(), WaitErr>
    {
        let mut vacancy = self.ring_buf_writer.vacant_len();
        while vacancy < required_size {
            tokio::select! {
                msg = self.rcx.recv() => {
                    match msg {
                        Some(StreamMessage::Seek(target)) => {
                            self.seek(target);
                            return Err(WaitErr::SeekRequested);
                        }
                        Some(StreamMessage::Vacancy) => {
                            vacancy = self.ring_buf_writer.vacant_len();
                        }
                        None => {
                            return Err(WaitErr::ChannelClosed);
                        }
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    vacancy = self.ring_buf_writer.vacant_len();
                }
            }
        }
        Ok(())
    }
} 
