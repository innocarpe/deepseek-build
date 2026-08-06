# Research: Reasonix

**Local path:** `OpenSources/DeepSeek-Reasonix`  
**Upstream:** https://github.com/esengine/DeepSeek-Reasonix  

**Binding product extraction:** [HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md) §5

---

## Why it matters

DeepSeek-native **cost and cache** culture. Long sessions stay cheap when the system/tool/memory **prefix is byte-stable**. Complements Deep Code’s cache-aware session layout with an even stronger “cache-first invariant” engineering culture.

## Takeaways

| Theme | Detail |
|-------|--------|
| Cache-first | System+tools+memory prefix byte-stable; dynamics on turn tail |
| Flash-first | Default cheap model; Pro escalate |
| Tool-call repair | Schema-aware repair before dispatch |
| Config-driven harness | Providers/tools as config (we take the mindset, not full desktop) |
| Single binary | Go reference for distribution simplicity |

## Promote to product

- Spec `10` cache contract  
- Spec `15` tool-call repair  
- Spec `20` model routing  

## Leave for later

Desktop app, full plugin/extension protocol surface.
