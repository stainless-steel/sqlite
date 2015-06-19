extern crate sqlite;
extern crate temporary;

macro_rules! ok(
    ($result:expr) => ($result.unwrap());
);

#[test]
fn workflow() {
    use sqlite::Binding::*;
    use sqlite::State;

    macro_rules! pair(
        ($one:expr, $two:expr) => (($one, Some($two)));
    );

    let database = ok!(sqlite::open(":memory:"));

    let sql = r#"CREATE TABLE `users` (id INTEGER, name VARCHAR(255), age REAL);"#;
    ok!(database.execute(sql));

    {
        let sql = r#"INSERT INTO `users` (id, name, age) VALUES (?, ?, ?);"#;
        let mut statement = ok!(database.prepare(sql));
        ok!(statement.bind(&[Integer(1, 1), Text(2, "Alice"), Float(3, 20.99)]));
        assert!(ok!(statement.step()) == State::Done);
    }

    {
        let mut done = false;
        let sql = r#"SELECT * FROM `users`;"#;
        ok!(database.process(sql, |pairs| {
            assert!(pairs.len() == 3);
            assert!(pairs[0] == pair!("id", "1"));
            assert!(pairs[1] == pair!("name", "Alice"));
            assert!(pairs[2] == pair!("age", "20.99"));
            done = true;
            true
        }));
        assert!(done);
    }

    {
        let sql = r#"SELECT * FROM `users`;"#;
        let mut statement = ok!(database.prepare(sql));
        assert!(ok!(statement.step()) == State::Row);
        assert!(ok!(statement.column::<i64>(0)) == 1);
        assert!(ok!(statement.column::<String>(1)) == String::from("Alice"));
        assert!(ok!(statement.column::<f64>(2)) == 20.99);
        assert!(ok!(statement.step()) == State::Done);
    }
}

#[test]
fn stress() {
    use sqlite::Binding::*;
    use sqlite::State;
    use std::path::PathBuf;
    use std::thread;
    use temporary::Directory;

    let directory = ok!(Directory::new("sqlite"));
    let path = directory.path().join("database.sqlite3");

    let database = ok!(sqlite::open(&path));
    let sql = r#"CREATE TABLE `users` (id INTEGER, name VARCHAR(255), age REAL);"#;
    ok!(database.execute(sql));

    let guards = (0..100).map(|_| {
        let path = PathBuf::from(&path);
        thread::spawn(move || {
            let mut database = ok!(sqlite::open(&path));
            ok!(database.set_busy_handler(|_| true));
            let sql = r#"INSERT INTO `users` (id, name, age) VALUES (?, ?, ?);"#;
            let mut statement = ok!(database.prepare(sql));
            ok!(statement.bind(&[Integer(1, 1), Text(2, "Alice"), Float(3, 20.99)]));
            assert!(ok!(statement.step()) == State::Done);
            true
        })
    }).collect::<Vec<_>>();

    for guard in guards {
        assert!(guard.join().unwrap());
    }
}
