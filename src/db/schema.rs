use rusqlite::Connection;

pub fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS services (
            name              TEXT PRIMARY KEY,
            description       TEXT NOT NULL,
            github_repo       TEXT,
            grpc_servers      TEXT NOT NULL DEFAULT '[]',
            grpc_clients      TEXT NOT NULL DEFAULT '[]',
            http_server       INTEGER NOT NULL DEFAULT 0,
            http_clients      TEXT NOT NULL DEFAULT '[]',
            queue_publishers  TEXT NOT NULL DEFAULT '[]',
            queue_subscribers TEXT NOT NULL DEFAULT '[]',
            owned_tables      TEXT NOT NULL DEFAULT '[]'
        );

        CREATE TABLE IF NOT EXISTS tables (
            name        TEXT PRIMARY KEY,
            database    TEXT NOT NULL,
            owner       TEXT NOT NULL,
            description TEXT NOT NULL,
            ddl         TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS queue_contracts (
            name        TEXT PRIMARY KEY,
            description TEXT NOT NULL,
            schema_json TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS proto_contracts (
            server      TEXT PRIMARY KEY,
            description TEXT NOT NULL,
            proto_raw   TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS http_contracts (
            service     TEXT PRIMARY KEY,
            description TEXT NOT NULL,
            spec_raw    TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS embeddings (
            entity_type TEXT NOT NULL,
            entity_key  TEXT NOT NULL,
            text_hash   TEXT NOT NULL,
            embedding   BLOB NOT NULL,
            PRIMARY KEY (entity_type, entity_key)
        );

        CREATE TABLE IF NOT EXISTS meta (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        ",
    )
}
