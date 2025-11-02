use crate::prelude::{Alternative, Applicative, Functor, Maybe};
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

impl<'a, A: 'a> Functor for Parser<'a, A> {
    type Inner = A;
    type Wrapped<B> = Parser<'a, B>;

    fn fmap<B, F>(self, f: F) -> Self::Wrapped<B>
    where
        F: Fn(A) -> B + 'a,
    {
        let Parser(p) = self;
        Parser(Arc::new(move |input: &'a str| {
            p(input).into_iter().map(|(a, rest)| (f(a), rest)).collect()
        }))
    }
}

impl<'a, A: Clone + 'a> Applicative<'a> for Parser<'a, A> {
    fn pure<B>(a: B) -> Self::Wrapped<B>
    where
        B: Clone + 'a,
    {
        Parser(Arc::new(move |input: &'a str| vec![(a.clone(), input)]))
    }

    fn ap<X, B, FFn>(fa: &Self::Wrapped<X>, fab: Self::Wrapped<FFn>) -> Self::Wrapped<B>
    where
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

    fn liftA2<X, Y, C, F2>(fa: &Self::Wrapped<X>, fb: &Self::Wrapped<Y>, f: F2) -> Self::Wrapped<C>
    where
        X: Clone + 'static,
        Y: Clone + 'static,
        F2: Fn(X, Y) -> C + 'static,
    {
        let f_rc = Arc::new(f);
        let funs = {
            let f_rc_outer = Arc::clone(&f_rc);
            Parser(fa.0.clone()).fmap(move |x: X| {
                let f_rc_inner = Arc::clone(&f_rc_outer);
                move |y: Y| (f_rc_inner)(x.clone(), y)
            })
        };
        Self::ap(&Parser(fb.0.clone()), funs)
    }

    fn then_keep_right<X, Y>(fa: &Self::Wrapped<X>, fb: &Self::Wrapped<Y>) -> Self::Wrapped<Y>
    where
        X: Clone + 'a,
        Y: Clone + 'a,
    {
        let Parser(p_left) = fa.clone();
        let Parser(p_right) = Parser(fb.0.clone());
        Parser(Arc::new(move |input: &'a str| {
            let mut out = Vec::new();
            for (_x, rest1) in p_left(input) {
                for (y, rest2) in p_right(rest1) {
                    out.push((y, rest2));
                }
            }
            out
        }))
    }

    fn then_keep_left<X, Y>(fa: &Self::Wrapped<X>, fb: &Self::Wrapped<Y>) -> Self::Wrapped<X>
    where
        X: Clone + 'a,
        Y: Clone + 'a,
    {
        let Parser(p_left) = fa.clone();
        let Parser(p_right) = Parser(fb.0.clone());
        Parser(Arc::new(move |input: &'a str| {
            let mut out = Vec::new();
            for (x, rest1) in p_left(input) {
                for (_y, rest2) in p_right(rest1) {
                    out.push((x.clone(), rest2));
                }
            }
            out
        }))
    }
}

impl<'a, A: 'a + Clone> Alternative<'a> for Parser<'a, A> {
    fn empty() -> Self {
        Parser(Arc::new(|_| vec![]))
    }

    // fn alt(&self, fb: &Self) -> Self {
    //     let f_left = self.0.clone();
    //     let f_right = fb.0.clone();
    //     Parser(Arc::new(move |input| {
    //         f_left(input)
    //             .iter()
    //             .chain(&f_right(input))
    //             .cloned()
    //             .collect()
    //     }))
    // }

    fn alt(&self, fb: &Self) -> Self {
        let f_left = self.0.clone();
        let f_right = fb.0.clone();
        Parser(Arc::new(move |input| {
            let left = f_left(input);
            if left.is_empty() {
                f_right(input)
            } else {
                left
            }
        }))
    }

    fn alt_with<F>(&self, fb: F) -> Self
    where
        F: FnOnce() -> Self,
    {
        unimplemented!()
    }

    fn optional(&self) -> Self::Wrapped<Maybe<A>> {
        unimplemented!()
    }

    fn many(&self) -> Self::Wrapped<Vec<A>>
    where
        A: 'static,
    {
        let some: Parser<'a, Vec<A>> = self.some();
        let empty: Parser<'a, Vec<A>> = Self::pure(Vec::new());
        Parser::alt(&some, &empty)
    }

    fn some(&self) -> Self::Wrapped<Vec<A>>
    where
        A: 'static,
    {
        let f_first = self.0.clone();
        let p_many: Parser<'a, A> = Parser(self.0.clone());
        Parser(Arc::new(move |input| {
            let mut out = Vec::new();
            for (x, rest1) in f_first(input) {
                let rest_parser: Parser<'a, Vec<A>> = Parser::many(&p_many);
                for (mut xs, rest2) in (rest_parser.0)(rest1) {
                    xs.insert(0, x.clone());
                    out.push((xs, rest2));
                }
            }
            out
        }))
    }
}

pub trait IntoPure<'a>: Sized {
    fn into_pure(self) -> Parser<'a, Self>;
}

impl<'a, T: 'a + Clone> IntoPure<'a> for T {
    #[inline]
    fn into_pure(self) -> Parser<'a, Self> {
        <Parser<'a, Self> as Applicative<'a>>::pure(self)
    }
}
