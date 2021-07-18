use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn rms_abs(buffer: &Vec<f32>) -> f32 {
    let mut sum = 0.0;
    for sample in buffer {
        sum += sample.abs();
    }
    sum / buffer.len() as f32
}

fn rms_pow(buffer: &Vec<f32>) -> f32 {
    let mut sum = 0.0;
    for sample in buffer {
        sum += sample * sample;
    }
    sum.sqrt() / buffer.len() as f32
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut oscillator = oscillator::Oscillator::sine(44100.0);
    oscillator.set_frequency(440.0);
    let mut output_buffer = Vec::new();
    output_buffer.resize(400000, 0.0);
    for sample in &mut output_buffer {
        *sample = oscillator.get();
        oscillator.tick();
    }

    c.bench_function("rms using `abs`", |b| {
        b.iter(|| rms_abs(black_box(&mut output_buffer)))
    });
    c.bench_function("rms using `sq root`", |b| {
        b.iter(|| rms_pow(black_box(&mut output_buffer)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
