extern crate sqlite;
extern crate temporary;

use std::path::PathBuf;
use temporary::Directory;

macro_rules! ok(
    ($result:expr) => ($result.unwrap());
);

#[test]
fn workflow() {
    use sqlite::Binding::*;
    use sqlite::ResultCode;

    macro_rules! pair(
        ($one:expr, $two:expr) => ((String::from($one), String::from($two)));
    );

    let (path, _directory) = setup();
    let mut database = ok!(sqlite::open(&path));

    let sql = r#"CREATE TABLE `users` (id INTEGER, name VARCHAR(255), age REAL);"#;
    ok!(database.execute(sql, None));

    {
        let sql = r#"INSERT INTO `users` (id, name, age) VALUES (?, ?, ?);"#;
        let mut statement = ok!(database.statement(sql));
        ok!(statement.bind(&[Integer(1, 1), Text(2, "Alice"), Float(3, 20.99)]));
        assert!(statement.step() == ResultCode::Done);
    }

    {
        let mut done = false;
        let sql = r#"SELECT * FROM `users`;"#;
        ok!(database.execute(sql, Some(&mut |pairs: Vec<(String, String)>| -> bool {
            assert!(pairs.len() == 3);
            assert!(pairs[0] == pair!("id", "1"));
            assert!(pairs[1] == pair!("name", "Alice"));
            assert!(pairs[2] == pair!("age", "20.99"));
            done = true;
            true
        })));
        assert!(done);
    }

    {
        let sql = r#"SELECT * FROM `users`;"#;
        let mut statement = ok!(database.statement(sql));
        assert!(statement.step() == ResultCode::Row);
        assert!(ok!(statement.column::<i64>(0)) == 1);
        assert!(ok!(statement.column::<String>(1)) == String::from("Alice"));
        assert!(ok!(statement.column::<f64>(2)) == 20.99);
        assert!(statement.step() == ResultCode::Done);
    }
}

fn setup() -> (PathBuf, Directory) {
    let directory = ok!(Directory::new("sqlite"));
    (directory.path().join("database.sqlite3"), directory)
}
