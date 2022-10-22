# SQLite [![Package][package-img]][package-url] [![Documentation][documentation-img]][documentation-url] [![Build][build-img]][build-url]

The package provides an interface to [SQLite].

## Example

Open a connection, create a table, and insert a few rows:

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

Run the same query but using a prepared statement, which is much more efficient
than the previous technique:

```rust
use sqlite::State;

let mut statement = connection
    .prepare("SELECT * FROM users WHERE age > ?")
    .unwrap()
    .bind(1, 50)
    .unwrap();

while let Ok(State::Row) = statement.next() {
    println!("name = {}", statement.read::<String>(0).unwrap());
    println!("age = {}", statement.read::<i64>(1).unwrap());
}
```

Run the same query but using a cursor, which is a wrapper around a prepared
statement providing the notion of row:

```rust
use sqlite::Value;

let mut cursor = connection
    .prepare("SELECT * FROM users WHERE age > ?")
    .unwrap()
    .into_cursor()
    .bind(&[Value::Integer(50)]).unwrap();

while let Some(Ok(row)) = cursor.next() {
    println!("name = {}", row.get::<String, _>(0));
    println!("age = {}", row.get::<i64, _>(1));
}
```

## Contribution

Your contribution is highly appreciated. Do not hesitate to open an issue or a
pull request. Note that any contribution submitted for inclusion in the project
will be licensed according to the terms given in [LICENSE.md](LICENSE.md).

[SQLite]: https://www.sqlite.org

[build-img]: https://github.com/stainless-steel/sqlite/workflows/build/badge.svg
[build-url]: https://github.com/stainless-steel/sqlite/actions/workflows/build.yml
[documentation-img]: https://docs.rs/sqlite/badge.svg
[documentation-url]: https://docs.rs/sqlite
[package-img]: https://img.shields.io/crates/v/sqlite.svg
[package-url]: https://crates.io/crates/sqlite
