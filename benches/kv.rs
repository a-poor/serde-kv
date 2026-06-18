//! Lightweight benchmarks for the serialize/deserialize hot paths.
//!
//! Run with `cargo bench`.

use std::collections::BTreeMap;
use std::time::Duration;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Record {
    foo: String,
    baz: u64,
    cool: bool,
    message: String,
}

fn benches(c: &mut Criterion) {
    let line = r#"foo=bar baz=42 cool=true message="hello world""#;
    let record = Record {
        foo: "bar".into(),
        baz: 42,
        cool: true,
        message: "hello world".into(),
    };

    c.bench_function("from_str/struct", |b| {
        b.iter(|| {
            let r: Record = serde_kv::from_str(black_box(line)).unwrap();
            black_box(r)
        })
    });

    c.bench_function("from_str/map", |b| {
        b.iter(|| {
            let m: BTreeMap<String, String> =
                serde_kv::from_str(black_box(line)).unwrap();
            black_box(m)
        })
    });

    c.bench_function("to_string/struct", |b| {
        b.iter(|| black_box(serde_kv::to_string(black_box(&record)).unwrap()))
    });

    // A wider line to show roughly linear scaling with field count.
    let wide: String = (0..50)
        .map(|i| format!("key{i}=value{i}"))
        .collect::<Vec<_>>()
        .join(" ");
    c.bench_function("from_str/map_50_fields", |b| {
        b.iter(|| {
            let m: BTreeMap<String, String> =
                serde_kv::from_str(black_box(&wide)).unwrap();
            black_box(m)
        })
    });
}

criterion_group! {
    name = kv;
    config = Criterion::default()
        .sample_size(50)
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(2));
    targets = benches
}
criterion_main!(kv);
