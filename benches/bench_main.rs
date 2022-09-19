use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::default_store::benches,
    benchmarks::cf_store::benches,
}
