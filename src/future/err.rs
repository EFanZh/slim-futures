use crate::future::{ready, Ready};

pub fn err<T, E>(error: E) -> Ready<Result<T, E>>
where
    T: Copy,
    E: Copy,
{
    ready(Err(error))
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_err() {
        assert_eq!(super::err::<u32, u32>(7).await, Err(7));
    }
}
