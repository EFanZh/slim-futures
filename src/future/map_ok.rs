use crate::future::map::Map;
use crate::support::FnMut1;
use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

struct MapOkFn<F> {
    inner: F,
}

impl<T, E, F> FnMut1<Result<T, E>> for MapOkFn<F>
where
    F: FnMut1<T>,
{
    type Output = Result<F::Output, E>;

    fn call_mut(&mut self, arg: Result<T, E>) -> Self::Output {
        arg.map(|value| self.inner.call_mut(value))
    }
}

pin_project_lite::pin_project! {
    pub struct MapOk<Fut, F> {
        #[pin]
        inner: Map<Fut, MapOkFn<F>>,
    }
}

impl<Fut, F> MapOk<Fut, F> {
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: Map::new(fut, MapOkFn { inner: f }),
        }
    }
}

impl<Fut, F, T, E> Future for MapOk<Fut, F>
where
    Fut: Future<Output = Result<T, E>>,
    F: FnMut1<T>,
{
    type Output = Result<F::Output, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F, T, E> FusedFuture for MapOk<Fut, F>
where
    Fut: FusedFuture<Output = Result<T, E>>,
    F: FnMut1<T>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}
