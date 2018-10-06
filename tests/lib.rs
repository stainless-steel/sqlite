extern crate sqlite;
extern crate temporary;

use sqlite::{Connection, State, Type, Value};
use std::path::Path;

macro_rules! ok(($result:expr) => ($result.unwrap()));

#[test]
fn connection_error() {
    let connection = setup_users(":memory:");
    match connection.execute(":)") {
        Err(error) => assert_eq!(
            error.message,
            Some(String::from(r#"unrecognized token: ":""#))
        ),
        _ => unreachable!(),
    }
}

#[test]
fn connection_iterate() {
    macro_rules! pair(
        ($one:expr, $two:expr) => (($one, Some($two)));
    );

    let connection = setup_users(":memory:");

    let mut done = false;
    let statement = "SELECT * FROM users";
    ok!(connection.iterate(statement, |pairs| {
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
    setup_users(&path);

    let guards = (0..100)
        .map(|_| {
            let path = path.to_path_buf();
            thread::spawn(move || {
                let mut connection = ok!(sqlite::open(&path));
                ok!(connection.set_busy_handler(|_| true));
                let statement = "INSERT INTO users (id, name, age, photo) VALUES (?, ?, ?, ?)";
                let mut statement = ok!(connection.prepare(statement));
                ok!(statement.bind(1, 2i64));
                ok!(statement.bind(2, "Bob"));
                ok!(statement.bind(3, 69.42));
                ok!(statement.bind(4, &[0x69u8, 0x42u8][..]));
                assert_eq!(ok!(statement.next()), State::Done);
                true
            })
        })
        .collect::<Vec<_>>();

    for guard in guards {
        assert!(ok!(guard.join()));
    }
}

#[test]
fn cursor_wildcard_with_binding() {
    let connection = setup_english(":memory:");
    let statement = "SELECT value FROM english WHERE value LIKE ?";
    let mut statement = ok!(connection.prepare(statement));
    ok!(statement.bind(1, "%type"));

    let mut count = 0;
    let mut cursor = statement.cursor();
    while let Some(_) = ok!(cursor.next()) {
        count += 1;
    }
    assert_eq!(count, 6);
}

#[test]
fn cursor_wildcard_without_binding() {
    let connection = setup_english(":memory:");
    let statement = "SELECT value FROM english WHERE value LIKE '%type'";
    let statement = ok!(connection.prepare(statement));

    let mut count = 0;
    let mut cursor = statement.cursor();
    while let Some(_) = ok!(cursor.next()) {
        count += 1;
    }
    assert_eq!(count, 6);
}

#[test]
fn cursor_workflow() {
    let connection = setup_users(":memory:");

    let select = "SELECT id, name FROM users WHERE id = ?";
    let mut select = ok!(connection.prepare(select)).cursor();

    let insert = "INSERT INTO users (id, name) VALUES (?, ?)";
    let mut insert = ok!(connection.prepare(insert)).cursor();

    for _ in 0..10 {
        ok!(select.bind(&[Value::Integer(1)]));
        assert_eq!(
            ok!(ok!(select.next())),
            &[Value::Integer(1), Value::String("Alice".to_string())]
        );
        assert_eq!(ok!(select.next()), None);
    }

    ok!(select.bind(&[Value::Integer(42)]));
    assert_eq!(ok!(select.next()), None);

    ok!(insert.bind(&[Value::Integer(42), Value::String("Bob".to_string())]));
    assert_eq!(ok!(insert.next()), None);

    ok!(select.bind(&[Value::Integer(42)]));
    assert_eq!(
        ok!(ok!(select.next())),
        &[Value::Integer(42), Value::String("Bob".to_string())]
    );
    assert_eq!(ok!(select.next()), None);
}

#[test]
fn statement_bind() {
    let connection = setup_users(":memory:");
    let statement = "INSERT INTO users (id, name, age, photo) VALUES (?, ?, ?, ?)";
    let mut statement = ok!(connection.prepare(statement));

    ok!(statement.bind(1, 2i64));
    ok!(statement.bind(2, "Bob"));
    ok!(statement.bind(3, 69.42));
    ok!(statement.bind(4, &[0x69u8, 0x42u8][..]));
    assert_eq!(ok!(statement.next()), State::Done);
}

#[test]
fn statement_count() {
    let connection = setup_users(":memory:");
    let statement = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(statement));

    assert_eq!(ok!(statement.next()), State::Row);

    assert_eq!(statement.count(), 4);
}

#[test]
fn statement_name() {
    let connection = setup_users(":memory:");
    let statement = "SELECT id, name, age, photo AS user_photo FROM users";
    let statement = ok!(connection.prepare(statement));

    let names = statement.names();
    assert_eq!(names, vec!["id", "name", "age", "user_photo"]);
    assert_eq!("user_photo", statement.name(3));
}

#[test]
fn statement_kind() {
    let connection = setup_users(":memory:");
    let statement = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(statement));

    assert_eq!(statement.kind(0), Type::Null);
    assert_eq!(statement.kind(1), Type::Null);
    assert_eq!(statement.kind(2), Type::Null);
    assert_eq!(statement.kind(3), Type::Null);

    assert_eq!(ok!(statement.next()), State::Row);

    assert_eq!(statement.kind(0), Type::Integer);
    assert_eq!(statement.kind(1), Type::String);
    assert_eq!(statement.kind(2), Type::Float);
    assert_eq!(statement.kind(3), Type::Binary);
}

#[test]
fn statement_read() {
    let connection = setup_users(":memory:");
    let statement = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(statement));

    assert_eq!(ok!(statement.next()), State::Row);
    assert_eq!(ok!(statement.read::<i64>(0)), 1);
    assert_eq!(ok!(statement.read::<String>(1)), String::from("Alice"));
    assert_eq!(ok!(statement.read::<f64>(2)), 42.69);
    assert_eq!(ok!(statement.read::<Vec<u8>>(3)), vec![0x42, 0x69]);
    assert_eq!(ok!(statement.next()), State::Done);
}

#[test]
fn statement_wildcard_with_binding() {
    let connection = setup_english(":memory:");
    let statement = "SELECT value FROM english WHERE value LIKE ?";
    let mut statement = ok!(connection.prepare(statement));
    ok!(statement.bind(1, "%type"));

    let mut count = 0;
    while let State::Row = ok!(statement.next()) {
        count += 1;
    }
    assert_eq!(count, 6);
}

#[test]
fn statement_wildcard_without_binding() {
    let connection = setup_english(":memory:");
    let statement = "SELECT value FROM english WHERE value LIKE '%type'";
    let mut statement = ok!(connection.prepare(statement));

    let mut count = 0;
    while let State::Row = ok!(statement.next()) {
        count += 1;
    }
    assert_eq!(count, 6);
}

fn setup_english<T: AsRef<Path>>(path: T) -> Connection {
    let connection = ok!(sqlite::open(path));
    ok!(connection.execute(
        "
        CREATE TABLE english (value TEXT);
        INSERT INTO english (value) VALUES ('cerotype');
        INSERT INTO english (value) VALUES ('metatype');
        INSERT INTO english (value) VALUES ('ozotype');
        INSERT INTO english (value) VALUES ('phenotype');
        INSERT INTO english (value) VALUES ('plastotype');
        INSERT INTO english (value) VALUES ('undertype');
        INSERT INTO english (value) VALUES ('nonsence');
        ",
    ));
    connection
}

fn setup_users<T: AsRef<Path>>(path: T) -> Connection {
    let connection = ok!(sqlite::open(path));
    ok!(connection.execute(
        "
        CREATE TABLE users (id INTEGER, name TEXT, age REAL, photo BLOB);
        INSERT INTO users (id, name, age, photo) VALUES (1, 'Alice', 42.69, X'4269');
        ",
    ));
    connection
}
