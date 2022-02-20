# SQLite [![Package][package-img]][package-url] [![Documentation][documentation-img]][documentation-url] [![Build][build-img]][build-url]

The package provides an interface to [SQLite][1].

## Example

Open a connection, create a table, and insert some rows:

```rust
let connection = sqlite::open(":memory:").unwrap();

connection
    .execute(
        "
        CREATE TABLE users (name TEXT, age INTEGER);
        INSERT INTO users VALUES ('Alice', 42);
        INSERT INTO users VALUES ('Bob', 69);
        ",
    )
    .unwrap();
```

Select some rows and process them one by one as plain text:

```rust
connection
    .iterate("SELECT * FROM users WHERE age > 50", |pairs| {
        for &(column, value) in pairs.iter() {
            println!("{} = {}", column, value.unwrap());
        }
        true
    })
    .unwrap();
```

The same query using a prepared statement, which is much more efficient than the
previous technique:

```rust
use sqlite::State;

let mut statement = connection
    .prepare("SELECT * FROM users WHERE age > ?")
    .unwrap();

statement.bind(1, 50).unwrap();

while let State::Row = statement.next().unwrap() {
    println!("name = {}", statement.read::<String>(0).unwrap());
    println!("age = {}", statement.read::<i64>(1).unwrap());
}
```

The same query using a cursor, which is a wrapper around a prepared statement
providing the notion of row and featuring all-at-once binding:

```rust
use sqlite::Value;

let mut cursor = connection
    .prepare("SELECT * FROM users WHERE age > ?")
    .unwrap()
    .into_cursor();

cursor.bind(&[Value::Integer(50)]).unwrap();

while let Some(row) = cursor.next().unwrap() {
    println!("name = {}", row[0].as_string().unwrap());
    println!("age = {}", row[1].as_integer().unwrap());
}
```

## Contribution

Your contribution is highly appreciated. Do not hesitate to open an issue or a
pull request. Note that any contribution submitted for inclusion in the project
will be licensed according to the terms given in [LICENSE.md](LICENSE.md).

[1]: https://www.sqlite.org

[build-img]: https://github.com/stainless-steel/sqlite/workflows/build/badge.svg
[build-url]: https://github.com/stainless-steel/sqlite/actions/workflows/build.yml
[documentation-img]: https://docs.rs/sqlite/badge.svg
[documentation-url]: https://docs.rs/sqlite
[package-img]: https://img.shields.io/crates/v/sqlite.svg
[package-url]: https://crates.io/crates/sqlite
