use marine_rs_sdk::marine;
use marine_sqlite_connector::State;
use marine_sqlite_connector::Value;

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

    let mut statement = connection
        .prepare("SELECT * FROM users WHERE age > ?")
        .unwrap();

    statement.bind(1, 50).unwrap();

    assert_eq!(statement.next().unwrap(), State::Row);
    assert_eq!(statement.read::<String>(0).unwrap(), "Bob");
    assert_eq!(statement.read::<i64>(1).unwrap(), 69);
}

#[marine]
pub fn test2() {
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
        assert_eq!(row[0].as_string().unwrap(), "Bob");
        assert_eq!(row[1].as_integer().unwrap(), 69);
    }
}

#[marine]
pub fn test3() {
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

    // check that blob is not null
    assert!(cursor.next().is_ok());
}

#[marine]
pub fn test4() {
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
        assert_eq!(row[0].as_binary().unwrap().to_vec(), vec![1, 2, 3]);
    }
}
