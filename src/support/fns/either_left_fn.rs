use crate::support::fn_mut_1::FnMut1;
use futures_util::future::Either;
use std::marker::PhantomData;

pub struct EitherLeftFn<A, B> {
    _phantom: PhantomData<fn(A) -> Either<A, B>>,
}

impl<A, B> Default for EitherLeftFn<A, B> {
    fn default() -> Self {
        Self { _phantom: PhantomData }
    }
}

impl<A, B> Clone for EitherLeftFn<A, B> {
    fn clone(&self) -> Self {
        Self { ..*self }
    }
}

impl<A, B> FnMut1<A> for EitherLeftFn<A, B> {
    type Output = Either<A, B>;

    fn call_mut(&mut self, arg: A) -> Self::Output {
        Either::Left(arg)
    }
}
