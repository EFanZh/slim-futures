use crate::support::fn_mut_1::FnMut1;
use core::marker::PhantomData;
use futures_util::future::Either;

pub struct EitherRightFn<A, B> {
    phantom: PhantomData<fn(B) -> Either<A, B>>,
}

impl<A, B> Default for EitherRightFn<A, B> {
    fn default() -> Self {
        Self { phantom: PhantomData }
    }
}

impl<A, B> Clone for EitherRightFn<A, B> {
    fn clone(&self) -> Self {
        Self { phantom: self.phantom }
    }
}

impl<A, B> FnMut1<B> for EitherRightFn<A, B> {
    type Output = Either<A, B>;

    fn call_mut(&mut self, arg: B) -> Self::Output {
        Either::Right(arg)
    }
}
