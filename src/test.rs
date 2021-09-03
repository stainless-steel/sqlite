use marine_rs_sdk::marine;
use marine_sqlite_connector::State;

pub fn main() {}

#[marine]
pub fn test1() {
    let connection = marine_sqlite_connector::open(":memory:").unwrap();

    connection
        .execute(
            "
            CREATE TABLE users (name TEXT, age INTEGER);
            INSERT INTO users VALUES ('Alice', 42);
            INSERT INTO users VALUES ('Bob', 69);
        ",
        )
        .unwrap();

    connection
        .iterate("SELECT * FROM users WHERE age > 50", |pairs| {
            for &(column, value) in pairs.iter() {
                println!("{} = {}", column, value.unwrap());
            }
            true
        })
        .unwrap();
}

#[marine]
pub fn test2() {
    let connection = marine_sqlite_connector::open(":memory:").unwrap();

    println!("connection id = {}\n", connection.as_raw());
    connection
        .execute(
            "
            CREATE TABLE users (name TEXT, age INTEGER);
            INSERT INTO users VALUES ('Alice', 42);
            INSERT INTO users VALUES ('Bob', 69);
        ",
        )
        .unwrap();

    let mut statement = connection
        .prepare("SELECT * FROM users WHERE age > ?")
        .unwrap();

    statement.bind(1, 50).unwrap();

    while let State::Row = statement.next().unwrap() {
        println!("name = {}", statement.read::<String>(0).unwrap());
        println!("age = {}", statement.read::<i64>(1).unwrap());
    }
}
#[marine]
pub fn test3() {
    use marine_sqlite_connector::Value;

    let connection = marine_sqlite_connector::open(":memory:").unwrap();

    connection
        .execute(
            "
            CREATE TABLE users (name TEXT, age INTEGER);
            INSERT INTO users VALUES ('Alice', 42);
            INSERT INTO users VALUES ('Bob', 69);
        ",
        )
        .unwrap();

    let mut cursor = connection
        .prepare("SELECT * FROM users WHERE age > ?")
        .unwrap()
        .cursor();

    cursor.bind(&[Value::Integer(50)]).unwrap();

    while let Some(row) = cursor.next().unwrap() {
        println!("name = {}", row[0].as_string().unwrap());
        println!("age = {}", row[1].as_integer().unwrap());
    }
}

#[marine]
pub fn test4() {
    use marine_sqlite_connector::Value;

    let connection = marine_sqlite_connector::open(":memory:").unwrap();

    connection
        .execute(
            "
            CREATE TABLE test (number INTEGER, blob BLOB NOT NULL);
        ",
        )
        .unwrap();

    let mut cursor = connection
        .prepare("INSERT OR REPLACE INTO test VALUES (?, ?)")
        .unwrap();

    cursor.bind(1, &Value::Integer(50)).unwrap();
    cursor.bind(2, &Value::Binary(vec![1, 2, 3])).unwrap();

    cursor.next().unwrap();
}

#[marine]
pub fn test5() {
    use marine_sqlite_connector::Value;

    let connection = marine_sqlite_connector::open(":memory:").unwrap();

    connection
        .execute(
            "
            CREATE TABLE test (number INTEGER, blob BLOB);
        ",
        )
        .unwrap();

    let mut cursor = connection
        .prepare("INSERT OR REPLACE INTO test VALUES (?, ?)")
        .unwrap();

    cursor.bind(1, &Value::Integer(50)).unwrap();
    cursor.bind(2, &Value::Binary(vec![1, 2, 3])).unwrap();

    cursor.next().unwrap();

    let mut cursor = connection
        .prepare("SELECT blob FROM test WHERE number = ?")
        .unwrap()
        .cursor();

    cursor.bind(&[Value::Integer(50)]).unwrap();

    while let Some(row) = cursor.next().unwrap() {
        if vec![1, 2, 3] != row[0].as_binary().unwrap().to_vec() {
            println!(
                "expected: {:?}, actual: {:?}",
                vec![1, 2, 3],
                row[0].as_binary().unwrap()
            );
        }
    }
}
