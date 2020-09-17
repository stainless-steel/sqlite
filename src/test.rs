extern crate fluence;
extern crate fce_sqlite_connector;

use fluence::fce;
use fce_sqlite_connector::State;

pub fn main() {}

#[fce]
pub fn test1() {
    let connection = fce_sqlite_connector::open(":memory:").unwrap();

    connection
        .execute(
            "
            CREATE TABLE users (name TEXT, age INTEGER);
            INSERT INTO users VALUES ('Alice', 42);
            INSERT INTO users VALUES ('Bob', 69);
        ",
        )
        .unwrap();

    connection
        .iterate("SELECT * FROM users WHERE age > 50", |pairs| {
            for &(column, value) in pairs.iter() {
                println!("{} = {}", column, value.unwrap());
            }
            true
        })
        .unwrap();
}

#[fce]
pub fn test2() {
    let connection = fce_sqlite_connector::open(":memory:").unwrap();

    println!("connection id = {}\n", connection.as_raw());
    connection
        .execute(
            "
            CREATE TABLE users (name TEXT, age INTEGER);
            INSERT INTO users VALUES ('Alice', 42);
            INSERT INTO users VALUES ('Bob', 69);
        ",
        )
        .unwrap();

    let mut statement = connection
        .prepare("SELECT * FROM users WHERE age > ?")
        .unwrap();

    statement.bind(1, 50).unwrap();

    while let State::Row = statement.next().unwrap() {
        println!("name = {}", statement.read::<String>(0).unwrap());
        println!("age = {}", statement.read::<i64>(1).unwrap());
    }
}
#[fce]
pub fn test3() {
    use fce_sqlite_connector::Value;

    let connection = fce_sqlite_connector::open(":memory:").unwrap();

    connection
        .execute(
            "
            CREATE TABLE users (name TEXT, age INTEGER);
            INSERT INTO users VALUES ('Alice', 42);
            INSERT INTO users VALUES ('Bob', 69);
        ",
        )
        .unwrap();

    let mut cursor = connection
        .prepare("SELECT * FROM users WHERE age > ?")
        .unwrap()
        .cursor();

    cursor.bind(&[Value::Integer(50)]).unwrap();

    while let Some(row) = cursor.next().unwrap() {
        println!("name = {}", row[0].as_string().unwrap());
        println!("age = {}", row[1].as_integer().unwrap());
    }
}
