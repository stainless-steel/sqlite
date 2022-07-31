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

    bencher.iter(|| {
        let mut cursor = ok!(
            ok!(connection.prepare("SELECT * FROM data WHERE a > ? AND b > ?"))
                .into_cursor()
                .bind(&[Integer(42), Float(42.0)])
        );
        while let Some(row) = cursor.next() {
            let row = ok!(row);
            assert!(row.get::<i64, _>(0) > 42);
            assert!(row.get::<f64, _>(1) > 42.0);
        }
    })
}

#[bench]
fn read_statement(bencher: &mut Bencher) {
    let connection = create();
    populate(&connection, 100);

    bencher.iter(|| {
        let mut statement = ok!(connection
            .prepare("SELECT * FROM data WHERE a > ? AND b > ?")
            .and_then(|statement| statement.bind(1, 42))
            .and_then(|statement| statement.bind(2, 42.0)));
        while let State::Row = ok!(statement.next()) {
            assert!(ok!(statement.read::<i64>(0)) > 42);
            assert!(ok!(statement.read::<f64>(1)) > 42.0);
        }
    })
}

#[bench]
fn write_cursor(bencher: &mut Bencher) {
    let connection = create();

    bencher.iter(|| {
        let mut cursor = ok!(connection
            .prepare("INSERT INTO data (a, b, c, d) VALUES (?, ?, ?, ?)")
            .map(|statement| statement.into_cursor())
            .and_then(|cursor| cursor.bind(&[Integer(42), Float(42.0), Float(42.0), Float(42.0)])));
        cursor.next();
    })
}

#[bench]
fn write_statement(bencher: &mut Bencher) {
    let connection = create();

    bencher.iter(|| {
        let mut statement = ok!(connection
            .prepare("INSERT INTO data (a, b, c, d) VALUES (?, ?, ?, ?)")
            .and_then(|statement| statement.bind(1, 42))
            .and_then(|statement| statement.bind(2, 42.0))
            .and_then(|statement| statement.bind(3, 42.0))
            .and_then(|statement| statement.bind(4, 42.0)));
        assert_eq!(ok!(statement.next()), State::Done);
    })
}

fn create() -> Connection {
    let connection = ok!(Connection::open(":memory:"));
    ok!(connection.execute("CREATE TABLE data (a INTEGER, b REAL, c REAL, d REAL)"));
    connection
}

fn populate(connection: &Connection, count: usize) {
    for i in 0..count {
        let mut statement = ok!(connection
            .prepare("INSERT INTO data (a, b, c, d) VALUES (?, ?, ?, ?)")
            .and_then(|statement| statement.bind(1, i as i64))
            .and_then(|statement| statement.bind(2, i as f64))
            .and_then(|statement| statement.bind(3, i as f64))
            .and_then(|statement| statement.bind(4, i as f64)));
        assert_eq!(ok!(statement.next()), State::Done);
    }
}
