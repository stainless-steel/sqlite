#![feature(test)]

extern crate test;

mod common;

use sqlite::State;
use test::Bencher;

use common::{create, populate};

macro_rules! ok(($result:expr) => ($result.unwrap()));

#[bench]
fn read(bencher: &mut Bencher) {
    let connection = create();
    populate(&connection, 100);
    let query = "SELECT * FROM data WHERE a > ? AND b > ?";
    let mut statement = ok!(connection.prepare(query));

    bencher.iter(|| {
        ok!(statement.reset());
        ok!(statement.bind((1, 42)));
        ok!(statement.bind((2, 42.0)));
        let mut count = 0;
        while let State::Row = ok!(statement.next()) {
            assert!(ok!(statement.read::<i64, _>(0)) > 42);
            assert!(ok!(statement.read::<f64, _>(1)) > 42.0);
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
        ok!(statement.reset());
        ok!(statement.bind((1, 42)));
        ok!(statement.bind((2, 42.0)));
        ok!(statement.bind((3, 42.0)));
        ok!(statement.bind((4, 42.0)));
        assert_eq!(ok!(statement.next()), State::Done);
    })
}
