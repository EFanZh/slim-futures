pin_project_lite::pin_project! {
    #[project = FoldStateProject]
    #[derive(Clone)]
    pub enum FoldState<T, Fut> {
        Accumulate {
            acc: T,
        },
        Future {
            #[pin]
            fut: Fut,
        },
    }
}
