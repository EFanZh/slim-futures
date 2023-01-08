use std::convert::Infallible;
use std::ops::ControlFlow;
use std::task::Poll;

pub trait FromResidual<R = <Self as Try>::Residual> {
    fn from_residual(residual: R) -> Self;
}

pub trait Try: FromResidual<Self::Residual> {
    type Output;
    type Residual;

    fn from_output(output: Self::Output) -> Self;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
}

fn infallible_into<T>(value: Infallible) -> T {
    match value {}
}

// `ControlFlow<B, C>`.

impl<B, C> FromResidual for ControlFlow<B, C> {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        match residual {
            ControlFlow::Continue(never) => infallible_into(never),
            ControlFlow::Break(residual) => Self::Break(residual),
        }
    }
}

impl<B, C> Try for ControlFlow<B, C> {
    type Output = C;
    type Residual = ControlFlow<B, Infallible>;

    fn from_output(output: Self::Output) -> Self {
        Self::Continue(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Continue(output) => ControlFlow::Continue(output),
            Self::Break(residual) => ControlFlow::Break(ControlFlow::Break(residual)),
        }
    }
}

// `Option<T>`.

impl<T> FromResidual<<Self as Try>::Residual> for Option<T> {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        residual.map(infallible_into)
    }
}

impl<T> Try for Option<T> {
    type Output = T;
    type Residual = Option<Infallible>;

    fn from_output(output: Self::Output) -> Self {
        Some(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            None => ControlFlow::Break(None),
            Some(output) => ControlFlow::Continue(output),
        }
    }
}

// `Result<T, E>`.

impl<T, F, E> FromResidual<Result<Infallible, F>> for Result<T, E>
where
    E: From<F>,
{
    fn from_residual(residual: Result<Infallible, F>) -> Self {
        residual.map_or_else(|residual| Err(residual.into()), infallible_into)
    }
}

impl<T, E> Try for Result<T, E> {
    type Output = T;
    type Residual = Result<Infallible, E>;

    fn from_output(output: Self::Output) -> Self {
        Ok(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Ok(output) => ControlFlow::Continue(output),
            Err(residual) => ControlFlow::Break(Err(residual)),
        }
    }
}

// `Poll<Option<Result<T, E>>>`.

impl<T, F, E> FromResidual<Result<Infallible, F>> for Poll<Option<Result<T, E>>>
where
    E: From<F>,
{
    fn from_residual(residual: Result<Infallible, F>) -> Self {
        residual.map_or_else(|residual| Self::Ready(Some(Err(residual.into()))), infallible_into)
    }
}

impl<T, E> Try for Poll<Option<Result<T, E>>> {
    type Output = Poll<Option<T>>;
    type Residual = Result<Infallible, E>;

    fn from_output(output: Self::Output) -> Self {
        output.map(|output| output.map(Ok))
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Ready(None) => ControlFlow::Continue(Poll::Ready(None)),
            Self::Ready(Some(Ok(output))) => ControlFlow::Continue(Poll::Ready(Some(output))),
            Self::Ready(Some(Err(residual))) => ControlFlow::Break(Err(residual)),
            Self::Pending => ControlFlow::Continue(Poll::Pending),
        }
    }
}

// `Poll<Result<T, F>>`.

impl<T, F, E> FromResidual<Result<Infallible, F>> for Poll<Result<T, E>>
where
    E: From<F>,
{
    fn from_residual(residual: Result<Infallible, F>) -> Self {
        residual.map_or_else(|residual| Self::Ready(Err(residual.into())), infallible_into)
    }
}

impl<T, E> Try for Poll<Result<T, E>> {
    type Output = Poll<T>;
    type Residual = Result<Infallible, E>;

    fn from_output(output: Self::Output) -> Self {
        output.map(Ok)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Ready(Ok(output)) => ControlFlow::Continue(Poll::Ready(output)),
            Self::Ready(Err(residual)) => ControlFlow::Break(Err(residual)),
            Self::Pending => ControlFlow::Continue(Poll::Pending),
        }
    }
}
