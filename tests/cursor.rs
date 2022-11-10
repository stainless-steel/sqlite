extern crate sqlite;

use sqlite::Value;
use std::collections::HashMap;

mod common;

use common::{setup_english, setup_users};

macro_rules! ok(($result:expr) => ($result.unwrap()));

#[test]
fn bind_iter() {
    let connection = ok!(sqlite::open(":memory:"));
    ok!(connection.execute("CREATE TABLE users (id INTEGER, name STRING)"));
    let statement = ok!(connection.prepare("INSERT INTO users VALUES (:id, :name)"));

    let mut map = HashMap::<_, Value>::new();
    map.insert(":name", "Bob".to_string().into());
    map.insert(":id", 42.into());

    let mut cursor = ok!(statement.into_cursor().bind_iter(map));
    assert!(cursor.next().is_none());
}

#[test]
fn count() {
    let connection = setup_english(":memory:");
    let query = "SELECT value FROM english WHERE value LIKE '%type'";
    let statement = ok!(connection.prepare(query));

    assert_eq!(statement.into_cursor().filter(|row| row.is_ok()).count(), 6);
}

#[test]
fn deref() {
    let connection = setup_english(":memory:");
    let query = "SELECT value FROM english WHERE value LIKE '%type'";
    let statement = ok!(connection.prepare(query));

    assert_eq!(statement.into_cursor().column_count(), 1);
}

#[test]
fn iter() {
    let connection = setup_users(":memory:");
    ok!(connection.execute("INSERT INTO users VALUES (2, 'Bob', NULL, NULL, NULL)"));
    let query = "SELECT id, age FROM users ORDER BY 1 DESC";
    let statement = ok!(connection.prepare(query));

    let mut count = 0;
    for row in statement.into_cursor().map(|row| ok!(row)) {
        let id = row.get::<i64, _>("id");
        if id == 1 {
            assert_eq!(row.get::<f64, _>("age"), 42.69);
        } else if id == 2 {
            assert_eq!(row.get::<Option<f64>, _>("age"), None);
        } else {
            assert!(false);
        }
        count += 1;
    }
    assert_eq!(count, 2);
}

#[test]
fn next_deref() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let statement = ok!(connection.prepare(query));

    let row = ok!(ok!(statement.into_cursor().next()));
    assert_eq!(row[0], Value::Integer(1));
    assert_eq!(row[2], Value::Float(42.69));
}

#[test]
fn next_get_with_name() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let statement = ok!(connection.prepare(query));

    let row = ok!(ok!(statement.into_cursor().next()));
    assert_eq!(row.get::<i64, _>("id"), 1);
    assert_eq!(row.get::<&str, _>("name"), "Alice");
    assert_eq!(row.get::<f64, _>("age"), 42.69);
    assert_eq!(row.get::<&[u8], _>("photo"), &[0x42u8, 0x69u8][..]);
}

#[test]
fn next_get_with_name_and_option() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let statement = ok!(connection.prepare(query));

    let row = ok!(ok!(statement.into_cursor().next()));
    assert!(row.get::<Option<i64>, _>("id").is_some());
    assert!(row.get::<Option<&str>, _>("name").is_some());
    assert!(row.get::<Option<f64>, _>("age").is_some());
    assert!(row.get::<Option<&[u8]>, _>("photo").is_some());
    assert!(row.get::<Option<&str>, _>("email").is_none());
}

#[test]
fn next_try_get_with_index() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let statement = ok!(connection.prepare(query));

    let row = ok!(ok!(statement.into_cursor().next()));
    assert!(row.try_get::<f64, _>(0).is_err());
    assert!(row.try_get::<i64, _>(0).is_ok());
    assert!(row.try_get::<&str, _>(1).is_ok());
    assert!(row.try_get::<f64, _>(2).is_ok());
    assert!(row.try_get::<&[u8], _>(3).is_ok());
    assert!(row.try_get::<&str, _>(4).is_err());
}

#[test]
fn next_try_get_with_index_and_option() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let statement = ok!(connection.prepare(query));

    let row = ok!(ok!(statement.into_cursor().next()));
    assert!(row.try_get::<Option<f64>, _>(0).is_err());
    assert!(ok!(row.try_get::<Option<i64>, _>(0)).is_some());
    assert!(ok!(row.try_get::<Option<&str>, _>(1)).is_some());
    assert!(ok!(row.try_get::<Option<f64>, _>(2)).is_some());
    assert!(ok!(row.try_get::<Option<&[u8]>, _>(3)).is_some());
    assert!(ok!(row.try_get::<Option<&str>, _>(4)).is_none());
}

#[test]
fn next_try_get_with_name() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let statement = ok!(connection.prepare(query));

    let row = ok!(ok!(statement.into_cursor().next()));
    assert!(row.try_get::<f64, _>("id").is_err());
    assert!(row.try_get::<i64, _>("id").is_ok());
    assert!(row.try_get::<&str, _>("name").is_ok());
    assert!(row.try_get::<f64, _>("age").is_ok());
    assert!(row.try_get::<&[u8], _>("photo").is_ok());
    assert!(row.try_get::<&str, _>("email").is_err());
}

#[test]
fn next_try_get_with_name_and_option() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let statement = ok!(connection.prepare(query));

    let row = ok!(ok!(statement.into_cursor().next()));
    assert!(row.try_get::<Option<f64>, _>("id").is_err());
    assert!(ok!(row.try_get::<Option<i64>, _>("id")).is_some());
    assert!(ok!(row.try_get::<Option<&str>, _>("name")).is_some());
    assert!(ok!(row.try_get::<Option<f64>, _>("age")).is_some());
    assert!(ok!(row.try_get::<Option<&[u8]>, _>("photo")).is_some());
    assert!(ok!(row.try_get::<Option<&str>, _>("email")).is_none());
}

#[test]
fn try_next_try_into() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let statement = ok!(connection.prepare(query));

    let mut cursor = statement.into_cursor();
    let row = ok!(ok!(cursor.try_next()));
    assert!(row[0].try_into::<f64>().is_err());
    assert!(row[0].try_into::<i64>().is_ok());
    assert!(row[1].try_into::<&str>().is_ok());
    assert!(row[2].try_into::<f64>().is_ok());
    assert!(row[3].try_into::<&[u8]>().is_ok());
    assert!(row[4].try_into::<&str>().is_err());
}

#[test]
fn workflow() {
    let connection = setup_users(":memory:");

    let select = "SELECT id, name FROM users WHERE id = ?";
    let mut select = ok!(connection.prepare(select)).into_cursor();

    let insert = "INSERT INTO users (id, name) VALUES (?, ?)";
    let insert = ok!(connection.prepare(insert)).into_cursor();

    for _ in 0..10 {
        select = ok!(select.bind((1, 1)));
        let row = ok!(ok!(select.next()));
        assert_eq!(row.get::<i64, _>("id"), 1);
        assert_eq!(row.get::<&str, _>("name"), "Alice");
        assert!(select.next().is_none());
    }

    let mut select = ok!(select.bind((1, 42)));
    assert!(select.next().is_none());

    let mut insert = ok!(insert.bind::<&[Value]>(&[42.into(), String::from("Bob").into()][..]));
    assert!(insert.next().is_none());

    let mut select = ok!(select.bind((1, 42)));
    let row = ok!(ok!(select.next()));
    assert_eq!(row.get::<i64, _>("id"), 42);
    assert_eq!(row.get::<&str, _>("name"), "Bob");
    assert!(select.next().is_none());
}
