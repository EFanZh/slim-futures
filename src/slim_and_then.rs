use crate::fn_mut_1::FnMut1;
use crate::slim_map::SlimMap;
use futures::future::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

struct AndThenFn<F> {
    inner: F,
}

impl<T, E, F, U> FnMut1<Result<T, E>> for AndThenFn<F>
where
    F: FnMut1<T, Output = Result<U, E>>,
{
    type Output = Result<U, E>;

    fn call_mut(&mut self, arg: Result<T, E>) -> Self::Output {
        arg.and_then(|value| self.inner.call_mut(value))
    }
}

pin_project_lite::pin_project! {
    pub struct SlimAndThen<Fut, F> {
        #[pin]
        inner: SlimMap<Fut, AndThenFn<F>>,
    }
}

impl<Fut, F> SlimAndThen<Fut, F> {
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: SlimMap::new(fut, AndThenFn { inner: f }),
        }
    }
}

impl<Fut, F, U, E, V> Future for SlimAndThen<Fut, F>
where
    Fut: Future<Output = Result<U, E>>,
    F: FnMut1<U, Output = Result<V, E>>,
{
    type Output = Result<V, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F, U, E, V> FusedFuture for SlimAndThen<Fut, F>
where
    Fut: FusedFuture<Output = Result<U, E>>,
    F: FnMut1<U, Output = Result<V, E>>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}
