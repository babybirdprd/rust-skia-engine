//! Audio Processing Tests
//!
//! Tests for audio resampling and mixing.

use director_core::audio::resample_audio;
use std::f32::consts::PI;

/// Test audio resampling from 44.1kHz to 48kHz.
///
/// Generates a sine sweep and verifies resampling produces valid output.
#[test]
fn audio_resampling_44100_to_48000() {
    let source_rate = 44100u32;
    let target_rate = 48000u32;
    let duration_secs = 1; // Shorter for faster test
    let num_samples = (source_rate as usize) * duration_secs;

    // Generate sine sweep (stereo)
    let mut samples: Vec<f32> = Vec::with_capacity(num_samples * 2);
    let start_freq = 440.0f32;

    for i in 0..num_samples {
        let t = i as f32 / source_rate as f32;
        let sample = (2.0 * PI * start_freq * t).sin();
        samples.push(sample); // Left
        samples.push(sample); // Right
    }

    // Resample
    let resampled = resample_audio(&samples, source_rate, target_rate).expect("Resampling failed");

    // Verify output
    let expected_samples = (target_rate as usize) * duration_secs * 2; // stereo
    let tolerance = expected_samples / 10; // 10% tolerance

    assert!(
        (resampled.len() as i64 - expected_samples as i64).abs() < tolerance as i64,
        "Resampled length {} should be close to expected {}",
        resampled.len(),
        expected_samples
    );

    // Verify no NaN or Inf values
    for (i, &sample) in resampled.iter().enumerate() {
        assert!(sample.is_finite(), "Sample {} is not finite: {}", i, sample);
    }
}

/// Test that resampling preserves approximate amplitude.
#[test]
fn audio_resampling_preserves_amplitude() {
    let source_rate = 44100u32;
    let target_rate = 48000u32;

    // Generate simple sine wave at 440Hz
    let num_samples = 4410; // 0.1 seconds
    let mut samples: Vec<f32> = Vec::with_capacity(num_samples * 2);

    for i in 0..num_samples {
        let t = i as f32 / source_rate as f32;
        let sample = (2.0 * PI * 440.0 * t).sin() * 0.5; // Half amplitude
        samples.push(sample);
        samples.push(sample);
    }

    let resampled = resample_audio(&samples, source_rate, target_rate).expect("Resampling failed");

    // Find max amplitude
    let max_amplitude = resampled
        .iter()
        .map(|s| s.abs())
        .fold(0.0f32, |a, b| a.max(b));

    // Should be approximately 0.5 (with some tolerance for interpolation)
    assert!(
        max_amplitude > 0.4 && max_amplitude < 0.6,
        "Max amplitude {} should be close to 0.5",
        max_amplitude
    );
}
