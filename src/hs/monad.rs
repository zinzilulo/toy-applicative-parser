use crate::hs::applicative::Applicative;

pub trait Monad<'a>: Applicative<'a> {
    fn bind<B, K>(self, k: K) -> Self::Wrapped<B>
    where
        Self: Sized,
        K: Fn(Self::Inner) -> Self::Wrapped<B> + 'static;
}
