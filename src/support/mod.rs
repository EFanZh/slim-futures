pub use self::fn_mut_1::FnMut1;
pub use self::fn_mut_2::FnMut2;
pub use self::pinned_and_not_pinned::PinnedAndNotPinned;
pub use self::try_::{FromResidual, Try};
pub use self::try_future::TryFuture;
pub use self::two_phases::TwoPhases;
use futures_core::Future;
pub use futures_core::{FusedStream as FusedAsyncIterator, Stream as AsyncIterator};
pub use std::convert::Infallible as Never;

mod fn_mut_1;
mod fn_mut_2;
pub mod fns;
mod pinned_and_not_pinned;
mod try_;
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
