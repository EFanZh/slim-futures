use core::future::{self, Future};
use criterion::async_executor::FuturesExecutor;
use criterion::Criterion;
use futures_util::{FutureExt, TryFutureExt};
use slim_futures::future::FutureExt as SlimFutureExt;
use std::future::Ready;
use std::{convert, hint};

// `and_then_async`.

fn benchmark_and_then_async_with<Fut>(
    name: &str,
    c: &mut Criterion,
    mut f: impl FnMut(Ready<Result<u32, ()>>, fn(u32) -> Ready<Result<u32, ()>>) -> Fut,
) where
    Fut: Future<Output = Result<u32, ()>>,
{
    c.bench_function(name, |b| {
        b.to_async(FuturesExecutor).iter(|| {
            f(
                future::ready(hint::black_box(Ok(2))),
                hint::black_box::<fn(_) -> _>(|x| future::ready(Ok(x))),
            )
        })
    });
}

fn benchmark_async_block_and_then_async(c: &mut Criterion) {
    benchmark_and_then_async_with("and_then_async - async block", c, |fut, f| async move {
        f(fut.await?).await
    });
}

fn benchmark_futures_and_then_async(c: &mut Criterion) {
    benchmark_and_then_async_with("and_then_async - futures", c, TryFutureExt::and_then);
}

fn benchmark_slim_futures_and_then_async(c: &mut Criterion) {
    benchmark_and_then_async_with("and_then_async - slim futures", c, SlimFutureExt::slim_and_then_async);
}

criterion::criterion_group!(
    benchmark_and_then_async,
    benchmark_async_block_and_then_async,
    benchmark_futures_and_then_async,
    benchmark_slim_futures_and_then_async,
);

// `map`.

fn benchmark_map_with<Fut>(name: &str, c: &mut Criterion, mut f: impl FnMut(Ready<u32>, fn(u32) -> u32) -> Fut)
where
    Fut: Future<Output = u32>,
{
    c.bench_function(name, |b| {
        b.to_async(FuturesExecutor).iter(|| {
            f(
                future::ready(hint::black_box(2)),
                hint::black_box::<fn(_) -> _>(convert::identity),
            )
        })
    });
}

fn benchmark_async_block_map(c: &mut Criterion) {
    benchmark_map_with("map - async block", c, |fut, f| async move { f(fut.await) });
}

fn benchmark_futures_map(c: &mut Criterion) {
    benchmark_map_with("map - futures", c, FutureExt::map);
}

fn benchmark_slim_futures_map(c: &mut Criterion) {
    benchmark_map_with("map - slim futures", c, SlimFutureExt::slim_map);
}

criterion::criterion_group!(
    benchmark_map,
    benchmark_async_block_map,
    benchmark_futures_map,
    benchmark_slim_futures_map,
);

// `map_async`.

fn benchmark_map_async_with<Fut>(
    name: &str,
    c: &mut Criterion,
    mut f: impl FnMut(Ready<u32>, fn(u32) -> Ready<u32>) -> Fut,
) where
    Fut: Future<Output = u32>,
{
    c.bench_function(name, |b| {
        b.to_async(FuturesExecutor).iter(|| {
            f(
                future::ready(hint::black_box(2)),
                hint::black_box::<fn(_) -> _>(future::ready),
            )
        })
    });
}

fn benchmark_async_block_map_async(c: &mut Criterion) {
    benchmark_map_async_with("map_async - async block", c, |fut, f| async move { f(fut.await).await });
}

fn benchmark_futures_map_async(c: &mut Criterion) {
    benchmark_map_async_with("map_async - futures", c, FutureExt::then);
}

fn benchmark_slim_futures_map_async(c: &mut Criterion) {
    benchmark_map_async_with("map_async - slim futures", c, SlimFutureExt::slim_map_async);
}

criterion::criterion_group!(
    benchmark_map_async,
    benchmark_async_block_map_async,
    benchmark_futures_map_async,
    benchmark_slim_futures_map_async,
);

// `map_err`.

fn benchmark_map_err_with<Fut>(
    name: &str,
    c: &mut Criterion,
    mut f: impl FnMut(Ready<Result<(), u32>>, fn(u32) -> u32) -> Fut,
) where
    Fut: Future<Output = Result<(), u32>>,
{
    c.bench_function(name, |b| {
        b.to_async(FuturesExecutor).iter(|| {
            f(
                future::ready(hint::black_box(Err(2))),
                hint::black_box::<fn(_) -> _>(convert::identity),
            )
        })
    });
}

fn benchmark_async_block_map_err(c: &mut Criterion) {
    benchmark_map_err_with("map_err - async block", c, |fut, f| async move { fut.await.map_err(f) });
}

fn benchmark_futures_map_err(c: &mut Criterion) {
    benchmark_map_err_with("map_err - futures", c, TryFutureExt::map_err);
}

fn benchmark_slim_futures_map_err(c: &mut Criterion) {
    benchmark_map_err_with("map_err - slim futures", c, SlimFutureExt::slim_map_err);
}

criterion::criterion_group!(
    benchmark_map_err,
    benchmark_async_block_map_err,
    benchmark_futures_map_err,
    benchmark_slim_futures_map_err,
);

// `map_ok`.

fn benchmark_map_ok_with<Fut>(
    name: &str,
    c: &mut Criterion,
    mut f: impl FnMut(Ready<Result<u32, ()>>, fn(u32) -> u32) -> Fut,
) where
    Fut: Future<Output = Result<u32, ()>>,
{
    c.bench_function(name, |b| {
        b.to_async(FuturesExecutor).iter(|| {
            f(
                future::ready(hint::black_box(Ok(2))),
                hint::black_box::<fn(_) -> _>(convert::identity),
            )
        })
    });
}

fn benchmark_async_block_map_ok(c: &mut Criterion) {
    benchmark_map_ok_with("map_ok - async block", c, |fut, f| async move { fut.await.map(f) });
}

fn benchmark_futures_map_ok(c: &mut Criterion) {
    benchmark_map_ok_with("map_ok - futures", c, TryFutureExt::map_ok);
}

fn benchmark_slim_futures_map_ok(c: &mut Criterion) {
    benchmark_map_ok_with("map_ok - slim futures", c, SlimFutureExt::slim_map_ok);
}

criterion::criterion_group!(
    benchmark_map_ok,
    benchmark_async_block_map_ok,
    benchmark_futures_map_ok,
    benchmark_slim_futures_map_ok,
);

// `map_map_map`.

fn benchmark_map_map_map_with<Fut>(name: &str, c: &mut Criterion, mut f: impl FnMut(Ready<u32>, fn(u32) -> u32) -> Fut)
where
    Fut: Future<Output = u32>,
{
    c.bench_function(name, |b| {
        b.to_async(FuturesExecutor).iter(|| {
            let identity = hint::black_box::<fn(_) -> _>(convert::identity);

            f(future::ready(hint::black_box(2)), identity)
        })
    });
}

fn benchmark_async_block_map_map_map(c: &mut Criterion) {
    benchmark_map_map_map_with(
        "map_map_map - async block",
        c,
        |fut, f| async move { f(f(f(fut.await))) },
    );
}

fn benchmark_futures_map_map_map(c: &mut Criterion) {
    benchmark_map_map_map_with("map_map_map - futures", c, |fut, f| fut.map(f).map(f).map(f));
}

fn benchmark_slim_futures_map_map_map(c: &mut Criterion) {
    benchmark_map_map_map_with("map_map_map - slim futures", c, |fut, f| {
        fut.slim_map(f).slim_map(f).slim_map(f)
    });
}

criterion::criterion_group!(
    benchmark_map_map_map,
    benchmark_async_block_map_map_map,
    benchmark_futures_map_map_map,
    benchmark_slim_futures_map_map_map,
);

// Main.

criterion::criterion_main!(
    benchmark_and_then_async,
    benchmark_map,
    benchmark_map_async,
    benchmark_map_err,
    benchmark_map_ok,
    benchmark_map_map_map,
);
