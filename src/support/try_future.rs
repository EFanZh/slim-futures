use core::future::Future;

pub trait TryFuture: Future<Output = Result<Self::Ok, Self::Error>> {
    type Ok;
    type Error;
}

impl<Fut, T, E> TryFuture for Fut
where
    Fut: Future<Output = Result<T, E>>,
{
    type Ok = T;
    type Error = E;
}
