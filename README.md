# rust-callgraph

## Build

```
$ cargo build
```

## Run

```
$ cargo new my_project && cd my_project
$ cat my_project/src/main.rs
fn main() {
    println!("In main");
    foo();
}

fn foo() {
    println!("In foo");
    bar();
}

fn bar() {
    println!("In bar");
    zoo();
}

fn zoo() {
    println!("In zoo");
    alpha();
}

fn alpha() {
    println!("In alpha");
    beta();
}

fn beta() {
    println!("In beta");
    gamma();
}

fn gamma() {
    println!("In gamma");
    delta();
}

fn delta() {
    println!("In delta");
    epsilon();
}

fn epsilon() {
    println!("In epsilon");
    zeta();
}

fn zeta() {
    println!("In zeta");
    // End of the call chain.
}
$ /path/to/target/debug/rust-callgraph .
digraph G {
    "foo" -> "bar" [label="my_project/src/main.rs:9:5"];
    "alpha" -> "beta" [label="my_project/src/main.rs:24:5"];
    "epsilon" -> "zeta" [label="my_project/src/main.rs:44:5"];
    "main" -> "foo" [label="my_project/src/main.rs:4:5"];
    "delta" -> "epsilon" [label="my_project/src/main.rs:39:5"];
    "gamma" -> "delta" [label="my_project/src/main.rs:34:5"];
    "zoo" -> "alpha" [label="my_project/src/main.rs:19:5"];
    "bar" -> "zoo" [label="my_project/src/main.rs:14:5"];
    "beta" -> "gamma" [label="my_project/src/main.rs:29:5"];
}
```

or generate png as:

```
$ sudo apt install graphviz
$ /path/to/target/debug/rust-callgraph . > tmp.graph
$ dot -Tpng tmp.graph -o test.png
```
