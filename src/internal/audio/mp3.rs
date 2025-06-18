use super::*;

use ringbuf::{ traits::*, HeapCons, HeapProd, HeapRb };

use rodio::{ Decoder, Source };

use std::fs::File;
use std::io::{ Read, Seek };
use std::path::PathBuf;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;


enum StreamMessage {
    Seek(Duration),
}

pub struct Mp3Stream {
    ring_buf_reader: HeapCons<f32>,
    handle: JoinHandle<()>,
    tcx: mpsc::Sender<StreamMessage>,
    channels: u16,
    sample_rate: u32,
    duration: Duration,
}

impl Mp3Stream {
    pub fn new(path: &PathBuf, duration: Duration) -> Self {
        let reader = Decoder::new_mp3(File::open(path).unwrap()).unwrap();
        let channels = reader.channels();
        let sample_rate = reader.sample_rate();

        let (tcx, rcx) = mpsc::channel(16);
        let (prod, cons) = HeapRb::<f32>::new(RINGBUF_CAPACITY).split();

        let handle = tokio::spawn(async move {
            let mut decoder = Mp3Decoder {
                inner: reader,
                ring_buf_writer: prod,
                rcx,
                rcx_buf: Vec::with_capacity(16),
            };
            decoder.run().await;
        });

        Self {
            ring_buf_reader: cons,
            handle,
            tcx,
            channels,
            sample_rate,
            duration,
        }
    }
}

impl Iterator for Mp3Stream {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.ring_buf_reader.try_pop();
        if res.is_none() {
            (!self.handle.is_finished()).then(|| 0_f32)
        } else {
            res
        }
    }
}

impl Source for Mp3Stream {
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
        Some(self.duration)
    }

    fn try_seek(&mut self, pos: Duration)
        -> Result<(), rodio::source::SeekError>
    {
        self.tcx.blocking_send(StreamMessage::Seek(pos));
        Ok(())
    }
}

struct Mp3Decoder<R>
where
    R: Read + Seek
{
    inner: Decoder<R>,
    ring_buf_writer: HeapProd<f32>,
    rcx: mpsc::Receiver<StreamMessage>,
    rcx_buf: Vec<StreamMessage>,
}

impl<R> Mp3Decoder<R>
where
    R: Read + Seek
{
    async fn run(&mut self) {
        let mut seek: Option<Duration>;
        loop {
            seek = None;
            self.rcx_buf.clear();
            let read = self.rcx.recv_many(&mut self.rcx_buf, 16).await;
            self.rcx_buf[..read].iter()
                .for_each(|msg| match msg {
                    StreamMessage::Seek(target) => {
                        seek = Some(*target);
                    }
                });
            seek.inspect(|target| { self.inner.try_seek(*target); });
            self.ring_buf_writer.push_iter(
                self.inner
                    .by_ref()
                    .map(|s| pcm16_to_ieee(s.into()))
            );
            if self.inner.by_ref().peekable().peek().is_none() {
                break;
            }
        }
    }
}
