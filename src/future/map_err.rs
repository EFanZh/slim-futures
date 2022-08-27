use crate::future::map::Map;
use crate::support::FnMut1;
use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Clone)]
struct MapErrFn<F> {
    inner: F,
}

impl<T, E, F> FnMut1<Result<T, E>> for MapErrFn<F>
where
    F: FnMut1<E>,
{
    type Output = Result<T, F::Output>;

    fn call_mut(&mut self, arg: Result<T, E>) -> Self::Output {
        arg.map_err(|value| self.inner.call_mut(value))
    }
}

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct MapErr<Fut, F> {
        #[pin]
        inner: Map<Fut, MapErrFn<F>>,
    }
}

impl<Fut, F> MapErr<Fut, F> {
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: Map::new(fut, MapErrFn { inner: f }),
        }
    }
}

impl<Fut, F, T, E> Future for MapErr<Fut, F>
where
    Fut: Future<Output = Result<T, E>>,
    F: FnMut1<E>,
{
    type Output = Result<T, F::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F, T, E> FusedFuture for MapErr<Fut, F>
where
    Fut: FusedFuture<Output = Result<T, E>>,
    F: FnMut1<E>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}
