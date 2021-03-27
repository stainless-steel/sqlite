#![feature(test)]

extern crate sqlite;
extern crate test;

use sqlite::Value::{Float, Integer};
use sqlite::{Connection, State};
use test::Bencher;

macro_rules! ok(($result:expr) => ($result.unwrap()));

#[bench]
fn read_cursor(bencher: &mut Bencher) {
    let connection = create();
    populate(&connection, 100);
    let mut cursor =
        ok!(connection.prepare("SELECT * FROM data WHERE a > ? AND b > ?")).into_cursor();

    bencher.iter(|| {
        ok!(cursor.bind(&[Integer(42), Float(42.0)]));
        while let Some(row) = ok!(cursor.next()) {
            assert!(ok!(row[0].as_integer()) > 42);
            assert!(ok!(row[1].as_float()) > 42.0);
        }
    })
}

#[bench]
fn read_statement(bencher: &mut Bencher) {
    let connection = create();
    populate(&connection, 100);
    let mut statement = ok!(connection.prepare("SELECT * FROM data WHERE a > ? AND b > ?"));

    bencher.iter(|| {
        ok!(statement.reset());
        ok!(statement.bind(1, 42));
        ok!(statement.bind(2, 42.0));
        while let State::Row = ok!(statement.next()) {
            assert!(ok!(statement.read::<i64>(0)) > 42);
            assert!(ok!(statement.read::<f64>(1)) > 42.0);
        }
    })
}

#[bench]
fn write_cursor(bencher: &mut Bencher) {
    let connection = create();
    let mut cursor =
        ok!(connection.prepare("INSERT INTO data (a, b, c, d) VALUES (?, ?, ?, ?)")).into_cursor();

    bencher.iter(|| {
        ok!(cursor.bind(&[Integer(42), Float(42.0), Float(42.0), Float(42.0)]));
        ok!(cursor.next());
    })
}

#[bench]
fn write_statement(bencher: &mut Bencher) {
    let connection = create();
    let mut statement =
        ok!(connection.prepare("INSERT INTO data (a, b, c, d) VALUES (?, ?, ?, ?)"));

    bencher.iter(|| {
        ok!(statement.reset());
        ok!(statement.bind(1, 42));
        ok!(statement.bind(2, 42.0));
        ok!(statement.bind(3, 42.0));
        ok!(statement.bind(4, 42.0));
        assert_eq!(ok!(statement.next()), State::Done);
    })
}

fn create() -> Connection {
    let connection = ok!(Connection::open(":memory:"));
    ok!(connection.execute("CREATE TABLE data (a INTEGER, b REAL, c REAL, d REAL)"));
    connection
}

fn populate(connection: &Connection, count: usize) {
    let mut statement =
        ok!(connection.prepare("INSERT INTO data (a, b, c, d) VALUES (?, ?, ?, ?)"));
    for i in 0..count {
        ok!(statement.reset());
        ok!(statement.bind(1, i as i64));
        ok!(statement.bind(2, i as f64));
        ok!(statement.bind(3, i as f64));
        ok!(statement.bind(4, i as f64));
        assert_eq!(ok!(statement.next()), State::Done);
    }
}
