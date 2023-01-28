use crate::support::{AsyncIterator, FusedAsyncIterator};
use core::mem;
use core::pin::Pin;
use core::task::{self, Context, Poll};

#[derive(Clone)]
enum Buffer<A, B> {
    Empty,
    Left(A),
    Right(B),
}

pin_project_lite::pin_project! {
    pub struct Zip<A, B>
    where
        A: AsyncIterator,
        B: AsyncIterator,
    {
        #[pin]
        left: A,
        #[pin]
        right: B,
        buffer: Buffer<A::Item, B::Item>,
    }
}

impl<A, B> Zip<A, B>
where
    A: AsyncIterator,
    B: AsyncIterator,
{
    pub(crate) fn new(left: A, right: B) -> Self {
        Self {
            left,
            right,
            buffer: Buffer::Empty,
        }
    }
}

impl<A, B> Clone for Zip<A, B>
where
    A: AsyncIterator + Clone,
    B: AsyncIterator + Clone,
    A::Item: Clone,
    B::Item: Clone,
{
    fn clone(&self) -> Self {
        Self {
            left: self.left.clone(),
            right: self.right.clone(),
            buffer: self.buffer.clone(),
        }
    }
}

impl<A, B> AsyncIterator for Zip<A, B>
where
    A: AsyncIterator,
    B: AsyncIterator,
{
    type Item = (A::Item, B::Item);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let buffer = this.buffer;

        Poll::Ready(loop {
            break match buffer {
                Buffer::Empty => match this.left.as_mut().poll_next(cx) {
                    Poll::Ready(None) => None,
                    Poll::Ready(Some(item)) => {
                        *buffer = Buffer::Left(item);

                        continue;
                    }
                    Poll::Pending => match task::ready!(this.right.as_mut().poll_next(cx)) {
                        None => None,
                        Some(item) => {
                            *buffer = Buffer::Right(item);

                            continue;
                        }
                    },
                },
                Buffer::Left(_) => {
                    task::ready!(this.right.poll_next(cx)).map(|right_item| match mem::replace(buffer, Buffer::Empty) {
                        Buffer::Left(left_item) => (left_item, right_item),
                        _ => unreachable!(),
                    })
                }
                Buffer::Right(_) => {
                    task::ready!(this.left.poll_next(cx)).map(|left_item| match mem::replace(buffer, Buffer::Empty) {
                        Buffer::Right(right_item) => (left_item, right_item),
                        _ => unreachable!(),
                    })
                }
            };
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (mut left_low, mut left_high) = self.left.size_hint();
        let (mut right_low, mut right_high) = self.right.size_hint();

        match &self.buffer {
            Buffer::Empty => {}
            Buffer::Left(_) => {
                left_low = left_low.saturating_add(1);
                left_high = left_high.and_then(|left_high| left_high.checked_add(1));
            }
            Buffer::Right(_) => {
                right_low = right_low.saturating_add(1);
                right_high = right_high.and_then(|right_high| right_high.checked_add(1));
            }
        }

        let low = left_low.min(right_low);

        let high = match (left_high, right_high) {
            (None, None) => None,
            (None, Some(right_high)) => Some(right_high),
            (Some(left_high), None) => Some(left_high),
            (Some(left_high), Some(right_high)) => Some(left_high.min(right_high)),
        };

        (low, high)
    }
}

impl<A, B> FusedAsyncIterator for Zip<A, B>
where
    A: FusedAsyncIterator,
    B: FusedAsyncIterator,
{
    fn is_terminated(&self) -> bool {
        match &self.buffer {
            Buffer::Empty => self.left.is_terminated() || self.right.is_terminated(),
            Buffer::Left(_) => self.right.is_terminated(),
            Buffer::Right(_) => self.left.is_terminated(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use futures_util::{stream, StreamExt};
    use std::vec::Vec;

    #[tokio::test]
    async fn test_zip() {
        let iter_1 = stream::iter(0..3).slim_zip(stream::iter(10..20));
        let iter_2 = stream::iter(10..20).slim_zip(stream::iter(0..3));

        assert_eq!(iter_1.collect::<Vec<_>>().await, [(0, 10), (1, 11), (2, 12)]);
        assert_eq!(iter_2.collect::<Vec<_>>().await, [(10, 0), (11, 1), (12, 2)]);
    }

    #[tokio::test]
    async fn test_zip_clone() {
        let iter_1 = stream::iter(0..3).slim_zip(stream::iter(10..20));
        let iter_1_clone = iter_1.clone();
        let iter_2 = stream::iter(10..20).slim_zip(stream::iter(0..3));
        let iter_2_clone = iter_2.clone();

        assert_eq!(iter_1.collect::<Vec<_>>().await, [(0, 10), (1, 11), (2, 12)]);
        assert_eq!(iter_1_clone.collect::<Vec<_>>().await, [(0, 10), (1, 11), (2, 12)]);
        assert_eq!(iter_2.collect::<Vec<_>>().await, [(10, 0), (11, 1), (12, 2)]);
        assert_eq!(iter_2_clone.collect::<Vec<_>>().await, [(10, 0), (11, 1), (12, 2)]);
    }
}
