use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use lottie_core::animatable::Animator;
use lottie_data::model::{Property, Value, Keyframe};

fn bench_animator_resolve(c: &mut Criterion) {
    let mut group = c.benchmark_group("Animator::resolve");

    // Create 10,000 keyframes
    let count = 10_000;
    let mut keyframes = Vec::with_capacity(count);
    for i in 0..count {
        keyframes.push(Keyframe {
            t: i as f32,
            s: Some(i as f32),
            e: Some((i + 1) as f32),
            i: None,
            o: None,
            to: None,
            ti: None,
            h: None,
        });
    }

    let property = Property {
        a: 1,
        k: Value::Animated(keyframes),
        ix: None,
    };

    let converter = |v: &f32| *v;

    // Test early, middle, late frames
    for &frame in &[100.0, 5000.0, 9990.0] {
        group.bench_with_input(BenchmarkId::new("resolve_frame", frame), &frame, |b, &f| {
            b.iter(|| {
                Animator::resolve(&property, f, converter, 0.0)
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_animator_resolve);
criterion_main!(benches);
