#![allow(non_snake_case)]

mod prelude;
use crate::prelude::liftA2;
mod miniparsec;
mod parser;

mod parser_demo {
    use crate::parser::{IntoPure, Parser, parse};
    use crate::prelude::{
        Just, Maybe, Nothing, alt, catMaybes, fmap, liftA2, many, then_keep_left, then_keep_right,
    };
    use std::sync::Arc;

    type P<'a> = Parser<'a, ()>;

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Command {
        F,
        L,
        R,
        B(Vec<Command>),
    }

    fn lazy<'a, A, F>(mk: F) -> Parser<'a, A>
    where
        F: Fn() -> Parser<'a, A> + 'a,
        A: 'a,
    {
        Parser(Arc::new(move |s: &'a str| parse(mk(), s)))
    }

    fn satisfy<'a, F>(f: F) -> Parser<'a, char>
    where
        F: Fn(char) -> bool + Clone + 'a,
    {
        fn eat<F>(s: &str, f: F) -> Vec<(char, &str)>
        where
            F: Fn(char) -> bool,
        {
            if let Just(c) = s.chars().next() {
                let rest = &s[c.len_utf8()..];
                if f(c) {
                    return vec![(c, rest)];
                }
            }
            vec![]
        }

        Parser(Arc::new(move |s: &'a str| eat(s, f.clone())))
    }

    fn char_<'a>(c: char) -> Parser<'a, char> {
        satisfy(move |chr| c == chr)
    }

    fn string_<'a, S>(gs: S) -> Parser<'a, String>
    where
        S: Into<String>,
    {
        let s = gs.into();
        if s.is_empty() {
            String::new().into_pure()
        } else {
            let mut it = s.chars();
            let c = it.next().unwrap();
            let cs = it.as_str();

            let head = char_(c);
            let tail = string_(cs.to_string());

            let out = liftA2::<P<'a>, _, _, _>(head, tail, |h, t| {
                let mut s = String::with_capacity(1 + t.len());
                s.push(h);
                s.push_str(&t);
                s
            });
            out
        }
    }

    fn parse_fully<'a, A: Clone>(p: Parser<'a, A>, s: &'a str) -> Vec<A> {
        parse(p, s)
            .into_iter()
            .filter_map(|(x, rest)| if rest.is_empty() { Just(x) } else { Nothing })
            .collect()
    }

    fn cmd<'a>() -> Parser<'a, Maybe<Command>> {
        let oneOf = |cs: String| satisfy(move |c| cs.contains(c));
        let noneOf = |cs: String| satisfy(move |c| !cs.contains(c));

        let inner_b = fmap::<P<'a>, _, _>(lazy(cmds), Command::B);

        let f =
            then_keep_left::<P<'a>, _, _>(Just(Command::F).into_pure(), oneOf("MN".to_string()));

        let l = then_keep_left::<P<'a>, _, _>(Just(Command::L).into_pure(), char_('+'));

        let r = then_keep_left::<P<'a>, _, _>(Just(Command::R).into_pure(), char_('-'));

        let b_head = then_keep_left::<P<'_>, _, _>(
            then_keep_right::<P<'a>, _, _>(char_('['), inner_b),
            char_(']'),
        );

        let b = fmap::<P<'a>, _, _>(b_head, Just);

        let n = fmap::<P<'a>, _, _>(noneOf("MN+-[]".to_string()), |_| Nothing::<Command>);

        alt::<P<'a>, _>(
            alt::<P<'a>, _>(alt::<P<'a>, _>(f, l), r),
            alt::<P<'a>, _>(b, n),
        )
    }

    fn cmds<'a>() -> Parser<'a, Vec<Command>> {
        fmap::<P<'a>, _, _>(many::<P<'a>, _>(cmd()), catMaybes)
    }

    pub fn run() {
        let item = satisfy(|_| true);

        let digit = satisfy(|c| c.is_ascii_digit());
        let digit_tests = vec![
            parse(digit.clone(), "a"),
            parse(digit.clone(), "0"),
            parse(digit.clone(), "23"),
        ];
        println!("{digit_tests:?}");

        let item_tests = vec![parse(item.clone(), ""), parse(item.clone(), "a1")];
        println!("{item_tests:?}");

        let multi_digit = liftA2::<P<'_>, _, _, _>(digit.clone(), digit.clone(), |x, y| (x, y));
        let pair_of_digits = parse(multi_digit, "423");
        println!("{pair_of_digits:?}");

        let string_bang = then_keep_left::<P<'_>, _, _>(string_("hello"), char_('!'));
        let string_tests = vec![
            parse(string_("hello"), "hello world"),
            parse(string_bang, "hello!"),
        ];
        println!("{string_tests:?}");

        let many_digit_values = parse(
            many::<P<'_>, _>(fmap::<P<'_>, _, _>(digit.clone(), |c: char| -> u32 {
                c.to_digit(10).unwrap()
            })),
            "12a",
        );
        println!("{many_digit_values:?}");

        let to_ord = |c: char| c as u32;
        let ord_item_tests = vec![
            parse(fmap::<P<'_>, _, _>(item.clone(), to_ord), "a"),
            parse(fmap::<P<'_>, _, _>(digit.clone(), to_ord), "1"),
        ];
        println!("{ord_item_tests:?}");

        let full_only = parse_fully(many::<P<'_>, _>(digit), "12a");
        println!("{full_only:?}");

        let command_parse = parse(cmds(), "M+X[-N]+[]");
        let first_result_commands = command_parse.first().unwrap().0.clone();
        println!("{first_result_commands:?}");
    }
}

mod miniparsec_demo {
    use crate::miniparsec::{IntoPure, Parser, runParser};
    use crate::prelude::{Maybe, alt, fmap, liftA2, then_keep_left, then_keep_right};
    use std::sync::Arc;

    fn satisfy<'a, F, R: Clone>(pf: F) -> Parser<'a, char, R>
    where
        F: Fn(char) -> bool + 'static,
    {
        Parser(Arc::new(
            move |inp: &'a str,
                  cok: Arc<dyn Fn(char, &'a str) -> R>,
                  _eok: Arc<dyn Fn(char) -> R>,
                  _cerr: Arc<dyn Fn(&'a str) -> R>,
                  eerr: Arc<dyn Fn(&'a str) -> R>|
                  -> R {
                let mut chars = inp.chars();

                match chars.next() {
                    Some(c) if pf(c) => cok(c, chars.as_str()),
                    _ => eerr(inp),
                }
            },
        ))
    }

    fn char_<'a>(c: char) -> Parser<'a, char, Maybe<String>> {
        satisfy(move |chr| c == chr)
    }

    fn string_<'a, S>(gs: S) -> Parser<'a, String, Maybe<String>>
    where
        S: Into<String>,
    {
        let s = gs.into();
        if s.is_empty() {
            String::new().into_pure()
        } else {
            let mut it = s.chars();
            let c = it.next().unwrap();
            let cs = it.as_str();

            let head = char_(c);
            let tail = string_(cs.to_string());

            let out = liftA2::<Parser<'a, char, Maybe<String>>, _, _, _>(head, tail, |h, t| {
                let mut s = String::with_capacity(1 + t.len());
                s.push(h);
                s.push_str(&t);
                s
            });
            out
        }
    }

    fn atomic<'a, A: 'a, R: 'a>(p: Parser<'a, A, R>) -> Parser<'a, A, R>
where {
        Parser(Arc::new(
            move |inp: &'a str,
                  cok: Arc<dyn Fn(A, &'a str) -> R>,
                  eok: Arc<dyn Fn(A) -> R>,
                  _cerr: Arc<dyn Fn(&'a str) -> R>,
                  eerr: Arc<dyn Fn(&'a str) -> R>|
                  -> R { (p.0)(inp, cok, eok, eerr.clone(), eerr.clone()) },
        ))
    }

    pub fn run() {
        let item = satisfy(|_| true);
        let item_tests = vec![runParser(item.clone(), ""), runParser(item.clone(), "a1")];
        println!("{item_tests:?}");

        let digit = satisfy(|c| c.is_ascii_digit());
        let digit_tests = vec![
            runParser(digit.clone(), "a"),
            runParser(digit.clone(), "0"),
            runParser(digit.clone(), "23"),
        ];
        println!("{digit_tests:?}");

        let non_atomic =
            alt::<Parser<'_, String, Maybe<String>>, _>(string_("hi"), string_("hello"));
        let non_atomic_tests = vec![
            runParser(non_atomic.clone(), "hihello"),
            runParser(non_atomic.clone(), "hellohi"),
        ];
        println!("{non_atomic_tests:?}");

        let full =
            alt::<Parser<'_, String, Maybe<String>>, _>(atomic(string_("hi")), string_("hello"));
        let full_tests = vec![
            runParser(full.clone(), "hihello"),
            runParser(full.clone(), "hellohi"),
        ];
        println!("{full_tests:?}");

        let empty_string = string_("");
        let tests = vec![
            runParser(empty_string.clone(), ""),
            runParser(empty_string.clone(), "abc"),
        ];
        println!("{tests:?}");

        fn char_item<'a>(c: char) -> Parser<'a, char, Maybe<char>> {
            satisfy(move |chr| c == chr)
        }
        let char_a = char_item('a');
        let char_b = char_item('b');
        let char_alt = alt::<Parser<'_, char, Maybe<char>>, _>(char_a, char_b);
        let char_alt_tests = vec![
            runParser(char_alt.clone(), "b"),
            runParser(char_alt.clone(), "a"),
            runParser(char_alt.clone(), "c"),
        ];
        println!("{char_alt_tests:?}");

        let prefix_no_atomic =
            alt::<Parser<'_, String, Maybe<String>>, _>(string_("hi"), string_("hello"));
        let prefix_no_atomic_tests = vec![
            runParser(prefix_no_atomic.clone(), "hello"),
            runParser(prefix_no_atomic.clone(), "hellohi"),
        ];
        println!("{prefix_no_atomic_tests:?}");

        let prefix_with_atomic =
            alt::<Parser<'_, String, Maybe<String>>, _>(atomic(string_("hi")), string_("hello"));
        let prefix_with_atomic_tests = vec![
            runParser(prefix_with_atomic.clone(), "hello"),
            runParser(prefix_with_atomic.clone(), "hellohi"),
        ];
        println!("{prefix_with_atomic_tests:?}");

        let hello_then_bang =
            then_keep_left::<Parser<'_, String, Maybe<String>>, _, _>(string_("hello"), char_('!'));
        let bang_then_hello = then_keep_right::<Parser<'_, String, Maybe<String>>, _, _>(
            char_('!'),
            string_("hello"),
        );
        let then_tests = vec![
            runParser(hello_then_bang.clone(), "hello!"),
            runParser(hello_then_bang.clone(), "hello"),
            runParser(bang_then_hello.clone(), "!hello"),
            runParser(bang_then_hello.clone(), "hello!"),
        ];
        println!("{then_tests:?}");

        const fn to_upper(c: char) -> char {
            c.to_ascii_uppercase()
        }
        const fn next_char(c: char) -> char {
            (c as u8 + 1) as char
        }
        let fmap_left = fmap::<Parser<'_, char, Maybe<char>>, _, _>(
            fmap::<Parser<'_, char, Maybe<char>>, _, _>(digit.clone(), to_upper),
            next_char,
        );
        let fmap_right =
            fmap::<Parser<'_, char, Maybe<char>>, _, _>(digit.clone(), |c| next_char(to_upper(c)));
        let fmap_tests = vec![
            runParser(fmap_left.clone(), "1"),
            runParser(fmap_right.clone(), "1"),
        ];
        println!("{fmap_tests:?}");
    }
}

fn main() {
    println!("-- List-of-results parser");
    parser_demo::run();

    println!("\n-- CPS Mini-Parsec");
    miniparsec_demo::run();

    println!("\n-- liftA2 on vector");
    let vec_liftA2_sum = liftA2::<Vec<()>, _, _, _>(vec![1, 3, 4], vec![2, 5, 6], |x, y| x + y);
    println!("{vec_liftA2_sum:?}");
}
