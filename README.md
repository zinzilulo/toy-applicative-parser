# Toy Applicative Parser in Rust

... for the lols.

Examples available in `main.rs`.

Expected Output:

```
===============================
[[], [('0', "")], [('2', "3")]]
[[], [('a', "1")]]
[(('4', '2'), "3")]
[[("hello", " world")], [("hello", "")]]
[([1, 2], "a")]
[[(97, "")], [(49, "")]]
[]
[F, L, B([R, F]), L, B([])]

===============================
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

===============================
[3, 6, 7, 5, 8, 9, 6, 9, 10]
```

**Credit:** Jamie Willis for his wonderful lecture note.
