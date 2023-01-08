pub use self::fn_mut_1::FnMut1;
pub use self::fn_mut_2::FnMut2;
pub use self::pinned_and_not_pinned::PinnedAndNotPinned;
pub use self::result_future::ResultFuture;
pub use self::try_::{FromResidual, Try};
pub use self::two_phases::TwoPhases;
pub use core::convert::Infallible as Never;
use futures_core::Future;
pub use futures_core::{FusedStream as FusedAsyncIterator, Stream as AsyncIterator};

mod fn_mut_1;
mod fn_mut_2;
pub mod fns;
mod pinned_and_not_pinned;
mod result_future;
mod try_;
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
