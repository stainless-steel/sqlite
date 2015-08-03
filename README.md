# SQLite [![Version][version-img]][version-url] [![Status][status-img]][status-url]

The package provides an interface to [SQLite][1].

## [Documentation][doc]

## Example

Create a table, insert a couple of rows, and fetch one:

```rust
let connection = sqlite::open(":memory:").unwrap();

connection.execute("
    CREATE TABLE users (name TEXT, age INTEGER);
    INSERT INTO users (name, age) VALUES ('Alice', 42);
    INSERT INTO users (name, age) VALUES ('Bob', 69);
").unwrap();

connection.process("SELECT * FROM users WHERE age > 50", |pairs| {
    for &(column, value) in pairs.iter() {
        println!("{} = {}", column, value.unwrap());
    }
    true
}).unwrap();
```

The same example using prepared statements:

```rust
let connection = sqlite::open(":memory:").unwrap();

connection.execute("
    CREATE TABLE users (name TEXT, age INTEGER)
").unwrap();

let mut statement = connection.prepare("
    INSERT INTO users (name, age) VALUES (?, ?)
").unwrap();

statement.bind(1, "Alice").unwrap();
statement.bind(2, 42).unwrap();
assert_eq!(statement.step().unwrap(), sqlite::State::Done);

statement.reset().unwrap();

statement.bind(1, "Bob").unwrap();
statement.bind(2, 69).unwrap();
assert_eq!(statement.step().unwrap(), sqlite::State::Done);

let mut statement = connection.prepare("
    SELECT * FROM users WHERE age > 50
").unwrap();

while let sqlite::State::Row = statement.step().unwrap() {
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
