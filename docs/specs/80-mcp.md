# Spec 80 — MCP (Model Context Protocol) surface

| Field | Value |
|-------|--------|
| Status | **ready-for-impl** |
| Philosophy | HARNESS §4 tools surface; L1 Deep Code; L2 cache epoch on schema change |
| Gate | **G6c** |
| Tests | **Automated required:** catalog fingerprint stability, epoch change on schema mutate, name safety |

## 1. Behavior

### 1.1 Role of MCP

MCP mounts **dynamic** tools from configured servers. Built-ins (spec 40) stay small; MCP must not dump unbounded schemas into the stable prefix without an explicit epoch.

### 1.2 Configuration (minimum)

Servers are declared in product config (paths product-specific):

```json
{
  "servers": [
    {
      "name": "example",
      "transport": "stdio",
      "command": "npx",
      "args": ["-y", "some-mcp-server"],
      "env": {}
    }
  ]
}
```

Search order for config files (first found wins for v1, or merge by server name — document choice):

1. `{workspace}/.deepseek-build/mcp.json`
2. `~/.deepseek-build/mcp.json`

**Server `name`:** `[a-z0-9_-]+` only.

### 1.3 Catalog + wire names

Each remote tool is exposed to the model as:

```text
mcp__{server}__{tool}
```

- `tool` is the MCP tool name sanitized to `[A-Za-z0-9_-]+`.  
- Unknown / unsafe characters → skip tool with audit warning (do not invent).

Catalog entry fields (internal):

| Field | Meaning |
|-------|---------|
| `wire_name` | `mcp__…` name shown to the model |
| `server` | Config server name |
| `remote_name` | Original MCP tool name |
| `description` | Short description |
| `input_schema` | JSON Schema object (or empty object) |

### 1.4 Cache epoch (normative)

When the **MCP tool schema document** changes (any add/remove/rename/schema hash change):

1. Recompute `mcp_schema_fingerprint` = SHA-256 of canonical JSON of the catalog (sorted by `wire_name`, sorted object keys).  
2. The stable prefix **must** include this fingerprint (or the full schema document) so the **prefix epoch** changes (spec 10).  
3. Mid-session hot-reload of MCP schemas is allowed **only** if it starts a **new** prefix epoch (explicit; rare). Default product: load catalog at process start.

### 1.5 Invocation

- Model calls tool by `wire_name`.  
- Runtime maps to server + remote tool, applies permission policy (`unknown` / future `mcp` scope → ask/deny per spec 90).  
- v1 may implement an **in-process catalog** for tests and a **stdio client** path when command is configured; fail closed if server unavailable.

### 1.6 Permissions

MCP tools are **not** YOLO. Minimum:

- Treat as requiring confirmation when headless unless pre-granted.  
- Out-of-process side effects remain the server’s responsibility; product still logs decision audit.

## 2. Non-goals (v1)

- Full MCP resources/prompts marketplace  
- Multi-hop OAuth MCP auth UX  
- Parallel MCP fan-out (Wave C / G4)  

## 3. Test plan

| ID | Case | Expect |
|----|------|--------|
| T1 | Fingerprint stable | same catalog → same hex |
| T2 | Schema change | add tool → fingerprint differs |
| T3 | Wire name format | `mcp__srv__tool` only for valid names |
| T4 | Invalid server name rejected | config error |
| T5 | Epoch integrates | prefix epoch changes when MCP catalog included and mutated |

## 4. Implementation map

| Area | Location |
|------|----------|
| Catalog + fingerprint | `crates/dsb-tools` / `mcp` module |
| Config load | same |
| Agent mount | `dsb-agent` tool definitions merge |
| Gate | `docs/GATES.md` **G6c** |

## 5. Ready-for-impl checklist

- [x] Config shape  
- [x] Wire naming  
- [x] Epoch / fingerprint rules  
- [x] Permission posture  
- [x] Automated tests listed  

**Status:** **ready-for-impl**.
