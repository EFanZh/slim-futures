pub use self::fn_mut_1::FnMut1;
pub use self::pinned_and_not_pinned::PinnedAndNotPinned;
pub use self::try_fn_mut_1::TryFnMut1;
pub use self::try_future::TryFuture;
pub use self::two_phases::TwoPhases;
use futures_core::Future;
pub use futures_core::{FusedStream as FusedAsyncIterator, Stream as AsyncIterator};
pub use std::convert::Infallible as Never;

mod fn_mut_1;
pub mod fns;
mod pinned_and_not_pinned;
mod try_fn_mut_1;
mod try_future;
mod two_phases;

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
