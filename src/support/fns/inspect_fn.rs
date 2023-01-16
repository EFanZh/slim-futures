use crate::support::fn_mut_1::FnMut1;

#[derive(Clone)]
pub struct InspectFn<F> {
    f: F,
}

impl<F> InspectFn<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<T, F> FnMut1<T> for InspectFn<F>
where
    F: for<'a> FnMut1<&'a T, Output = ()>,
{
    type Output = T;

    fn call_mut(&mut self, arg: T) -> Self::Output {
        self.f.call_mut(&arg);

        arg
    }
}
