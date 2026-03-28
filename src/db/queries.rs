use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::models::*;

// ── Services ──

pub fn insert_service(conn: &Connection, s: &Service) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO services (name, description, github_repo, grpc_servers, grpc_clients, http_server, http_clients, queue_publishers, queue_subscribers, owned_tables)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            s.name,
            s.description,
            s.github_repo,
            serde_json::to_string(&s.grpc_servers).unwrap(),
            serde_json::to_string(&s.grpc_clients).unwrap(),
            s.http_server as i32,
            serde_json::to_string(&s.http_clients).unwrap(),
            serde_json::to_string(&s.queue_publishers).unwrap(),
            serde_json::to_string(&s.queue_subscribers).unwrap(),
            serde_json::to_string(&s.tables).unwrap(),
        ],
    )?;
    Ok(())
}

pub fn update_service(conn: &Connection, name: &str, s: &Service) -> Result<(), AppError> {
    let updated = conn.execute(
        "UPDATE services SET description=?1, github_repo=?2, grpc_servers=?3, grpc_clients=?4, http_server=?5, http_clients=?6, queue_publishers=?7, queue_subscribers=?8, owned_tables=?9
         WHERE name=?10",
        params![
            s.description,
            s.github_repo,
            serde_json::to_string(&s.grpc_servers).unwrap(),
            serde_json::to_string(&s.grpc_clients).unwrap(),
            s.http_server as i32,
            serde_json::to_string(&s.http_clients).unwrap(),
            serde_json::to_string(&s.queue_publishers).unwrap(),
            serde_json::to_string(&s.queue_subscribers).unwrap(),
            serde_json::to_string(&s.tables).unwrap(),
            name,
        ],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub fn delete_service(conn: &Connection, name: &str) -> Result<(), AppError> {
    let deleted = conn.execute("DELETE FROM services WHERE name=?1", params![name])?;
    if deleted == 0 {
        return Err(AppError::NotFound);
    }
    conn.execute(
        "DELETE FROM embeddings WHERE entity_type='service' AND entity_key=?1",
        params![name],
    )?;
    Ok(())
}

pub fn get_service(conn: &Connection, name: &str) -> Result<Option<Service>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT name, description, github_repo, grpc_servers, grpc_clients, http_server, http_clients, queue_publishers, queue_subscribers, owned_tables
         FROM services WHERE name=?1",
    )?;
    let mut rows = stmt.query_map(params![name], row_to_service)?;
    Ok(rows.next().transpose()?)
}

pub fn list_services(conn: &Connection) -> Result<Vec<ServiceSummary>, AppError> {
    let mut stmt = conn.prepare("SELECT name, description FROM services ORDER BY name")?;
    let rows = stmt.query_map([], |row| {
        Ok(ServiceSummary {
            name: row.get(0)?,
            description: row.get(1)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn list_services_full(conn: &Connection) -> Result<Vec<Service>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT name, description, github_repo, grpc_servers, grpc_clients, http_server, http_clients, queue_publishers, queue_subscribers, owned_tables
         FROM services ORDER BY name",
    )?;
    let rows = stmt.query_map([], row_to_service)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn row_to_service(row: &rusqlite::Row) -> rusqlite::Result<Service> {
    let grpc_servers: String = row.get(3)?;
    let grpc_clients: String = row.get(4)?;
    let http_server: i32 = row.get(5)?;
    let http_clients: String = row.get(6)?;
    let queue_publishers: String = row.get(7)?;
    let queue_subscribers: String = row.get(8)?;
    let owned_tables: String = row.get(9)?;
    Ok(Service {
        name: row.get(0)?,
        description: row.get(1)?,
        github_repo: row.get(2)?,
        grpc_servers: serde_json::from_str(&grpc_servers).unwrap_or_default(),
        grpc_clients: serde_json::from_str(&grpc_clients).unwrap_or_default(),
        http_server: http_server != 0,
        http_clients: serde_json::from_str(&http_clients).unwrap_or_default(),
        queue_publishers: serde_json::from_str(&queue_publishers).unwrap_or_default(),
        queue_subscribers: serde_json::from_str(&queue_subscribers).unwrap_or_default(),
        tables: serde_json::from_str(&owned_tables).unwrap_or_default(),
    })
}

// ── Tables ──

pub fn insert_table(conn: &Connection, t: &Table) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO tables (name, database, owner, description, ddl) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![t.name, t.database, t.owner, t.description, t.ddl],
    )?;
    Ok(())
}

pub fn update_table(conn: &Connection, name: &str, t: &Table) -> Result<(), AppError> {
    let updated = conn.execute(
        "UPDATE tables SET database=?1, owner=?2, description=?3, ddl=?4 WHERE name=?5",
        params![t.database, t.owner, t.description, t.ddl, name],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub fn delete_table(conn: &Connection, name: &str) -> Result<(), AppError> {
    let deleted = conn.execute("DELETE FROM tables WHERE name=?1", params![name])?;
    if deleted == 0 {
        return Err(AppError::NotFound);
    }
    conn.execute(
        "DELETE FROM embeddings WHERE entity_type='table' AND entity_key=?1",
        params![name],
    )?;
    Ok(())
}

pub fn get_table(conn: &Connection, name: &str) -> Result<Option<Table>, AppError> {
    let mut stmt =
        conn.prepare("SELECT name, database, owner, description, ddl FROM tables WHERE name=?1")?;
    let mut rows = stmt.query_map(params![name], |row| {
        Ok(Table {
            name: row.get(0)?,
            database: row.get(1)?,
            owner: row.get(2)?,
            description: row.get(3)?,
            ddl: row.get(4)?,
        })
    })?;
    Ok(rows.next().transpose()?)
}

pub fn list_tables(conn: &Connection) -> Result<Vec<TableSummary>, AppError> {
    let mut stmt = conn.prepare("SELECT name, description FROM tables ORDER BY name")?;
    let rows = stmt.query_map([], |row| {
        Ok(TableSummary {
            name: row.get(0)?,
            description: row.get(1)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn get_tables_by_names(conn: &Connection, names: &[String]) -> Result<Vec<Table>, AppError> {
    if names.is_empty() {
        return Ok(vec![]);
    }
    let placeholders: Vec<String> = (1..=names.len()).map(|i| format!("?{i}")).collect();
    let sql = format!(
        "SELECT name, database, owner, description, ddl FROM tables WHERE name IN ({})",
        placeholders.join(",")
    );
    let mut stmt = conn.prepare(&sql)?;
    let params: Vec<&dyn rusqlite::types::ToSql> =
        names.iter().map(|n| n as &dyn rusqlite::types::ToSql).collect();
    let rows = stmt.query_map(params.as_slice(), |row| {
        Ok(Table {
            name: row.get(0)?,
            database: row.get(1)?,
            owner: row.get(2)?,
            description: row.get(3)?,
            ddl: row.get(4)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

// ── Queue Contracts ──

pub fn insert_queue(conn: &Connection, q: &QueueContract) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO queue_contracts (name, description, schema_json) VALUES (?1, ?2, ?3)",
        params![q.name, q.description, serde_json::to_string(&q.schema).unwrap()],
    )?;
    Ok(())
}

pub fn update_queue(conn: &Connection, name: &str, q: &QueueContract) -> Result<(), AppError> {
    let updated = conn.execute(
        "UPDATE queue_contracts SET description=?1, schema_json=?2 WHERE name=?3",
        params![q.description, serde_json::to_string(&q.schema).unwrap(), name],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub fn delete_queue(conn: &Connection, name: &str) -> Result<(), AppError> {
    let deleted = conn.execute("DELETE FROM queue_contracts WHERE name=?1", params![name])?;
    if deleted == 0 {
        return Err(AppError::NotFound);
    }
    conn.execute(
        "DELETE FROM embeddings WHERE entity_type='queue' AND entity_key=?1",
        params![name],
    )?;
    Ok(())
}

pub fn get_queue(conn: &Connection, name: &str) -> Result<Option<QueueContract>, AppError> {
    let mut stmt = conn
        .prepare("SELECT name, description, schema_json FROM queue_contracts WHERE name=?1")?;
    let mut rows = stmt.query_map(params![name], |row| {
        let schema_json: String = row.get(2)?;
        Ok(QueueContract {
            name: row.get(0)?,
            description: row.get(1)?,
            schema: serde_json::from_str(&schema_json).unwrap_or_else(|_| QueueSchema {
                name: String::new(),
                fields: vec![],
                notes: String::new(),
            }),
        })
    })?;
    Ok(rows.next().transpose()?)
}

pub fn list_queues(conn: &Connection) -> Result<Vec<QueueSummary>, AppError> {
    let mut stmt =
        conn.prepare("SELECT name, description FROM queue_contracts ORDER BY name")?;
    let rows = stmt.query_map([], |row| {
        Ok(QueueSummary {
            name: row.get(0)?,
            description: row.get(1)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn get_queues_by_names(
    conn: &Connection,
    names: &[String],
) -> Result<Vec<QueueContract>, AppError> {
    if names.is_empty() {
        return Ok(vec![]);
    }
    let placeholders: Vec<String> = (1..=names.len()).map(|i| format!("?{i}")).collect();
    let sql = format!(
        "SELECT name, description, schema_json FROM queue_contracts WHERE name IN ({})",
        placeholders.join(",")
    );
    let mut stmt = conn.prepare(&sql)?;
    let params: Vec<&dyn rusqlite::types::ToSql> =
        names.iter().map(|n| n as &dyn rusqlite::types::ToSql).collect();
    let rows = stmt.query_map(params.as_slice(), |row| {
        let schema_json: String = row.get(2)?;
        Ok(QueueContract {
            name: row.get(0)?,
            description: row.get(1)?,
            schema: serde_json::from_str(&schema_json).unwrap_or_else(|_| QueueSchema {
                name: String::new(),
                fields: vec![],
                notes: String::new(),
            }),
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

// ── Proto Contracts ──

pub fn insert_proto(conn: &Connection, p: &ProtoContract) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO proto_contracts (server, description, proto_raw) VALUES (?1, ?2, ?3)",
        params![p.server, p.description, p.proto_raw],
    )?;
    Ok(())
}

pub fn update_proto(conn: &Connection, server: &str, p: &ProtoContract) -> Result<(), AppError> {
    let updated = conn.execute(
        "UPDATE proto_contracts SET description=?1, proto_raw=?2 WHERE server=?3",
        params![p.description, p.proto_raw, server],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub fn delete_proto(conn: &Connection, server: &str) -> Result<(), AppError> {
    let deleted = conn.execute("DELETE FROM proto_contracts WHERE server=?1", params![server])?;
    if deleted == 0 {
        return Err(AppError::NotFound);
    }
    conn.execute(
        "DELETE FROM embeddings WHERE entity_type='proto' AND entity_key=?1",
        params![server],
    )?;
    Ok(())
}

pub fn get_proto(conn: &Connection, server: &str) -> Result<Option<ProtoContract>, AppError> {
    let mut stmt = conn
        .prepare("SELECT server, description, proto_raw FROM proto_contracts WHERE server=?1")?;
    let mut rows = stmt.query_map(params![server], |row| {
        Ok(ProtoContract {
            server: row.get(0)?,
            description: row.get(1)?,
            proto_raw: row.get(2)?,
        })
    })?;
    Ok(rows.next().transpose()?)
}

pub fn list_protos(conn: &Connection) -> Result<Vec<ProtoSummary>, AppError> {
    let mut stmt =
        conn.prepare("SELECT server, description FROM proto_contracts ORDER BY server")?;
    let rows = stmt.query_map([], |row| {
        Ok(ProtoSummary {
            server: row.get(0)?,
            description: row.get(1)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn get_protos_by_servers(
    conn: &Connection,
    servers: &[String],
) -> Result<Vec<ProtoContract>, AppError> {
    if servers.is_empty() {
        return Ok(vec![]);
    }
    let placeholders: Vec<String> = (1..=servers.len()).map(|i| format!("?{i}")).collect();
    let sql = format!(
        "SELECT server, description, proto_raw FROM proto_contracts WHERE server IN ({})",
        placeholders.join(",")
    );
    let mut stmt = conn.prepare(&sql)?;
    let params: Vec<&dyn rusqlite::types::ToSql> = servers
        .iter()
        .map(|s| s as &dyn rusqlite::types::ToSql)
        .collect();
    let rows = stmt.query_map(params.as_slice(), |row| {
        Ok(ProtoContract {
            server: row.get(0)?,
            description: row.get(1)?,
            proto_raw: row.get(2)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

// ── HTTP Contracts ──

pub fn insert_http_contract(conn: &Connection, h: &HttpContract) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO http_contracts (service, description, spec_raw) VALUES (?1, ?2, ?3)",
        params![h.service, h.description, h.spec_raw],
    )?;
    Ok(())
}

pub fn update_http_contract(
    conn: &Connection,
    service: &str,
    h: &HttpContract,
) -> Result<(), AppError> {
    let updated = conn.execute(
        "UPDATE http_contracts SET description=?1, spec_raw=?2 WHERE service=?3",
        params![h.description, h.spec_raw, service],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub fn delete_http_contract(conn: &Connection, service: &str) -> Result<(), AppError> {
    let deleted = conn.execute(
        "DELETE FROM http_contracts WHERE service=?1",
        params![service],
    )?;
    if deleted == 0 {
        return Err(AppError::NotFound);
    }
    conn.execute(
        "DELETE FROM embeddings WHERE entity_type='http' AND entity_key=?1",
        params![service],
    )?;
    Ok(())
}

pub fn get_http_contract(
    conn: &Connection,
    service: &str,
) -> Result<Option<HttpContract>, AppError> {
    let mut stmt = conn
        .prepare("SELECT service, description, spec_raw FROM http_contracts WHERE service=?1")?;
    let mut rows = stmt.query_map(params![service], |row| {
        Ok(HttpContract {
            service: row.get(0)?,
            description: row.get(1)?,
            spec_raw: row.get(2)?,
        })
    })?;
    Ok(rows.next().transpose()?)
}

pub fn list_http_contracts(conn: &Connection) -> Result<Vec<HttpContractSummary>, AppError> {
    let mut stmt =
        conn.prepare("SELECT service, description FROM http_contracts ORDER BY service")?;
    let rows = stmt.query_map([], |row| {
        Ok(HttpContractSummary {
            service: row.get(0)?,
            description: row.get(1)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

// ── Entity counts (for health) ──

pub fn entity_counts(conn: &Connection) -> Result<serde_json::Value, AppError> {
    let count = |table: &str| -> Result<i64, AppError> {
        Ok(conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))?)
    };
    Ok(serde_json::json!({
        "services": count("services")?,
        "tables": count("tables")?,
        "queues": count("queue_contracts")?,
        "protos": count("proto_contracts")?,
        "http_contracts": count("http_contracts")?,
    }))
}

pub fn embedding_counts(conn: &Connection) -> Result<serde_json::Value, AppError> {
    let mut stmt = conn.prepare(
        "SELECT entity_type, COUNT(*) FROM embeddings GROUP BY entity_type",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;
    let mut coverage = serde_json::Map::new();
    let mut total: i64 = 0;
    for row in rows {
        let (etype, cnt) = row?;
        total += cnt;
        coverage.insert(etype, serde_json::Value::from(cnt));
    }
    Ok(serde_json::json!({
        "total": total,
        "coverage": coverage,
    }))
}

// ── Embeddings ──

pub fn upsert_embedding(
    conn: &Connection,
    entity_type: &str,
    entity_key: &str,
    text_hash: &str,
    embedding: &[u8],
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO embeddings (entity_type, entity_key, text_hash, embedding)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(entity_type, entity_key) DO UPDATE SET text_hash=?3, embedding=?4",
        params![entity_type, entity_key, text_hash, embedding],
    )?;
    Ok(())
}

pub fn get_embedding_hash(
    conn: &Connection,
    entity_type: &str,
    entity_key: &str,
) -> Result<Option<String>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT text_hash FROM embeddings WHERE entity_type=?1 AND entity_key=?2",
    )?;
    let mut rows = stmt.query_map(params![entity_type, entity_key], |row| row.get(0))?;
    Ok(rows.next().transpose()?)
}

pub struct EmbeddingRow {
    pub entity_type: String,
    pub entity_key: String,
    pub embedding: Vec<u8>,
}

pub fn load_all_embeddings(conn: &Connection) -> Result<Vec<EmbeddingRow>, AppError> {
    let mut stmt =
        conn.prepare("SELECT entity_type, entity_key, embedding FROM embeddings")?;
    let rows = stmt.query_map([], |row| {
        Ok(EmbeddingRow {
            entity_type: row.get(0)?,
            entity_key: row.get(1)?,
            embedding: row.get(2)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema::migrate(&conn).unwrap();
        conn
    }

    fn sample_service() -> Service {
        Service {
            name: "test-svc".to_string(),
            description: "A test service".to_string(),
            github_repo: Some("https://github.com/org/test-svc".to_string()),
            grpc_servers: vec!["TestGrpcServer".to_string()],
            grpc_clients: vec!["OtherGrpcServer".to_string()],
            http_server: false,
            http_clients: vec![],
            queue_publishers: vec!["test-event".to_string()],
            queue_subscribers: vec![],
            tables: vec!["test_table".to_string()],
        }
    }

    // ── Service tests ──

    #[test]
    fn test_insert_and_get_service() {
        let conn = setup_db();
        let svc = sample_service();
        insert_service(&conn, &svc).unwrap();
        let got = get_service(&conn, "test-svc").unwrap().unwrap();
        assert_eq!(got, svc);
    }

    #[test]
    fn test_list_services_returns_summaries() {
        let conn = setup_db();
        for i in 0..3 {
            let mut svc = sample_service();
            svc.name = format!("svc-{i}");
            svc.description = format!("Service {i}");
            insert_service(&conn, &svc).unwrap();
        }
        let list = list_services(&conn).unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].name, "svc-0");
        assert_eq!(list[0].description, "Service 0");
    }

    #[test]
    fn test_update_service() {
        let conn = setup_db();
        let mut svc = sample_service();
        insert_service(&conn, &svc).unwrap();
        svc.description = "Updated description".to_string();
        update_service(&conn, "test-svc", &svc).unwrap();
        let got = get_service(&conn, "test-svc").unwrap().unwrap();
        assert_eq!(got.description, "Updated description");
    }

    #[test]
    fn test_delete_service() {
        let conn = setup_db();
        insert_service(&conn, &sample_service()).unwrap();
        delete_service(&conn, "test-svc").unwrap();
        assert!(get_service(&conn, "test-svc").unwrap().is_none());
    }

    #[test]
    fn test_insert_duplicate_service_fails() {
        let conn = setup_db();
        insert_service(&conn, &sample_service()).unwrap();
        let result = insert_service(&conn, &sample_service());
        assert!(result.is_err());
    }

    // ── Table tests ──

    fn sample_table() -> Table {
        Table {
            name: "users".to_string(),
            database: "users-db".to_string(),
            owner: "users-svc".to_string(),
            description: "User accounts".to_string(),
            ddl: "CREATE TABLE users (id UUID PRIMARY KEY);".to_string(),
        }
    }

    #[test]
    fn test_insert_and_get_table() {
        let conn = setup_db();
        let t = sample_table();
        insert_table(&conn, &t).unwrap();
        let got = get_table(&conn, "users").unwrap().unwrap();
        assert_eq!(got, t);
    }

    #[test]
    fn test_list_tables_returns_summaries() {
        let conn = setup_db();
        insert_table(&conn, &sample_table()).unwrap();
        let list = list_tables(&conn).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "users");
    }

    #[test]
    fn test_update_table() {
        let conn = setup_db();
        let mut t = sample_table();
        insert_table(&conn, &t).unwrap();
        t.description = "Updated".to_string();
        update_table(&conn, "users", &t).unwrap();
        let got = get_table(&conn, "users").unwrap().unwrap();
        assert_eq!(got.description, "Updated");
    }

    #[test]
    fn test_delete_table() {
        let conn = setup_db();
        insert_table(&conn, &sample_table()).unwrap();
        delete_table(&conn, "users").unwrap();
        assert!(get_table(&conn, "users").unwrap().is_none());
    }

    #[test]
    fn test_insert_duplicate_table_fails() {
        let conn = setup_db();
        insert_table(&conn, &sample_table()).unwrap();
        assert!(insert_table(&conn, &sample_table()).is_err());
    }

    // ── Queue tests ──

    fn sample_queue() -> QueueContract {
        QueueContract {
            name: "user-registered".to_string(),
            description: "User registration event".to_string(),
            schema: QueueSchema {
                name: "UserRegisteredEvent".to_string(),
                fields: vec![QueueField {
                    name: "user_id".to_string(),
                    field_type: "uuid".to_string(),
                    description: "Unique user ID".to_string(),
                }],
                notes: "First event".to_string(),
            },
        }
    }

    #[test]
    fn test_insert_and_get_queue() {
        let conn = setup_db();
        let q = sample_queue();
        insert_queue(&conn, &q).unwrap();
        let got = get_queue(&conn, "user-registered").unwrap().unwrap();
        assert_eq!(got, q);
    }

    #[test]
    fn test_list_queues_returns_summaries() {
        let conn = setup_db();
        insert_queue(&conn, &sample_queue()).unwrap();
        let list = list_queues(&conn).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "user-registered");
    }

    #[test]
    fn test_update_queue() {
        let conn = setup_db();
        let mut q = sample_queue();
        insert_queue(&conn, &q).unwrap();
        q.description = "Updated".to_string();
        update_queue(&conn, "user-registered", &q).unwrap();
        let got = get_queue(&conn, "user-registered").unwrap().unwrap();
        assert_eq!(got.description, "Updated");
    }

    #[test]
    fn test_delete_queue() {
        let conn = setup_db();
        insert_queue(&conn, &sample_queue()).unwrap();
        delete_queue(&conn, "user-registered").unwrap();
        assert!(get_queue(&conn, "user-registered").unwrap().is_none());
    }

    #[test]
    fn test_insert_duplicate_queue_fails() {
        let conn = setup_db();
        insert_queue(&conn, &sample_queue()).unwrap();
        assert!(insert_queue(&conn, &sample_queue()).is_err());
    }

    // ── Proto tests ──

    fn sample_proto() -> ProtoContract {
        ProtoContract {
            server: "TestGrpcService".to_string(),
            description: "Test gRPC service".to_string(),
            proto_raw: "syntax = \"proto3\";".to_string(),
        }
    }

    #[test]
    fn test_insert_and_get_proto() {
        let conn = setup_db();
        let p = sample_proto();
        insert_proto(&conn, &p).unwrap();
        let got = get_proto(&conn, "TestGrpcService").unwrap().unwrap();
        assert_eq!(got, p);
    }

    #[test]
    fn test_list_protos_returns_summaries() {
        let conn = setup_db();
        insert_proto(&conn, &sample_proto()).unwrap();
        let list = list_protos(&conn).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].server, "TestGrpcService");
    }

    #[test]
    fn test_update_proto() {
        let conn = setup_db();
        let mut p = sample_proto();
        insert_proto(&conn, &p).unwrap();
        p.description = "Updated".to_string();
        update_proto(&conn, "TestGrpcService", &p).unwrap();
        let got = get_proto(&conn, "TestGrpcService").unwrap().unwrap();
        assert_eq!(got.description, "Updated");
    }

    #[test]
    fn test_delete_proto() {
        let conn = setup_db();
        insert_proto(&conn, &sample_proto()).unwrap();
        delete_proto(&conn, "TestGrpcService").unwrap();
        assert!(get_proto(&conn, "TestGrpcService").unwrap().is_none());
    }

    #[test]
    fn test_insert_duplicate_proto_fails() {
        let conn = setup_db();
        insert_proto(&conn, &sample_proto()).unwrap();
        assert!(insert_proto(&conn, &sample_proto()).is_err());
    }

    // ── HTTP Contract tests ──

    fn sample_http_contract() -> HttpContract {
        HttpContract {
            service: "api-gateway".to_string(),
            description: "REST API gateway".to_string(),
            spec_raw: "openapi: 3.0.0".to_string(),
        }
    }

    #[test]
    fn test_insert_and_get_http_contract() {
        let conn = setup_db();
        let h = sample_http_contract();
        insert_http_contract(&conn, &h).unwrap();
        let got = get_http_contract(&conn, "api-gateway").unwrap().unwrap();
        assert_eq!(got, h);
    }

    #[test]
    fn test_list_http_contracts_returns_summaries() {
        let conn = setup_db();
        insert_http_contract(&conn, &sample_http_contract()).unwrap();
        let list = list_http_contracts(&conn).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].service, "api-gateway");
    }

    #[test]
    fn test_update_http_contract() {
        let conn = setup_db();
        let mut h = sample_http_contract();
        insert_http_contract(&conn, &h).unwrap();
        h.description = "Updated".to_string();
        update_http_contract(&conn, "api-gateway", &h).unwrap();
        let got = get_http_contract(&conn, "api-gateway").unwrap().unwrap();
        assert_eq!(got.description, "Updated");
    }

    #[test]
    fn test_delete_http_contract() {
        let conn = setup_db();
        insert_http_contract(&conn, &sample_http_contract()).unwrap();
        delete_http_contract(&conn, "api-gateway").unwrap();
        assert!(get_http_contract(&conn, "api-gateway").unwrap().is_none());
    }

    #[test]
    fn test_insert_duplicate_http_contract_fails() {
        let conn = setup_db();
        insert_http_contract(&conn, &sample_http_contract()).unwrap();
        assert!(insert_http_contract(&conn, &sample_http_contract()).is_err());
    }
}
