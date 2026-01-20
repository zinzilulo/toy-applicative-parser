# Toy Applicative Parser

For the lols.

A deliberately literal translation of simple Haskell applicative parsers into Rust.

The goal is not performance. Rather, this project explores how closely Rust can mirror Haskell’s shape and style.

To support this, a minimal subset of the Haskell concepts are reimplemented in Rust.

## What’s here

### List-of-results parser

```haskell
type Parser a = String -> [(a, String)]
```

- Backtracking via sequencing (left-biased choice)
- Demonstrated in `mod parser_demo`

### CPS “Mini-Parsec”

```haskell
type Parser a = forall r. String
                -> (a -> String -> r)  -- cok
                -> (a -> r)            -- eok
                -> (String -> r)       -- cerr
                -> (String -> r)       -- eerr
                -> r
```

- Continuation-passing style
- Left-biased choice
- Includes `atomic` (Parsec’s `try`)
- Demonstrated in `mod miniparsec_demo`

## Layout

```text
src/
├── main.rs
├── parser.rs
├── miniparsec.rs
├── prelude.rs
└── prelude/
```

## Build and run

```bash
cargo run
```

Expected output:

```text
-- List-of-results parser
[[], [('0', "")], [('2', "3")]]
[[], [('a', "1")]]
[(('4', '2'), "3")]
[[("hello", " world")], [("hello", "")]]
[([1, 2], "a")]
[[(97, "")], [(49, "")]]
[]
[F, L, B([R, F]), L, B([])]

-- CPS Mini-Parsec
[None, Some('a')]
[None, Some('0'), Some('2')]
[Some("hi"), None]
[Some("hi"), Some("hello")]
[Some(""), Some("")]
[Some('b'), Some('a'), None]
[None, None]
[Some("hello"), Some("hello")]
[None, None, Some("hello"), None]
[Some('2'), Some('2')]

-- liftA2 on vector
[3, 6, 7, 5, 8, 9, 6, 9, 10]
```

**Credit:** [Yoda](https://github.com/zenzike/yoda) by Nicolas Wu for inspiration and [Jamie Willis](https://github.com/j-mie6) for his wonderful lectures.

**License:** Apache License, Version 2.0
