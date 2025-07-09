# rlox 🦀
#### Yet another implementation of `jlox` in Rust, via _Robert Nystrom’s_ Crafting Interpreters
#

This isn't a line-for-line port but rather (an attempt at) a faithful, idiomatic re-implementation. More than anything, it's an exercise.  

# Usage
1. Clone and build
```zsh
git clone https://github.com/cachebag/rlox.git
cd rlox
cargo build --release
```
2. Run REPL (currently stateless)
```zsh
cargo run

> print "Hello, world!";
Hello, World!

> var msg = 10 != 100 ? "true" : "false"; print msg;
true
```
3. Run a `lox` file
```JavaScript
// examples/recursive_fib.lox
fn fib(n) {
  return n <= 1 ? n : fib(n - 2) + fib(n - 1);
}

for (var i = 0; i < 20; i = i + 1) {
  print fib(i);
}
```

```bash
cargo run examples/recursive_fib.lox

0
1
1
2
3
5
8
13
21
...
```
# Resources
- [_Crafting Interpreters_](https://craftinginterpreters.com/)
- _[The Rust Programming Language](https://doc.rust-lang.org/book/title-page.html)_ 
- [_Rust by Example_](https://doc.rust-lang.org/rust-by-example/)
- [_Rust Ownership, Borrowing and Lifetimes_](https://www.integralist.co.uk/posts/rust-ownership/)
- [_Becoming a Rust master_](https://www.youtube.com/watch?v=dQw4w9WgXcQ)
