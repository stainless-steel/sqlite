# SQLite [![Version][version-img]][version-url] [![Status][status-img]][status-url]

The package provides an interface to [SQLite][1].

## [Documentation][doc]

## Example

The example given below can be ran using the following command:

```
cargo run --example workflow
```

```rust
extern crate sqlite;

use std::path::PathBuf;

fn main() {
    let path = setup();
    let database = sqlite::open(&path).unwrap();

    database.instruct(r#"
        CREATE TABLE `users` (id INTEGER, name VARCHAR(255));
        INSERT INTO `users` (id, name) VALUES (1, 'Alice');
    "#).unwrap();

    database.iterate("SELECT * FROM `users`;", |pairs| {
        for (ref column, ref value) in pairs {
            println!("{} = {}", column, value);
        }
        true
    }).unwrap();
}

fn setup() -> PathBuf {
    let path = PathBuf::from("database.sqlite3");
    if std::fs::metadata(&path).is_ok() {
        std::fs::remove_file(&path).unwrap();
    }
    path
}
```

## Contributing

1. Fork the project.
2. Implement your idea.
3. Open a pull request.

[1]: https://www.sqlite.org

[version-img]: https://img.shields.io/crates/v/sqlite.svg
[version-url]: https://crates.io/crates/sqlite
[status-img]: https://travis-ci.org/stainless-steel/sqlite.svg?branch=master
[status-url]: https://travis-ci.org/stainless-steel/sqlite
[doc]: https://stainless-steel.github.io/sqlite
