use crate::fn_mut_1::FnMut1;
use crate::slim_map::SlimMap;
use futures::future::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

struct InspectFn<F> {
    inner: F,
}

impl<T, F> FnMut1<T> for InspectFn<F>
where
    F: for<'a> FnMut1<&'a T, Output = ()>,
{
    type Output = T;

    fn call_mut(&mut self, arg: T) -> Self::Output {
        self.inner.call_mut(&arg);

        arg
    }
}

pin_project_lite::pin_project! {
    pub struct SlimInspect<T, F> {
        #[pin]
        inner: SlimMap<T, InspectFn<F>>,
    }
}

impl<T, F> SlimInspect<T, F> {
    pub(crate) fn new(fut: T, f: F) -> Self {
        Self {
            inner: SlimMap::new(fut, InspectFn { inner: f }),
        }
    }
}

impl<T, F> Future for SlimInspect<T, F>
where
    T: Future,
    F: FnMut(&T::Output),
{
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<T, F> FusedFuture for SlimInspect<T, F>
where
    T: FusedFuture,
    F: FnMut(&T::Output),
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}
