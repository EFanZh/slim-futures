use core::future::IntoFuture;

pub trait IntoResultFuture: IntoFuture<Output = Result<Self::Ok, Self::Error>> {
    type Ok;
    type Error;
}

impl<Fut, T, E> IntoResultFuture for Fut
where
    Fut: IntoFuture<Output = Result<T, E>>,
{
    type Ok = T;
    type Error = E;
}
