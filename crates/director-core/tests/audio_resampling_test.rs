use director_engine::audio::resample_audio;
use hound;
use std::f32::consts::PI;

#[test]
fn test_audio_resampling_sine_sweep() {
    // 1. Generate Sine Sweep (20Hz - 20kHz) at 44.1kHz
    let source_rate = 44100;
    let target_rate = 48000;
    let duration_secs = 5;
    let num_samples = source_rate * duration_secs;

    let mut samples: Vec<f32> = Vec::with_capacity(num_samples * 2);

    let start_freq = 20.0f32;
    let end_freq = 20000.0f32;
    // Exponential sweep formula: f(t) = f_start * (f_end / f_start)^(t / T)
    // Phase phi(t) = integral(f(t)) = (f_start * T / ln(f_end/f_start)) * ((f_end/f_start)^(t/T) - 1)

    let k = (end_freq / start_freq).ln() / duration_secs as f32;

    for i in 0..num_samples {
        let t = i as f32 / source_rate as f32;
        // Frequency at time t
        // let freq = start_freq * (k * t).exp();

        // Phase calculation for chirp
        let phase = 2.0 * PI * start_freq * ((k * t).exp() - 1.0) / k;

        let sample = phase.sin();

        // Push Stereo
        samples.push(sample); // Left
        samples.push(sample); // Right
    }

    // 2. Resample to 48kHz
    let resampled = resample_audio(&samples, source_rate as u32, target_rate as u32)
        .expect("Resampling failed");

    // 3. Save to WAV using Hound
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: target_rate as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    // Use tempfile to avoid polluting the repo
    let temp_file = tempfile::Builder::new()
        .suffix(".wav")
        .tempfile()
        .unwrap();
    let temp_path = temp_file.path().to_owned();

    let mut writer = hound::WavWriter::create(&temp_path, spec).unwrap();

    for sample in resampled {
        let amplitude = i16::MAX as f32;
        let s = (sample * amplitude).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        writer.write_sample(s).unwrap();
    }

    writer.finalize().unwrap();

    // Verification: Check if file exists and has content
    assert!(temp_path.exists());
    let reader = hound::WavReader::open(&temp_path).unwrap();
    assert_eq!(reader.spec().sample_rate, 48000);
    assert!(reader.duration() > 0);
}
