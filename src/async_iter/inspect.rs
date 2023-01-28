use crate::async_iter::map::Map;
use crate::support::fns::InspectFn;
use crate::support::{AsyncIterator, FusedAsyncIterator};
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;

pin_project_lite::pin_project! {
    pub struct Inspect<I, F> {
        #[pin]
        inner: Map<I, InspectFn<F>>,
    }
}

impl<I, F> Inspect<I, F> {
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: Map::new(iter, InspectFn::new(f)),
        }
    }
}

impl<I, F> Clone for Inspect<I, F>
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

impl<I, F> AsyncIterator for Inspect<I, F>
where
    I: AsyncIterator,
    F: for<'a> FnMut<(&'a I::Item,), Output = ()>,
{
    type Item = I::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I, F> FusedAsyncIterator for Inspect<I, F>
where
    I: FusedAsyncIterator,
    F: for<'a> FnMut<(&'a I::Item,), Output = ()>,
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
    async fn test_inspect() {
        let mut result = Vec::new();
        let iter = stream::iter([2, 3, 5]).slim_inspect(|&x| result.push(x));

        assert_eq!(iter.collect::<Vec<_>>().await, [2, 3, 5]);
        assert_eq!(result, [2, 3, 5]);
    }

    #[tokio::test]
    async fn test_inspect_clone() {
        let result = Mutex::new(Vec::new());
        let iter = stream::iter([2, 3, 5]).slim_inspect(|&x| result.lock().unwrap().push(x));
        let iter_2 = iter.clone();

        assert_eq!(iter.collect::<Vec<_>>().await, [2, 3, 5]);
        assert_eq!(*result.lock().unwrap(), [2, 3, 5]);

        assert_eq!(iter_2.collect::<Vec<_>>().await, [2, 3, 5]);
        assert_eq!(*result.lock().unwrap(), [2, 3, 5, 2, 3, 5]);
    }
}
