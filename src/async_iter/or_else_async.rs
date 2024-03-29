use crate::support::{AsyncIterator, FusedAsyncIterator, ResultAsyncIterator, Try};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;
use option_entry::{OptionEntryExt, OptionPinnedEntry};

pin_project_lite::pin_project! {
    pub struct OrElseAsync<I, F>
    where
        I: ResultAsyncIterator,
        F: FnMut<(I::Error,)>,
        F: ?Sized,
        F::Output: IntoFuture,
    {
        #[pin]
        iter: I,
        #[pin]
        state: Option<<F::Output as IntoFuture>::IntoFuture>,
        f: F,
    }
}

impl<I, F> OrElseAsync<I, F>
where
    I: ResultAsyncIterator,
    F: FnMut<(I::Error,)>,
    F::Output: IntoFuture,
{
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self { iter, state: None, f }
    }
}

impl<I, F> Clone for OrElseAsync<I, F>
where
    I: ResultAsyncIterator + Clone,
    F: FnMut<(I::Error,)> + Clone,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            state: self.state.clone(),
            f: self.f.clone(),
        }
    }
}

impl<I, F> AsyncIterator for OrElseAsync<I, F>
where
    I: ResultAsyncIterator,
    F: FnMut<(I::Error,)> + ?Sized,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: Try<Output = I::Ok>,
{
    type Item = <F::Output as IntoFuture>::Output;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut iter = this.iter;
        let state = this.state.pinned_entry();
        let f = this.f;

        let mut fut = match state {
            OptionPinnedEntry::None(none_state) => match task::ready!(iter.as_mut().poll_next(cx)) {
                None => return Poll::Ready(None),
                Some(item) => match item {
                    Ok(value) => return Poll::Ready(Some(Self::Item::from_output(value))),
                    Err(error) => none_state.replace_some(f.call_mut((error,)).into_future()),
                },
            },
            OptionPinnedEntry::Some(some_state) => some_state,
        };

        let item = task::ready!(fut.get_pin_mut().poll(cx));

        fut.replace_none();

        Poll::Ready(Some(item))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut candidate = self.iter.size_hint();

        if self.state.is_some() {
            candidate.0 = candidate.0.saturating_add(1);
            candidate.1 = candidate.1.and_then(|high| high.checked_add(1));
        }

        candidate
    }
}

impl<I, F> FusedAsyncIterator for OrElseAsync<I, F>
where
    I: ResultAsyncIterator + FusedAsyncIterator,
    F: FnMut<(I::Error,)> + ?Sized,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: Try<Output = I::Ok>,
    <F::Output as IntoFuture>::IntoFuture: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        self.state
            .as_ref()
            .map_or_else(|| self.iter.is_terminated(), FusedFuture::is_terminated)
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use futures_util::future::Ready;
    use futures_util::{future, stream, StreamExt};
    use std::vec::Vec;

    fn or_else_async_fn(x: u32) -> Ready<Result<u64, u64>> {
        if x < 7 {
            future::ok(u64::from(x * 100))
        } else {
            future::err(u64::from(x * 1000))
        }
    }

    #[tokio::test]
    async fn test_or_else_async() {
        let iter = stream::iter([Ok(2), Ok(3), Err(5), Err(7), Ok(11), Ok(13)]).slim_or_else_async(or_else_async_fn);

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Ok(2), Ok(3), Ok(500), Err(7000), Ok(11), Ok(13)],
        );
    }

    #[tokio::test]
    async fn test_or_else_async_with_option() {
        let iter = stream::iter([Ok(2), Ok(3), Err(5), Err(7), Ok(11), Ok(13)])
            .slim_or_else_async(|x: u32| future::ready((x < 7).then_some(u64::from(x * 100))));

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Some(2), Some(3), Some(500), None, Some(11), Some(13)],
        );
    }

    #[tokio::test]
    async fn test_or_else_async_clone() {
        let iter =
            stream::iter([Ok(2), Ok(3), Err(5_u32), Err(7), Ok(11), Ok(13)]).slim_or_else_async(or_else_async_fn);
        let iter_2 = iter.clone();

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Ok(2), Ok(3), Ok(500), Err(7000), Ok(11), Ok(13)],
        );

        assert_eq!(
            iter_2.collect::<Vec<_>>().await,
            [Ok(2), Ok(3), Ok(500), Err(7000), Ok(11), Ok(13)],
        );
    }
}
