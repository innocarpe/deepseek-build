# GitHub Actions

**No product CI is defined yet.**

Early scaffolding accidentally used Actions to police PR titles/labels and
markdown path inventories. That is **not** product development CI and was
removed.

## When CI belongs here

Add workflows only when they verify **shipped product behavior**, for example:

| Milestone-ish | Plausible jobs |
|---------------|----------------|
| M1+ | Build binary / package; provider client unit tests |
| M2+ | Tool runtime tests; golden prefix-hash tests |
| M3+ | Skills discovery tests |
| M6 | Install smoke, release artifact checks |

Process quality (PR narrative, kind labels, Orca-level body) is enforced by:

- `docs/contributing/*` (normative human process)
- root `AGENTS.md` + `skills/pr-authoring/` (agent harness)
- Review / self-merge checklist

—not by green-check theater on empty product surface.
