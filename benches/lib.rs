#![feature(test)]

extern crate sqlite;
extern crate test;

use sqlite::{Connection, State, Value};
use test::Bencher;

#[bench]
fn read_cursor(bencher: &mut Bencher) {
    let connection = create();
    populate(&connection, 100);
    let mut cursor = connection.prepare("
        SELECT * FROM data WHERE integer > ? AND real > ?
    ").unwrap().cursor();

    bencher.iter(|| {
        cursor.bind(&[Value::Integer(42), Value::Float(69.0)]).unwrap();
        while let Some(row) = cursor.next().unwrap() {
            assert!(row[0].as_integer().unwrap() > 42);
            assert!(row[1].as_float().unwrap() > 69.0);
        }
    })
}

#[bench]
fn read_statement(bencher: &mut Bencher) {
    let connection = create();
    populate(&connection, 100);
    let mut statement = connection.prepare("
        SELECT * FROM data WHERE integer > ? AND real > ?
    ").unwrap();

    bencher.iter(|| {
        statement.reset().unwrap();
        statement.bind(1, 42).unwrap();
        statement.bind(2, 69.0).unwrap();
        while let State::Row = statement.next().unwrap() {
            assert!(statement.read::<i64>(0).unwrap() > 42);
            assert!(statement.read::<f64>(1).unwrap() > 69.0);
        }
    })
}

#[bench]
fn write_cursor(bencher: &mut Bencher) {
    let connection = create();
    let mut cursor = connection.prepare("
        INSERT INTO data (integer, real) VALUES (?, ?)
    ").unwrap().cursor();

    bencher.iter(|| {
        cursor.bind(&[Value::Integer(42), Value::Float(69.0)]).unwrap();
        cursor.next().unwrap();
    })
}

#[bench]
fn write_statement(bencher: &mut Bencher) {
    let connection = create();
    let mut statement = connection.prepare("
        INSERT INTO data (integer, real) VALUES (?, ?)
    ").unwrap();

    bencher.iter(|| {
        statement.reset().unwrap();
        statement.bind(1, 42).unwrap();
        statement.bind(2, 69.0).unwrap();
        assert_eq!(statement.next().unwrap(), State::Done);
    })
}

fn create() -> Connection {
    let connection = Connection::open(":memory:").unwrap();
    connection.execute("CREATE TABLE data (integer INTEGER, real REAL)").unwrap();
    connection
}

fn populate(connection: &Connection, count: usize) {
    let mut statement = connection.prepare("
        INSERT INTO data (integer, real) VALUES (?, ?)
    ").unwrap();
    for i in 0..count {
        statement.reset().unwrap();
        statement.bind(1, i as i64).unwrap();
        statement.bind(2, i as f64).unwrap();
        assert_eq!(statement.next().unwrap(), State::Done);
    }
}
