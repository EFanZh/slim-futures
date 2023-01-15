use core::future::Future;

pub trait OptionFuture: Future<Output = Option<Self::Item>> {
    type Item;
}

impl<Fut, T> OptionFuture for Fut
where
    Fut: Future<Output = Option<T>>,
{
    type Item = T;
}
