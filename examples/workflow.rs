extern crate sqlite;

use std::fs;
use std::path::PathBuf;

fn main() {
    let path = setup();
    let database = sqlite::open(&path).unwrap();

    database.execute(r#"
        CREATE TABLE `users` (id INTEGER, name VARCHAR(255));
        INSERT INTO `users` (id, name) VALUES (1, 'Alice');
    "#).unwrap();

    database.process("SELECT * FROM `users`;", |pairs| {
        for &(column, value) in pairs.iter() {
            println!("{} = {}", column, value.unwrap());
        }
        true
    }).unwrap();
}

fn setup() -> PathBuf {
    let path = PathBuf::from("database.sqlite3");
    if fs::metadata(&path).is_ok() {
        fs::remove_file(&path).unwrap();
    }
    path
}
