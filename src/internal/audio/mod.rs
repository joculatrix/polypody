use symphonia::core::sample::{ i24, u24 };

pub mod output;

const WRITE_THRESHOLD: usize = 8;
const RINGBUF_CAPACITY: usize = 65536 + WRITE_THRESHOLD;

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

const I24_MIN: i32 = -8_388_608;
const I24_MAX: i32 = 8_388_607;
const U24_MAX: i32 = 16_777_215;

fn spcm_ieee_(sample: f32, min_scale: f32, max_scale: f32) -> f32 {
    if sample < 0.0 {
        sample * min_scale
    } else {
        sample * max_scale
    }
}

fn pcm_s8_to_ieee(sample: i8) -> f32 {
    const P_S8MIN_I: f32 = -1.0 / (i8::MIN as f32);
    const P_S8MAX_I: f32 = 1.0 / (i8::MAX as f32);

    spcm_ieee_(sample as f32, P_S8MIN_I, P_S8MAX_I)
}

fn pcm_u8_to_ieee(sample: u8) -> f32 {
    const P_U8MAX_I: f32 = 2.0 / (u8::MAX as f32);
    (sample as f32 * P_U8MAX_I) - 1.0 
}

fn pcm_s16_to_ieee(sample: i16) -> f32 {
    const P_S16MIN_I: f32 = -1.0 / (i16::MIN as f32);
    const P_S16MAX_I: f32 = 1.0 / (i16::MAX as f32);

    spcm_ieee_(sample as f32, P_S16MIN_I, P_S16MAX_I)
}

fn pcm_u16_to_ieee(sample: u16) -> f32 {
    const P_U16MAX_I: f32 = 2.0 / (u16::MAX as f32);
    (sample as f32 * P_U16MAX_I) - 1.0
}

fn pcm_s24_to_ieee(sample: i24) -> f32 {
    const P_S24MIN_I: f32 = -1.0 / (I24_MIN as f32);
    const P_S24MAX_I: f32 = 1.0 / (I24_MAX as f32);

    spcm_ieee_(sample.inner() as f32, P_S24MIN_I, P_S24MAX_I)
}

fn pcm_u24_to_ieee(sample: u24) -> f32 {
    const P_U24MAX_I: f32 = 2.0 / (U24_MAX as f32);
    (sample.inner() as f32 * P_U24MAX_I) - 1.0
}

fn pcm_s32_to_ieee(sample: i32) -> f32 {
    const P_S32MIN_I: f32 = -1.0 / (i32::MIN as f32);
    const P_S32MAX_I: f32 = 1.0 / (i32::MAX as f32);

    spcm_ieee_(sample as f32, P_S32MIN_I, P_S32MAX_I)
}

fn pcm_u32_to_ieee(sample: u32) -> f32 {
    const P_U32MAX_I: f32 = 2.0 / (u32::MAX as f32);
    (sample as f32 * P_U32MAX_I) - 1.0
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

    test_pcm!(pcm_s8, i8::MIN, i8::MAX);
    test_pcm!(pcm_u8, u8::MIN, u8::MAX);
    test_pcm!(pcm_s16, i16::MIN, i16::MAX);
    test_pcm!(pcm_u16, u16::MIN, u16::MAX);
    test_pcm!(pcm_s24, i24::MIN, i24::MAX);
    test_pcm!(pcm_u24, u24::MIN, u24::MAX);
    test_pcm!(pcm_s32, i32::MIN, i32::MAX);
    test_pcm!(pcm_u32, u32::MIN, u32::MAX);
}
