use crate::fn_mut_1::FnMut1;
use crate::slim_flatten::SlimFlatten;
use crate::slim_map::SlimMap;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct SlimMapAsync<T, F>
    where
        T: Future,
        F: FnMut1<T::Output>,
    {
        #[pin]
        inner: SlimFlatten<SlimMap<T, F>>
    }
}

impl<T, F> SlimMapAsync<T, F>
where
    T: Future,
    F: FnMut1<T::Output>,
{
    pub(crate) fn new(fut: T, f: F) -> Self {
        Self {
            inner: SlimFlatten::new(SlimMap::new(fut, f)),
        }
    }
}

impl<T, F, U> Future for SlimMapAsync<T, F>
where
    T: Future,
    F: FnMut(T::Output) -> U,
    U: Future,
{
    type Output = U::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}
