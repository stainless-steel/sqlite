extern crate sqlite;
extern crate temporary;

use sqlite::{Connection, State, Type};
use std::path::Path;

macro_rules! ok(
    ($result:expr) => ($result.unwrap());
);

#[test]
fn connection_error() {
    let connection = setup(":memory:");
    match connection.execute(":)") {
        Err(error) => assert_eq!(error.message, Some(String::from(r#"unrecognized token: ":""#))),
        _ => unreachable!(),
    }
}

#[test]
fn connection_process() {
    macro_rules! pair(
        ($one:expr, $two:expr) => (($one, Some($two)));
    );

    let connection = setup(":memory:");

    let mut done = false;
    let query = "SELECT * FROM users";
    ok!(connection.process(query, |pairs| {
        assert_eq!(pairs.len(), 4);
        assert_eq!(pairs[0], pair!("id", "1"));
        assert_eq!(pairs[1], pair!("name", "Alice"));
        assert_eq!(pairs[2], pair!("age", "42.69"));
        assert_eq!(pairs[3], pair!("photo", "\x42\x69"));
        done = true;
        true
    }));
    assert!(done);
}

#[test]
fn connection_set_busy_handler() {
    use std::thread;
    use temporary::Directory;

    let directory = ok!(Directory::new("sqlite"));
    let path = directory.path().join("database.sqlite3");
    setup(&path);

    let guards = (0..100).map(|_| {
        let path = path.to_path_buf();
        thread::spawn(move || {
            let mut connection = ok!(sqlite::open(&path));
            ok!(connection.set_busy_handler(|_| true));
            let query = "INSERT INTO `users` (id, name, age, photo) VALUES (?, ?, ?, ?)";
            let mut statement = ok!(connection.prepare(query));
            ok!(statement.bind(1, 2i64));
            ok!(statement.bind(2, "Bob"));
            ok!(statement.bind(3, 69.42));
            ok!(statement.bind(4, &[0x69u8, 0x42u8][..]));
            assert_eq!(ok!(statement.step()), State::Done);
            true
        })
    }).collect::<Vec<_>>();

    for guard in guards {
        assert!(guard.join().unwrap());
    }
}

#[test]
fn statement_columns() {
    let connection = setup(":memory:");
    let query = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(query));

    assert_eq!(statement.columns(), 4);

    assert_eq!(ok!(statement.step()), State::Row);

    assert_eq!(statement.columns(), 4);
}

#[test]
fn statement_kind() {
    let connection = setup(":memory:");
    let query = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(query));

    assert_eq!(statement.kind(0), Type::Null);
    assert_eq!(statement.kind(1), Type::Null);
    assert_eq!(statement.kind(2), Type::Null);
    assert_eq!(statement.kind(3), Type::Null);

    assert_eq!(ok!(statement.step()), State::Row);

    assert_eq!(statement.kind(0), Type::Integer);
    assert_eq!(statement.kind(1), Type::String);
    assert_eq!(statement.kind(2), Type::Float);
    assert_eq!(statement.kind(3), Type::Blob);
}

#[test]
fn statement_bind() {
    let connection = setup(":memory:");
    let query = "INSERT INTO users (id, name, age, photo) VALUES (?, ?, ?, ?)";
    let mut statement = ok!(connection.prepare(query));

    ok!(statement.bind(1, 2i64));
    ok!(statement.bind(2, "Bob"));
    ok!(statement.bind(3, 69.42));
    ok!(statement.bind(4, &[0x69u8, 0x42u8][..]));
    assert_eq!(ok!(statement.step()), State::Done);
}

#[test]
fn statement_read() {
    let connection = setup(":memory:");
    let query = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(query));

    assert_eq!(ok!(statement.step()), State::Row);
    assert_eq!(ok!(statement.read::<i64>(0)), 1);
    assert_eq!(ok!(statement.read::<String>(1)), String::from("Alice"));
    assert_eq!(ok!(statement.read::<f64>(2)), 42.69);
    assert_eq!(ok!(statement.read::<Vec<u8>>(3)), vec![0x42, 0x69]);
    assert_eq!(ok!(statement.step()), State::Done);
}

fn setup<T: AsRef<Path>>(path: T) -> Connection {
    let connection = ok!(sqlite::open(path));
    ok!(connection.execute("
        CREATE TABLE users (id INTEGER, name VARCHAR(255), age REAL, photo BLOB);
        INSERT INTO users (id, name, age, photo) VALUES (1, 'Alice', 42.69, X'4269');
    "));
    connection
}
