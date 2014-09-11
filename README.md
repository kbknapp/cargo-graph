cargo-dot
=========

A tool to graph transitive dependencies for rust projects using Cargo

Usage
-----
In a rust project using Cargo, run the following commands (assuming
cargo-dot is on your PATH)
```sh
cargo build # If you don't have a Cargo.lock file
cargo dot | dot -Tsvg > Cargo.svg
```

Examples
--------
![cargo-dot dependencies](etc/cargo-dot.png)

![servo dependencies](etc/servo.png)
