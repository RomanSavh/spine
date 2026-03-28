# Spine MCP Specification

## Overview

Spine exposes an MCP (Model Context Protocol) server over Streamable HTTP (SSE) transport at `/mcp`. AI agents use it to query and register system architecture knowledge — services, database schemas, queue contracts, gRPC/HTTP APIs, and service dependencies.

**Endpoint:** `http://<host>:<port>/mcp`
**Transport:** Streamable HTTP (Server-Sent Events)
**Protocol:** MCP 2025-03-26

## Connection

```json
{
  "mcpServers": {
    "spine": {
      "url": "http://localhost:3000/mcp"
    }
  }
}
```

## Server Info

```json
{
  "name": "rmcp",
  "version": "1.3.0",
  "capabilities": { "tools": {} },
  "instructions": "Spine is a system knowledge registry. Use it to understand and register microservice architecture, database schemas, queue contracts, gRPC/HTTP APIs, and service dependencies."
}
```

## Tools

### list_services

List all registered services (names and descriptions).

**Parameters:** none

**Returns:**
```json
[
  { "name": "users-grpc", "description": "User management service." },
  { "name": "contest-engine-grpc", "description": "Core contest orchestration engine." }
]
```

---

### get_service

Get full details of a service by name, including gRPC servers/clients, queue publishers/subscribers, and owned tables.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | yes | Service name |

**Returns:**
```json
{
  "name": "contest-engine-grpc",
  "description": "Core contest orchestration engine.",
  "github_repo": "https://github.com/org/contest-engine-grpc",
  "grpc_servers": ["ContestEngineGrpcService"],
  "grpc_clients": ["ClientWalletsGrpcService", "UsersGrpcService"],
  "http_server": false,
  "http_clients": [],
  "queue_publishers": ["contest-registration-update"],
  "queue_subscribers": ["contest-accounts-updates"],
  "tables": ["contests", "contest_participants"]
}
```

---

### list_tables

List all registered database tables (names and descriptions).

**Parameters:** none

**Returns:**
```json
[
  { "name": "contests", "description": "Active and historical contests" },
  { "name": "users", "description": "User accounts" }
]
```

---

### get_table

Get full details of a database table by name, including DDL schema.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | yes | Table name |

**Returns:**
```json
{
  "name": "contests",
  "database": "contests-db",
  "owner": "contest-engine-grpc",
  "description": "Active and historical contests",
  "ddl": "CREATE TABLE contests (\n  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),\n  title VARCHAR(255) NOT NULL,\n  status VARCHAR(50) NOT NULL DEFAULT 'draft',\n  starts_at TIMESTAMP NOT NULL,\n  created_at TIMESTAMP NOT NULL DEFAULT now()\n);"
}
```

---

### list_queues

List all registered queue contracts (names and descriptions).

**Parameters:** none

**Returns:**
```json
[
  { "name": "user-registered", "description": "Emitted when a new user completes registration." }
]
```

---

### get_queue

Get full details of a queue contract by name, including message schema.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | yes | Queue name |

**Returns:**
```json
{
  "name": "user-registered",
  "description": "Emitted when a new user completes registration.",
  "schema": {
    "name": "UserRegisteredEvent",
    "fields": [
      { "name": "user_id", "type": "uuid", "description": "Unique user ID." },
      { "name": "email", "type": "string", "description": "" },
      { "name": "registered_at", "type": "datetime", "description": "" }
    ],
    "notes": "First event in user lifecycle."
  }
}
```

---

### get_proto

Get proto contract by gRPC server name, including raw .proto definition.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `server` | string | yes | gRPC server name |

**Returns:**
```json
{
  "server": "ContestEngineGrpcService",
  "description": "Contest lifecycle gRPC service",
  "proto_raw": "syntax = \"proto3\";\npackage contest;\n\nservice ContestEngineGrpcService {\n  rpc CreateContest (CreateContestRequest) returns (Contest);\n}"
}
```

---

### get_http_contract

Get HTTP contract by service name, including OpenAPI spec.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `service` | string | yes | Service name |

**Returns:**
```json
{
  "service": "api-gateway",
  "description": "Public-facing REST API gateway",
  "spec_raw": "openapi: \"3.0.0\"\ninfo:\n  title: API Gateway\n  version: \"1.0\"\npaths: ..."
}
```

---

### get_context

Get the full context for a service: the service itself plus all its tables, queue contracts, proto contracts, and HTTP contracts.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `service` | string | yes | Service name |

**Returns:**
```json
{
  "service": { "name": "contest-engine-grpc", "..." : "..." },
  "tables": [
    { "name": "contests", "database": "contests-db", "ddl": "..." }
  ],
  "queue_contracts": [
    { "name": "contest-registration-update", "schema": { "..." : "..." } }
  ],
  "proto_contracts": [
    { "server": "ContestEngineGrpcService", "proto_raw": "..." }
  ],
  "http_contracts": []
}
```

---

### get_dependencies

Get the dependency graph for a service: which services it depends on and which depend on it, via gRPC, HTTP, or queue relationships.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `service` | string | yes | Service name |

**Returns:**
```json
{
  "service": "contest-engine-grpc",
  "depends_on": [
    { "service": "users-grpc", "relation": "grpc", "via": "UsersGrpcService" },
    { "service": "wallets-grpc", "relation": "grpc", "via": "ClientWalletsGrpcService" }
  ],
  "depended_by": [
    { "service": "api-gateway", "relation": "grpc", "via": "ContestEngineGrpcService" }
  ]
}
```

**Dependency derivation rules:**
- `grpc`: service A lists X in `grpc_clients`, service B lists X in `grpc_servers` → A depends on B
- `http`: service A lists B in `http_clients`, B has `http_server: true` → A depends on B
- `queue`: service A lists Q in `queue_subscribers`, service B lists Q in `queue_publishers` → A depends on B

---

### search

Semantic search across all entities (services, tables, queues, protos, HTTP contracts) using natural language. Returns results ranked by cosine similarity.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `query` | string | yes | Natural language search query |
| `entity_type` | string | no | Filter: `service`, `table`, `queue`, `proto`, `http` |
| `limit` | integer | no | Max results (default 10) |

**Returns:**
```json
[
  { "type": "service", "name": "users-grpc", "score": 0.94 },
  { "type": "queue", "name": "user-registered", "score": 0.91 },
  { "type": "table", "name": "users", "score": 0.87 }
]
```

**Note:** Requires the embedding service (`spine-embed`) to be running. Returns an error if unavailable.

## Write Tools

All write tools automatically update embeddings for semantic search.

---

### register_service

Register a new service. Provide name, description, and relationship fields.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | yes | Unique service name |
| `description` | string | yes | What the service does |
| `github_repo` | string | no | Link to source repository |
| `grpc_servers` | string[] | no | gRPC servers this service implements |
| `grpc_clients` | string[] | no | gRPC servers this service calls |
| `http_server` | boolean | no | Whether it exposes an HTTP API |
| `http_clients` | string[] | no | Services it makes HTTP calls to |
| `queue_publishers` | string[] | no | Queues/topics this service publishes to |
| `queue_subscribers` | string[] | no | Queues/topics this service subscribes to |
| `tables` | string[] | no | Database tables this service owns |

**Returns:** Confirmation message + full service JSON.

---

### update_service

Update an existing service. All fields are replaced.

**Parameters:** Same as `register_service`.

---

### delete_service

Delete a service by name.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | yes | Service name |

**Returns:** Confirmation message.

---

### register_table

Register a new database table with its DDL schema.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | yes | Table name |
| `database` | string | yes | Which database it belongs to |
| `owner` | string | yes | Which service owns this table |
| `description` | string | yes | What data it holds |
| `ddl` | string | yes | Raw PostgreSQL CREATE TABLE statement |

**Returns:** Confirmation message + full table JSON.

---

### update_table

Update an existing table. All fields are replaced.

**Parameters:** Same as `register_table`.

---

### delete_table

Delete a table by name.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | yes | Table name |

---

### register_queue

Register a new queue contract with its message schema.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | yes | Queue/topic name |
| `description` | string | yes | What events flow through it |
| `schema` | object | yes | Message schema (see below) |

**`schema` object:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | yes | Schema/event type name |
| `fields` | array | yes | Typed fields: `{name, type, description}` |
| `notes` | string | no | Additional notes |

**Returns:** Confirmation message + full queue contract JSON.

---

### update_queue

Update an existing queue contract. All fields are replaced.

**Parameters:** Same as `register_queue`.

---

### delete_queue

Delete a queue contract by name.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | yes | Queue name |

---

### register_proto

Register a new proto contract with raw .proto content.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `server` | string | yes | gRPC server name |
| `description` | string | yes | What the service does |
| `proto_raw` | string | yes | Raw .proto file content |

**Returns:** Confirmation message + full proto contract JSON.

---

### update_proto

Update an existing proto contract. All fields are replaced.

**Parameters:** Same as `register_proto`.

---

### delete_proto

Delete a proto contract by gRPC server name.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `server` | string | yes | gRPC server name |

---

### register_http_contract

Register a new HTTP contract with OpenAPI/Swagger spec.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `service` | string | yes | Service name |
| `description` | string | yes | What the API does |
| `spec_raw` | string | yes | Raw OpenAPI/Swagger spec content |

**Returns:** Confirmation message + full HTTP contract JSON.

---

### update_http_contract

Update an existing HTTP contract. All fields are replaced.

**Parameters:** Same as `register_http_contract`.

---

### delete_http_contract

Delete an HTTP contract by service name.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `service` | string | yes | Service name |

---

## Entity Types

| Type | Primary Key | Description |
|------|-------------|-------------|
| `service` | `name` | Microservice with gRPC/HTTP/queue relationships |
| `table` | `name` | PostgreSQL table with DDL schema |
| `queue` | `name` | Queue/topic with typed message schema |
| `proto` | `server` | gRPC proto definition (raw .proto content) |
| `http` | `service` | HTTP/OpenAPI contract (raw spec content) |

## Error Handling

Tools return errors in two ways:

**Not found** — tool succeeds but returns `is_error: true`:
```json
{
  "content": [{ "type": "text", "text": "Service 'nonexistent' not found" }],
  "isError": true
}
```

**Internal error** — MCP error response:
```json
{
  "code": -32603,
  "message": "database error: ..."
}
```
