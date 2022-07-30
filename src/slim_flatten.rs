use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    #[project = SlimFlattenInnerProject]
    enum SlimFlattenInner<T>
    where
        T: Future,
    {
        First {
            #[pin]
            fut: T,
        },
        Second {
            #[pin]
            fut: T::Output,
        },
    }
}

pin_project_lite::pin_project! {
    pub struct SlimFlatten<T>
    where
        T: Future,
    {
        #[pin]
        inner: SlimFlattenInner<T>,
    }
}

impl<T> SlimFlatten<T>
where
    T: Future,
{
    pub(crate) fn new(fut: T) -> Self {
        Self {
            inner: SlimFlattenInner::First { fut },
        }
    }
}

impl<T> Future for SlimFlatten<T>
where
    T: Future,
    T::Output: Future,
{
    type Output = <T::Output as Future>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut inner = self.project().inner;

        loop {
            match inner.as_mut().project() {
                SlimFlattenInnerProject::First { fut } => {
                    let fut = futures::ready!(fut.poll(cx));

                    inner.set(SlimFlattenInner::Second { fut });
                }
                SlimFlattenInnerProject::Second { fut } => return fut.poll(cx),
            }
        }
    }
}
