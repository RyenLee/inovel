use rusqlite::{Connection, Result as SqliteResult};

pub fn begin_transaction(conn: &mut Connection) -> SqliteResult<rusqlite::Transaction<'_>> {
    conn.transaction()
}

pub fn commit_transaction(tx: rusqlite::Transaction<'_>) -> SqliteResult<()> {
    tx.commit()
}

pub fn rollback_transaction(tx: rusqlite::Transaction<'_>) -> SqliteResult<()> {
    tx.rollback()
}
