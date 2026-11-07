extern crate ndk;

use crate::{
    BackendSpecificError, BuildStreamError, PauseStreamError, PlayStreamError, StreamError,
    StreamInstant,
};

/// The [`StreamInstant`] of the present moment on `CLOCK_MONOTONIC`, the clock AAudio states
/// its timestamps on.
pub fn now_stream_instant() -> StreamInstant {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: `ts` is a valid timespec for the call to write.
    let res = unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts) };
    if res != 0 {
        return StreamInstant::new(0, 0);
    }
    StreamInstant::new(ts.tv_sec as i64, ts.tv_nsec as u32)
}

/// The instant `app_frame` is presented (or was captured), projected from a hardware timestamp
/// anchor: frame `anchor_frame` at `anchor_nanos` on `CLOCK_MONOTONIC`.
fn stream_instant_from_anchor(
    anchor_frame: i64,
    anchor_nanos: i64,
    app_frame: i64,
    sample_rate: u32,
) -> StreamInstant {
    let offset_nanos =
        (app_frame as i128 - anchor_frame as i128) * 1_000_000_000 / sample_rate.max(1) as i128;
    let nanos = (anchor_nanos as i128 + offset_nanos).max(0) as u64;
    StreamInstant::new((nanos / 1_000_000_000) as i64, (nanos % 1_000_000_000) as u32)
}

/// When the first frame of the current output callback is presented at the DAC.
pub fn output_stream_instant(stream: &ndk::audio::AudioStream, sample_rate: u32) -> StreamInstant {
    match stream.timestamp(ndk::audio::Clockid::Monotonic) {
        Ok(ts) => stream_instant_from_anchor(
            ts.frame_position,
            ts.time_nanoseconds,
            stream.frames_written(),
            sample_rate,
        ),
        Err(_) => now_stream_instant(),
    }
}

/// When the first frame of the current input callback was captured at the ADC.
pub fn input_stream_instant(stream: &ndk::audio::AudioStream, sample_rate: u32) -> StreamInstant {
    match stream.timestamp(ndk::audio::Clockid::Monotonic) {
        Ok(ts) => stream_instant_from_anchor(
            ts.frame_position,
            ts.time_nanoseconds,
            stream.frames_read(),
            sample_rate,
        ),
        Err(_) => now_stream_instant(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_after_the_anchor_is_presented_after_it_by_its_distance_in_time() {
        // 480 frames at 48 kHz past a frame presented at 1 s: 10 ms later.
        let at = stream_instant_from_anchor(1000, 1_000_000_000, 1480, 48_000);
        assert_eq!(at, StreamInstant::new(1, 10_000_000));
    }

    #[test]
    fn a_frame_before_the_anchor_was_presented_before_it() {
        let at = stream_instant_from_anchor(1480, 1_010_000_000, 1000, 48_000);
        assert_eq!(at, StreamInstant::new(1, 0));
    }
}

impl From<ndk::audio::AudioError> for StreamError {
    fn from(error: ndk::audio::AudioError) -> Self {
        use self::ndk::audio::AudioError::*;
        match error {
            Disconnected | Unavailable => Self::DeviceNotAvailable,
            e => (BackendSpecificError {
                description: e.to_string(),
            })
            .into(),
        }
    }
}

impl From<ndk::audio::AudioError> for PlayStreamError {
    fn from(error: ndk::audio::AudioError) -> Self {
        use self::ndk::audio::AudioError::*;
        match error {
            Disconnected | Unavailable => Self::DeviceNotAvailable,
            e => (BackendSpecificError {
                description: e.to_string(),
            })
            .into(),
        }
    }
}

impl From<ndk::audio::AudioError> for PauseStreamError {
    fn from(error: ndk::audio::AudioError) -> Self {
        use self::ndk::audio::AudioError::*;
        match error {
            Disconnected | Unavailable => Self::DeviceNotAvailable,
            e => (BackendSpecificError {
                description: e.to_string(),
            })
            .into(),
        }
    }
}

impl From<ndk::audio::AudioError> for BuildStreamError {
    fn from(error: ndk::audio::AudioError) -> Self {
        use self::ndk::audio::AudioError::*;
        match error {
            Disconnected | Unavailable => Self::DeviceNotAvailable,
            NoFreeHandles => Self::StreamIdOverflow,
            InvalidFormat | InvalidRate => Self::StreamConfigNotSupported,
            IllegalArgument => Self::InvalidArgument,
            e => (BackendSpecificError {
                description: e.to_string(),
            })
            .into(),
        }
    }
}
