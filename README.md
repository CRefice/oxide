# reel
A tiny scripting language written in Rust.

## Getting started
Fire up the REPL:
```
cargo run
```
Or, to run a script file:
```
cargo run -- script.reel
```

## Examples
Computing the log base 2 of a number:
```
let x = 256
let log = 1
while x / 2 > 1 {
	x = x / 2
	log = log + 1
}
```

## Limitations
The REPL will print the value of an entered expression, but since there are no functions(yet), you cannot print a value from a script file. To cope with that, at the end of execution, the interpreter will print the state of its global variables.
