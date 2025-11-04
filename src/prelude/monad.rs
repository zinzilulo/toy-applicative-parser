use crate::prelude::applicative::Applicative;
use crate::prelude::functor::Functor;

pub trait Monad<'a>: Applicative<'a> {
    fn bind<A, B, K>(ma: &Self::Wrapped<A>, k: K) -> Self::Wrapped<B>
    where
        A: 'a + Clone,
        B: 'a,
        K: Fn(A) -> Self::Wrapped<B> + 'a;

    fn join<A>(mma: &Self::Wrapped<Self::Wrapped<A>>) -> Self::Wrapped<A>
    where
        Self: 'a + Clone,
        <Self as Functor<'a>>::Wrapped<A>: std::clone::Clone,
        A: 'a + Clone,
    {
        Self::bind(mma, |ma| ma)
    }
}

pub fn bind<'a, C, A, B>(pa: &C::Wrapped<A>, f: impl Fn(A) -> C::Wrapped<B> + 'a) -> C::Wrapped<B>
where
    A: Clone,
    C: Monad<'a>,
{
    C::bind(pa, f)
}

pub fn join<'a, C, A>(mma: &C::Wrapped<C::Wrapped<A>>) -> C::Wrapped<A>
where
    A: Clone + 'a,
    C: Monad<'a> + Clone + 'a,
    C::Wrapped<A>: Clone,
{
    bind::<C, _, _>(mma, |ma| ma)
}
