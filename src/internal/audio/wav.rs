use super::*;

use hound::{ Sample, SampleFormat, WavReader, WavSpec };

use ringbuf::traits::*;
use ringbuf::{ HeapRb, HeapCons, HeapProd };

use rodio::Source;

use std::io::{ Read, Seek };
use std::path::PathBuf;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;

enum StreamMessage {
    Seek(Duration),
    Vacancy,
}

pub struct WavStream {
    duration: Duration,
    handle: JoinHandle<()>,
    ring_buf_reader: HeapCons<f32>,
    spec: WavSpec,
    tcx: mpsc::Sender<StreamMessage>,
}

impl WavStream {
    pub fn new(path: PathBuf) -> Self {
        let reader = WavReader::open(path).unwrap();
        let spec = reader.spec();

        let duration = Duration::from_secs_f32(
            reader.duration() as f32
                / spec.sample_rate as f32);

        let (format, bits_per_sample) =
            (spec.sample_format, spec.bits_per_sample);

        let (prod, cons) = HeapRb::<f32>::new(RINGBUF_CAPACITY).split();
        let (tcx, rcx) = mpsc::channel(16);

        let handle = tokio::spawn(async move {
            let decoder = match (format, bits_per_sample) {
                (SampleFormat::Float, _) =>
                    Decoder::FloatDecoder(WavDecoder {
                        convert: |x| x,
                        ring_buf_writer: prod,
                        rcx,
                        rcx_buf: Vec::with_capacity(16),
                        wav: reader,
                    }),
                (SampleFormat::Int, n) =>
                    Decoder::IntDecoder(WavDecoder {
                        convert: match n {
                            8  => pcm8_to_ieee,
                            12 => pcm12_to_ieee,
                            16 => pcm16_to_ieee,
                            20 => pcm20_to_ieee,
                            24 => pcm24_to_ieee,
                            32 => pcm32_to_ieee,
                            _ => unreachable!(),
                        },
                        ring_buf_writer: prod,
                        rcx,
                        rcx_buf: Vec::with_capacity(16),
                        wav: reader,
                    }),
            };
            match decoder {
                Decoder::FloatDecoder(mut d) => {
                    d.run().await;
                }
                Decoder::IntDecoder(mut d) => {
                    d.run().await;
                }
            }
        });

        Self {
            duration,
            handle,
            ring_buf_reader: cons,
            spec,
            tcx,
        }
    }
}

impl Iterator for WavStream {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.tcx.blocking_send(StreamMessage::Vacancy);
        let res = self.ring_buf_reader.try_pop();
        if res.is_none() {
            (!self.handle.is_finished()).then(|| 0_f32)
        } else {
            res
        }
    }
}

impl Source for WavStream {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.spec.channels
    }

    fn sample_rate(&self) -> u32 {
        self.spec.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(self.duration)
    }

    fn try_seek(&mut self, pos: Duration)
        -> Result<(), rodio::source::SeekError>
    {
        self.tcx.blocking_send(StreamMessage::Seek(pos));
        Ok(())
    }
}

impl Drop for WavStream {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

enum Decoder<R>
where
    R: Read + Seek
{
    FloatDecoder(WavDecoder<R, f32>),
    IntDecoder(WavDecoder<R, i32>)
}

struct WavDecoder<R, S>
where
    R: Read + Seek,
    S: Sample + Default
{
    convert: fn(S) -> f32,
    ring_buf_writer: HeapProd<f32>,
    rcx: mpsc::Receiver<StreamMessage>,
    rcx_buf: Vec<StreamMessage>,
    wav: WavReader<R>,
}

impl<R, S> WavDecoder<R, S>
where
    R: Read + Seek,
    S: Sample + Default
{
    async fn run(&mut self) {
        let mut seek: Option<Duration>;
        let mut vacancy: bool;
        loop {
            seek = None;
            vacancy = !self.ring_buf_writer.is_full();
            while !vacancy {
                self.rcx_buf.clear();
                let read = self.rcx.recv_many(&mut self.rcx_buf, 16).await;
                self.rcx_buf[..read].iter()
                    .for_each(|msg| match msg {
                        StreamMessage::Seek(target) => {
                            seek = Some(*target);
                        }
                        StreamMessage::Vacancy => {
                            vacancy = true;
                        }
                    });
                seek.inspect(|target| self.seek(*target));
            }
            self.ring_buf_writer.push_iter(
                self.wav.samples()
                    .map(|s|
                        (self.convert)(s.unwrap_or(S::default()))
                    )
            );
            if self.wav.samples::<S>().len() == 0 {
                break;
            }
        }
    }

    fn seek(&mut self, target: Duration) {
        let target = target.as_secs() as u32 * self.wav.spec().sample_rate;
        self.wav.seek(target).unwrap();
    }
}
