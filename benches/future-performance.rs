use core::future::{self, Future};
use criterion::async_executor::FuturesExecutor;
use criterion::measurement::Measurement;
use criterion::{BenchmarkGroup, Criterion};
use futures_util::{FutureExt, TryFutureExt};
use slim_futures::future::FutureExt as SlimFutureExt;
use std::future::Ready;
use std::{convert, hint};

// `and_then_async`.

fn benchmark_and_then_async_with<Fut>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut(Ready<Result<u32, ()>>, fn(u32) -> Ready<Result<u32, ()>>) -> Fut,
) where
    Fut: Future<Output = Result<u32, ()>>,
{
    benchmark_group.bench_function(name, |b| {
        b.to_async(FuturesExecutor).iter(|| {
            f(
                future::ready(hint::black_box(Ok(2))),
                hint::black_box::<fn(_) -> _>(|x| future::ready(Ok(x))),
            )
        })
    });
}

fn benchmark_and_then_async(c: &mut Criterion) {
    let mut benchmark_group = c.benchmark_group("future - and_then_async");

    benchmark_and_then_async_with(&mut benchmark_group, "async block", |fut, f| async move {
        f(fut.await?).await
    });

    benchmark_and_then_async_with(&mut benchmark_group, "futures", TryFutureExt::and_then);

    benchmark_and_then_async_with(&mut benchmark_group, "slim-futures", SlimFutureExt::slim_and_then_async);

    benchmark_group.finish()
}

// `map`.

fn benchmark_map_with<Fut>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut(Ready<u32>, fn(u32) -> u32) -> Fut,
) where
    Fut: Future<Output = u32>,
{
    benchmark_group.bench_function(name, |b| {
        b.to_async(FuturesExecutor).iter(|| {
            f(
                future::ready(hint::black_box(2)),
                hint::black_box::<fn(_) -> _>(convert::identity),
            )
        })
    });
}

fn benchmark_map(c: &mut Criterion) {
    let mut benchmark_group = c.benchmark_group("future - map");

    benchmark_map_with(
        &mut benchmark_group,
        "async block",
        |fut, f| async move { f(fut.await) },
    );

    benchmark_map_with(&mut benchmark_group, "futures", FutureExt::map);

    benchmark_map_with(&mut benchmark_group, "slim-futures", SlimFutureExt::slim_map);

    benchmark_group.finish()
}

// `map_async`.

fn benchmark_map_async_with<Fut>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut(Ready<u32>, fn(u32) -> Ready<u32>) -> Fut,
) where
    Fut: Future<Output = u32>,
{
    benchmark_group.bench_function(name, |b| {
        b.to_async(FuturesExecutor).iter(|| {
            f(
                future::ready(hint::black_box(2)),
                hint::black_box::<fn(_) -> _>(future::ready),
            )
        })
    });
}

fn benchmark_map_async(c: &mut Criterion) {
    let mut benchmark_group = c.benchmark_group("future - map_async");

    benchmark_map_async_with(&mut benchmark_group, "async block", |fut, f| async move {
        f(fut.await).await
    });

    benchmark_map_async_with(&mut benchmark_group, "futures", FutureExt::then);

    benchmark_map_async_with(&mut benchmark_group, "slim-futures", SlimFutureExt::slim_map_async);

    benchmark_group.finish()
}

// `map_err`.

fn benchmark_map_err_with<Fut>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut(Ready<Result<(), u32>>, fn(u32) -> u32) -> Fut,
) where
    Fut: Future<Output = Result<(), u32>>,
{
    benchmark_group.bench_function(name, |b| {
        b.to_async(FuturesExecutor).iter(|| {
            f(
                future::ready(hint::black_box(Err(2))),
                hint::black_box::<fn(_) -> _>(convert::identity),
            )
        })
    });
}

fn benchmark_map_err(c: &mut Criterion) {
    let mut benchmark_group = c.benchmark_group("future - map_err");

    benchmark_map_err_with(&mut benchmark_group, "async block", |fut, f| async move {
        fut.await.map_err(f)
    });

    benchmark_map_err_with(&mut benchmark_group, "futures", TryFutureExt::map_err);

    benchmark_map_err_with(&mut benchmark_group, "slim-futures", SlimFutureExt::slim_map_err);

    benchmark_group.finish()
}

// `map_ok`.

fn benchmark_map_ok_with<Fut>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut(Ready<Result<u32, ()>>, fn(u32) -> u32) -> Fut,
) where
    Fut: Future<Output = Result<u32, ()>>,
{
    benchmark_group.bench_function(name, |b| {
        b.to_async(FuturesExecutor).iter(|| {
            f(
                future::ready(hint::black_box(Ok(2))),
                hint::black_box::<fn(_) -> _>(convert::identity),
            )
        })
    });
}

fn benchmark_map_ok(c: &mut Criterion) {
    let mut benchmark_group = c.benchmark_group("future - map_ok");

    benchmark_map_ok_with(
        &mut benchmark_group,
        "async block",
        |fut, f| async move { fut.await.map(f) },
    );

    benchmark_map_ok_with(&mut benchmark_group, "futures", TryFutureExt::map_ok);

    benchmark_map_ok_with(&mut benchmark_group, "slim-futures", SlimFutureExt::slim_map_ok);

    benchmark_group.finish()
}

// `map_chain`.

fn benchmark_map_chain_with<Fut>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut(Ready<u32>, fn(u32) -> u32) -> Fut,
) where
    Fut: Future<Output = u32>,
{
    benchmark_group.bench_function(name, |b| {
        b.to_async(FuturesExecutor).iter(|| {
            let identity = hint::black_box::<fn(_) -> _>(convert::identity);

            f(future::ready(hint::black_box(2)), identity)
        })
    });
}

fn benchmark_map_chain(c: &mut Criterion) {
    let mut benchmark_group = c.benchmark_group("future - map chain");

    benchmark_map_chain_with(&mut benchmark_group, "async block", |fut, f| async move {
        f(f(f(f(fut.await))))
    });

    benchmark_map_chain_with(&mut benchmark_group, "futures", |fut, f| {
        fut.map(f).map(f).map(f).map(f)
    });

    benchmark_map_chain_with(&mut benchmark_group, "slim-futures", |fut, f| {
        fut.slim_map(f).slim_map(f).slim_map(f).slim_map(f)
    });

    benchmark_group.finish()
}

// Main.

criterion::criterion_group!(
    benchmarks,
    benchmark_and_then_async,
    benchmark_map,
    benchmark_map_async,
    benchmark_map_err,
    benchmark_map_ok,
    benchmark_map_chain,
);

criterion::criterion_main!(benchmarks);
