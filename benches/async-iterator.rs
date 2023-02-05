use criterion::async_executor::FuturesExecutor;
use criterion::measurement::Measurement;
use criterion::{BenchmarkGroup, Criterion};
use fn_traits::fns::CopyFn;
use futures_core::Stream;
use futures_util::stream::Iter;
use futures_util::{stream, StreamExt, TryStreamExt};
use slim_futures::async_iter::AsyncIteratorExt;
use slim_futures::future;
use std::future::Future;
use std::iter::Map;
use std::ops::Range;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{convert, hint};

type Ready<T> = future::Ready<CopyFn, T>;
type StreamType<R> = NoInlineStream<Iter<Map<Range<usize>, fn(usize) -> R>>>;

pin_project_lite::pin_project! {
    struct NoInlineStream<S>{
        #[pin]
        inner: S,
    }
}

impl<S> Stream for NoInlineStream<S>
where
    S: Stream,
{
    type Item = S::Item;

    #[inline(never)]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }

    #[inline(never)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

fn gen_stream<R>(f: fn(usize) -> R) -> StreamType<R> {
    hint::black_box(NoInlineStream {
        inner: stream::iter((0..1_000_000).map(f)),
    })
}

fn benchmark_async_iter_with<I>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut() -> I,
) where
    I: Stream,
{
    hint::black_box::<fn(_, _) -> _>(I::poll_next);

    benchmark_group.bench_function(name, |b| {
        b.to_async(FuturesExecutor).iter(|| {
            let iter = f();

            async {
                futures_util::pin_mut!(iter);

                while let Some(item) = iter.next().await {
                    hint::black_box(item);
                }
            }
        })
    });
}

fn benchmark_future_with<Fut>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut() -> Fut,
) where
    Fut: Future,
{
    hint::black_box::<fn(_, _) -> _>(Fut::poll);

    benchmark_group.bench_function(name, |b| b.to_async(FuturesExecutor).iter(&mut f));
}

// `and_then`.

fn benchmark_and_then_with<I>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut(StreamType<Result<usize, usize>>, fn(usize) -> Result<usize, usize>) -> I,
) where
    I: Stream<Item = Result<usize, usize>>,
{
    benchmark_async_iter_with(benchmark_group, name, || {
        f(
            gen_stream(|x| if x % 2 == 0 { Ok(x) } else { Err(x) }),
            hint::black_box::<fn(_) -> _>(|x| if x % 4 == 0 { Ok(x) } else { Err(x) }),
        )
    });
}

fn benchmark_and_then(c: &mut Criterion<impl Measurement>) {
    let mut benchmark_group = c.benchmark_group("async iter/and_then");

    benchmark_and_then_with(&mut benchmark_group, "futures", |iter, f| {
        iter.map(move |item| item.and_then(f))
    });

    benchmark_and_then_with(&mut benchmark_group, "slim-futures", AsyncIteratorExt::slim_and_then);

    benchmark_group.finish()
}

// `and_then_async`.

fn benchmark_and_then_async_with<I>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut(StreamType<Result<usize, usize>>, fn(usize) -> Ready<Result<usize, usize>>) -> I,
) where
    I: Stream<Item = Result<usize, usize>>,
{
    benchmark_async_iter_with(benchmark_group, name, || {
        f(
            gen_stream(|x| if x % 2 == 0 { Ok(x) } else { Err(x) }),
            hint::black_box::<fn(_) -> _>(|x| future::ready_by_copy(if x % 4 == 0 { Ok(x) } else { Err(x) })),
        )
    });
}

fn benchmark_and_then_async(c: &mut Criterion<impl Measurement>) {
    let mut benchmark_group = c.benchmark_group("async iter/and_then_async");

    benchmark_and_then_async_with(&mut benchmark_group, "futures", TryStreamExt::and_then);

    benchmark_and_then_async_with(
        &mut benchmark_group,
        "slim-futures",
        AsyncIteratorExt::slim_and_then_async,
    );

    benchmark_group.finish()
}

// `filter_async`.

fn benchmark_filter_async_with<I>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut(StreamType<usize>, fn(&usize) -> Ready<bool>) -> I,
) where
    I: Stream<Item = usize>,
{
    benchmark_async_iter_with(benchmark_group, name, || {
        f(
            gen_stream(convert::identity),
            hint::black_box::<fn(&_) -> _>(|&x| future::ready_by_copy(x % 2 == 0)),
        )
    });
}

fn benchmark_filter_async(c: &mut Criterion<impl Measurement>) {
    let mut benchmark_group = c.benchmark_group("async iter/filter_async");

    benchmark_filter_async_with(&mut benchmark_group, "futures", StreamExt::filter);

    benchmark_filter_async_with(
        &mut benchmark_group,
        "slim-futures",
        AsyncIteratorExt::slim_filter_async,
    );

    benchmark_group.finish()
}

// `filter_map_async`.

fn benchmark_filter_map_async_with<I>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut(StreamType<usize>, fn(usize) -> Ready<Option<usize>>) -> I,
) where
    I: Stream<Item = usize>,
{
    benchmark_async_iter_with(benchmark_group, name, || {
        f(
            gen_stream(convert::identity),
            hint::black_box::<fn(_) -> _>(|x| future::ready_by_copy((x % 2 == 0).then_some(x))),
        )
    });
}

fn benchmark_filter_map_async(c: &mut Criterion<impl Measurement>) {
    let mut benchmark_group = c.benchmark_group("async iter/filter_map_async");

    benchmark_filter_map_async_with(&mut benchmark_group, "futures", StreamExt::filter_map);

    benchmark_filter_map_async_with(
        &mut benchmark_group,
        "slim-futures",
        AsyncIteratorExt::slim_filter_map_async,
    );

    benchmark_group.finish()
}

// `fold_async`.

fn benchmark_fold_async_with<Fut>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut(StreamType<usize>, usize, fn(usize, usize) -> Ready<usize>) -> Fut,
) where
    Fut: Future<Output = usize>,
{
    benchmark_future_with(benchmark_group, name, || {
        f(
            gen_stream(convert::identity),
            0,
            hint::black_box::<fn(_, _) -> _>(|x, y| future::ready_by_copy(x ^ y)),
        )
    });
}

fn benchmark_fold_async(c: &mut Criterion<impl Measurement>) {
    let mut benchmark_group = c.benchmark_group("async iter/fold_async");

    benchmark_fold_async_with(
        &mut benchmark_group,
        "async block",
        |mut iter, mut init, f| async move {
            while let Some(item) = iter.next().await {
                init = f(init, item).await;
            }

            init
        },
    );

    benchmark_fold_async_with(&mut benchmark_group, "futures", StreamExt::fold);

    benchmark_fold_async_with(
        &mut benchmark_group,
        "slim-futures/clone",
        AsyncIteratorExt::slim_fold_async_by_clone,
    );

    benchmark_fold_async_with(
        &mut benchmark_group,
        "slim-futures/copy",
        AsyncIteratorExt::slim_fold_async_by_copy,
    );

    benchmark_fold_async_with(
        &mut benchmark_group,
        "slim-futures/take",
        AsyncIteratorExt::slim_fold_async_by_take,
    );

    benchmark_group.finish()
}

// `for_each_async`.

fn benchmark_for_each_async_with<Fut>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut(StreamType<usize>, fn(usize) -> Ready<()>) -> Fut,
) where
    Fut: Future<Output = ()>,
{
    benchmark_future_with(benchmark_group, name, || {
        f(
            gen_stream(convert::identity),
            hint::black_box::<fn(_) -> _>(|_| future::ready_by_copy(())),
        )
    });
}

fn benchmark_for_each_async(c: &mut Criterion<impl Measurement>) {
    let mut benchmark_group = c.benchmark_group("async iter/for_each_async");

    benchmark_for_each_async_with(&mut benchmark_group, "async block", |mut iter, f| async move {
        while let Some(item) = iter.next().await {
            f(item).await;
        }
    });

    benchmark_for_each_async_with(&mut benchmark_group, "futures", StreamExt::for_each);

    benchmark_for_each_async_with(
        &mut benchmark_group,
        "slim-futures",
        AsyncIteratorExt::slim_for_each_async,
    );

    benchmark_group.finish()
}

// `map`.

fn benchmark_map_with<I>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut(StreamType<usize>, fn(usize) -> usize) -> I,
) where
    I: Stream<Item = usize>,
{
    benchmark_async_iter_with(benchmark_group, name, || {
        f(
            gen_stream(convert::identity),
            hint::black_box::<fn(_) -> _>(convert::identity),
        )
    });
}

fn benchmark_map(c: &mut Criterion<impl Measurement>) {
    let mut benchmark_group = c.benchmark_group("async iter/map");

    benchmark_map_with(&mut benchmark_group, "futures", StreamExt::map);
    benchmark_map_with(&mut benchmark_group, "slim-futures", AsyncIteratorExt::slim_map);

    benchmark_group.finish()
}

// `map_async`.

fn benchmark_map_async_with<I>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut(StreamType<usize>, fn(usize) -> Ready<usize>) -> I,
) where
    I: Stream<Item = usize>,
{
    benchmark_async_iter_with(benchmark_group, name, || {
        f(
            gen_stream(convert::identity),
            hint::black_box::<fn(_) -> _>(future::ready_by_copy),
        )
    });
}

fn benchmark_map_async(c: &mut Criterion<impl Measurement>) {
    let mut benchmark_group = c.benchmark_group("async iter/map_async");

    benchmark_map_async_with(&mut benchmark_group, "futures", StreamExt::then);
    benchmark_map_async_with(&mut benchmark_group, "slim-futures", AsyncIteratorExt::slim_map_async);

    benchmark_group.finish()
}

// `map_err`.

fn benchmark_map_err_with<I>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut(StreamType<Result<usize, usize>>, fn(usize) -> usize) -> I,
) where
    I: Stream<Item = Result<usize, usize>>,
{
    benchmark_async_iter_with(benchmark_group, name, || {
        f(
            gen_stream(|x| if x % 2 == 0 { Ok(x) } else { Err(x) }),
            hint::black_box::<fn(_) -> _>(convert::identity),
        )
    });
}

fn benchmark_map_err(c: &mut Criterion<impl Measurement>) {
    let mut benchmark_group = c.benchmark_group("async iter/map_err");

    benchmark_map_err_with(&mut benchmark_group, "futures", TryStreamExt::map_err);
    benchmark_map_err_with(&mut benchmark_group, "slim-futures", AsyncIteratorExt::slim_map_err);

    benchmark_group.finish()
}

// `map_ok`.

fn benchmark_map_ok_with<I>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut(StreamType<Result<usize, usize>>, fn(usize) -> usize) -> I,
) where
    I: Stream<Item = Result<usize, usize>>,
{
    benchmark_async_iter_with(benchmark_group, name, || {
        f(
            gen_stream(|x| if x % 2 == 0 { Ok(x) } else { Err(x) }),
            hint::black_box::<fn(_) -> _>(convert::identity),
        )
    });
}

fn benchmark_map_ok(c: &mut Criterion<impl Measurement>) {
    let mut benchmark_group = c.benchmark_group("async iter/map_ok");

    benchmark_map_ok_with(&mut benchmark_group, "futures", TryStreamExt::map_ok);
    benchmark_map_ok_with(&mut benchmark_group, "slim-futures", AsyncIteratorExt::slim_map_ok);

    benchmark_group.finish()
}

// `scan_async`.

fn benchmark_scan_async_with<I>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut(StreamType<usize>, (), fn(&mut (), usize) -> Ready<Option<usize>>) -> I,
) where
    I: Stream<Item = usize>,
{
    benchmark_async_iter_with(benchmark_group, name, || {
        f(
            gen_stream(convert::identity),
            (),
            hint::black_box::<fn(&mut _, _) -> _>(|_, x| future::ready_by_copy(Some(x))),
        )
    });
}

fn benchmark_scan_async(c: &mut Criterion<impl Measurement>) {
    let mut benchmark_group = c.benchmark_group("async iter/scan_async");

    benchmark_scan_async_with(&mut benchmark_group, "futures", StreamExt::scan);
    benchmark_scan_async_with(&mut benchmark_group, "slim-futures", AsyncIteratorExt::slim_scan_async);

    benchmark_group.finish()
}

// `try_fold_async`.

fn benchmark_try_fold_async_with<Fut>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut(StreamType<Result<usize, ()>>, usize, fn(usize, usize) -> Ready<Result<usize, ()>>) -> Fut,
) where
    Fut: Future<Output = Result<usize, ()>>,
{
    benchmark_future_with(benchmark_group, name, || {
        f(
            gen_stream(Ok),
            0,
            hint::black_box::<fn(_, _) -> _>(|x, y| future::ready_by_copy(Ok(x ^ y))),
        )
    });
}

fn benchmark_try_fold_async(c: &mut Criterion<impl Measurement>) {
    fn try_fold_fn(
        f: fn(usize, usize) -> Ready<Result<usize, ()>>,
    ) -> impl Fn(usize, Result<usize, ()>) -> Ready<Result<usize, ()>> {
        move |state, item| item.map_or_else(|error| future::ready_by_copy(Err(error)), |x| f(state, x))
    }

    let mut benchmark_group = c.benchmark_group("async iter/try_fold_async");

    benchmark_try_fold_async_with(
        &mut benchmark_group,
        "async block",
        |mut iter, mut init, f| async move {
            while let Some(item) = iter.next().await {
                init = f(init, item?).await?;
            }

            Ok(init)
        },
    );

    benchmark_try_fold_async_with(&mut benchmark_group, "futures", TryStreamExt::try_fold);

    benchmark_try_fold_async_with(&mut benchmark_group, "slim-futures/clone", |iter, init, f| {
        iter.slim_try_fold_async_by_clone(init, try_fold_fn(f))
    });

    benchmark_try_fold_async_with(&mut benchmark_group, "slim-futures/copy", |iter, init, f| {
        iter.slim_try_fold_async_by_copy(init, try_fold_fn(f))
    });

    benchmark_try_fold_async_with(&mut benchmark_group, "slim-futures/take", |iter, init, f| {
        iter.slim_try_fold_async_by_take(init, try_fold_fn(f))
    });

    benchmark_group.finish()
}

// `try_for_each_async`.

fn benchmark_try_for_each_async_with<Fut>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut(StreamType<Result<usize, ()>>, fn(usize) -> Ready<Result<(), ()>>) -> Fut,
) where
    Fut: Future<Output = Result<(), ()>>,
{
    benchmark_future_with(benchmark_group, name, || {
        f(
            gen_stream(Ok),
            hint::black_box::<fn(_) -> _>(|_| future::ready_by_copy(Ok(()))),
        )
    });
}

fn benchmark_try_for_each_async(c: &mut Criterion<impl Measurement>) {
    let mut benchmark_group = c.benchmark_group("async iter/try_for_each_async");

    benchmark_try_for_each_async_with(&mut benchmark_group, "async block", |mut iter, f| async move {
        while let Some(item) = iter.next().await {
            f(item?).await?;
        }

        Ok(())
    });

    benchmark_try_for_each_async_with(&mut benchmark_group, "futures", TryStreamExt::try_for_each);

    benchmark_try_for_each_async_with(&mut benchmark_group, "slim-futures", |iter, f| {
        iter.slim_try_for_each_async(move |item| item.map_or_else(|error| future::ready_by_copy(Err(error)), f))
    });

    benchmark_group.finish()
}

// `zip`.

fn benchmark_zip_with<I>(
    benchmark_group: &mut BenchmarkGroup<impl Measurement>,
    name: &str,
    mut f: impl FnMut(StreamType<usize>, StreamType<usize>) -> I,
) where
    I: Stream<Item = (usize, usize)>,
{
    benchmark_async_iter_with(benchmark_group, name, || {
        f(gen_stream(convert::identity), gen_stream(convert::identity))
    });
}

fn benchmark_zip(c: &mut Criterion<impl Measurement>) {
    let mut benchmark_group = c.benchmark_group("async iter/zip");

    benchmark_zip_with(&mut benchmark_group, "futures", StreamExt::zip);
    benchmark_zip_with(&mut benchmark_group, "slim-futures", AsyncIteratorExt::slim_zip);

    benchmark_group.finish()
}

// Main.

criterion::criterion_group!(
    benchmarks,
    benchmark_and_then,
    benchmark_and_then_async,
    benchmark_filter_async,
    benchmark_filter_map_async,
    benchmark_fold_async,
    benchmark_for_each_async,
    benchmark_map,
    benchmark_map_async,
    benchmark_map_err,
    benchmark_map_ok,
    benchmark_scan_async,
    benchmark_try_fold_async,
    benchmark_try_for_each_async,
    benchmark_zip,
);

criterion::criterion_main!(benchmarks);
