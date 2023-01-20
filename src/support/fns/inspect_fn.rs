use fn_traits::FnMut;

#[derive(Clone)]
pub struct InspectFn<F> {
    f: F,
}

impl<F> InspectFn<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<T, F> FnMut<(T,)> for InspectFn<F>
where
    F: for<'a> FnMut<(&'a T,), Output = ()>,
{
    type Output = T;

    fn call_mut(&mut self, args: (T,)) -> Self::Output {
        self.f.call_mut((&args.0,));

        args.0
    }
}
