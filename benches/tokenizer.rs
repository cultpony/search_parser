use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use search_parser::tokenizers::fsm::tokenize as tokenize_fsm;
use search_parser::{span::TokenSpan, tokenizers::dlln::*};

const TEST: &'static str =
    "(((field.gte:1000)AND data.neq:20)||bla.gte:100.2,tag),test.lte:-10,tag";

const TEST997: &'static str = "(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,aba,bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb)";

fn tokenize<'a>(
    expr: &'a str,
    alloc: &'a bumpalo::Bump,
) -> bumpalo::collections::Vec<'a, TokenSpan<'a>> {
    let mut tokenizer = Tokenizer::new(&alloc, expr);
    tokenizer.consume_all().unwrap()
}

fn criterion_benchmark(b: &mut Criterion) {
    let mut c = b.benchmark_group("input-latency-test");
    let c = c.sample_size(500);
    c.throughput(criterion::Throughput::Bytes(TEST.as_bytes().len() as u64));
    c.bench_function(&format!("{}-byte input, standard", TEST.len()), |b| {
        b.iter(|| {
            let alloc = bumpalo::Bump::new();
            let res = tokenize(black_box(TEST), &alloc);
            let _ = black_box(res);
        })
    });
}

fn criterion_benchmark_long_str(b: &mut Criterion) {
    let mut c = b.benchmark_group("input-latency-test");
    let c = c.sample_size(500);
    c.throughput(criterion::Throughput::Bytes(TEST997.as_bytes().len() as u64));
    c.bench_function(&format!("{}-byte input, standard", TEST997.len()), |b| {
        b.iter(|| {
            let alloc = bumpalo::Bump::new();
            let res = tokenize(black_box(TEST997), &alloc);
            let _ = black_box(res);
        })
    });
}

fn criterion_benchmark_exc_long_str(b: &mut Criterion) {
    let mut c = b.benchmark_group("input-latency-test");
    let c = c.sample_size(500);
    let c = c.measurement_time(Duration::from_secs(30));

    let data = TEST997.to_string();
    let data = data.clone() + "," + &data; // 2000
    let data = data.clone() + "," + &data; // 4000
    let data = data.clone() + "," + &data; // 8000
    let data = data.clone() + "," + &data; // 16000
    let data = data.clone() + "," + &data; // 32000
    let data = data.clone() + "," + &data; // 64000
    let data = data.clone() + "," + &data; // 128000
    c.throughput(criterion::Throughput::Bytes(data.as_bytes().len() as u64));
    c.bench_function(&format!("{}-byte input, standard", data.len()), |b| {
        b.iter(|| {
            let alloc = bumpalo::Bump::new();
            let res = tokenize(black_box(&data), &alloc);
            let _ = black_box(res);
        })
    });
}

fn criterion_benchmark_fsm(b: &mut Criterion) {
    let mut c = b.benchmark_group("input-latency-test");
    let c = c.sample_size(500);
    c.throughput(criterion::Throughput::Bytes(TEST.as_bytes().len() as u64));
    c.bench_function(&format!("{}-byte input, FSM", TEST.len()), |b| {
        b.iter(|| {
            let alloc = bumpalo::Bump::new();
            let res = tokenize_fsm(black_box(TEST), &alloc);
            let _ = black_box(res);
        })
    });
}

fn criterion_benchmark_fsm_long_str(b: &mut Criterion) {
    let mut c = b.benchmark_group("input-latency-test");
    let c = c.sample_size(500);
    c.throughput(criterion::Throughput::Bytes(TEST997.as_bytes().len() as u64));
    c.bench_function(&format!("{}-byte input, FSM", TEST997.len()), |b| {
        b.iter(|| {
            let alloc = bumpalo::Bump::new();
            let res = tokenize_fsm(black_box(TEST997), &alloc);
            let _ = black_box(res);
        })
    });
}

fn criterion_benchmark_fsm_exc_long_str(b: &mut Criterion) {
    let mut c = b.benchmark_group("input-latency-test");
    let c = c.sample_size(500);
    let c = c.measurement_time(Duration::from_secs(30));

    let data = TEST997.to_string();
    let data = data.clone() + "," + &data; // 2000
    let data = data.clone() + "," + &data; // 4000
    let data = data.clone() + "," + &data; // 8000
    let data = data.clone() + "," + &data; // 16000
    let data = data.clone() + "," + &data; // 32000
    let data = data.clone() + "," + &data; // 64000
    let data = data.clone() + "," + &data; // 128000
    c.throughput(criterion::Throughput::Bytes(data.as_bytes().len() as u64));
    c.bench_function(&format!("{}-byte input, FSM", data.len()), |b| {
        b.iter(|| {
            let alloc = bumpalo::Bump::new();
            let res = tokenize_fsm(black_box(&data), &alloc);
            let _ = black_box(res);
        })
    });
}

criterion_group!(
    benches,
    criterion_benchmark,
    criterion_benchmark_long_str,
    criterion_benchmark_exc_long_str,
    criterion_benchmark_fsm,
    criterion_benchmark_fsm_long_str,
    criterion_benchmark_fsm_exc_long_str
);
criterion_main!(benches);
