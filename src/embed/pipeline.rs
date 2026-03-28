use sha2::{Digest, Sha256};

use crate::models::*;

pub fn embed_text_for_service(s: &Service) -> String {
    format!("{}: {}", s.name, s.description)
}

pub fn embed_text_for_table(t: &Table) -> String {
    format!("{} ({}): {}\n{}", t.name, t.database, t.description, t.ddl)
}

pub fn embed_text_for_queue(q: &QueueContract) -> String {
    let field_names: Vec<&str> = q.schema.fields.iter().map(|f| f.name.as_str()).collect();
    format!(
        "{}: {}. Fields: {}",
        q.name,
        q.description,
        field_names.join(", ")
    )
}

pub fn embed_text_for_proto(p: &ProtoContract) -> String {
    format!("{}: {}\n{}", p.server, p.description, p.proto_raw)
}

pub fn embed_text_for_http(h: &HttpContract) -> String {
    let spec = if h.spec_raw.len() > 4000 {
        &h.spec_raw[..4000]
    } else {
        &h.spec_raw
    };
    format!("{}: {}\n{}", h.service, h.description, spec)
}

pub fn text_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}
