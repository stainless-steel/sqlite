# SQLite [![Package][package-img]][package-url] [![Documentation][documentation-img]][documentation-url] [![Build][build-img]][build-url]

The package provides an interface to [SQLite].

## Example

Open a connection, create a table, and insert a few rows:

```rust
let connection = sqlite::open(":memory:").unwrap();

let query = "
    CREATE TABLE users (name TEXT, age INTEGER);
    INSERT INTO users VALUES ('Alice', 42);
    INSERT INTO users VALUES ('Bob', 69);
";
connection.execute(query).unwrap();
```

Select some rows and process them one by one as plain text, which is generally
not efficient:

```rust
let query = "SELECT * FROM users WHERE age > 50";

connection
    .iterate(query, |pairs| {
        for &(name, value) in pairs.iter() {
            println!("{} = {}", name, value.unwrap());
        }
        true
    })
    .unwrap();
```

Run the same query but using a prepared statement, which is much more efficient
than the previous technique:

```rust
use sqlite::State;

let query = "SELECT * FROM users WHERE age > ?";
let mut statement = connection.prepare(query).unwrap();
statement.bind((1, 50)).unwrap();

while let Ok(State::Row) = statement.next() {
    println!("name = {}", statement.read::<String, _>("name").unwrap());
    println!("age = {}", statement.read::<i64, _>("age").unwrap());
}
```

Run the same query but using a cursor, which is iterable:

```rust
let query = "SELECT * FROM users WHERE age > ?";

for row in connection
    .prepare(query)
    .unwrap()
    .into_iter()
    .bind((1, 50))
    .unwrap()
    .map(|row| row.unwrap())
{
    println!("name = {}", row.read::<&str, _>("name"));
    println!("age = {}", row.read::<i64, _>("age"));
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
