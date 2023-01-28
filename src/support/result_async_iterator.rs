use crate::support::AsyncIterator;

pub trait ResultAsyncIterator: AsyncIterator<Item = Result<Self::Ok, Self::Error>> {
    type Ok;
    type Error;
}

impl<Fut, T, E> ResultAsyncIterator for Fut
where
    Fut: AsyncIterator<Item = Result<T, E>> + ?Sized,
{
    type Ok = T;
    type Error = E;
}
