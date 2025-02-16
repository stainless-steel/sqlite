use sqlite::Value;

mod common;

use common::setup_users;

macro_rules! ok(($result:expr) => ($result.unwrap()));

#[test]
fn index() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(query));

    let row = ok!(ok!(statement.iter().next()));

    assert_eq!(row[0], Value::Integer(1));
    assert_eq!(row[2], Value::Float(42.69));

    assert_eq!(row["id"], Value::Integer(1));
    assert_eq!(row["age"], Value::Float(42.69));
}

#[test]
fn iter_count() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(query));
    let mut cursor = statement.iter();
    let row = ok!(ok!(cursor.next()));
    let row = row.iter();
    assert_eq!(5, row.count());
}

#[test]
fn iter_order() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(query));
    let mut cursor = statement.iter();
    let row = ok!(ok!(cursor.next()));
    let mut row = row.iter();
    assert_eq!(Some(("id", &Value::Integer(1))), row.next());
    assert_eq!(
        Some(("name", &Value::String("Alice".to_owned()))),
        row.next(),
    );
    assert_eq!(Some(("age", &Value::Float(42.69))), row.next());
    assert_eq!(Some(("photo", &Value::Binary(vec![66, 105]))), row.next());
    assert_eq!(Some(("email", &Value::Null)), row.next());
    assert_eq!(None, row.next());
}

#[test]
fn read_with_name() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(query));

    let row = ok!(ok!(statement.iter().next()));
    assert_eq!(row.read::<i64, _>("id"), 1);
    assert_eq!(row.read::<&str, _>("name"), "Alice");
    assert_eq!(row.read::<f64, _>("age"), 42.69);
    assert_eq!(row.read::<&[u8], _>("photo"), &[0x42u8, 0x69u8][..]);
}

#[test]
fn read_with_name_and_option() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(query));

    let row = ok!(ok!(statement.iter().next()));
    assert!(row.read::<Option<i64>, _>("id").is_some());
    assert!(row.read::<Option<&str>, _>("name").is_some());
    assert!(row.read::<Option<f64>, _>("age").is_some());
    assert!(row.read::<Option<&[u8]>, _>("photo").is_some());
    assert!(row.read::<Option<&str>, _>("email").is_none());
}

#[test]
fn take() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(query));

    let mut row = ok!(ok!(statement.iter().next()));
    assert_eq!(row.take("name"), Value::String("Alice".into()));
    assert_eq!(row.take("name"), Value::Null);
}

#[test]
fn try_read_with_index() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(query));

    let row = ok!(ok!(statement.iter().next()));
    assert!(row.try_read::<f64, _>(0).is_err());
    assert!(row.try_read::<i64, _>(0).is_ok());
    assert!(row.try_read::<&str, _>(1).is_ok());
    assert!(row.try_read::<f64, _>(2).is_ok());
    assert!(row.try_read::<&[u8], _>(3).is_ok());
    assert!(row.try_read::<&str, _>(4).is_err());
}

#[test]
fn try_read_with_index_out_of_range() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(query));

    let row = ok!(ok!(statement.iter().next()));
    assert!(row.try_read::<&str, _>(5).is_err());
}

#[test]
fn try_read_with_index_and_option() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(query));

    let row = ok!(ok!(statement.iter().next()));
    assert!(row.try_read::<Option<f64>, _>(0).is_err());
    assert!(ok!(row.try_read::<Option<i64>, _>(0)).is_some());
    assert!(ok!(row.try_read::<Option<&str>, _>(1)).is_some());
    assert!(ok!(row.try_read::<Option<f64>, _>(2)).is_some());
    assert!(ok!(row.try_read::<Option<&[u8]>, _>(3)).is_some());
    assert!(ok!(row.try_read::<Option<&str>, _>(4)).is_none());
}

#[test]
fn try_read_with_name() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(query));

    let row = ok!(ok!(statement.iter().next()));
    assert!(row.try_read::<f64, _>("id").is_err());
    assert!(row.try_read::<i64, _>("id").is_ok());
    assert!(row.try_read::<&str, _>("name").is_ok());
    assert!(row.try_read::<f64, _>("age").is_ok());
    assert!(row.try_read::<&[u8], _>("photo").is_ok());
    assert!(row.try_read::<&str, _>("email").is_err());
}

#[test]
fn try_read_with_name_and_option() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(query));

    let row = ok!(ok!(statement.iter().next()));
    assert!(row.try_read::<Option<f64>, _>("id").is_err());
    assert!(ok!(row.try_read::<Option<i64>, _>("id")).is_some());
    assert!(ok!(row.try_read::<Option<&str>, _>("name")).is_some());
    assert!(ok!(row.try_read::<Option<f64>, _>("age")).is_some());
    assert!(ok!(row.try_read::<Option<&[u8]>, _>("photo")).is_some());
    assert!(ok!(row.try_read::<Option<&str>, _>("email")).is_none());
}

#[test]
fn try_into() {
    let connection = setup_users(":memory:");
    let query = "SELECT * FROM users";
    let mut statement = ok!(connection.prepare(query));

    let mut cursor = statement.iter();
    let row = ok!(ok!(cursor.try_next()));
    assert!((&row[0]).try_into::<f64>().is_err());
    assert!((&row[0]).try_into::<i64>().is_ok());
    assert!((&row[1]).try_into::<&str>().is_ok());
    assert!((&row[2]).try_into::<f64>().is_ok());
    assert!((&row[3]).try_into::<&[u8]>().is_ok());
    assert!((&row[4]).try_into::<&str>().is_err());
}
