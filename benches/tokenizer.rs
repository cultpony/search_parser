use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use search_parser::tokenizer;
use search_parser::TokenSpan;

const TEST: &'static str =
    "(((field.gte:1000)AND data.neq:20)||bla.gte:100.2,tag),test.lte:-10,tag";

const TEST997: &'static str = "(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,(aa,aba,bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb),bb)";

fn tokenize(
    expr: &str,
) -> Vec<TokenSpan> {
    let mut tokenizer = tokenizer("fsm", expr).unwrap();
    tokenizer.token_spans().unwrap()
}

fn criterion_benchmark(b: &mut Criterion) {
    let mut c = b.benchmark_group("input-latency-test");
    let c = c.sample_size(500);
    c.throughput(criterion::Throughput::Bytes(TEST.as_bytes().len() as u64));
    c.bench_function(&format!("{}-byte input, standard", TEST.len()), |b| {
        b.iter(|| {
            let res = tokenize(black_box(TEST));
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
            let res = tokenize(black_box(TEST997));
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
            let res = tokenize(black_box(&data));
            let _ = black_box(res);
        })
    });
}

criterion_group!(
    benches,
    criterion_benchmark,
    criterion_benchmark_long_str,
    criterion_benchmark_exc_long_str,
);
criterion_main!(benches);
