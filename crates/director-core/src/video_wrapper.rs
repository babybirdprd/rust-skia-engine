// Conditional re-export or mock of video-rs types
use anyhow::Result;
use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::thread;

/// Hint for determining how to handle resource loading (e.g. video decoding buffering).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderMode {
    /// Optimized for playback; may drop frames to keep up or buffer opportunistically.
    Preview,
    /// Must render every frame perfectly; blocking operations allowed.
    Export,
}

/// Commands sent to the async video decoder thread.
pub enum VideoCommand {
    /// Request a frame at the specified timestamp (in seconds).
    GetFrame(f64),
}

/// Responses from the async video decoder thread.
#[derive(Debug)]
pub enum VideoResponse {
    /// Decoded frame: timestamp, raw RGBA bytes, width, height.
    Frame(f64, Vec<u8>, u32, u32),
    /// Reached end of video file.
    EndOfStream,
    /// Critical error during decoding.
    Error(String),
}

#[cfg(feature = "video-rs")]
mod real {
    use super::*;
    use ndarray::Array3;
    use video_rs::ffmpeg::{self, codec, format, software, ChannelLayout};
    pub use video_rs::{Location as Locator, Time};

    /// Hardware acceleration options for video encoding.
    #[derive(Debug, Clone, Copy, Default)]
    pub enum HardwareAccel {
        /// Auto-detect best available hardware encoder (NVENC -> QSV -> AMF -> Software)
        #[default]
        Auto,
        /// Force NVIDIA NVENC
        Nvenc,
        /// Force Intel QuickSync
        Qsv,
        /// Force AMD AMF
        Amf,
        /// Force software encoding (libx264)
        Software,
    }

    /// Configuration for video encoding.
    pub struct EncoderSettings {
        pub width: usize,
        pub height: usize,
        pub sample_rate: i32,
        pub hardware_accel: HardwareAccel,
    }

    impl EncoderSettings {
        /// Creates a default H.264 / AAC preset with hardware acceleration.
        pub fn preset_h264_yuv420p(w: usize, h: usize, _b: bool) -> Self {
            Self {
                width: w,
                height: h,
                sample_rate: 48000,
                hardware_accel: HardwareAccel::Auto,
            }
        }

        /// Set hardware acceleration mode.
        pub fn with_hardware_accel(mut self, accel: HardwareAccel) -> Self {
            self.hardware_accel = accel;
            self
        }
    }

    // NOTE: Hardware encoder selection (NVENC/QSV/AMF) requires additional API work.
    // For now, we use software H264. HardwareAccel enum reserved for future use.
    /// A custom encoder wrapping `ffmpeg-next` (via `video-rs` bindings) to support
    /// simultaneous Audio + Video encoding in a single process.
    pub struct Encoder {
        output: format::context::Output,
        video_idx: usize,
        audio_idx: usize,
        video_encoder: codec::encoder::video::Encoder,
        audio_encoder: codec::encoder::audio::Encoder,
        scaler: software::scaling::Context,
        audio_buffer: Vec<f32>,
        audio_samples_processed: i64,
        // Pre-allocated buffers for performance
        rgba_frame: ffmpeg::util::frame::Video,
        yuv_frame: ffmpeg::util::frame::Video,
        audio_left: Vec<f32>,
        audio_right: Vec<f32>,
    }

    impl Encoder {
        /// Initializes the encoder and output file.
        pub fn new(dest: &Locator, settings: EncoderSettings) -> Result<Self> {
            ffmpeg::init().unwrap();

            let path = match dest {
                Locator::File(p) => p,
                _ => return Err(anyhow::anyhow!("Network not supported")),
            };

            let mut output = format::output(&path)?;

            // Video Setup
            let global_header = output
                .format()
                .flags()
                .contains(format::flag::Flags::GLOBAL_HEADER);

            // Log which encoder we're using
            let codec_v =
                codec::encoder::find(codec::Id::H264).ok_or(anyhow::anyhow!("H264 not found"))?;
            tracing::info!("[Encoder] Using codec: H264 (software)");

            let mut v_encoder = codec::context::Context::new_with_codec(codec_v)
                .encoder()
                .video()?;

            v_encoder.set_height(settings.height as u32);
            v_encoder.set_width(settings.width as u32);
            v_encoder.set_aspect_ratio((settings.height as i32, settings.width as i32));
            v_encoder.set_format(format::Pixel::YUV420P);
            v_encoder.set_time_base((1, 90000));

            if global_header {
                v_encoder.set_flags(codec::flag::Flags::GLOBAL_HEADER);
            }

            let v_encoder = v_encoder.open_as(codec_v)?;
            let mut o_stream_v = output.add_stream(codec_v)?;
            o_stream_v.set_parameters(&v_encoder);
            let video_idx = o_stream_v.index();

            // Audio Setup
            let codec_a =
                codec::encoder::find(codec::Id::AAC).ok_or(anyhow::anyhow!("AAC not found"))?;
            let mut a_encoder = codec::context::Context::new_with_codec(codec_a)
                .encoder()
                .audio()?;

            a_encoder.set_rate(settings.sample_rate);
            a_encoder.set_channel_layout(ChannelLayout::STEREO);
            a_encoder.set_format(format::Sample::F32(format::sample::Type::Planar));
            a_encoder.set_time_base((1, settings.sample_rate));

            if global_header {
                a_encoder.set_flags(codec::flag::Flags::GLOBAL_HEADER);
            }

            let a_encoder = a_encoder.open_as(codec_a)?;
            let mut o_stream_a = output.add_stream(codec_a)?;
            o_stream_a.set_parameters(&a_encoder);
            let audio_idx = o_stream_a.index();

            // Scaler
            let scaler = software::scaling::Context::get(
                format::Pixel::RGBA,
                settings.width as u32,
                settings.height as u32,
                format::Pixel::YUV420P,
                settings.width as u32,
                settings.height as u32,
                software::scaling::flag::Flags::BILINEAR,
            )?;

            output.write_header()?;

            // Pre-allocate frame buffers for performance
            let rgba_frame = ffmpeg::util::frame::Video::new(
                format::Pixel::RGBA,
                settings.width as u32,
                settings.height as u32,
            );
            let yuv_frame = ffmpeg::util::frame::Video::new(
                format::Pixel::YUV420P,
                settings.width as u32,
                settings.height as u32,
            );

            // Pre-allocate audio channel buffers (AAC frame size is typically 1024)
            let audio_frame_size = a_encoder.frame_size() as usize;
            let audio_left = vec![0.0f32; audio_frame_size];
            let audio_right = vec![0.0f32; audio_frame_size];

            Ok(Self {
                output,
                video_idx,
                audio_idx,
                video_encoder: v_encoder,
                audio_encoder: a_encoder,
                scaler,
                audio_buffer: Vec::new(),
                audio_samples_processed: 0,
                rgba_frame,
                yuv_frame,
                audio_left,
                audio_right,
            })
        }

        fn write_video_packets(&mut self) -> Result<()> {
            let mut packet = codec::packet::Packet::empty();
            while self.video_encoder.receive_packet(&mut packet).is_ok() {
                packet.set_stream(self.video_idx);
                packet.rescale_ts(
                    self.video_encoder.time_base(),
                    self.output.stream(self.video_idx).unwrap().time_base(),
                );
                packet.write_interleaved(&mut self.output)?;
            }
            Ok(())
        }

        fn write_audio_packets(&mut self) -> Result<()> {
            let mut packet = codec::packet::Packet::empty();
            while self.audio_encoder.receive_packet(&mut packet).is_ok() {
                packet.set_stream(self.audio_idx);
                packet.rescale_ts(
                    self.audio_encoder.time_base(),
                    self.output.stream(self.audio_idx).unwrap().time_base(),
                );
                packet.write_interleaved(&mut self.output)?;
            }
            Ok(())
        }

        /// Encodes a video frame.
        ///
        /// `frame_array` must be RGBA (height, width, 4).
        pub fn encode(&mut self, frame_array: &Array3<u8>, time: Time) -> Result<()> {
            let (h, w, c) = frame_array.dim();
            assert_eq!(c, 4);

            // Reuse pre-allocated RGBA frame buffer
            let stride = self.rgba_frame.stride(0);
            let width_bytes = w * 4;
            let src = frame_array.as_slice().unwrap();

            if stride == width_bytes {
                self.rgba_frame.data_mut(0)[..src.len()].copy_from_slice(src);
            } else {
                for y in 0..h {
                    let src_row = &src[y * width_bytes..(y + 1) * width_bytes];
                    let dest_row =
                        &mut self.rgba_frame.data_mut(0)[y * stride..y * stride + width_bytes];
                    dest_row.copy_from_slice(src_row);
                }
            }

            // Reuse pre-allocated YUV frame buffer
            self.scaler.run(&self.rgba_frame, &mut self.yuv_frame)?;

            let secs = time.as_secs_f64();
            let pts = (secs * 90000.0) as i64;
            self.yuv_frame.set_pts(Some(pts));

            self.video_encoder.send_frame(&self.yuv_frame)?;
            self.write_video_packets()?;
            Ok(())
        }

        /// Encodes audio samples.
        ///
        /// `samples` must be interleaved stereo floats.
        pub fn encode_audio(&mut self, samples: &[f32], _time: Time) -> Result<()> {
            self.audio_buffer.extend_from_slice(samples);

            let frame_size = self.audio_encoder.frame_size() as usize;
            let channels = 2;
            let chunk_size = frame_size * channels;

            while self.audio_buffer.len() >= chunk_size {
                let chunk: Vec<f32> = self.audio_buffer.drain(0..chunk_size).collect();

                let mut frame = ffmpeg::util::frame::Audio::new(
                    format::Sample::F32(format::sample::Type::Planar),
                    frame_size,
                    ChannelLayout::STEREO,
                );

                // Use pre-allocated buffers - deinterleave stereo to L/R
                for i in 0..frame_size {
                    self.audio_left[i] = chunk[i * 2];
                    self.audio_right[i] = chunk[i * 2 + 1];
                }

                frame.plane_mut(0).copy_from_slice(&self.audio_left);
                frame.plane_mut(1).copy_from_slice(&self.audio_right);

                frame.set_pts(Some(self.audio_samples_processed));
                self.audio_samples_processed += frame_size as i64;

                self.audio_encoder.send_frame(&frame)?;
                self.write_audio_packets()?;
            }
            Ok(())
        }

        /// Finalizes the stream, flushing buffers and writing trailers.
        pub fn finish(mut self) -> Result<()> {
            self.video_encoder.send_eof()?;
            self.write_video_packets()?;

            if !self.audio_buffer.is_empty() {
                let frame_size = self.audio_encoder.frame_size() as usize;
                let channels = 2;
                let needed = frame_size * channels - self.audio_buffer.len();
                for _ in 0..needed {
                    self.audio_buffer.push(0.0);
                }
                let chunk = std::mem::take(&mut self.audio_buffer);

                let mut frame = ffmpeg::util::frame::Audio::new(
                    format::Sample::F32(format::sample::Type::Planar),
                    frame_size,
                    ChannelLayout::STEREO,
                );

                let mut left = Vec::with_capacity(frame_size);
                let mut right = Vec::with_capacity(frame_size);
                for i in 0..frame_size {
                    left.push(chunk[i * 2]);
                    right.push(chunk[i * 2 + 1]);
                }
                frame.plane_mut(0).copy_from_slice(&left);
                frame.plane_mut(1).copy_from_slice(&right);
                frame.set_pts(Some(self.audio_samples_processed));

                self.audio_encoder.send_frame(&frame)?;
                self.write_audio_packets()?;
            }

            self.audio_encoder.send_eof()?;
            self.write_audio_packets()?;

            self.output.write_trailer()?;
            Ok(())
        }
    }

    /// Asynchronous video decoder running on a separate thread.
    ///
    /// Useful for pre-fetching frames during preview to avoid stuttering.
    #[derive(Debug)]
    pub struct ThreadedDecoder {
        cmd_tx: Sender<VideoCommand>,
        resp_rx: Receiver<VideoResponse>,
        mode: RenderMode,
    }

    impl ThreadedDecoder {
        pub fn new(path: PathBuf, mode: RenderMode) -> Result<Self> {
            let (cmd_tx, cmd_rx) = unbounded();
            // In Threaded mode (Preview), we use a small buffer.
            let (resp_tx, resp_rx) = bounded(5);

            thread::spawn(move || {
                let mut decoder = match video_rs::Decoder::new(path.clone()) {
                    Ok(d) => d,
                    Err(e) => {
                        let _ = resp_tx.send(VideoResponse::Error(e.to_string()));
                        return;
                    }
                };

                let mut cache: VecDeque<(f64, Vec<u8>, u32, u32)> = VecDeque::with_capacity(15);
                let mut current_decoder_time = 0.0;

                loop {
                    // In Preview/Threaded mode, we skip outdated requests
                    let target_time = match cmd_rx.recv() {
                        Ok(VideoCommand::GetFrame(mut t)) => {
                            while let Ok(VideoCommand::GetFrame(next_t)) = cmd_rx.try_recv() {
                                t = next_t;
                            }
                            t
                        }
                        Err(_) => break,
                    };

                    // Check Cache
                    if let Some(frame_idx) = cache
                        .iter()
                        .position(|(t, _, _, _)| (t - target_time).abs() < 0.02)
                    {
                        let (t, data, w, h) = &cache[frame_idx];
                        if resp_tx
                            .send(VideoResponse::Frame(*t, data.clone(), *w, *h))
                            .is_err()
                        {
                            break;
                        }
                        continue;
                    }

                    // Seek if needed
                    let diff = target_time - current_decoder_time;
                    if diff < -0.1 || diff > 2.0 {
                        let ms = (target_time * 1000.0) as i64;
                        if let Err(_) = decoder.seek(ms) {
                            // In preview, ignore seek errors or just continue
                            continue;
                        }
                        current_decoder_time = target_time;
                    }

                    let max_decode_steps = 60;
                    let mut steps = 0;
                    let mut found = false;

                    loop {
                        match decoder.decode() {
                            Ok((time, frame)) => {
                                steps += 1;
                                let t = time.as_secs_f64();
                                current_decoder_time = t;

                                let shape = frame.shape();
                                if shape.len() == 3 && shape[2] >= 3 {
                                    let h = shape[0] as u32;
                                    let w = shape[1] as u32;
                                    let channels = shape[2];
                                    let (bytes, _) = frame.into_raw_vec_and_offset();

                                    let data = if channels == 3 {
                                        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
                                        for chunk in bytes.chunks(3) {
                                            rgba.extend_from_slice(chunk);
                                            rgba.push(255);
                                        }
                                        rgba
                                    } else {
                                        bytes
                                    };

                                    if cache.len() >= 15 {
                                        cache.pop_front();
                                    }
                                    cache.push_back((t, data.clone(), w, h));

                                    if (t - target_time).abs() < 0.04 {
                                        if resp_tx
                                            .send(VideoResponse::Frame(t, data, w, h))
                                            .is_err()
                                        {
                                            return;
                                        }
                                        found = true;
                                        break;
                                    }
                                }

                                if t > target_time + 0.1 {
                                    break;
                                }
                                if steps > max_decode_steps {
                                    break;
                                }
                            }
                            Err(_) => {
                                // End of stream or error, stop trying to decode
                                break;
                            }
                        }
                    }

                    if !found {
                        // In preview, if we didn't find the frame, we just don't send anything.
                        // The UI will keep the old frame.
                    }
                }
            });

            Ok(Self {
                cmd_tx,
                resp_rx,
                mode,
            })
        }

        pub fn send_request(&self, time: f64) {
            let _ = self.cmd_tx.send(VideoCommand::GetFrame(time));
        }

        pub fn get_response(&self) -> Option<VideoResponse> {
            self.resp_rx.try_recv().ok()
        }
    }

    /// Synchronous video decoder running on the main thread.
    ///
    /// Essential for deterministic Export mode to ensure frame-perfect decoding.
    pub struct SyncDecoder {
        decoder: video_rs::Decoder,
        current_time: f64,
        last_frame: Option<(f64, Vec<u8>, u32, u32)>,
    }

    impl std::fmt::Debug for SyncDecoder {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("SyncDecoder")
                .field("current_time", &self.current_time)
                .field(
                    "last_frame",
                    &self.last_frame.as_ref().map(|(t, _, w, h)| (*t, *w, *h)),
                )
                .finish()
        }
    }

    impl SyncDecoder {
        pub fn new(path: PathBuf) -> Result<Self> {
            let decoder = video_rs::Decoder::new(path)?;
            Ok(Self {
                decoder,
                current_time: -1.0,
                last_frame: None,
            })
        }

        pub fn get_frame_at(&mut self, target_time: f64) -> Result<(f64, Vec<u8>, u32, u32)> {
            // 1. Check if we need to seek
            // If target is behind current, or too far ahead (more than 1 sec), seek.
            if target_time < self.current_time || (target_time - self.current_time) > 1.0 {
                let ms = (target_time * 1000.0) as i64;
                self.decoder.seek(ms)?;
                // Reset current time to slightly before target to account for seek precision
                self.current_time = target_time - 0.1;
            }

            // 2. Decode loop (Blocking)
            let max_steps = 200; // Safety break
            let mut steps = 0;

            loop {
                match self.decoder.decode() {
                    Ok((time, frame)) => {
                        steps += 1;
                        let t = time.as_secs_f64();
                        self.current_time = t;

                        // Check if we reached target
                        // We accept frames that are equal or slightly after target
                        if t >= target_time - 0.01 {
                            let shape_vec = frame.shape().to_vec();
                            let shape = &shape_vec;

                            if shape.len() == 3 && shape[2] >= 3 {
                                let h = shape[0] as u32;
                                let w = shape[1] as u32;
                                let (bytes, _) = frame.into_raw_vec_and_offset();
                                let data = if shape[2] == 3 {
                                    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
                                    for chunk in bytes.chunks(3) {
                                        rgba.extend_from_slice(chunk);
                                        rgba.push(255);
                                    }
                                    rgba
                                } else {
                                    bytes
                                };

                                let result = (t, data, w, h);
                                self.last_frame = Some(result.clone());
                                return Ok(result);
                            }
                        }

                        if steps > max_steps {
                            break;
                        }
                    }
                    Err(_) => {
                        // EOF
                        break;
                    }
                }
            }

            // If we failed to find a new frame (EOF or max steps), return the last one
            if let Some(frame) = &self.last_frame {
                Ok(frame.clone())
            } else {
                Err(anyhow::anyhow!("Could not decode frame at {}", target_time))
            }
        }
    }

    /// Wrapper that selects the backend strategy based on RenderMode.
    #[derive(Debug)]
    pub enum VideoLoader {
        Threaded(ThreadedDecoder),
        Sync(SyncDecoder),
    }

    impl VideoLoader {
        pub fn new(path: PathBuf, mode: RenderMode) -> Result<Self> {
            match mode {
                RenderMode::Preview => Ok(Self::Threaded(ThreadedDecoder::new(path, mode)?)),
                RenderMode::Export => Ok(Self::Sync(SyncDecoder::new(path)?)),
            }
        }
    }
}

#[cfg(feature = "video-rs")]
pub use real::*;
#[cfg(feature = "video-rs")]
pub use video_rs::{ffmpeg, Frame, Time};

#[cfg(not(feature = "video-rs"))]
pub mod mock {
    use super::*;
    use ndarray::Array3;
    use std::path::Path;

    #[derive(Debug)]
    pub struct Decoder;
    impl Decoder {
        pub fn new(_path: &Path) -> Result<Self, String> {
            Ok(Self)
        }
        pub fn decode(&mut self) -> Result<(Time, Array3<u8>), anyhow::Error> {
            Ok((Time, Array3::zeros((10, 10, 3))))
        }
        pub fn seek(&mut self, _ms: i64) -> Result<(), anyhow::Error> {
            Ok(())
        }
    }

    pub struct Encoder;
    impl Encoder {
        pub fn new(_dest: &Locator, _settings: EncoderSettings) -> Result<Self> {
            Ok(Self)
        }
        pub fn finish(self) -> Result<()> {
            Ok(())
        }
        pub fn encode(&mut self, _frame: &Array3<u8>, _time: Time) -> Result<()> {
            Ok(())
        }
        pub fn encode_audio(&mut self, _samples: &[f32], _time: Time) -> Result<()> {
            Ok(())
        }
    }

    pub struct Locator;
    impl From<std::path::PathBuf> for Locator {
        fn from(_: std::path::PathBuf) -> Self {
            Self
        }
    }

    pub struct EncoderSettings;
    impl EncoderSettings {
        pub fn preset_h264_yuv420p(_w: usize, _h: usize, _b: bool) -> Self {
            Self
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Time;
    impl Time {
        pub fn from_nth_of_second(_n: usize, _fps: u32) -> Self {
            Self
        }
        pub fn from_secs(_s: f64) -> Self {
            Self
        }
        pub fn from_secs_f64(_s: f64) -> Self {
            Self
        }
        pub fn as_secs_f64(&self) -> f64 {
            0.0
        }
    }

    pub struct Frame;

    #[derive(Debug)]
    pub struct ThreadedDecoder {
        cmd_tx: Sender<VideoCommand>,
        resp_rx: Receiver<VideoResponse>,
        mode: RenderMode,
    }

    impl ThreadedDecoder {
        pub fn new(_path: PathBuf, mode: RenderMode) -> Result<Self> {
            let (cmd_tx, cmd_rx) = unbounded();
            let (resp_tx, resp_rx) = bounded(5);

            thread::spawn(move || {
                loop {
                    let t = match cmd_rx.recv() {
                        Ok(VideoCommand::GetFrame(t)) => t,
                        Err(_) => break,
                    };
                    thread::sleep(std::time::Duration::from_millis(10));
                    // Return Mock Frame (Red)
                    let _ = resp_tx.send(VideoResponse::Frame(t, vec![255, 0, 0, 255], 1, 1));
                }
            });

            Ok(Self {
                cmd_tx,
                resp_rx,
                mode,
            })
        }
        pub fn send_request(&self, time: f64) {
            let _ = self.cmd_tx.send(VideoCommand::GetFrame(time));
        }
        pub fn get_response(&self) -> Option<VideoResponse> {
            self.resp_rx.try_recv().ok()
        }
    }

    #[derive(Debug)]
    pub struct SyncDecoder {
        last_time: f64,
    }

    impl SyncDecoder {
        pub fn new(_path: PathBuf) -> Result<Self> {
            Ok(Self { last_time: 0.0 })
        }
        pub fn get_frame_at(&mut self, target_time: f64) -> Result<(f64, Vec<u8>, u32, u32)> {
            // Simulate work
            std::thread::sleep(std::time::Duration::from_millis(5));
            self.last_time = target_time;
            // Return dummy red frame
            Ok((target_time, vec![255, 0, 0, 255], 1, 1))
        }
    }

    #[derive(Debug)]
    pub enum VideoLoader {
        Threaded(ThreadedDecoder),
        Sync(SyncDecoder),
    }

    impl VideoLoader {
        pub fn new(path: PathBuf, mode: RenderMode) -> Result<Self> {
            match mode {
                RenderMode::Preview => Ok(Self::Threaded(ThreadedDecoder::new(path, mode)?)),
                RenderMode::Export => Ok(Self::Sync(SyncDecoder::new(path)?)),
            }
        }
    }
}

#[cfg(not(feature = "video-rs"))]
pub use mock::*;
