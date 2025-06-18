use super::*;

use lewton::header::IdentHeader;
use lewton::inside_ogg::OggStreamReader;

use ringbuf::{ traits::*, HeapCons, HeapProd, HeapRb };

use rodio::Source;

use std::fs::File;
use std::io::{ Read, Seek };
use std::path::PathBuf;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;


enum StreamMessage {
    Seek(Duration),
    Vacancy,
}

pub struct VorbisStream {
    ring_buf_reader: HeapCons<f32>,
    handle: JoinHandle<()>,
    tcx: mpsc::Sender<StreamMessage>,
    channels: u8,
    duration: Option<Duration>,
    sample_rate: u32,
}

impl VorbisStream {
    pub fn new(path: PathBuf, duration: Option<Duration>) -> Self {
        let reader = OggStreamReader::new(File::open(path).unwrap())
            .unwrap();

        let (channels, sample_rate) =
            (reader.ident_hdr.audio_channels, reader.ident_hdr.audio_sample_rate);

        let (prod, cons) = HeapRb::<f32>::new(RINGBUF_CAPACITY).split();
        let (tcx, rcx) = mpsc::channel(16);

        let handle = tokio::spawn(async move {
            let mut decoder = VorbisDecoder {
                reader,
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
            duration,
            sample_rate
        }
    }
}

impl Iterator for VorbisStream {
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

impl Source for VorbisStream {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels.into()
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        self.duration
    }

    fn try_seek(&mut self, pos: Duration)
        -> Result<(), rodio::source::SeekError>
    {
        self.tcx.blocking_send(StreamMessage::Seek(pos));
        Ok(())
    }
}

impl Drop for VorbisStream {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

struct VorbisDecoder<R>
where
    R: Read + Seek
{
    reader: OggStreamReader<R>,
    ring_buf_writer: HeapProd<f32>,
    rcx: mpsc::Receiver<StreamMessage>,
    rcx_buf: Vec<StreamMessage>,
}

impl<R> VorbisDecoder<R>
where
    R: Read + Seek
{
    async fn run(&mut self) {
        let mut seek: Option<Duration>;
        let mut vacancy: usize;
        loop {
            let packet = self.reader.read_dec_packet_itl().unwrap();
            let Some(packet) = packet else {
                break;
            };

            seek = None;
            vacancy = self.ring_buf_writer.vacant_len();
            let packet_len = packet.len();

            while vacancy < packet_len {
                self.rcx_buf.clear();
                let read = self.rcx.recv_many(&mut self.rcx_buf, 16).await;
                let mut samples_read = false;
                self.rcx_buf[..read].iter()
                    .for_each(|msg| match msg {
                        StreamMessage::Seek(target) => {
                            seek = Some(*target);
                        }
                        StreamMessage::Vacancy => {
                            samples_read = true;
                        }
                    });
                if samples_read {
                    vacancy = self.ring_buf_writer.vacant_len();
                }
            }
           
            if let Some(target) = seek {
                self.seek(target);
                continue;
            }

            let res = self.ring_buf_writer
                .push_iter(packet.into_iter()
                    .map(|s| pcm16_to_ieee(s.into())));
            assert_eq!(res, packet_len);
        }
    }

    fn seek(&mut self, target: Duration) {
        let target = target.as_secs()
            * self.reader.ident_hdr.audio_sample_rate as u64;
        self.reader.seek_absgp_pg(target);
    }
}
