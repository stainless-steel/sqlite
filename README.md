# SQLite [![Version][version-img]][version-url] [![Status][status-img]][status-url]

The package provides an interface to [SQLite][1].

## [Documentation][doc]

## Example

```rust
let connection = sqlite::open(":memory:").unwrap();

connection.execute(r#"
    CREATE TABLE `users` (id INTEGER, name VARCHAR(255));
    INSERT INTO `users` (id, name) VALUES (1, 'Alice');
"#).unwrap();

connection.process("SELECT * FROM `users`;", |pairs| {
    for &(column, value) in pairs.iter() {
        println!("{} = {}", column, value.unwrap());
    }
    true
}).unwrap();
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
