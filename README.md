# Oxide

A small, general-purpose imperative programming language written with simplicity in mind. Programs written in Oxide are compiled to run on a custom VM with minimal runtime overhead and memory footprint.


## Running the project
You'll need Cargo to compile this project.

Spin up a REPL by simply building and running the project:
```bash
cargo run --release
```

Or run a script file instead:
```bash
cargo run --release -- my_beautiful_script.o2
```

## Whirlwind tour

Let's take a look at the main features and syntax of Oxide. As you'll see, the syntax should be mostly familiar, as it takes inspiration from its host language, Rust.

### Data types

Oxide (currently) supports 3 basic data types: real numbers, booleans, and strings.
The total amount of types is 5, which includes `null` and functions, but we'll look at those later.

```rust
// Numbers
let x = 10
// Not just integers, either!
x = 42.5

// Strings
let greeting = "hello, world!"

// Booleans
let b = true and false // false
```

Oxide is dynamically typed, which means doing this is fine:

```rust
let what_is_it = 15
what_is_it = "i don't know!"
```

Also, yes, as you might have noticed, there's no need for semicolons at the end of statements.

### Variables
Local variables can be declared through the `let` keyword and follow standard lexical scoping rules:

```rust
let a = 1
let b = 1
print(a + b) // 30

b = 100 
print(a + b) // 101

let x = "outer" 
{
	// New scope: a is a new variable (shadowing)
	let x = "inner" 
	print(x) // "inner"
}
print(x) // "outer" 
```

### Expressions
Mathematical and boolean expressions are expressions, and as such return a value:

```rust
let x = 20 * 2 + 1 * 2 // x == 42

let first_name = "john"
let last_name = "doe"
let full_name = first_name + " " + last_name // "john doe"

let is_john = first_name == "john" or last_name == "doe"
```

Blocks are expressions too!
```rust
let x = {
	let a = 5
	let b = 10 
	a + b
} // x == 15
```

In fact, everything is an expression, including control flow (which would typically be statements in other programming languages!) We'll see how that works in the next section.

### Control flow

There's only two types of control flow statements: `if` and `while`:

```rust
// If with block:
let a = ... // Some number
if a > 42 {
	print("that's a pretty huge number")
} else if a == 42 {
	print("my favorite number!")
} else {
	print("get outta here with that puny stuff")
}

// One-line if-expression with "then":
let clamped = if a < 42 then 42 else a
```

I said previously that all statements return a value. But what if the condition isn't taken?

```rust
let x = if false then 50
print(x) // ???
```
In this case, we return a special value: `null`. That is also the value returned by functions which don't return anything, as we'll see later.

Looping is only performed through `while`:

```rust
let x = 100
let sqrt = x
while sqrt * sqrt > x {
	sqrt = sqrt - 1
}
// sqrt == 10
```

Loops also return a value. More precisely, they return the last value they evaluated to:

```rust
let a = 100
let x = while a > 42 {
	a = a / 2
	a // Note: this can be omitted, since assignments evaluate to their right hand side.
}
// a == 25
```

### Functions

Functions are the final datatype we'll be looking at. They're declared as follows:

```rust
fn factorial(x) -> if x > 1 then x * factorial(x-1) else 1
```

This is a simple, one-line function. You can use a block (with or without an arrow) to declare a multi-line function:
```rust
fn fib(n) {
	let a = 0
	let b = 1
	while n != 0 {
		n = n - 1
		let tmp = a
		a = b
		b = b + tmp
	}
	a
}
```

Note that there's no need to use a `return` keyword: just like loops (and every other "statement" in Oxide) functions evaluate to the last expression they execute.

(Unfortunately, for now you also _can't_ use the `return` keyword to exit early from a function. This will be fixed in a future release.)

Note that functions are values just like any other, meaning they can themselves be passed to other functions (yay for functional programming!)
