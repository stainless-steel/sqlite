extern crate sqlite;
extern crate temporary;

use sqlite::{Connection, OpenFlags, State, Type, Value};
use std::collections::HashMap;
use std::path::Path;

macro_rules! ok(($result:expr) => ($result.unwrap()));

#[test]
fn connection_change_count() {
    let connection = setup_users(":memory:");
    assert_eq!(connection.change_count(), 1);
    assert_eq!(connection.total_change_count(), 1);

    ok!(connection.execute("INSERT INTO users VALUES (2, 'Bob', NULL, NULL, NULL)"));
    assert_eq!(connection.change_count(), 1);
    assert_eq!(connection.total_change_count(), 2);

    ok!(connection.execute("UPDATE users SET name = 'Bob' WHERE id = 1"));
    assert_eq!(connection.change_count(), 1);
    assert_eq!(connection.total_change_count(), 3);

    ok!(connection.execute("DELETE FROM users"));
    assert_eq!(connection.change_count(), 2);
    assert_eq!(connection.total_change_count(), 5);
}

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
        assert_eq!(pairs.len(), 5);
        assert_eq!(pairs[0], pair!("id", "1"));
        assert_eq!(pairs[1], pair!("name", "Alice"));
        assert_eq!(pairs[2], pair!("age", "42.69"));
        assert_eq!(pairs[3], pair!("photo", "\x42\x69"));
        assert_eq!(pairs[4], ("email", None));
        done = true;
        true
    }));
    assert!(done);
}

#[test]
fn connection_open_with_flags() {
    use temporary::Directory;

    let directory = ok!(Directory::new("sqlite"));
    let path = directory.path().join("database.sqlite3");
    setup_users(&path);

    let flags = OpenFlags::new().set_read_only();
    let connection = ok!(Connection::open_with_flags(path, flags));
    match connection.execute("INSERT INTO users VALUES (2, 'Bob', NULL, NULL)") {
        Err(_) => {}
        _ => unreachable!(),
    }
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
                let statement = "INSERT INTO users VALUES (?, ?, ?, ?, ?)";
                let mut statement = ok!(connection.prepare(statement));
                ok!(statement.bind(1, 2i64));
                ok!(statement.bind(2, "Bob"));
                ok!(statement.bind(3, 69.42));
                ok!(statement.bind(4, &[0x69u8, 0x42u8][..]));
                ok!(statement.bind(5, ()));
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
fn cursor_bind_by_name() {
    let connection = ok!(sqlite::open(":memory:"));
    ok!(connection.execute("CREATE TABLE users (id INTEGER, name STRING)"));
    let statement = ok!(connection.prepare("INSERT INTO users VALUES (:id, :name)"));

    let mut map = HashMap::new();
    map.insert(":name".to_string(), Value::String("Bob".to_string()));
    map.insert(":id".to_string(), Value::Integer(42));

    let mut cursor = statement.cursor();
    ok!(cursor.bind_by_name(map));
    assert_eq!(ok!(cursor.next()), None);
}

#[test]
fn cursor_read() {
    let connection = setup_users(":memory:");
    ok!(connection.execute("INSERT INTO users VALUES (2, 'Bob', NULL, NULL, NULL)"));
    let statement = "SELECT id, age FROM users ORDER BY 1 DESC";
    let statement = ok!(connection.prepare(statement));

    let mut count = 0;
    let mut cursor = statement.cursor();
    while let Some(row) = ok!(cursor.next()) {
        let id = row[0].as_integer().unwrap();
        if id == 1 {
            assert_eq!(row[1].as_float().unwrap(), 42.69);
        } else if id == 2 {
            assert_eq!(row[1].as_float().unwrap_or(69.42), 69.42);
        } else {
            assert!(false);
        }
        count += 1;
    }
    assert_eq!(count, 2);
}

#[test]
fn cursor_wildcard() {
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
    let statement = "INSERT INTO users VALUES (?, ?, ?, ?, ?)";
    let mut statement = ok!(connection.prepare(statement));

    ok!(statement.bind(1, 2i64));
    ok!(statement.bind(2, "Bob"));
    ok!(statement.bind(3, 69.42));
    ok!(statement.bind(4, &[0x69u8, 0x42u8][..]));
    ok!(statement.bind(5, ()));
    assert_eq!(ok!(statement.next()), State::Done);
}

#[test]
fn statement_bind_with_optional() {
    let connection = setup_users(":memory:");
    let statement = "INSERT INTO users VALUES (?, ?, ?, ?, ?)";
    let mut statement = ok!(connection.prepare(statement));

    ok!(statement.bind(1, None::<i64>));
    ok!(statement.bind(2, None::<&str>));
    ok!(statement.bind(3, None::<f64>));
    ok!(statement.bind(4, None::<&[u8]>));
    ok!(statement.bind(5, None::<&str>));
    assert_eq!(ok!(statement.next()), State::Done);

    let statement = "INSERT INTO users VALUES (?, ?, ?, ?, ?)";
    let mut statement = ok!(connection.prepare(statement));

    ok!(statement.bind(1, Some(2i64)));
    ok!(statement.bind(2, Some("Bob")));
    ok!(statement.bind(3, Some(69.42)));
    ok!(statement.bind(4, Some(&[0x69u8, 0x42u8][..])));
    ok!(statement.bind(5, None::<&str>));
    assert_eq!(ok!(statement.next()), State::Done);
}

#[test]
<<<<<<< HEAD
fn statement_bind_by_name() {
    let connection = setup_users(":memory:");
    let statement = "INSERT INTO users VALUES (:id, :name, :age, :photo, :email)";
    let mut statement = ok!(connection.prepare(statement));

    ok!(statement.bind_by_name(":id", 2i64));
    ok!(statement.bind_by_name(":name", "Bob"));
    ok!(statement.bind_by_name(":age", 69.42));
    ok!(statement.bind_by_name(":photo", &[0x69u8, 0x42u8][..]));
    ok!(statement.bind_by_name(":email", ()));
    assert!(statement.bind_by_name(":missing", 404).is_err());
    assert_eq!(ok!(statement.next()), State::Done);
}

#[test]
fn statement_bind_by_name_multiple() {
    let connection = setup_users(":memory:");
    let query = "SELECT name FROM users WHERE age > :age - 5 AND age < :age + 5";
    let mut statement = ok!(connection.prepare(query));
    let index = ok!(statement.parameter_index(":age")).unwrap();

    ok!(statement.bind(index, 40));
    let mut cursor = statement.cursor();
    let row = ok!(cursor.next()).unwrap();
    assert_eq!(row[0].as_string(), Some("Alice"));

    let mut statement = ok!(connection.prepare(query));
    ok!(statement.bind(index, 45));
    let mut cursor = statement.cursor();
    let row = ok!(cursor.next()).unwrap();
    assert_eq!(row[0].as_string(), Some("Alice"));
}

#[test]
fn statement_count() {
=======
fn statement_column_count() {
>>>>>>> e6da9fd (Rename count to column_count)
    let connection = setup_users(":memory:");
    let statement = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(statement));

    assert_eq!(ok!(statement.next()), State::Row);

    assert_eq!(statement.column_count(), 5);
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
fn statement_parameter_index() {
    let connection = setup_users(":memory:");
    let statement = "INSERT INTO users VALUES (:id, :name, :age, :photo, :email)";
    let mut statement = ok!(connection.prepare(statement));

    ok!(statement.bind(ok!(statement.parameter_index(":id")).unwrap().into(), 2i64));
    ok!(statement.bind(
        ok!(statement.parameter_index(":name")).unwrap().into(),
        "Bob",
    ));
    ok!(statement.bind(
        ok!(statement.parameter_index(":age")).unwrap().into(),
        69.42,
    ));
    ok!(statement.bind(
        ok!(statement.parameter_index(":photo")).unwrap().into(),
        &[0x69u8, 0x42u8][..],
    ));
    ok!(statement.bind(ok!(statement.parameter_index(":email")).unwrap().into(), ()));
    assert_eq!(ok!(statement.parameter_index(":missing")), None);
    assert_eq!(ok!(statement.next()), State::Done);
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
    assert_eq!(ok!(statement.read::<Value>(4)), Value::Null);
    assert_eq!(ok!(statement.next()), State::Done);
}

#[test]
fn statement_read_with_optional() {
    let connection = setup_users(":memory:");
    let statement = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(statement));

    assert_eq!(ok!(statement.next()), State::Row);
    assert_eq!(ok!(statement.read::<Option<i64>>(0)), Some(1));
    assert_eq!(
        ok!(statement.read::<Option<String>>(1)),
        Some(String::from("Alice"))
    );
    assert_eq!(ok!(statement.read::<Option<f64>>(2)), Some(42.69));
    assert_eq!(
        ok!(statement.read::<Option<Vec<u8>>>(3)),
        Some(vec![0x42, 0x69])
    );
    assert_eq!(ok!(statement.read::<Option<String>>(4)), None);
    assert_eq!(ok!(statement.next()), State::Done);
}

#[test]
fn statement_wildcard() {
    let connection = setup_english(":memory:");
    let statement = "SELECT value FROM english WHERE value LIKE '%type'";
    let mut statement = ok!(connection.prepare(statement));

    let mut count = 0;
    while let State::Row = ok!(statement.next()) {
        count += 1;
    }
    assert_eq!(count, 6);
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

fn setup_english<T: AsRef<Path>>(path: T) -> Connection {
    let connection = ok!(sqlite::open(path));
    ok!(connection.execute(
        "
        CREATE TABLE english (value TEXT);
        INSERT INTO english VALUES ('cerotype');
        INSERT INTO english VALUES ('metatype');
        INSERT INTO english VALUES ('ozotype');
        INSERT INTO english VALUES ('phenotype');
        INSERT INTO english VALUES ('plastotype');
        INSERT INTO english VALUES ('undertype');
        INSERT INTO english VALUES ('nonsence');
        ",
    ));
    connection
}

fn setup_users<T: AsRef<Path>>(path: T) -> Connection {
    let connection = ok!(sqlite::open(path));
    ok!(connection.execute(
        "
        CREATE TABLE users (id INTEGER, name TEXT, age REAL, photo BLOB, email TEXT);
        INSERT INTO users VALUES (1, 'Alice', 42.69, X'4269', NULL);
        ",
    ));
    connection
}
