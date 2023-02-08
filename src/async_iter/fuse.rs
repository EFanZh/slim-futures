use crate::support::{AsyncIterator, FusedAsyncIterator};
use core::pin::Pin;
use core::task::{self, Context, Poll};
use option_entry::{OptionEntryExt, OptionPinnedEntry};

pin_project_lite::pin_project! {
    pub struct Fuse<I> {
        #[pin]
        iter: Option<I>
    }
}

impl<I> Fuse<I> {
    pub(crate) fn new(iter: I) -> Self {
        Self { iter: Some(iter) }
    }
}

impl<I> Clone for Fuse<I>
where
    I: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<I> AsyncIterator for Fuse<I>
where
    I: AsyncIterator,
{
    type Item = I::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let iter = self.project().iter.pinned_entry();

        Poll::Ready(if let OptionPinnedEntry::Some(mut iter) = iter {
            let item = task::ready!(iter.get_pin_mut().poll_next(cx));

            if item.is_none() {
                iter.replace_none();
            }

            item
        } else {
            None
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.as_ref().map_or((0, Some(0)), I::size_hint)
    }
}

impl<I> FusedAsyncIterator for Fuse<I>
where
    I: AsyncIterator,
{
    fn is_terminated(&self) -> bool {
        self.iter.is_none()
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use crate::support::AsyncIterator;
    use core::pin::Pin;
    use core::task::{Context, Poll};
    use futures_core::FusedStream;
    use futures_util::StreamExt;
    use std::vec::Vec;

    #[derive(Clone)]
    struct Foo(u32);

    impl AsyncIterator for Foo {
        type Item = u32;

        fn poll_next(mut self: Pin<&mut Self>, _: &mut Context) -> Poll<Option<Self::Item>> {
            let item = self.0.checked_sub(1);

            if let Some(item) = item {
                self.0 = item;
            }

            Poll::Ready(item)
        }
    }

    #[tokio::test]
    async fn test_fuse() {
        let mut iter = Foo(4).slim_fuse();

        assert!(!iter.is_terminated());
        assert_eq!(iter.next().await, Some(3));
        assert!(!iter.is_terminated());
        assert_eq!(iter.next().await, Some(2));
        assert!(!iter.is_terminated());
        assert_eq!(iter.next().await, Some(1));
        assert!(!iter.is_terminated());
        assert_eq!(iter.next().await, Some(0));
        assert!(!iter.is_terminated());
        assert_eq!(iter.next().await, None);
        assert!(iter.is_terminated());
        assert_eq!(iter.next().await, None);
    }

    #[tokio::test]
    async fn test_fuse_clone() {
        let iter = Foo(4).slim_fuse();
        let iter_2 = iter.clone();

        assert_eq!(iter.collect::<Vec<_>>().await, [3, 2, 1, 0]);
        assert_eq!(iter_2.collect::<Vec<_>>().await, [3, 2, 1, 0]);
    }
}
