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

> 10 != 100 ? print "true" : print "false";
true
```
3. Run a `lox` file
```JavaScript
// test.lox
fn sayHi(first,last) {
  print "Hi, " + first + " " + last + "!";
}

sayHi("Dear", "Reader");
```

```bash
cargo run test.lox

Hi, Dear Reader!
```
# Resources
- [_Crafting Interpreters_](https://craftinginterpreters.com/)
- _[The Rust Programming Language](https://doc.rust-lang.org/book/title-page.html)_ 
- [_Rust by Example_](https://doc.rust-lang.org/rust-by-example/)
- [_Rust Ownership, Borrowing and Lifetimes_](https://www.integralist.co.uk/posts/rust-ownership/)
- [_Becoming a Rust master_](https://www.youtube.com/watch?v=dQw4w9WgXcQ)
