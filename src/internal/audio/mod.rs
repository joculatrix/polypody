use interflow::prelude::*;


mod flac;
mod mp3;
mod vorbis;
mod wav;

const RINGBUF_CAPACITY: usize = 2048;
const WRITE_THRESHOLD: usize = 8;

// FLAC and WAV samples are encoded in signed PCM format of varying depths,
// whereas Interflow's interface for the audio device takes samples in IEEE
// format: floats in the range [-1.0..1.0].
//
// To convert a sample P of size S in a range [Smin..Smax] (where Smin < 0 and
// Smax > 0) to a sample I in a range [-1.0..1.0]:
//
//       P / -Smin,  P < 0  (sign flipped to avoid positive I for negative P)
// I = { 0,          P == 0
//       P / Smax,   P > 0
//
// On computers, multiplication is much faster than division. Luckily,
// P * (1 / Smax) is equivalent to P / Smax.

const I12_MIN: i32 = -2_048;
const I12_MAX: i32 = 2_047;
const I20_MIN: i32 = -524_288;
const I20_MAX: i32 = 524_287;
const I24_MIN: i32 = -8_388_608;
const I24_MAX: i32 = 8_388_607;

fn pcm_ieee_(sample: f32, min_scale: f32, max_scale: f32) -> f32 {
    if sample < 0.0 {
        sample * min_scale
    } else {
        sample * max_scale
    }
}

fn pcm8_to_ieee(sample: i32) -> f32 {
    const P8MIN_I: f32 = -1.0 / (i8::MIN as f32);
    const P8MAX_I: f32 = 1.0 / (i8::MAX as f32);

    pcm_ieee_(sample as f32, P8MIN_I, P8MAX_I)
}

fn pcm12_to_ieee(sample: i32) -> f32 {
    const P12MIN_I: f32 = -1.0 / (I12_MIN as f32);
    const P12MAX_I: f32 = 1.0 / (I12_MAX as f32);

    pcm_ieee_(sample as f32, P12MIN_I, P12MAX_I)
}

fn pcm16_to_ieee(sample: i32) -> f32 {
    const P16MIN_I: f32 = -1.0 / (i16::MIN as f32);
    const P16MAX_I: f32 = 1.0 / (i16::MAX as f32);

    pcm_ieee_(sample as f32, P16MIN_I, P16MAX_I)
}

fn pcm20_to_ieee(sample: i32) -> f32 {
    const P20MIN_I: f32 = -1.0 / (I20_MIN as f32);
    const P20MAX_I: f32 = 1.0 / (I20_MAX as f32);

    pcm_ieee_(sample as f32, P20MIN_I, P20MAX_I)
}

fn pcm24_to_ieee(sample: i32) -> f32 {
    const P24MIN_I: f32 = -1.0 / (I24_MIN as f32);
    const P24MAX_I: f32 = 1.0 / (I24_MAX as f32);

    pcm_ieee_(sample as f32, P24MIN_I, P24MAX_I)
}

fn pcm32_to_ieee(sample: i32) -> f32 {
    const P32MIN_I: f32 = -1.0 / (i32::MIN as f32);
    const P32MAX_I: f32 = 1.0 / (i32::MAX as f32);

    pcm_ieee_(sample as f32, P32MIN_I, P32MAX_I)
}

#[cfg(test)]
mod test {
    use super::*;
    use paste::paste;

    macro_rules! test_pcm {
        ($pcm:ident, $min:expr, $max:expr) => {
            paste!{
                #[test]
                fn [<$pcm _max_is_correct>]() {
                    assert!([<$pcm _to_ieee>]($max) == 1.0); 
                }
            }

            paste!{
                #[test]
                fn [<$pcm _min_is_correct>]() {
                    assert!([<$pcm _to_ieee>]($min) == -1.0); 
                }
            }
        };
    }

    test_pcm!(pcm8, i8::MIN.into(), i8::MAX.into());
    test_pcm!(pcm12, I12_MIN.into(), I12_MAX.into());
    test_pcm!(pcm16, i16::MIN.into(), i16::MAX.into());
    test_pcm!(pcm20, I20_MIN.into(), I20_MAX.into());
    test_pcm!(pcm24, I24_MIN.into(), I24_MAX.into());
    test_pcm!(pcm32, i32::MIN, i32::MAX);
}
