# Shell.rs
Work in progres...

[![Build Status](https://travis-ci.org/icorderi/shell.rs.png?branch=master)](https://travis-ci.org/icorderi/shell.rs)

## Getting Started

If you are using [Cargo] and want to use the latest stable shell.rs [crate] available at [crates.io] add this to your `Cargo.toml`:

```toml
[dependencies.shell]
```

To get the dependency to be linked directly to the GitHub repo add this instead:

```toml
[dependencies.shell]
    git = "https://github.com/icorderi/shell.rs"
```

> **Note:** For more information on handling [dependencies] check the official cargo site.

[Cargo]: http://doc.crates.io/index.html
[crate]: https://crates.io/crates/shell.rs
[crates.io]: https://crates.io/
[dependencies]: http://doc.crates.io/guide.html#adding-dependencies

### Importing Shell.rs

To import Shell.rs from your code add this statement:

```rust
extern crate shell;
```

### [Optional] Installing Kinetic-rust from source

    git clone https://github.com/icorderi/shell.rs.git
    cd shell.rs
    cargo build

Additionally you can run the tests or compile the documentation locally:

    cargo test
    cargo doc

The local HTML documentation will be available at `./target/doc/shell/index.html`.

## Documentation

If you need help don't forget to checkout the online [documentation] for the library.

[documentation]: http://icorderi.github.io/shell.rs/doc/shell

## Contributing

Get involved with the [issues] or submit a [PR].

[issues]: https://github.com/icorderi/shell.rs/issues
[PR]: https://github.com/icorderi/shell.rs/pulls

## License

This project is licensed under The MIT License (MIT)
* [Markdown](LICENSE/mit.md) version
* [Original](LICENSE/mit.txt) version
