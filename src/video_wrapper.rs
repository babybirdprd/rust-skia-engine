// Conditional re-export or mock of video-rs types
#[cfg(feature = "video-rs")]
pub use video_rs::{
    Decoder, Encoder, Location as Locator, Time, Frame,
    encode::Settings as EncoderSettings,
};

#[cfg(not(feature = "video-rs"))]
pub mod mock {
    use std::path::Path;
    use anyhow::Result;
    use ndarray::Array3;

    #[derive(Debug)]
    pub struct Decoder;
    impl Decoder {
        pub fn new(_path: &Path) -> Result<Self, String> { Ok(Self) }
        // Mock decode: returns time and frame (RGB)
        pub fn decode(&mut self) -> Result<(Time, Array3<u8>), anyhow::Error> {
             Ok((Time, Array3::zeros((10, 10, 3))))
        }
        pub fn seek(&mut self, _ms: i64) -> Result<(), anyhow::Error> { Ok(()) }
    }

    pub struct Encoder;
    impl Encoder {
        pub fn new(_dest: &Locator, _settings: EncoderSettings) -> Result<Self> { Ok(Self) }
        pub fn finish(self) -> Result<()> { Ok(()) }

        pub fn encode(&mut self, _frame: &Array3<u8>, _time: Time) -> Result<()> {
            Ok(())
        }
    }

    pub struct Locator;
    impl From<std::path::PathBuf> for Locator {
        fn from(_: std::path::PathBuf) -> Self { Self }
    }

    pub struct EncoderSettings;
    impl EncoderSettings {
        pub fn preset_h264_yuv420p(_w: usize, _h: usize, _b: bool) -> Self { Self }
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Time;
    impl Time {
        pub fn from_nth_of_second(_n: usize, _fps: u32) -> Self { Self }
        pub fn from_secs(_s: f64) -> Self { Self }
        pub fn from_secs_f64(_s: f64) -> Self { Self }
    }

    pub struct Frame;
}

#[cfg(not(feature = "video-rs"))]
pub use mock::*;
