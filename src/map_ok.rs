use crate::fn_mut_1::FnMut1;
use crate::map::Map;
use futures::future::FusedFuture;
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
    pub struct MapOk<T, F> {
        #[pin]
        inner: Map<T, MapOkFn<F>>,
    }
}

impl<T, F> MapOk<T, F> {
    pub(crate) fn new(fut: T, f: F) -> Self {
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

impl<T, F, E> FusedFuture for MapOk<T, F>
where
    T: FusedFuture<Output = Result<T, E>>,
    F: FnMut1<T>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}
