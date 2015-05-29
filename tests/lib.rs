extern crate sqlite;
extern crate temporary;

use std::path::PathBuf;
use temporary::Directory;

macro_rules! ok(
    ($result:expr) => ($result.unwrap());
);

#[test]
fn execute() {
    macro_rules! pair(
        ($one:expr, $two:expr) => ((String::from($one), String::from($two)));
    );

    let (path, _directory) = setup();
    let mut database = ok!(sqlite::open(&path));

    let sql = r#"CREATE TABLE `users` (id INTEGER, name VARCHAR(255), age REAL);"#;
    ok!(database.execute(sql, Some(|_| -> bool { true })));

    let sql = r#"INSERT INTO `users` (id, name, age) VALUES (1, "Alice", 20.99);"#;
    ok!(database.execute(sql, Some(|_| -> bool { true })));

    let mut done = false;
    let sql = r#"SELECT * FROM `users`;"#;
    ok!(database.execute(sql, Some(|pairs: Vec<(String, String)>| -> bool {
        assert!(pairs.len() == 3);
        assert!(pairs[0] == pair!("id", "1"));
        assert!(pairs[1] == pair!("name", "Alice"));
        assert!(pairs[2] == pair!("age", "20.99"));
        done = true;
        true
    })));
    assert!(done);
}

fn setup() -> (PathBuf, Directory) {
    let directory = ok!(Directory::new("sqlite"));
    (directory.path().join("database.sqlite3"), directory)
}
