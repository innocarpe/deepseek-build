# Spec 20 — Model routing (Flash / Pro)

| Field | Value |
|-------|--------|
| Status | **ready-for-impl** |
| Philosophy | HARNESS §5, §7; Reasonix Flash-first |
| Gate | Part of **G2** |
| Tests | Automated routing table tests required |

## 1. Pinned wire IDs (ADR 0005)

| Tier | Wire model |
|------|------------|
| Flash | `deepseek-v4-flash` |
| Pro | `deepseek-v4-pro` |

## 2. Behavior

### 2.1 Defaults

| Session preset | Model | Default effort (spec 30) |
|----------------|-------|---------------------------|
| `flash` (product default) | Flash | `low` or `high` (product default **`high`** for coding quality; user can lower) |
| `balanced` | Flash for tools; Pro for escalations | Flash `high`, Pro `high` |
| `max` | Pro | `max` |

M1 minimum: implement **session default Flash** + **explicit escalate to Pro**.

### 2.2 Escalation triggers (product)

| Trigger | Result |
|---------|--------|
| User `/pro` or “use pro for this turn” | Next turn Pro, then return to default unless sticky preset |
| User `/preset max` | Session sticky Pro |
| User `/preset flash` | Session sticky Flash |
| Automatic router (optional M1) | May escalate **one turn** to Pro when: plan/architecture keywords **and** user did not force flash; must log `escalate_reason` |

**Precedence:** explicit user model/preset **>** sticky session preset **>** automatic router **>** default Flash.

### 2.3 Visibility

- Every turn log/UI must show **which wire model** ran.  
- Never silently stay on Pro after one-shot `/pro` unless preset is `max`.

### 2.4 Unavailable model

If Pro returns 404/unsupported: fall back Flash + user-visible warning; do not crash loop.

### 2.5 Cost

Do not auto-loop Pro on every tool turn. Subagents (M4) default Flash (HARNESS worker law).

## 3. Non-goals

- Multi-provider routing  
- Responses API as primary (Flash-only; ADR 0005)  

## 4. Test plan (automated)

| Test | Expect |
|------|--------|
| `default_is_flash` | builder selects `deepseek-v4-flash` |
| `pro_oneshot_then_flash` | after one-shot, next is flash |
| `preset_max_sticky` | remains pro |
| `user_beats_router` | forced flash not overridden |
| `pro_unavailable_fallback` | flash + warning flag |

## 5. Implementation notes

- Config: `~/.deepseek-build/config.toml` keys `default_preset`, `default_model`.  
