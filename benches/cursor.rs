#![feature(test)]

extern crate test;

mod common;

use sqlite::Value;
use test::Bencher;

use common::{create, populate};

macro_rules! ok(($result:expr) => ($result.unwrap()));

#[bench]
fn read_next(bencher: &mut Bencher) {
    let connection = create();
    populate(&connection, 100);
    let query = "SELECT * FROM data WHERE a > ? AND b > ?";
    let mut statement = ok!(connection.prepare(query));

    bencher.iter(|| {
        let mut count = 0;
        for row in statement
            .iter()
            .bind::<&[Value]>(&[42.into(), 42.0.into()][..])
            .unwrap()
            .map(|row| row.unwrap())
        {
            assert!(row.read::<i64, _>(0) > 42);
            assert!(row.read::<f64, _>(1) > 42.0);
            count += 1;
        }
        assert_eq!(count, 100 - 42);
    })
}

#[bench]
fn read_try_next(bencher: &mut Bencher) {
    let connection = create();
    populate(&connection, 100);
    let query = "SELECT * FROM data WHERE a > ? AND b > ?";
    let mut statement = ok!(connection.prepare(query));

    bencher.iter(|| {
        let mut cursor = statement
            .iter()
            .bind::<&[Value]>(&[42.into(), 42.0.into()][..])
            .unwrap();
        let mut count = 0;
        while let Ok(Some(row)) = cursor.try_next() {
            assert!(ok!((&row[0]).try_into::<i64>()) > 42);
            assert!(ok!((&row[1]).try_into::<f64>()) > 42.0);
            count += 1;
        }
        assert_eq!(count, 100 - 42);
    })
}

#[bench]
fn write(bencher: &mut Bencher) {
    let connection = create();
    let query = "INSERT INTO data (a, b, c, d) VALUES (?, ?, ?, ?)";
    let mut statement = ok!(connection.prepare(query));

    bencher.iter(|| {
        let mut cursor = statement
            .iter()
            .bind::<&[Value]>(&[42.into(), 42.0.into(), 42.0.into(), 42.0.into()][..])
            .unwrap();
        match cursor.next() {
            None => {}
            _ => unreachable!(),
        }
    })
}
