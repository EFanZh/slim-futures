pub use self::fn_mut_1::FnMut1;
pub use self::try_fn_mut_1::TryFnMut1;
pub use self::try_future::TryFuture;
pub use futures_core::FusedStream as FusedAsyncIterator;
use futures_core::Future;
pub use futures_core::Stream as AsyncIterator;

mod fn_mut_1;
mod try_fn_mut_1;
mod try_future;

pub fn assert_future<Fut, T>(fut: Fut) -> Fut
where
    Fut: Future<Output = T>,
{
    fut
}

pub fn assert_async_iter<I, T>(iter: I) -> I
where
    I: AsyncIterator<Item = T>,
{
    iter
}
