extern crate sqlite;

use std::path::Path;

fn main() {
    let database = sqlite::open(&Path::new(":memory:")).unwrap();

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
