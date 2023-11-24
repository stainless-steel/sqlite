#![allow(dead_code)]

use sqlite::{Connection, Value};
use std::path::Path;

macro_rules! ok(($result:expr) => ($result.unwrap()));

pub fn setup_english<T: AsRef<Path>>(path: T) -> Connection {
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

pub fn setup_users<T: AsRef<Path>>(path: T) -> Connection {
    let connection = ok!(sqlite::open(path));
    ok!(connection.execute(
        "
        CREATE TABLE users (id INTEGER, name TEXT, age REAL, photo BLOB, email TEXT);
        INSERT INTO users VALUES (1, 'Alice', 42.69, X'4269', NULL);
        ",
    ));
    connection
}

pub fn count_users(connection: &Connection) -> i64 {
    let query = "SELECT COUNT(*) FROM users";
    let mut statement = ok!(connection.prepare(query));
    ok!(statement.next());
    ok!(statement.read::<i64, _>(0))
}

pub fn check_user(connection: &Connection, id: i64, name: &str, age: f64, photo: Vec<u8>) {
    let query = "SELECT * FROM users WHERE id = :id";
    let mut statement = ok!(connection.prepare(query));
    ok!(statement.bind((":id", id)));
    ok!(statement.next());
    assert_eq!(ok!(statement.read::<i64, _>(0)), id);
    assert_eq!(ok!(statement.read::<String, _>(1)), name);
    assert_eq!(ok!(statement.read::<f64, _>(2)), age);
    assert_eq!(ok!(statement.read::<Vec<u8>, _>(3)), photo);
    assert_eq!(ok!(statement.read::<Value, _>(4)), Value::Null);
}
