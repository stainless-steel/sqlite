extern crate sqlite;

use sqlite::Value;
use std::collections::HashMap;

mod common;

use common::{setup_english, setup_users};

macro_rules! ok(($result:expr) => ($result.unwrap()));

#[test]
fn bind_by_name() {
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
fn read() {
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
            // assert_eq!(row.get::<Option<f64>, _>("age"), None);
        } else {
            assert!(false);
        }
        count += 1;
    }
    assert_eq!(count, 2);
}

#[test]
fn next_with_nullable() {
    let connection = setup_users(":memory:");
    let query = "SELECT id, name, email FROM users";
    let statement = ok!(connection.prepare(query));

    let row = ok!(ok!(statement.into_cursor().next()));
    assert_eq!(row.get::<i64, _>("id"), 1);
    // assert_eq!(row.get::<Value, _>("id"), 1.into());
    assert_eq!(row.get::<&str, _>("name"), "Alice");
    // assert_eq!(row.get::<Value, _>("name"), String::from("Alice").into());
    // assert_eq!(row.get::<Option<String>, _>("email"), None);
    // assert_eq!(row.get::<Value, _>("email"), Value::Null);
    // assert_eq!(ok!(row.try_get::<Option<String>, _>("email")), None);
}

#[test]
fn try_next_with_nullable() {
    let connection = setup_users(":memory:");
    let query = "SELECT id, name, email FROM users";
    let statement = ok!(connection.prepare(query));

    let mut cursor = statement.into_cursor();
    let row = ok!(ok!(cursor.try_next()));
    assert_eq!(ok!(row[0].try_into::<i64>()), 1);
    assert_eq!(ok!(row[1].try_into::<&str>()), "Alice");
    assert!(row[2].try_into::<&str>().is_err());
}

#[test]
fn wildcard() {
    let connection = setup_english(":memory:");
    let query = "SELECT value FROM english WHERE value LIKE '%type'";
    let statement = ok!(connection.prepare(query));

    assert_eq!(statement.into_cursor().filter(|row| row.is_ok()).count(), 6);
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
