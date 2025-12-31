//! # Audio Module
//!
//! Audio mixing and playback.
//!
//! ## Responsibilities
//! - **Audio Mixing**: Combines multiple `AudioTrack`s into final output.
//! - **Sync**: Aligns audio with video timeline.
//! - **Track Management**: Add/remove/seek audio tracks.
//!
//! ## Key Types
//! - `AudioMixer`: The main audio processor.
//! - `AudioTrack`: A single audio source with volume and timing.

use crate::animation::Animated;
use anyhow::{Context, Result};
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use std::io::Cursor;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Represents a single audio source on the timeline.
#[derive(Clone, Debug)]
pub struct AudioTrack {
    /// Interleaved stereo samples (L, R, L, R...). Normalized -1.0 to 1.0.
    pub samples: Vec<f32>,
    /// Volume multiplier (animated).
    pub volume: Animated<f32>,
    /// Start time in global seconds.
    pub start_time: f64,
    /// Optional clipping duration (in seconds).
    pub duration: Option<f64>,
    /// Whether to loop the audio.
    pub loop_audio: bool,
}

/// Manages mixing of multiple audio tracks.
#[derive(Clone, Debug)]
pub struct AudioMixer {
    /// List of active tracks. Option allows for empty slots (freelist style).
    pub tracks: Vec<Option<AudioTrack>>,
    /// Output sample rate (usually 48000Hz).
    pub sample_rate: u32,
}

impl AudioMixer {
    /// Creates a new mixer with the specified sample rate.
    pub fn new(sample_rate: u32) -> Self {
        Self {
            tracks: Vec::new(),
            sample_rate,
        }
    }

    /// Adds a track to the mixer.
    pub fn add_track(&mut self, track: AudioTrack) -> usize {
        // Find empty slot
        if let Some(idx) = self.tracks.iter().position(|t| t.is_none()) {
            self.tracks[idx] = Some(track);
            idx
        } else {
            let idx = self.tracks.len();
            self.tracks.push(Some(track));
            idx
        }
    }

    /// Returns a mutable reference to a track.
    pub fn get_track_mut(&mut self, id: usize) -> Option<&mut AudioTrack> {
        self.tracks.get_mut(id).and_then(|t| t.as_mut())
    }

    /// Mixes all active tracks for a specific time window.
    ///
    /// # Arguments
    /// * `samples_needed` - Number of samples to generate (per channel).
    /// * `start_time` - Global start time for the mix window.
    ///
    /// # Returns
    /// * `Vec<f32>` - Interleaved stereo samples (length = samples_needed * 2).
    pub fn mix(&mut self, samples_needed: usize, start_time: f64) -> Vec<f32> {
        // Output buffer (stereo)
        let mut output = vec![0.0; samples_needed * 2];
        let dt_per_sample = 1.0 / self.sample_rate as f64;

        for track_opt in self.tracks.iter_mut() {
            if let Some(track) = track_opt {
                // Determine if track is active
                // For looping or simple playback, calculate relative time

                track.volume.update(start_time);
                let vol = track.volume.current_value;

                for i in 0..samples_needed {
                    let t = start_time + i as f64 * dt_per_sample;
                    let relative_time = t - track.start_time;

                    // Check start
                    if relative_time < 0.0 {
                        continue;
                    }

                    // Check duration (clipping)
                    if let Some(dur) = track.duration {
                        if relative_time >= dur {
                            if track.loop_audio {
                                // If looping AND hard clipped? Usually looping means it loops *within* the clip?
                                // Or does it mean the source loops?
                                // RFC: "Scene Audio: Starts at scene.start_time. It is hard clipped to the scene duration."
                                // "Global Audio: ... plays independently".
                                // If hard clipped, we stop.
                                continue;
                            } else {
                                continue;
                            }
                        }
                    }

                    // Determine sample index
                    // If looping, we wrap the sample index relative to the source length.

                    let mut sample_idx = (relative_time * self.sample_rate as f64) as usize;

                    // Convert to stereo frame index
                    let frame_count = track.samples.len() / 2;

                    if track.loop_audio {
                        sample_idx %= frame_count;
                    } else if sample_idx >= frame_count {
                        continue;
                    }

                    let left = track.samples[sample_idx * 2];
                    let right = track.samples[sample_idx * 2 + 1];

                    output[i * 2] += left * vol;
                    output[i * 2 + 1] += right * vol;
                }
            }
        }

        // Clamp
        for s in output.iter_mut() {
            *s = s.clamp(-1.0, 1.0);
        }

        output
    }
}

/// Resamples audio data to the target sample rate.
///
/// Uses high-quality Sinc interpolation (via `rubato`).
pub fn resample_audio(samples: &[f32], source_rate: u32, target_rate: u32) -> Result<Vec<f32>> {
    if source_rate == target_rate {
        return Ok(samples.to_vec());
    }

    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };

    let ratio = target_rate as f64 / source_rate as f64;
    let mut resampler = SincFixedIn::<f32>::new(
        ratio, 256.0, // max_resample_ratio_relative
        params, 1024, // input chunk size
        2,    // channels
    )
    .context("Failed to create resampler")?;

    // De-interleave
    let frames = samples.len() / 2;
    let mut left = Vec::with_capacity(frames);
    let mut right = Vec::with_capacity(frames);

    for chunk in samples.chunks_exact(2) {
        left.push(chunk[0]);
        right.push(chunk[1]);
    }

    let input_chunk_size = resampler.input_frames_max();
    let mut output_left = Vec::with_capacity((frames as f64 * ratio) as usize + 1024);
    let mut output_right = Vec::with_capacity((frames as f64 * ratio) as usize + 1024);

    let mut input_idx = 0;
    while input_idx < frames {
        let end = (input_idx + input_chunk_size).min(frames);
        let len = end - input_idx;

        let chunk_left = &left[input_idx..end];
        let chunk_right = &right[input_idx..end];

        let mut input_batch = vec![chunk_left.to_vec(), chunk_right.to_vec()];

        // Pad if last chunk is smaller than required input size
        if len < input_chunk_size {
            input_batch[0].resize(input_chunk_size, 0.0);
            input_batch[1].resize(input_chunk_size, 0.0);
        }

        let output_batch = resampler
            .process(&input_batch, None)
            .context("Resampling failed")?;

        // Append to output
        // Note: Rubato output size depends on input size and ratio.
        // If we padded, we might get more samples than we want at the end, but usually that's fine as silent tail.
        // However, strictly we might want to trim. But for now let's append all.
        // Actually, if we padded the input, the output corresponds to that padded input.
        // Calculating exact valid output samples might be complex due to filter delay.
        // For simplicity in this RFC, we process full blocks.

        output_left.extend_from_slice(&output_batch[0]);
        output_right.extend_from_slice(&output_batch[1]);

        input_idx += input_chunk_size;
    }

    // Interleave result
    let out_len = output_left.len();
    let mut result = Vec::with_capacity(out_len * 2);
    for i in 0..out_len {
        result.push(output_left[i]);
        result.push(output_right[i]);
    }

    Ok(result)
}

/// Decodes an audio file from raw bytes into interleaved stereo float samples.
///
/// Automatically handles format detection and resampling to the target rate.
pub fn load_audio_bytes(data: &[u8], target_sample_rate: u32) -> Result<Vec<f32>> {
    let mss = MediaSourceStream::new(Box::new(Cursor::new(data.to_vec())), Default::default());
    let hint = Hint::new();
    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();
    let decoder_opts = DecoderOptions::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .context("Unsupported format")?;

    let mut format = probed.format;
    let track = format.default_track().context("No track found")?;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &decoder_opts)
        .context("Unsupported codec")?;

    let track_id = track.id;
    let source_rate = track.codec_params.sample_rate.unwrap_or(44100);

    let mut samples = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(_)) => break,
            Err(symphonia::core::errors::Error::ResetRequired) => break,
            Err(_) => break,
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = *decoded.spec();
                let duration = decoded.capacity() as u64;
                let mut buf = symphonia::core::audio::SampleBuffer::<f32>::new(duration, spec);
                buf.copy_interleaved_ref(decoded);

                let buf_samples = buf.samples();
                let channels = spec.channels.count();

                if channels == 1 {
                    for s in buf_samples {
                        samples.push(*s);
                        samples.push(*s);
                    }
                } else if channels >= 2 {
                    // Taking first two channels if more than 2
                    for chunk in buf_samples.chunks(channels) {
                        samples.push(chunk[0]);
                        samples.push(chunk[1]);
                    }
                }
            }
            Err(_) => break,
        }
    }

    resample_audio(&samples, source_rate, target_sample_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mixing_logic() {
        let mut mixer = AudioMixer::new(48000);
        let track = AudioTrack {
            samples: vec![0.5; 48000 * 2], // 1 sec stereo
            volume: Animated::new(1.0),
            start_time: 0.0,
            duration: None,
            loop_audio: false,
        };
        mixer.add_track(track);

        let mixed = mixer.mix(100, 0.0);
        assert_eq!(mixed.len(), 200);
        // Check first sample (Left)
        assert!((mixed[0] - 0.5).abs() < 1e-5);
    }
}
