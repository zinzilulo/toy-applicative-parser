#![allow(non_snake_case)]

mod hs;
use crate::hs::{Alternative, Applicative, Functor, Just, Nothing, catMaybes};

mod parser;
use crate::parser::*;

use std::sync::Arc;

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
    Parser(Arc::new(move |s: &'a str| parse(&mk(), s)))
}

fn satisfy<'a, F>(f: F) -> Parser<'a, char>
where
    F: Fn(char) -> bool + 'a,
{
    fn eat<'a, F>(s: &'a str, f: &F) -> Vec<(char, &'a str)>
    where
        F: Fn(char) -> bool,
    {
        if let Some(c) = s.chars().next() {
            let rest = &s[c.len_utf8()..];
            if f(c) {
                return vec![(c, rest)];
            }
        }
        vec![]
    }

    Parser(Arc::new(move |s: &'a str| eat(s, &f)))
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
        let head: Parser<'a, char> = char_(c);
        let tail: Parser<'a, String> = string_(cs.to_string());

        let out: Parser<'a, String> = <Parser<'a, char>>::liftA2(&head, &tail, |h, t| {
            let mut s = String::with_capacity(1 + t.len());
            s.push(h);
            s.push_str(&t);
            s
        });
        out
    }
}

fn parse_fully<'a, A: Clone>(p: &Parser<'a, A>, s: &'a str) -> Vec<A> {
    parse(p, s)
        .into_iter()
        .filter_map(|(x, rest)| if rest.is_empty() { Some(x) } else { None })
        .collect()
}

fn cmd<'a>() -> Parser<'a, Option<Command>> {
    let oneOf = |cs: String| satisfy(move |c| cs.contains(c));
    let noneOf = |cs: String| satisfy(move |c| !cs.contains(c));

    let inner_b: Parser<'a, Command> = lazy(cmds).fmap(Command::B);

    let f: Parser<'a, Option<Command>> = <Parser<'a, Option<Command>>>::then_keep_left(
        &(Just(Command::F).into_pure()),
        &oneOf("MN".to_string()),
    );

    let l: Parser<'a, Option<Command>> =
        <Parser<'a, Option<Command>>>::then_keep_left(&(Just(Command::L).into_pure()), &char_('+'));

    let r: Parser<'a, Option<Command>> =
        <Parser<'a, Option<Command>>>::then_keep_left(&(Just(Command::R).into_pure()), &char_('-'));

    let b_head: Parser<'a, Command> = <Parser<'a, char>>::then_keep_left(
        &<Parser<'a, char>>::then_keep_right(&char_('['), &inner_b),
        &char_(']'),
    );

    let b: Parser<'a, Option<Command>> = b_head.fmap(Some);
    let n: Parser<Option<Command>> = noneOf("MN+-[]".to_string()).fmap(|_| Nothing::<Command>);

    f.alt(&l).alt(&r).alt(&b).alt(&n)
}

fn cmds<'a>() -> Parser<'a, Vec<Command>> {
    cmd().many().fmap(catMaybes)
}

fn main() {
    let item = satisfy(|_| true);

    let digit = satisfy(|c| c.is_ascii_digit());
    let digit_tests = vec![parse(&digit, "a"), parse(&digit, "0"), parse(&digit, "23")];
    println!("{digit_tests:?}");

    let item_tests = vec![parse(&item, ""), parse(&item, "a1")];
    println!("{item_tests:?}");

    let pair_of_digits = parse(
        &<Parser<'_, char> as Applicative<'_>>::liftA2(&digit, &digit, |x, y| (x, y)),
        "423",
    );
    println!("{pair_of_digits:?}");

    let string_bang =
        <Parser<'_, String> as Applicative<'_>>::then_keep_left(&string_("hello"), &char_('!'));
    let string_tests = vec![
        parse(&string_("hello"), "hello world"),
        parse(&string_bang, "hello!"),
    ];
    println!("{string_tests:?}");

    let many_digit_values = parse(
        &(digit.clone().fmap(|c: char| c.to_digit(10).unwrap()).many()),
        "12a",
    );
    println!("{many_digit_values:?}");

    let to_ord = |c: char| c as u32;
    let ord_item_tests = vec![
        parse(&(item.fmap(to_ord)), "a"),
        parse(&(digit.clone().fmap(to_ord)), "1"),
    ];
    println!("{ord_item_tests:?}");

    let full_only = parse_fully(&(digit.clone().many()), "12a");
    println!("{full_only:?}");

    let command_parse = parse(&cmds(), "M+X[-N]+[]");
    let first_result_commands = &(command_parse.first().unwrap()).0;
    println!("{first_result_commands:?}");

    let vec_liftA2_sum =
        <Vec<i32> as Applicative<'_>>::liftA2(&vec![1, 3, 4], &vec![2, 5, 6], |x, y| x + y);
    println!("{vec_liftA2_sum:?}");
}
