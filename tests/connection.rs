extern crate sqlite;
extern crate temporary;

use sqlite::{Connection, OpenFlags, State};

mod common;

use common::setup_users;

macro_rules! ok(($result:expr) => ($result.unwrap()));

#[test]
fn change_count() {
    let connection = setup_users(":memory:");
    assert_eq!(connection.change_count(), 1);
    assert_eq!(connection.total_change_count(), 1);

    ok!(connection.execute("INSERT INTO users VALUES (2, 'Bob', NULL, NULL, NULL)"));
    assert_eq!(connection.change_count(), 1);
    assert_eq!(connection.total_change_count(), 2);

    ok!(connection.execute("UPDATE users SET name = 'Bob' WHERE id = 1"));
    assert_eq!(connection.change_count(), 1);
    assert_eq!(connection.total_change_count(), 3);

    ok!(connection.execute("DELETE FROM users"));
    assert_eq!(connection.change_count(), 2);
    assert_eq!(connection.total_change_count(), 5);
}

#[test]
fn execute() {
    let connection = setup_users(":memory:");
    match connection.execute(":)") {
        Err(error) => assert_eq!(
            error.message,
            Some(String::from(r#"unrecognized token: ":""#))
        ),
        _ => unreachable!(),
    }
}

#[test]
fn iterate() {
    macro_rules! pair(
        ($one:expr, $two:expr) => (($one, Some($two)));
    );

    let connection = setup_users(":memory:");

    let mut done = false;
    let query = "SELECT * FROM users";
    ok!(connection.iterate(query, |pairs| {
        assert_eq!(pairs.len(), 5);
        assert_eq!(pairs[0], pair!("id", "1"));
        assert_eq!(pairs[1], pair!("name", "Alice"));
        assert_eq!(pairs[2], pair!("age", "42.69"));
        assert_eq!(pairs[3], pair!("photo", "\x42\x69"));
        assert_eq!(pairs[4], ("email", None));
        done = true;
        true
    }));
    assert!(done);
}

#[test]
fn open_with_flags() {
    use temporary::Directory;

    let directory = ok!(Directory::new("sqlite"));
    let path = directory.path().join("database.sqlite3");
    setup_users(&path);

    let flags = OpenFlags::new().set_read_only();
    let connection = ok!(Connection::open_with_flags(path, flags));
    match connection.execute("INSERT INTO users VALUES (2, 'Bob', NULL, NULL)") {
        Err(_) => {}
        _ => unreachable!(),
    }
}

#[test]
fn open_with_full_mutex() {
    use std::sync::Arc;
    use std::thread;

    let connection = ok!(Connection::open_with_full_mutex(":memory:"));
    let connection = Arc::new(connection);

    let mut threads = Vec::new();
    for _ in 0..5 {
        let connection_ = connection.clone();
        let thread = thread::spawn(move || {
            ok!(connection_.execute("SELECT 1"));
        });
        threads.push(thread);
    }
    for thread in threads {
        ok!(thread.join());
    }
}

#[test]
fn set_busy_handler() {
    use std::thread;
    use temporary::Directory;

    let directory = ok!(Directory::new("sqlite"));
    let path = directory.path().join("database.sqlite3");
    setup_users(&path);

    let guards = (0..10)
        .map(|_| {
            let path = path.to_path_buf();
            thread::spawn(move || {
                let mut connection = ok!(sqlite::open(&path));
                ok!(connection.set_busy_handler(|_| true));
                let query = "INSERT INTO users VALUES (?, ?, ?, ?, ?)";
                let mut statement = ok!(connection.prepare(query));
                ok!(statement.bind((1, 2i64)));
                ok!(statement.bind((2, "Bob")));
                ok!(statement.bind((3, 69.42)));
                ok!(statement.bind((4, &[0x69u8, 0x42u8][..])));
                ok!(statement.bind((5, ())));
                assert_eq!(ok!(statement.next()), State::Done);
                true
            })
        })
        .collect::<Vec<_>>();

    for guard in guards {
        assert!(ok!(guard.join()));
    }
}
