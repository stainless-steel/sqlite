extern crate sqlite;
extern crate temporary;

use std::path::PathBuf;
use temporary::Directory;

macro_rules! ok(
    ($result:expr) => ($result.unwrap());
);

#[test]
fn open() {
    let (path, _directory) = setup();
    let mut database = ok!(sqlite::open(&path));

    let sql = r#"CREATE TABLE `users` (id INTEGER, name VARCHAR(255), age REAL);"#;
    ok!(database.execute(sql, Some(|_| -> bool { true })));

    let sql = r#"INSERT INTO `users` (id, name, age) VALUES (1, "Alice", 20.99);"#;
    ok!(database.execute(sql, Some(|_| -> bool { true })));

    let sql = r#"SELECT * FROM `users`;"#;
    ok!(database.execute(sql, Some(|_| -> bool {
        true
    })));
}

fn setup() -> (PathBuf, Directory) {
    let directory = ok!(Directory::new("sqlite"));
    (directory.path().join("database.sqlite3"), directory)
}
