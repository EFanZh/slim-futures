use std::future::Future;

pub fn assert_future<T, R>(value: T) -> T
where
    T: Future<Output = R>,
{
    value
}
