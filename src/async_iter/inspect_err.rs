use crate::async_iter::inspect::Inspect;
use crate::support::fns::InspectErrFn;
use crate::support::{AsyncIterator, FusedAsyncIterator, ResultAsyncIterator};
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;

pin_project_lite::pin_project! {
    pub struct InspectErr<I, F>
    where
        F: ?Sized,
    {
        #[pin]
        inner: Inspect<I, InspectErrFn<F>>,
    }
}

impl<I, F> InspectErr<I, F> {
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: Inspect::new(iter, InspectErrFn::new(f)),
        }
    }
}

impl<I, F> Clone for InspectErr<I, F>
where
    I: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, F> AsyncIterator for InspectErr<I, F>
where
    I: ResultAsyncIterator,
    F: for<'a> FnMut<(&'a I::Error,), Output = ()> + ?Sized,
{
    type Item = I::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I, F> FusedAsyncIterator for InspectErr<I, F>
where
    I: ResultAsyncIterator + FusedAsyncIterator,
    F: for<'a> FnMut<(&'a I::Error,), Output = ()> + ?Sized,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use futures_util::{stream, StreamExt};
    use std::sync::Mutex;
    use std::vec::Vec;

    #[tokio::test]
    async fn test_inspect_err() {
        let mut result = Vec::new();
        let iter = stream::iter([Err(2), Ok(3), Err(5)]).slim_inspect_err(|&x| result.push(x));

        assert_eq!(iter.collect::<Vec<_>>().await, [Err(2), Ok(3), Err(5)]);
        assert_eq!(result, [2, 5]);
    }

    #[tokio::test]
    async fn test_inspect_err_clone() {
        let result = Mutex::new(Vec::new());
        let iter = stream::iter([Err(2), Ok(3), Err(5)]).slim_inspect_err(|&x| result.lock().unwrap().push(x));
        let iter_2 = iter.clone();

        assert_eq!(iter.collect::<Vec<_>>().await, [Err(2), Ok(3), Err(5)]);
        assert_eq!(*result.lock().unwrap(), [2, 5]);

        assert_eq!(iter_2.collect::<Vec<_>>().await, [Err(2), Ok(3), Err(5)]);
        assert_eq!(*result.lock().unwrap(), [2, 5, 2, 5]);
    }
}
