extern crate marine_rs_sdk;

use self::marine_rs_sdk::marine;

pub(crate) type Sqlite3DbHandle = u32;
pub(crate) type Sqlite3StmtHandle = u32;

#[marine]
#[derive(Clone, Debug)]
pub struct DBOpenDescriptor {
    pub ret_code: i32,
    pub db_handle: u32,
}

#[marine]
#[derive(Clone, Debug)]
pub struct DBPrepareDescriptor {
    pub ret_code: i32,
    pub stmt_handle: u32,
    pub tail: u32,
}

#[marine]
#[derive(Clone, Debug)]
pub struct DBExecDescriptor {
    pub ret_code: i32,
    pub err_msg: String,
}

#[marine]
#[link(wasm_import_module = "sqlite3")]
extern "C" {
    /*
     SQLITE_API int sqlite3_open_v2(
       const char *filename,   /* Database filename (UTF-8) */
       sqlite3 **ppDb,         /* OUT: SQLite db handle */
       int flags,              /* Flags */
       const char *zVfs        /* Name of VFS module to use */
      );
    */
    pub fn sqlite3_open_v2(filename: String, flags: i32, vfs: String) -> DBOpenDescriptor;

    // SQLITE_API int sqlite3_close(sqlite3*);
    pub fn sqlite3_close(db_handle: u32) -> i32;

    /*
     SQLITE_API int sqlite3_prepare_v2(
       sqlite3 *db,            /* Database handle */
       const char *zSql,       /* SQL statement, UTF-8 encoded */
       int nByte,              /* Maximum length of zSql in bytes. */
       sqlite3_stmt **ppStmt,  /* OUT: Statement handle */
       const char **pzTail     /* OUT: Pointer to unused portion of zSql */
     );
    */
    pub fn sqlite3_prepare_v2(db_handle: u32, sql: String) -> DBPrepareDescriptor;

    /*
     SQLITE_API int sqlite3_exec(
       sqlite3*,                                  /* An open database */
       const char *sql,                           /* SQL to be evaluated */
       int (*callback)(void*,int,char**,char**),  /* Callback function */
       void *,                                    /* 1st argument to callback */
       char **errmsg                              /* Error msg written here */
     );
    */
    pub fn sqlite3_exec(
        db_handle: u32,
        sql: String,
        callback_id: i32,
        callback_arg: i32,
    ) -> DBExecDescriptor;

    // SQLITE_API int sqlite3_libversion_number(void);
    pub fn sqlite3_libversion_number() -> i32;

    // SQLITE_API int sqlite3_changes(sqlite3*);
    pub fn sqlite3_changes(db_handle: u32) -> i32;

    // SQLITE_API int sqlite3_total_changes(sqlite3*);
    pub fn sqlite3_total_changes(db_handle: u32) -> i32;

    // SQLITE_API int sqlite3_busy_timeout(sqlite3*, int ms);
    pub fn sqlite3_busy_timeout(db_handle: u32, ms: u32) -> i32;

    // SQLITE_API const char *sqlite3_errmsg(sqlite3*);
    pub fn sqlite3_errmsg(db_handle: u32) -> String;

    // SQLITE_API int sqlite3_errcode(sqlite3 *db);
    pub fn sqlite3_errcode(db: u32) -> i32;

    // SQLITE_API int sqlite3_column_type(sqlite3_stmt*, int iCol);
    pub fn sqlite3_column_type(stmt_handle: u32, icol: u32) -> i32;

    // SQLITE_API const char *sqlite3_column_name(sqlite3_stmt*, int N);
    pub fn sqlite3_column_name(stmt_handle: u32, N: u32) -> String;

    // SQLITE_API int sqlite3_step(sqlite3_stmt*);
    pub fn sqlite3_step(stmt_handle: u32) -> i32;

    // SQLITE_API int sqlite3_reset(sqlite3_stmt *pStmt);
    pub fn sqlite3_reset(stmt_handle: u32) -> i32;

    // SQLITE_API int sqlite3_bind_blob(sqlite3_stmt*, int, const void*, int n, void(*)(void*));
    pub fn sqlite3_bind_blob(stmt_handle: u32, pos: i32, blob: Vec<u8>, xDel: i32) -> i32;

    // SQLITE_API int sqlite3_bind_double(sqlite3_stmt*, int, double);
    pub fn sqlite3_bind_double(stmt_handle: u32, pos: i32, value: f64) -> i32;

    // SQLITE_API int sqlite3_bind_int64(sqlite3_stmt*, int, sqlite3_int64);
    pub fn sqlite3_bind_int64(stmt_handle: u32, pos: i32, value: i64) -> i32;

    // SQLITE_API int sqlite3_bind_null(sqlite3_stmt*, int);
    pub fn sqlite3_bind_null(stmt_handle: u32, pos: i32) -> i32;

    // SQLITE_API int sqlite3_bind_text(sqlite3_stmt*,int,const char*,int,void(*)(void*));
    pub fn sqlite3_bind_text(stmt_handle: u32, pos: i32, text: String, xDel: i32) -> i32;

    // SQLITE_API int sqlite3_column_count(sqlite3_stmt *pStmt)
    pub fn sqlite3_column_count(stmt_handle: u32) -> i32;

    // SQLITE_API const void *sqlite3_column_blob(sqlite3_stmt*, int iCol);
    pub fn sqlite3_column_blob(stmt_handle: u32, icol: i32) -> Vec<u8>;

    // SQLITE_API double sqlite3_column_double(sqlite3_stmt*, int iCol);
    pub fn sqlite3_column_double(stmt_handle: u32, icol: i32) -> f64;

    // SQLITE_API sqlite3_int64 sqlite3_column_int64(sqlite3_stmt*, int iCol);
    pub fn sqlite3_column_int64(stmt_handle: u32, icol: u32) -> i64;

    // SQLITE_API const unsigned char *sqlite3_column_text(sqlite3_stmt*, int iCol);
    pub fn sqlite3_column_text(stmt_handle: u32, icol: u32) -> String;

    // SQLITE_API int sqlite3_column_bytes(sqlite3_stmt*, int iCol);
    pub fn sqlite3_column_bytes(stmt_handle: u32, icol: u32) -> i32;

    // SQLITE_API int sqlite3_finalize(sqlite3_stmt *pStmt);
    pub fn sqlite3_finalize(stmt_handle: u32) -> i32;
}
