extern crate sqlite;
extern crate temporary;

macro_rules! ok(
    ($result:expr) => ($result.unwrap());
);

#[test]
fn workflow() {
    use sqlite::State;

    macro_rules! pair(
        ($one:expr, $two:expr) => (($one, Some($two)));
    );

    let connection = ok!(sqlite::open(":memory:"));

    let sql = "CREATE TABLE `users` (id INTEGER, name VARCHAR(255), age REAL)";
    ok!(connection.execute(sql));

    {
        let sql = "INSERT INTO `users` (id, name, age) VALUES (?, ?, ?)";
        let mut statement = ok!(connection.prepare(sql));
        ok!(statement.bind(1, 1i64));
        ok!(statement.bind(2, "Alice"));
        ok!(statement.bind(3, 20.99));
        assert_eq!(ok!(statement.step()), State::Done);
    }

    {
        let mut done = false;
        let sql = "SELECT * FROM `users`";
        ok!(connection.process(sql, |pairs| {
            assert_eq!(pairs.len(), 3);
            assert_eq!(pairs[0], pair!("id", "1"));
            assert_eq!(pairs[1], pair!("name", "Alice"));
            assert_eq!(pairs[2], pair!("age", "20.99"));
            done = true;
            true
        }));
        assert!(done);
    }

    {
        let sql = "SELECT * FROM `users`";
        let mut statement = ok!(connection.prepare(sql));
        assert_eq!(ok!(statement.step()), State::Row);
        assert_eq!(ok!(statement.read::<i64>(0)), 1);
        assert_eq!(ok!(statement.read::<String>(1)), String::from("Alice"));
        assert_eq!(ok!(statement.read::<f64>(2)), 20.99);
        assert_eq!(ok!(statement.step()), State::Done);
    }
}

#[test]
fn stress() {
    use sqlite::State;
    use std::path::PathBuf;
    use std::thread;
    use temporary::Directory;

    let directory = ok!(Directory::new("sqlite"));
    let path = directory.path().join("database.sqlite3");

    let connection = ok!(sqlite::open(&path));
    let sql = "CREATE TABLE `users` (id INTEGER, name VARCHAR(255), age REAL)";
    ok!(connection.execute(sql));

    let guards = (0..100).map(|_| {
        let path = PathBuf::from(&path);
        thread::spawn(move || {
            let mut connection = ok!(sqlite::open(&path));
            ok!(connection.set_busy_handler(|_| true));
            let sql = "INSERT INTO `users` (id, name, age) VALUES (?, ?, ?)";
            let mut statement = ok!(connection.prepare(sql));
            ok!(statement.bind(1, 1i64));
            ok!(statement.bind(2, "Alice"));
            ok!(statement.bind(3, 20.99));
            assert_eq!(ok!(statement.step()), State::Done);
            true
        })
    }).collect::<Vec<_>>();

    for guard in guards {
        assert!(guard.join().unwrap());
    }
}
