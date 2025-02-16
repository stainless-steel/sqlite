use sqlite::{Connection, State, Statement, Type, Value};

mod common;

use common::{setup_english, setup_users};

macro_rules! ok(($result:expr) => ($result.unwrap()));

#[test]
fn bind_with_index() {
    let connection = setup_users(":memory:");
    let query = "INSERT INTO users VALUES (?, ?, ?, ?, ?)";
    let mut statement = ok!(connection.prepare(query));

    ok!(statement.reset());
    ok!(statement.bind(&[(1, 2i64)][..]));
    ok!(statement.bind((2, "Bob")));
    ok!(statement.bind((3, 69.42)));
    ok!(statement.bind((4, &[0x69u8, 0x42u8][..])));
    ok!(statement.bind((5, ())));
    assert_eq!(ok!(statement.next()), State::Done);

    ok!(statement.reset());
    ok!(statement.bind((1, Some(2i64))));
    ok!(statement.bind((2, Some("Bob"))));
    ok!(statement.bind((3, Some(69.42))));
    ok!(statement.bind((4, Some(&[0x69u8, 0x42u8][..]))));
    ok!(statement.bind((5, None::<&str>)));
    assert_eq!(ok!(statement.next()), State::Done);

    ok!(statement.reset());
    ok!(statement.bind(
        &[
            Value::Integer(2),
            Value::String("Bob".into()),
            Value::Float(69.42),
            Value::Binary([0x69u8, 0x42u8].to_vec()),
            Value::Null,
        ][..]
    ));
    assert_eq!(ok!(statement.next()), State::Done);

    ok!(statement.reset());
    ok!(statement.bind(
        &[
            Some(Value::Integer(2)),
            Some(Value::String("Bob".into())),
            Some(Value::Float(69.42)),
            Some(Value::Binary([0x69u8, 0x42u8].to_vec())),
            Some(Value::Null),
        ][..]
    ));
    assert_eq!(ok!(statement.next()), State::Done);

    ok!(statement.reset());
    ok!(statement.bind(
        &[
            (1, Value::Integer(2)),
            (2, Value::String("Bob".into())),
            (3, Value::Float(69.42)),
            (4, Value::Binary([0x69u8, 0x42u8].to_vec())),
            (5, Value::Null),
        ][..]
    ));
    assert_eq!(ok!(statement.next()), State::Done);
}

#[test]
fn bind_with_name() {
    let connection = setup_users(":memory:");
    let query = "INSERT INTO users VALUES (:id, :name, :age, :photo, :email)";
    let mut statement = ok!(connection.prepare(query));

    ok!(statement.reset());
    ok!(statement.bind(&[(":id", 2i64)][..]));
    ok!(statement.bind((":name", "Bob")));
    ok!(statement.bind((":age", 69.42)));
    ok!(statement.bind((":photo", &[0x69u8, 0x42u8][..])));
    ok!(statement.bind((":email", ())));
    assert_eq!(ok!(statement.next()), State::Done);

    ok!(statement.reset());
    assert!(statement.bind((":missing", 404)).is_err());

    ok!(statement.reset());
    ok!(statement.bind(
        &[
            (":id", Value::Integer(2)),
            (":name", Value::String("Bob".into())),
            (":age", Value::Float(69.42)),
            (":photo", Value::Binary([0x69u8, 0x42u8].to_vec())),
            (":email", Value::Null),
        ][..]
    ));
    assert_eq!(ok!(statement.next()), State::Done);
}

#[test]
fn count() {
    let connection = setup_english(":memory:");

    let query = "SELECT value FROM english WHERE value LIKE ?";
    let mut statement = ok!(connection.prepare(query));
    ok!(statement.bind((1, "%type")));
    let mut count = 0;
    while let State::Row = ok!(statement.next()) {
        count += 1;
    }
    assert_eq!(count, 6);

    let query = "SELECT value FROM english WHERE value LIKE '%type'";
    let mut statement = ok!(connection.prepare(query));
    let mut count = 0;
    while let State::Row = ok!(statement.next()) {
        count += 1;
    }
    assert_eq!(count, 6);
}

#[test]
fn read_with_index() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(query));

    assert_eq!(ok!(statement.next()), State::Row);
    assert_eq!(ok!(statement.read::<i64, _>(0)), 1);
    assert_eq!(ok!(statement.read::<String, _>(1)), String::from("Alice"));
    assert_eq!(ok!(statement.read::<f64, _>(2)), 42.69);
    assert_eq!(ok!(statement.read::<Vec<u8>, _>(3)), vec![0x42, 0x69]);
    assert_eq!(ok!(statement.read::<Value, _>(4)), Value::Null);
    assert_eq!(ok!(statement.next()), State::Done);
}

#[test]
fn read_with_index_and_option() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(query));

    assert_eq!(ok!(statement.next()), State::Row);
    assert_eq!(ok!(statement.read::<Option<i64>, _>(0)), Some(1));
    assert_eq!(
        ok!(statement.read::<Option<String>, _>(1)),
        Some(String::from("Alice"))
    );
    assert_eq!(ok!(statement.read::<Option<f64>, _>(2)), Some(42.69));
    assert_eq!(
        ok!(statement.read::<Option<Vec<u8>>, _>(3)),
        Some(vec![0x42, 0x69])
    );
    assert_eq!(ok!(statement.read::<Option<String>, _>(4)), None);
    assert_eq!(ok!(statement.next()), State::Done);
}

#[test]
fn read_with_name_and_option() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(query));

    assert_eq!(ok!(statement.next()), State::Row);
    assert_eq!(ok!(statement.read::<Option<i64>, _>("id")), Some(1));
    assert_eq!(
        ok!(statement.read::<Option<String>, _>("name")),
        Some(String::from("Alice"))
    );
    assert_eq!(ok!(statement.read::<Option<f64>, _>("age")), Some(42.69));
    assert_eq!(
        ok!(statement.read::<Option<Vec<u8>>, _>("photo")),
        Some(vec![0x42, 0x69])
    );
    assert_eq!(ok!(statement.read::<Option<String>, _>("email")), None);
    assert_eq!(ok!(statement.next()), State::Done);
}

#[test]
fn read_with_name() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(query));

    assert_eq!(ok!(statement.next()), State::Row);
    assert_eq!(ok!(statement.read::<i64, _>("id")), 1);
    assert_eq!(
        ok!(statement.read::<String, _>("name")),
        String::from("Alice")
    );
    assert_eq!(ok!(statement.read::<f64, _>("age")), 42.69);
    assert_eq!(ok!(statement.read::<Vec<u8>, _>("photo")), vec![0x42, 0x69]);
    assert_eq!(ok!(statement.read::<Value, _>("email")), Value::Null);
    assert_eq!(ok!(statement.next()), State::Done);
}

#[test]
fn column_count() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(query));

    assert_eq!(ok!(statement.next()), State::Row);
    assert_eq!(statement.column_count(), 5);
}

#[test]
fn column_name() {
    let connection = setup_users(":memory:");
    let query = "SELECT id, name, age, photo AS user_photo FROM users";
    let statement = ok!(connection.prepare(query));

    let names = statement.column_names();
    assert_eq!(names, vec!["id", "name", "age", "user_photo"]);
    assert_eq!("user_photo", ok!(statement.column_name(3)));
}

#[test]
fn column_type() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(query));

    assert_eq!(ok!(statement.column_type(0)), Type::Null);
    assert_eq!(ok!(statement.column_type(1)), Type::Null);
    assert_eq!(ok!(statement.column_type(2)), Type::Null);
    assert_eq!(ok!(statement.column_type(3)), Type::Null);

    assert_eq!(ok!(statement.next()), State::Row);

    assert_eq!(ok!(statement.column_type(0)), Type::Integer);
    assert_eq!(ok!(statement.column_type(1)), Type::String);
    assert_eq!(ok!(statement.column_type(2)), Type::Float);
    assert_eq!(ok!(statement.column_type(3)), Type::Binary);
}

#[test]
fn parameter_index() {
    let connection = setup_users(":memory:");
    let query = "INSERT INTO users VALUES (:id, :name, :age, :photo, :email)";
    let mut statement = ok!(connection.prepare(query));
    ok!(statement.bind((":id", 2i64)));
    ok!(statement.bind((":name", "Bob")));
    ok!(statement.bind((":age", 69.42)));
    ok!(statement.bind((":photo", &[0x69u8, 0x42u8][..])));
    ok!(statement.bind((":email", ())));
    assert_eq!(ok!(statement.parameter_index(":missing")), None);
    assert_eq!(ok!(statement.next()), State::Done);
}

#[test]
fn workflow_1() {
    struct Database<'l> {
        #[allow(dead_code)]
        connection: &'l Connection,
        statement: Statement<'l>,
    }

    impl Database<'_> {
        fn run_once(&mut self) -> sqlite::Result<()> {
            self.statement.reset()?;
            self.statement.bind((":age", 40))?;
            assert_eq!(ok!(self.statement.next()), State::Row);
            Ok(())
        }
    }

    let connection = setup_users(":memory:");
    let query = "SELECT name FROM users WHERE age > :age";
    let statement = ok!(connection.prepare(query));

    let mut database = Database {
        connection: &connection,
        statement,
    };

    for _ in 0..5 {
        assert!(database.run_once().is_ok());
    }
}

#[test]
fn workflow_2() {
    let connection = ok!(Connection::open(":memory:"));
    ok!(connection.execute("CREATE TABLE users (name TEXT, age INTEGER, PRIMARY KEY (name))"));

    let mut statement = ok!(connection.prepare(
        "INSERT INTO users (name, age) VALUES ('jean', 49) ON CONFLICT DO UPDATE SET age = 49"
    ));
    ok!(statement.next());

    let mut statement = ok!(connection.prepare(
        "INSERT INTO users (name, age) VALUES ('jean', 50) ON CONFLICT DO UPDATE SET age = 50"
    ));
    ok!(statement.next());

    let mut statement = ok!(connection.prepare("SELECT * FROM users WHERE name = 'jean'"));
    ok!(statement.next());

    let age = ok!(statement.read::<i64, _>("age"));
    assert_eq!(age, 50);
}
