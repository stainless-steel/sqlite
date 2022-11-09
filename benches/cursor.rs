#![feature(test)]

extern crate sqlite;
extern crate test;

use sqlite::Value;
use test::Bencher;

mod common;

use common::{create, populate};

macro_rules! ok(($result:expr) => ($result.unwrap()));

#[bench]
fn read(bencher: &mut Bencher) {
    let connection = create();
    populate(&connection, 100);
    let query = "SELECT * FROM data WHERE a > ? AND b > ?";
    let mut cursor = Some(ok!(connection.prepare(query)).into_cursor());

    bencher.iter(|| {
        let mut cursor_ = cursor
            .take()
            .unwrap()
            .bind::<&[Value]>(&[42.into(), 42.0.into()][..])
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
fn read_try_next(bencher: &mut Bencher) {
    let connection = create();
    populate(&connection, 100);
    let query = "SELECT * FROM data WHERE a > ? AND b > ?";
    let mut cursor = Some(ok!(connection.prepare(query)).into_cursor());

    bencher.iter(|| {
        let mut cursor_ = cursor
            .take()
            .unwrap()
            .bind::<&[Value]>(&[42.into(), 42.0.into()][..])
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
fn write(bencher: &mut Bencher) {
    let connection = create();
    let query = "INSERT INTO data (a, b, c, d) VALUES (?, ?, ?, ?)";
    let mut cursor = Some(ok!(connection.prepare(query)).into_cursor());

    bencher.iter(|| {
        let mut cursor_ = cursor
            .take()
            .unwrap()
            .bind::<&[Value]>(&[42.into(), 42.0.into(), 42.0.into(), 42.0.into()][..])
            .unwrap();
        match cursor_.next() {
            None => {}
            _ => unreachable!(),
        }
        cursor = Some(cursor_);
    })
}
