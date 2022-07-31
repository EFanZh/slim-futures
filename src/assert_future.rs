use std::future::Future;

pub fn assert_future<Fut, T>(fut: Fut) -> Fut
where
    Fut: Future<Output = T>,
{
    fut
}
