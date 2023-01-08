use core::future::Future;

pub trait ResultFuture: Future<Output = Result<Self::Ok, Self::Error>> {
    type Ok;
    type Error;
}

impl<Fut, T, E> ResultFuture for Fut
where
    Fut: Future<Output = Result<T, E>>,
{
    type Ok = T;
    type Error = E;
}
