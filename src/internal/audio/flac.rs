use super::*;

use claxon::{
    frame::FrameReader,
    input::BufferedReader,
    metadata::StreamInfo,
    FlacReader, FlacReaderOptions,
};

use ringbuf::{ traits::*, HeapRb, HeapCons, HeapProd };

use rodio::Source;

use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub enum StreamMessage {
    Seek(Duration),
    VacantLen(usize),
}

pub struct FlacStream {
    ring_buf_reader: HeapCons<f32>,
    handle: JoinHandle<()>,
    stream_info: StreamInfo,
    tcx: mpsc::Sender<StreamMessage>,
}

impl FlacStream {
    pub fn new(path: PathBuf) -> Self {
        let reader = FlacReader::open_ext(
            &path,
            FlacReaderOptions {
                metadata_only: false,
                read_vorbis_comment: false,
            }
        )
            .unwrap();
        let stream_info = reader.streaminfo();
        let blocks = FrameReader::new(BufferedReader::new(reader.into_inner()));

        let ring_buf = HeapRb::<f32>::new(RINGBUF_CAPACITY);
        let (prod, cons) = ring_buf.split();

        let (tcx, rcx) = mpsc::channel(16);

        let (min_block_size, sample_size, sample_rate) =
            (stream_info.min_block_size, stream_info.bits_per_sample,
                stream_info.sample_rate);

        let handle = tokio::spawn(async move {
            let mut decoder = FlacDecoder::new(
                blocks,
                match sample_size {
                    8  => pcm8_to_ieee,
                    12 => pcm12_to_ieee,
                    16 => pcm16_to_ieee,
                    20 => pcm20_to_ieee,
                    24 => pcm24_to_ieee,
                    32 => pcm32_to_ieee,
                    _ => unreachable!(),
                },
                path,
                min_block_size.into(),
                prod,
                rcx,
                sample_rate.into(),
            );
            decoder.run().await;
        });

        Self {
            ring_buf_reader: cons,
            handle,
            stream_info,
            tcx,
        }
    }
}

impl Iterator for FlacStream {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.tcx
            .blocking_send(
                StreamMessage::VacantLen(self.ring_buf_reader.vacant_len())
            );
        let res = self.ring_buf_reader
            .try_pop();
        if res.is_none() {
            (!self.handle.is_finished()).then(|| 0_f32)
        } else {
            res
        }
    }
}

impl Source for FlacStream {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.stream_info
            .channels as u16
    }

    fn sample_rate(&self) -> u32 {
        self.stream_info
            .sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.stream_info
            .samples
            .map(|s|
                Duration::from_secs(
                    s / <u32 as Into<u64>>::into(self.sample_rate())
                )
            )
    }

    fn try_seek(&mut self, pos: Duration)
        -> Result<(), rodio::source::SeekError>
    {
        self.tcx.blocking_send(StreamMessage::Seek(pos)).unwrap();
        Ok(())
    }
}

impl Drop for FlacStream {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

struct FlacDecoder {
    blocks: Option<FrameReader<BufferedReader<File>>>,
    convert: fn(i32) -> f32,
    path: PathBuf,
    read_buf: Option<Vec<i32>>,
    ring_buf_writer: HeapProd<f32>,
    rcx: mpsc::Receiver<StreamMessage>,
    rcx_buf: Vec<StreamMessage>,
    sample_rate: u64,
    samples_read: u64,
}

impl FlacDecoder {
    fn new(
        blocks: FrameReader<BufferedReader<File>>,
        convert: fn(i32) -> f32,
        path: PathBuf,
        read_buf_capacity: usize,
        ring_buf_writer: HeapProd<f32>,
        rcx: mpsc::Receiver<StreamMessage>,
        sample_rate: u64,
    ) -> Self {
        Self {
            blocks: Some(blocks),
            convert,
            path,
            read_buf: Some(Vec::with_capacity(read_buf_capacity)),
            ring_buf_writer,
            rcx,
            rcx_buf: Vec::with_capacity(16),
            sample_rate,
            samples_read: 0,
        }
    }

    async fn run(&mut self) {
        let mut seek: Option<Duration>;
        let mut vacant: usize;
        loop {
            // prep reused variables
            seek = None;
            vacant = self.ring_buf_writer.vacant_len();
            self.rcx_buf.clear();

            let read = self.rcx.recv_many(&mut self.rcx_buf, 16).await;
            self.rcx_buf[..read].iter()
                .for_each(|msg| match msg {
                    StreamMessage::Seek(time) => {
                        seek = Some(*time);
                    }
                    StreamMessage::VacantLen(len) => {
                        vacant = *len;
                    }
                });
            let position_changed = seek.take()
                .map(|seek_pos| self.seek(seek_pos))
                .unwrap_or(false);
            // FlacDecoder::seek() prepares the buffer with the appropriate
            // samples already, so we don't want to discard that and read a
            // new block after seeking
            let samples = if position_changed {
                unsafe {
                    Some(self.read_buf
                        .as_ref()
                        .unwrap_unchecked()
                        .iter()
                        .map(|s| (self.convert)(*s))
                        .collect::<Vec<f32>>())
                }
            } else {
                unsafe {
                    if let Ok(Some(block)) =
                        self.blocks
                            .as_mut()
                            .unwrap_unchecked()
                            .read_next_or_eof(self.read_buf
                                .take()
                                .unwrap_unchecked())
                    {
                        self.read_buf = Some(block.into_buffer());
                        Some(self.read_buf
                            .as_ref()
                            .unwrap_unchecked()
                            .iter()
                            .map(|s| (self.convert)(*s))
                            .collect::<Vec<f32>>())
                    } else {
                        None
                    }
                }
            };
            if let Some(samples) = samples {
                self.dump_packet(&samples).await;
            } else {
                break;
            }
        }        
    }

    fn seek(&mut self, target: Duration) -> bool {
        let current_time =
            Duration::from_secs(self.samples_read / self.sample_rate);
        match target.cmp(&current_time) {
            std::cmp::Ordering::Less => {
                if (current_time - target) < Duration::from_secs(1) {
                    return false;
                }
                { // drop current file reader:
                    let _ = self.blocks.take();
                }
                // use flac reader to skip to main audio data before taking
                // the new reader:
                self.blocks = Some(FrameReader::new(
                    BufferedReader::new(
                        FlacReader::open_ext(
                            self.path.clone(),
                            FlacReaderOptions {
                                metadata_only: false,
                                read_vorbis_comment: false,
                            }
                        )
                            .unwrap()
                            .into_inner()
                    )
                ));
                self.samples_read = 0;
                let target = target.as_secs();
                self.seek_ahead(target);
                true
            }
            std::cmp::Ordering::Equal => false,
            std::cmp::Ordering::Greater => {
                let target = target.as_secs();
                self.seek_ahead(target);
                true
            }
        }
    }

    fn seek_ahead(&mut self, target: u64) {
        loop {
            let Ok(Some(block)) =
                self.blocks
                    .as_mut()
                    .unwrap()
                    .read_next_or_eof(self.read_buf.take().unwrap())
            else {
                todo!()
            };
            
            let block_timestamp = block.time() / self.sample_rate;
            self.samples_read +=
                <u32 as Into<u64>>::into(block.duration());
            
            self.read_buf = Some(block.into_buffer());

            if (block_timestamp..(self.samples_read / self.sample_rate))
                .contains(&target)
            {
                match &mut self.read_buf {
                    Some(buf) => {
                        let i = (target - block_timestamp) * self.sample_rate;
                        *buf = buf.split_off(i as usize);
                    }
                    None => unreachable!(),
                }
                break;
            }
        }
    }

    async fn dump_packet(&mut self, mut samples: &[f32]) {
        while let samples_len = samples.len() && samples_len != 0 {
            if self.ring_buf_writer.is_full() {
                loop {
                    match self.rcx.recv().await {
                        Some(StreamMessage::VacantLen(s))
                            if s >= samples_len || s >= WRITE_THRESHOLD =>
                        {
                            break;
                        }
                        _ => (),
                    }
                }
            }
            let samples_written = self.ring_buf_writer.push_slice(samples);
            if samples_written < samples_len {
                samples = &samples[samples_written..];   
            } else {
                return;
            }
        }
    }
}

