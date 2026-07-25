# sql-peas

sql-peas is yet another [sqlite3](https://www.sqlite.org) synchronous Rust crate.

With sql-peas, you can store prepared statements in the same struct as your sqlite connection, like peas in a pod:

```
/// This structure cannot be constructed in other Rust sqlite libraries
pub struct MyStruct {
    my_cxn: sql_peas::Connection,
    
    my_insert_handle: sql_peas::StatementHandle,
    my_update_handle: sql_peas::StatementHandle,
}
```

In sql-peas, preparing a statement gives you a _handle_, not a _reference_. References have lifetimes and they borrow their connection. Handles are opaque integers that implement `Clone` and `Copy`, at the slight cost of a fallible hash lookup when you convert them to a reference to execute a query.

Comparison with other Rust sqlite crates:

1. [stainless-steel/sqlite](https://github.com/stainless-steel/sqlite) is the upstream project of sql-peas. stainless-steel/sqlite is more mature and has more contributors.
2. [rusqlite](https://github.com/rusqlite/rusqlite) solves the same problem using a statement cache. rusqlite works fine and it's a mature project with many contributors.
3. [sqlx](https://github.com/transact-rs/sqlx) is a popular, mature crate with many contributors, but it is async-first, so it fulfills a difference niche than sql-peas.

## Using prepared statements in sql-peas

In sql-peas, preparing a statement returns an opaque `StatementHandle`.

```
let my_insert_handle: sql_peas::StatementHandle = my_cxn.prepare("INSERT INTO my_table (x) VALUES (my_value)")?;
```

The handle itself is only an integer and cannot make database queries. It's like the key to a bank box or PO box. To use the handle, call `borrow_statement` to borrow the real statement locked inside the connection object: 

```
// This _does_ borrow the connection
let stmt: &sql_peas::Statement = my_cxn.borrow_statement(my_insert_handle)?;
```

This `borrow_statement` call incurs a slight overhead - The connection must look up the handle in its internal hash table, and if the handle is invalid (e.g. the handle belongs to a different connection), then it will return an error.

Once you've borrowed the real statement, you can use it just like any other sqlite crate:

```
stmt.bind((1, "1"))?;
stmt.bind((2, "two"))?;
assert_eq!(stmt.next(), sql_peas::State::Row);
```

You can call `stmt.reset()` to reset the statement and run it multiple times without incurring further overhead:

```
stmt.reset()?;
stmt.bind((6, "six"))?;
stmt.bind((7, "seven"))?;
assert_eq!(stmt.next(), sql_peas::State::Row);
```

## Freeing prepared statements in sql-peas

To free a prepared statement, call `drop_statement`.

```
my_cxn.drop_statement(my_insert_handle)?;
```

This will return an error if the handle was already dropped, or if the handle was not from this connection.

After a statement is dropped, any call to `borrow_statement` with the handle will return an error.

Statements are not garbage-collected nor reference-counted. If you lose all handles to a statement, you won't be able to drop it.

Dropping a `Connection` frees all its prepared statements.
