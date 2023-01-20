use crate::support::{AsyncIterator, FusedAsyncIterator};
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;

pin_project_lite::pin_project! {
    pub struct SkipWhile<I, F> {
        #[pin]
        iter: I,
        f: Option<F>,
    }
}

impl<I, F> SkipWhile<I, F> {
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self { iter, f: Some(f) }
    }
}

impl<I, F> Clone for SkipWhile<I, F>
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

impl<I, F> AsyncIterator for SkipWhile<I, F>
where
    I: AsyncIterator,
    F: for<'a> FnMut<(&'a I::Item,), Output = bool>,
{
    type Item = I::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut iter = this.iter;
        let f_slot = this.f;

        if let Some(f) = f_slot {
            loop {
                let item = task::ready!(iter.as_mut().poll_next(cx));

                if let Some(item) = &item {
                    if f.call_mut((item,)) {
                        continue;
                    } else {
                        *f_slot = None;
                    }
                }

                return Poll::Ready(item);
            }
        }

        iter.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }
}

impl<I, F> FusedAsyncIterator for SkipWhile<I, F>
where
    I: FusedAsyncIterator,
    F: for<'a> FnMut<(&'a I::Item,), Output = bool>,
{
    fn is_terminated(&self) -> bool {
        self.iter.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
    use futures_util::{stream, StreamExt};
    use std::vec::Vec;

    #[tokio::test]
    async fn test_skip_while() {
        let iter = stream::iter(0..10).slim_skip_while(|&x| x < 5);

        assert_eq!(iter.collect::<Vec<_>>().await, [5, 6, 7, 8, 9]);
    }

    #[tokio::test]
    async fn test_skip_while_clone() {
        let iter = stream::iter(0..10).slim_skip_while(|&x| x < 5);
        let iter_2 = iter.clone();

        assert_eq!(iter.collect::<Vec<_>>().await, [5, 6, 7, 8, 9]);
        assert_eq!(iter_2.collect::<Vec<_>>().await, [5, 6, 7, 8, 9]);
    }
}
