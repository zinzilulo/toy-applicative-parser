use crate::prelude::applicative::Applicative;
use crate::prelude::maybe::{Just, Maybe, Nothing};

pub trait Alternative<'a>: Applicative<'a> {
    fn empty<A>() -> Self::Wrapped<A>
    where
        A: 'a;

    fn alt<A>(fa: Self::Wrapped<A>, fb: Self::Wrapped<A>) -> Self::Wrapped<A>
    where
        A: 'a;

    fn alt_with<A, F>(fa: Self::Wrapped<A>, fb: F) -> Self::Wrapped<A>
    where
        A: 'a,
        F: Fn() -> Self::Wrapped<A>,
    {
        let b = fb();
        Self::alt(fa, b)
    }

    fn optional<A>(p: Self::Wrapped<A>) -> Self::Wrapped<Maybe<A>>
    where
        A: 'a + Clone,
    {
        let just = Self::fmap(p, Just);
        let nothing = Self::pure::<Maybe<A>>(Nothing);
        Self::alt(just, nothing)
    }

    fn some<A>(p: Self::Wrapped<A>) -> Self::Wrapped<Vec<A>>
    where
        Self: Sized,
        A: 'a + Clone,
        Self::Wrapped<A>: Clone,
    {
        let tail = Self::many(p.clone());
        Self::liftA2(p.clone(), tail, |x, mut xs| {
            xs.insert(0, x);
            xs
        })
    }

    fn many<A>(p: Self::Wrapped<A>) -> Self::Wrapped<Vec<A>>
    where
        Self: Sized,
        A: 'a + Clone,
        Self::Wrapped<A>: Clone,
    {
        let some = Self::some(p);
        let empty_vec: Self::Wrapped<Vec<A>> = Self::empty::<Vec<A>>();
        Self::alt(some, empty_vec)
    }
}

pub fn empty<'a, C, A>() -> C::Wrapped<A>
where
    C: Alternative<'a>,
    A: 'a,
{
    C::empty::<A>()
}

pub fn alt<'a, C, A>(fa: C::Wrapped<A>, fb: C::Wrapped<A>) -> C::Wrapped<A>
where
    C: Alternative<'a>,
    A: 'a,
{
    C::alt(fa, fb)
}

pub fn alt_with<'a, C, A, F>(fa: C::Wrapped<A>, fb: F) -> C::Wrapped<A>
where
    C: Alternative<'a>,
    A: 'a,
    F: Fn() -> C::Wrapped<A>,
{
    C::alt_with(fa, fb)
}

pub fn optional<'a, C, A>(p: C::Wrapped<A>) -> C::Wrapped<Maybe<A>>
where
    C: Alternative<'a>,
    A: 'a + Clone,
{
    C::optional(p)
}

pub fn some<'a, C, A>(p: C::Wrapped<A>) -> C::Wrapped<Vec<A>>
where
    C: Alternative<'a>,
    A: 'a + Clone,
    C::Wrapped<A>: Clone,
{
    C::some(p)
}

pub fn many<'a, C, A>(p: C::Wrapped<A>) -> C::Wrapped<Vec<A>>
where
    C: Alternative<'a>,
    A: 'a + Clone,
    C::Wrapped<A>: Clone,
{
    C::many(p)
}
