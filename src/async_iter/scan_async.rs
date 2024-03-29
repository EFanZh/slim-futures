use crate::support::AsyncIterator;
use core::future::IntoFuture;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;
use futures_core::Future;
use option_entry::{OptionEntryExt, OptionPinnedEntry};

pub trait ScanFn<S, T>: for<'a> FnMut<(&'a mut S, T), Output = <Self as ScanFn<S, T>>::Output> {
    type Output;
}

impl<S, T, F, R> ScanFn<S, T> for F
where
    F: for<'a> FnMut<(&'a mut S, T), Output = R> + ?Sized,
{
    type Output = R;
}

pin_project_lite::pin_project! {
    pub struct ScanAsync<I, S, F>
    where
        I: AsyncIterator,
        F: ScanFn<S, I::Item>,
        F: ?Sized,
        <F as ScanFn<S, I::Item>>::Output: IntoFuture,
    {
        #[pin]
        iter: I,
        state: S,
        #[pin]
        fut: Option<<<F as ScanFn<S, I::Item>>::Output as IntoFuture>::IntoFuture>,
        f: F,
    }
}

impl<I, S, F> ScanAsync<I, S, F>
where
    I: AsyncIterator,
    F: ScanFn<S, I::Item>,
    <F as ScanFn<S, I::Item>>::Output: IntoFuture,
{
    pub(crate) fn new(iter: I, state: S, f: F) -> Self {
        Self {
            iter,
            state,
            fut: None,
            f,
        }
    }
}

impl<I, S, F> Clone for ScanAsync<I, S, F>
where
    I: AsyncIterator + Clone,
    S: Clone,
    F: ScanFn<S, I::Item> + Clone,
    <F as ScanFn<S, I::Item>>::Output: IntoFuture,
    <<F as ScanFn<S, I::Item>>::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            state: self.state.clone(),
            fut: self.fut.clone(),
            f: self.f.clone(),
        }
    }
}

impl<I, S, F, T> AsyncIterator for ScanAsync<I, S, F>
where
    I: AsyncIterator,
    F: ScanFn<S, I::Item> + ?Sized,
    <F as ScanFn<S, I::Item>>::Output: IntoFuture<Output = Option<T>>,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut iter = this.iter;
        let state = this.state;
        let fut = this.fut.pinned_entry();
        let f = this.f;

        let mut fut = match fut {
            OptionPinnedEntry::None(none_state) => match task::ready!(iter.as_mut().poll_next(cx)) {
                None => return Poll::Ready(None),
                Some(item) => none_state.replace_some(f.call_mut((state, item)).into_future()),
            },
            OptionPinnedEntry::Some(some_state) => some_state,
        };

        let item = task::ready!(fut.get_pin_mut().poll(cx));

        fut.replace_none();

        Poll::Ready(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut candidate = (0, self.iter.size_hint().1);

        if self.fut.is_some() {
            candidate.1 = candidate.1.and_then(|high| high.checked_add(1));
        }

        candidate
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use futures_util::future::{self, Ready};
    use futures_util::{stream, StreamExt};
    use std::vec::Vec;

    fn scan_async_fn(state: &mut u32, item: u16) -> Ready<Option<u64>> {
        *state += u32::from(item);

        future::ready((*state < 10).then_some(u64::from(*state)))
    }

    #[tokio::test]
    async fn test_scan_async() {
        let iter = stream::iter(0..10).slim_scan_async(0, scan_async_fn);

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 1, 3, 6]);
    }

    #[tokio::test]
    async fn test_scan_async_clone() {
        let iter = stream::iter(0..10).slim_scan_async(0, scan_async_fn);
        let iter_2 = iter.clone();

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 1, 3, 6]);
        assert_eq!(iter_2.collect::<Vec<_>>().await, [0, 1, 3, 6]);
    }
}
