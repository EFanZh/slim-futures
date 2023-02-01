use crate::support::AsyncIterator;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;

pin_project_lite::pin_project! {
    pub struct TakeWhile<I, F>
    where
        F: ?Sized,
    {
        #[pin]
        iter: I,
        f: F,
    }
}

impl<I, F> TakeWhile<I, F> {
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self { iter, f }
    }
}

impl<I, F> Clone for TakeWhile<I, F>
where
    I: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            f: self.f.clone(),
        }
    }
}

impl<I, F> AsyncIterator for TakeWhile<I, F>
where
    I: AsyncIterator,
    F: for<'a> FnMut<(&'a I::Item,), Output = bool> + ?Sized,
{
    type Item = I::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();

        Poll::Ready(task::ready!(this.iter.poll_next(cx)).filter(|item| this.f.call_mut((item,))))
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

    #[tokio::test]
    async fn test_take_while() {
        let iter = stream::iter(0..10).slim_take_while(|&x| x < 5);

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_take_while_clone() {
        let iter = stream::iter(0..10).slim_take_while(|&x| x < 5);
        let iter_2 = iter.clone();

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 1, 2, 3, 4]);
        assert_eq!(iter_2.collect::<Vec<_>>().await, [0, 1, 2, 3, 4]);
    }
}
