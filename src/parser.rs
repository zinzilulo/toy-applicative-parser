use crate::prelude::{Alternative, Applicative, Functor, Just};
use std::sync::Arc;

pub struct Parser<'a, A>(pub Arc<dyn Fn(&'a str) -> Vec<(A, &'a str)> + 'a>);

impl<'a, A> Clone for Parser<'a, A> {
    fn clone(&self) -> Self {
        Parser(self.0.clone())
    }
}

pub fn parse<'a, A>(p: &Parser<'a, A>, input: &'a str) -> Vec<(A, &'a str)> {
    (p.0)(input)
}

impl<'a, A: 'a> Functor<'a> for Parser<'a, A> {
    type Wrapped<B>
        = Parser<'a, B>
    where
        B: 'a;

    fn fmap<X, B, F>(fa: &Self::Wrapped<X>, f: F) -> Self::Wrapped<B>
    where
        B: 'a,
        X: 'a,
        F: Fn(X) -> B + 'a,
    {
        let Parser(p) = fa.clone();
        Parser(Arc::new(move |input: &'a str| {
            p(input).into_iter().map(|(x, rest)| (f(x), rest)).collect()
        }))
    }
}

impl<'a, A: 'a> Applicative<'a> for Parser<'a, A> {
    fn pure<B>(a: &B) -> Self::Wrapped<B>
    where
        B: Clone + 'a,
    {
        let b = a.clone();
        Parser(Arc::new(move |input: &'a str| vec![(b.clone(), input)]))
    }

    fn ap<X, B, FFn>(fa: &Self::Wrapped<X>, fab: Self::Wrapped<FFn>) -> Self::Wrapped<B>
    where
        B: 'a,
        X: Clone + 'a,
        FFn: Fn(X) -> B + 'a,
    {
        let Parser(p_val) = fa.clone();
        let Parser(p_fun) = fab;
        Parser(Arc::new(move |input: &'a str| {
            let mut out = Vec::new();
            for (f, rest1) in p_fun(input) {
                for (x, rest2) in p_val(rest1) {
                    out.push((f(x), rest2));
                }
            }
            out
        }))
    }
}

impl<'a, A: 'a> Alternative<'a> for Parser<'a, A> {
    fn empty<B>() -> Self::Wrapped<B>
    where
        B: 'a,
    {
        Parser(Arc::new(|_| vec![]))
    }

    fn alt<B>(fa: &Self::Wrapped<B>, fb: &Self::Wrapped<B>) -> Self::Wrapped<B>
    where
        B: 'a,
    {
        let left = fa.0.clone();
        let right = fb.0.clone();
        Parser(Arc::new(move |input| {
            let l = left(input);

            if !l.is_empty() {
                return l;
            }

            right(input)
        }))
    }

    fn many<B>(p: &Self::Wrapped<B>) -> Self::Wrapped<Vec<B>>
    where
        Self: Sized,
        B: 'a + Clone,
    {
        let p = p.clone();
        Parser(Arc::new(move |mut input: &'a str| {
            let mut acc: Vec<B> = Vec::new();
            loop {
                let mut step = (p.0)(input).into_iter();

                let Just((b, rest)) = step.next() else { break };

                if rest.len() == input.len() {
                    break;
                }

                acc.push(b);
                input = rest;
            }
            vec![(acc, input)]
        }))
    }
}

pub trait IntoPure<'a>: Sized {
    fn into_pure(self) -> Parser<'a, Self>;
}

impl<'a, T: 'a + Clone> IntoPure<'a> for T {
    #[inline]
    fn into_pure(self) -> Parser<'a, Self> {
        <Parser<'a, Self> as Applicative<'a>>::pure(&self)
    }
}
