use crate::hs::{Applicative, Functor, Monad};

pub type Maybe<T> = Option<T>;
pub use std::option::Option::{None as Nothing, Some as Just};

pub fn catMaybes<T, I>(iter: I) -> Vec<T>
where
    I: IntoIterator<Item = Option<T>>,
{
    iter.into_iter().flatten().collect()
}

impl<T> Functor for Option<T> {
    type Inner = T;
    type Wrapped<B> = Option<B>;

    fn fmap<B, F>(self, f: F) -> Self::Wrapped<B>
    where
        F: Fn(Self::Inner) -> B + 'static,
    {
        self.map(f)
    }
}

impl<'a, T> Applicative<'a> for Option<T> {
    fn pure<A>(a: A) -> Self::Wrapped<A>
    where
        A: Clone + 'a,
    {
        Some(a)
    }

    fn ap<X, B, FFn>(fa: &Self::Wrapped<X>, fab: Self::Wrapped<FFn>) -> Self::Wrapped<B>
    where
        X: Clone + 'a,
        FFn: Fn(X) -> B + 'a,
    {
        match (fa, fab) {
            (Some(x), Some(f)) => Some(f(x.clone())),
            _ => None,
        }
    }

    fn liftA2<X, Y, C, F2>(fa: &Self::Wrapped<X>, fb: &Self::Wrapped<Y>, f: F2) -> Self::Wrapped<C>
    where
        X: Clone + 'a,
        Y: Clone + 'a,
        F2: Fn(X, Y) -> C + 'a,
    {
        match (fa, fb) {
            (Some(x), Some(y)) => Some(f(x.clone(), y.clone())),
            _ => None,
        }
    }

    fn then_keep_right<X, Y>(fa: &Self::Wrapped<X>, fb: &Self::Wrapped<Y>) -> Self::Wrapped<Y>
    where
        Y: Clone + 'a,
    {
        match (fa, fb) {
            (Some(_), Some(y)) => Some(y.clone()),
            _ => None,
        }
    }

    fn then_keep_left<X, Y>(fa: &Self::Wrapped<X>, fb: &Self::Wrapped<Y>) -> Self::Wrapped<X>
    where
        X: Clone + 'a,
    {
        match (fa, fb) {
            (Some(x), Some(_)) => Some(x.clone()),
            _ => None,
        }
    }
}

impl<'a, T> Monad<'a> for Option<T> {
    fn bind<B, K>(self, k: K) -> Self::Wrapped<B>
    where
        Self: Sized,
        K: Fn(Self::Inner) -> Self::Wrapped<B> + 'a,
    {
        self.and_then(k)
    }
}
