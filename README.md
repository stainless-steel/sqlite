# SQLite [![Version][version-img]][version-url] [![Status][status-img]][status-url]

The package provides an interface to [SQLite][1].

## [Documentation][doc]

## Example

```rust
let connection = sqlite::open(":memory:").unwrap();

connection.execute("
    CREATE TABLE `users` (id INTEGER, name VARCHAR(255));
    INSERT INTO `users` (id, name) VALUES (42, 'Alice');
").unwrap();

connection.process("SELECT * FROM `users`", |pairs| {
    for &(column, value) in pairs.iter() {
        println!("{} = {}", column, value.unwrap());
    }
    true
}).unwrap();
```

The same example using prepared statements:

```rust
use sqlite::State;

let connection = sqlite::open(":memory:").unwrap();

connection.execute("
    CREATE TABLE `users` (id INTEGER, name VARCHAR(255))
");

let mut statement = connection.prepare("
    INSERT INTO `users` (id, name) VALUES (?, ?)
").unwrap();
statement.bind(1, 42).unwrap();
statement.bind(2, "Alice").unwrap();
assert_eq!(statement.step().unwrap(), State::Done);

let mut statement = connection.prepare("SELECT * FROM `users`").unwrap();
while let State::Row = statement.step().unwrap() {
    println!("id = {}", statement.read::<i64>(0).unwrap());
    println!("name = {}", statement.read::<String>(1).unwrap());
}
```

## Contributing

1. Fork the project.
2. Implement your idea.
3. Open a pull request.

[1]: https://www.sqlite.org

[version-img]: http://stainless-steel.github.io/images/crates.svg
[version-url]: https://crates.io/crates/sqlite
[status-img]: https://travis-ci.org/stainless-steel/sqlite.svg?branch=master
[status-url]: https://travis-ci.org/stainless-steel/sqlite
[doc]: https://stainless-steel.github.io/sqlite
