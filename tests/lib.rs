extern crate sqlite;
extern crate temporary;

use std::path::PathBuf;
use temporary::Directory;

macro_rules! ok(
    ($result:expr) => ($result.unwrap());
);

#[test]
fn open() {
    let (path, _directory) = setup();
    let _database = ok!(sqlite::open(&path));
}

fn setup() -> (PathBuf, Directory) {
    let directory = ok!(Directory::new("sqlite"));
    (directory.path().join("database.sqlite3"), directory)
}
