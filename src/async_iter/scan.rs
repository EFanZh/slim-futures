use crate::support::AsyncIterator;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;

pin_project_lite::pin_project! {
    pub struct Scan<I, S, F>
    where
        F: ?Sized,
    {
        #[pin]
        iter: I,
        state: S,
        f: F,
    }
}

impl<I, S, F> Scan<I, S, F> {
    pub(crate) fn new(iter: I, state: S, f: F) -> Self {
        Self { iter, state, f }
    }
}

impl<I, S, F> Clone for Scan<I, S, F>
where
    I: Clone,
    S: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            state: self.state.clone(),
            f: self.f.clone(),
        }
    }
}

impl<I, S, F, T> AsyncIterator for Scan<I, S, F>
where
    I: AsyncIterator,
    F: for<'a> FnMut<(&'a mut S, I::Item), Output = Option<T>> + ?Sized,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();

        Poll::Ready(task::ready!(this.iter.poll_next(cx)).and_then(|item| this.f.call_mut((this.state, item))))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use futures_util::{stream, StreamExt};
    use std::vec::Vec;

    fn scan_fn(state: &mut u32, item: u16) -> Option<u64> {
        *state += u32::from(item);

        (*state < 10).then_some(u64::from(*state))
    }

    #[tokio::test]
    async fn test_scan() {
        let iter = stream::iter(0..10).slim_scan(0, scan_fn);

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 1, 3, 6]);
    }

    #[tokio::test]
    async fn test_scan_clone() {
        let iter = stream::iter(0..10).slim_scan(0, scan_fn);
        let iter_2 = iter.clone();

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 1, 3, 6]);
        assert_eq!(iter_2.collect::<Vec<_>>().await, [0, 1, 3, 6]);
    }
}
