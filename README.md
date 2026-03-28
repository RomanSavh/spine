# Spine

System Knowledge Registry for AI Agents.

Spine stores structured knowledge about your microservice architecture — services, database schemas, queue contracts, gRPC/HTTP APIs — and exposes it via REST API and MCP. AI agents query Spine to understand the system before making decisions.

## Features

- **REST API** with full CRUD for services, tables, queues, proto contracts, HTTP contracts
- **MCP server** (SSE/HTTP) with 12 tools for AI agent consumption
- **Semantic search** via embeddings (nomic-embed-text-v1.5)
- **Dependency graph** derived from service relationships
- **Context endpoint** — get everything about a service in one call
- **Swagger UI** at `/swagger-ui/`
- **SQLite** storage

## Quick Start

### Docker Compose

```bash
docker compose up
```

This starts two containers:
- `spine` (Rust) — API + MCP on port 3000
- `spine-embed` (Python) — embedding service

The embedding model downloads on first startup (~500MB) and is cached in a Docker volume.

### From Source

```bash
# Start the embedding service
cd embed-service
pip install -r requirements.txt
uvicorn main:app --port 8100 &

# Start Spine
cargo run
```

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `SPINE_PORT` | `3000` | HTTP server port |
| `SPINE_DB_PATH` | `spine.db` | SQLite database path |
| `SPINE_EMBED_URL` | `http://localhost:8100` | Embedding service URL |

## API

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/services` | List services |
| GET/POST | `/services/{name}` | Get/Create/Update/Delete service |
| GET | `/tables` | List tables |
| GET/POST | `/tables/{name}` | Get/Create/Update/Delete table |
| GET | `/queues` | List queue contracts |
| GET/POST | `/queues/{name}` | Get/Create/Update/Delete queue |
| GET | `/protos/{server}` | Get proto contract |
| GET | `/http-contracts/{service}` | Get HTTP contract |
| GET | `/graph/dependencies/{service}` | Service dependency graph |
| GET | `/search?q=...&type=...` | Semantic search |
| GET | `/context/{service}` | Bundled service context |
| GET | `/health` | Health check |

Full API docs at `/swagger-ui/`.

### MCP

MCP endpoint at `/mcp` (Streamable HTTP / SSE transport).

**Read tools:** `list_services`, `get_service`, `list_tables`, `get_table`, `list_queues`, `get_queue`, `get_proto`, `get_http_contract`, `get_context`, `get_dependencies`, `search`.

**Write tools:** `register_service`, `update_service`, `delete_service`, `register_table`, `update_table`, `delete_table`, `register_queue`, `update_queue`, `delete_queue`, `register_proto`, `update_proto`, `delete_proto`, `register_http_contract`, `update_http_contract`, `delete_http_contract`.

See [MCP.md](MCP.md) for full specification.

Connect from any MCP client:

```json
{
  "mcpServers": {
    "spine": {
      "url": "http://localhost:3000/mcp"
    }
  }
}
```

## Example

```bash
# Create a service
curl -X POST http://localhost:3000/services \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "users-grpc",
    "description": "User management service",
    "grpc_servers": ["UsersGrpcService"],
    "queue_publishers": ["user-registered"],
    "tables": ["users"]
  }'

# Create a table
curl -X POST http://localhost:3000/tables \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "users",
    "database": "users-db",
    "owner": "users-grpc",
    "description": "User accounts",
    "ddl": "CREATE TABLE users (id UUID PRIMARY KEY, email TEXT NOT NULL);"
  }'

# Get full context
curl http://localhost:3000/context/users-grpc

# Search
curl "http://localhost:3000/search?q=user+registration"

# Dependency graph
curl http://localhost:3000/graph/dependencies/users-grpc
```

## Docker Images

Published to GitHub Container Registry on each release:

```
ghcr.io/romansavh/spine:latest
ghcr.io/romansavh/spine-embed:latest
```
