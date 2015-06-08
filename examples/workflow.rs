extern crate sqlite;

use std::path::PathBuf;

fn main() {
    let path = setup();
    let database = sqlite::open(&path).unwrap();

    database.instruct(r#"
        CREATE TABLE `users` (id INTEGER, name VARCHAR(255));
        INSERT INTO `users` (id, name) VALUES (1, 'Alice');
    "#).unwrap();

    database.iterate("SELECT * FROM `users`;", |pairs| {
        for (ref column, ref value) in pairs {
            println!("{} = {}", column, value);
        }
        true
    }).unwrap();
}

fn setup() -> PathBuf {
    let path = PathBuf::from("database.sqlite3");
    if std::fs::metadata(&path).is_ok() {
        std::fs::remove_file(&path).unwrap();
    }
    path
}
