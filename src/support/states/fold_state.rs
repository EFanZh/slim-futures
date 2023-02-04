use core::pin::Pin;

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

impl<T, Fut> FoldState<T, Fut> {
    pub fn set_future(mut self: Pin<&mut Self>, fut: Fut) -> Pin<&mut Fut> {
        self.set(Self::Future { fut });

        let FoldStateProject::Future { fut, .. } = self.project() else { unreachable!() };

        fut
    }
}
