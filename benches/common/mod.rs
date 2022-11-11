use sqlite::{Connection, State};

macro_rules! ok(($result:expr) => ($result.unwrap()));

pub fn create() -> Connection {
    let connection = ok!(Connection::open(":memory:"));
    let query = "CREATE TABLE data (a INTEGER, b REAL, c REAL, d REAL)";
    ok!(connection.execute(query));
    connection
}

pub fn populate(connection: &Connection, count: usize) {
    let query = "INSERT INTO data (a, b, c, d) VALUES (?, ?, ?, ?)";
    let mut statement = ok!(connection.prepare(query));
    for i in 1..(count + 1) {
        ok!(statement.reset());
        ok!(statement.bind((1, i as i64)));
        ok!(statement.bind((2, i as f64)));
        ok!(statement.bind((3, i as f64)));
        ok!(statement.bind((4, i as f64)));
        assert_eq!(ok!(statement.next()), State::Done);
    }
}
