use crate::future::and_then_async::AndThenAsync;
use crate::future::map::Map;
use crate::support::fns::OkFn;
use crate::support::{FnMut1, TryFuture};
use futures_core::FusedFuture;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

struct MapOkAsyncFn<F, E> {
    inner: F,
    _phantom: PhantomData<fn() -> E>,
}

impl<T, F, E> FnMut1<T> for MapOkAsyncFn<F, E>
where
    F: FnMut1<T>,
    F::Output: Future,
{
    type Output = Map<F::Output, OkFn<<F::Output as Future>::Output, E>>;

    fn call_mut(&mut self, arg: T) -> Self::Output {
        Map::new(self.inner.call_mut(arg), OkFn::default())
    }
}

pin_project_lite::pin_project! {
    pub struct MapOkAsync<Fut, F>
    where
        Fut: TryFuture,
        F: FnMut1<Fut::Ok>,
        F::Output: Future,
    {
        #[pin]
        inner: AndThenAsync<Fut, MapOkAsyncFn<F, Fut::Error>>,
    }
}

impl<Fut, F, T, E> MapOkAsync<Fut, F>
where
    Fut: Future<Output = Result<T, E>>,
    F: FnMut1<T>,
    F::Output: Future,
{
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: AndThenAsync::new(
                fut,
                MapOkAsyncFn {
                    inner: f,
                    _phantom: PhantomData,
                },
            ),
        }
    }
}

impl<Fut, F, T, E> Future for MapOkAsync<Fut, F>
where
    Fut: Future<Output = Result<T, E>>,
    F: FnMut1<T>,
    F::Output: Future,
{
    type Output = Result<<F::Output as Future>::Output, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F, T, E> FusedFuture for MapOkAsync<Fut, F>
where
    Fut: FusedFuture<Output = Result<T, E>>,
    F: FnMut1<T>,
    F::Output: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}
