pub use output::AudioStream;
pub use pcm::*;

pub mod output;
pub mod pcm;

const WRITE_THRESHOLD: usize = 8;
const RINGBUF_CAPACITY: usize = 65536 + WRITE_THRESHOLD;

