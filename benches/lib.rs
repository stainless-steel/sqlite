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
    let query = "SELECT * FROM data WHERE a > ? AND b > ?";
    let mut cursor = Some(ok!(connection.prepare(query)).into_cursor());

    bencher.iter(|| {
        let mut cursor_ = cursor
            .take()
            .unwrap()
            .bind(&[Integer(42), Float(42.0)])
            .unwrap();
        let mut count = 0;
        while let Some(Ok(row)) = cursor_.next() {
            assert!(row.get::<i64, _>(0) > 42);
            assert!(row.get::<f64, _>(1) > 42.0);
            count += 1;
        }
        assert_eq!(count, 100 - 42);
        cursor = Some(cursor_);
    })
}

#[bench]
fn read_cursor_try_next(bencher: &mut Bencher) {
    let connection = create();
    populate(&connection, 100);
    let query = "SELECT * FROM data WHERE a > ? AND b > ?";
    let mut cursor = Some(ok!(connection.prepare(query)).into_cursor());

    bencher.iter(|| {
        let mut cursor_ = cursor
            .take()
            .unwrap()
            .bind(&[Integer(42), Float(42.0)])
            .unwrap();
        let mut count = 0;
        while let Ok(Some(row)) = cursor_.try_next() {
            assert!(ok!(row[0].as_integer()) > 42);
            assert!(ok!(row[1].as_float()) > 42.0);
            count += 1;
        }
        assert_eq!(count, 100 - 42);
        cursor = Some(cursor_);
    })
}

#[bench]
fn read_statement(bencher: &mut Bencher) {
    let connection = create();
    populate(&connection, 100);
    let query = "SELECT * FROM data WHERE a > ? AND b > ?";
    let mut statement = Some(ok!(connection.prepare(query)));

    bencher.iter(|| {
        let mut statement_ = statement
            .take()
            .unwrap()
            .reset()
            .unwrap()
            .bind(1, 42)
            .unwrap()
            .bind(2, 42.0)
            .unwrap();
        let mut count = 0;
        while let State::Row = ok!(statement_.next()) {
            assert!(ok!(statement_.read::<i64>(0)) > 42);
            assert!(ok!(statement_.read::<f64>(1)) > 42.0);
            count += 1;
        }
        assert_eq!(count, 100 - 42);
        statement = Some(statement_);
    })
}

#[bench]
fn write_cursor(bencher: &mut Bencher) {
    let connection = create();
    let query = "INSERT INTO data (a, b, c, d) VALUES (?, ?, ?, ?)";
    let mut cursor = Some(ok!(connection.prepare(query)).into_cursor());

    bencher.iter(|| {
        let mut cursor_ = cursor
            .take()
            .unwrap()
            .bind(&[Integer(42), Float(42.0), Float(42.0), Float(42.0)])
            .unwrap();
        match cursor_.next() {
            None => {}
            _ => unreachable!(),
        }
        cursor = Some(cursor_);
    })
}

#[bench]
fn write_statement(bencher: &mut Bencher) {
    let connection = create();
    let query = "INSERT INTO data (a, b, c, d) VALUES (?, ?, ?, ?)";
    let mut statement = Some(ok!(connection.prepare(query)));

    bencher.iter(|| {
        let mut statement_ = statement
            .take()
            .unwrap()
            .reset()
            .unwrap()
            .bind(1, 42)
            .unwrap()
            .bind(2, 42.0)
            .unwrap()
            .bind(3, 42.0)
            .unwrap()
            .bind(4, 42.0)
            .unwrap();
        assert_eq!(ok!(statement_.next()), State::Done);
        statement = Some(statement_);
    })
}

fn create() -> Connection {
    let connection = ok!(Connection::open(":memory:"));
    let query = "CREATE TABLE data (a INTEGER, b REAL, c REAL, d REAL)";
    ok!(connection.execute(query));
    connection
}

fn populate(connection: &Connection, count: usize) {
    let query = "INSERT INTO data (a, b, c, d) VALUES (?, ?, ?, ?)";
    let mut statement = ok!(connection.prepare(query));
    for i in 1..(count + 1) {
        statement = statement
            .reset()
            .unwrap()
            .bind(1, i as i64)
            .unwrap()
            .bind(2, i as f64)
            .unwrap()
            .bind(3, i as f64)
            .unwrap()
            .bind(4, i as f64)
            .unwrap();
        assert_eq!(ok!(statement.next()), State::Done);
    }
}
