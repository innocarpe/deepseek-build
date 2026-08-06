## Verdict

**FAIL** — Codex P0 중 문서 모순형(P0-6 repair M1/M6, P0-2 milestone 순서)은 실제로 닫혔지만, 제품의 존재 여부를 결정하는 두 축(provider/model 계약, executable spec 산출물)은 **한 글자도 산출물이 없다**: `docs/specs/`는 여전히 `00-overview.md` 단 하나, toolchain ADR 없음, provider contract 없음, 그리고 이것을 강제한다는 게이트 체계는 CI를 의도적으로 삭제한 뒤 **원장도 검증자도 없는 자기신고(self-attestation)**로만 남았다.

## Codex P0 status

| # | Codex P0 | 상태 | 비고 |
|---|---|---|---|
| 1 | "specs before code" 게이트가 실체 없음 | **부분 수정** | 규칙은 생겼다(`HARNESS_PHILOSOPHY.md:270-286`, `PRD-v1.md:230-246`). 그러나 (a) G0는 이미 green인 no-op, (b) 게이트 상태를 기록하는 파일이 없음, (c) 검증자는 단독 메인테이너의 self-merge 체크리스트, (d) `8dd1f40`에서 CI를 삭제하고 `AGENTS.md:47`이 "process-police CI" 재도입을 금지 — 강제 수단이 0 |
| 2 | Grok이 실행 순서를 지배 | **수정됨** | `MILESTONES.md:9-14` invariant gates, M2 순서 `45 → 90min → 40 → 50`, "Failure if" 행, G3/G4. 잔여: `MILESTONES.md:12`의 `"(when edit exists)"` 조건절이 invariant를 스스로 무력화 |
| 3 | edit 계약에 우회로 존재 | **부분 수정** | Bypass law 표 추가(`HARNESS_PHILOSOPHY.md:96-100`). 그러나 `write`의 "explicit flag" 주체 미정, `bash` "ask (or deny)"는 정책이 아니라 정책 두 개, allowlist 미정의, **shell이 파일을 바꿨을 때 outstanding snippet 무효화 규칙 전무**, binary/generated 파일 거동 여전히 없음(Codex가 명시 요구) |
| 4 | byte-stable cache가 구현 불가 계약 | **부분 수정** | `HARNESS_PHILOSOPHY.md:139-144`가 canonical serialization·epoch·golden fixture·"intent to reuse is not acceptance"를 명문화. 그러나 **탈출구 3개 생존**: `PRD-v1.md:37` "(or proxy)", `PRD-v1.md:110` "when API reports it", `MILESTONES.md:60` "telemetry **or** golden prefix equality" |
| 5 | Reasonix 경제학이 슬로건 | **미수정** | escalation signal, budget, user-choice vs router 우선순위, price source, cache billing 가정 전부 부재. `deepseek-v4-flash`/`deepseek-v4-pro`는 Codex가 "pin하라"고 지목했음에도 `PRD-v1.md:180`·`VISION.md:28-29`에 사실인 양 그대로 |
| 6 | repair가 M1이자 M6 | **수정됨** | `PRD-v1.md:91` "M1 (must)", M6 should-have 목록에서 제거. 모순 해소 확인 |
| 7 | provider 거동이 계약에 없음 | **미수정(게이트로만 이관)** | 문서 자체가 없음. `PRD-v1.md:244` 체크박스와 `MILESTONES.md:54` work item으로 "쓰겠다"는 약속만 추가됨. Codex의 요구는 "쓰라"였지 "쓰기로 하라"가 아니었음 |

## Critical (P0)

1. **모델 식별자가 여전히 검증되지 않은 가정** — `docs/product/PRD-v1.md:180`, `docs/product/VISION.md:28-29`, `HARNESS_PHILOSOPHY.md:218-222`.
   `deepseek-v4-flash` / `deepseek-v4-pro`가 실재하는 API model id인지, Flash/Pro라는 티어 구분이 존재하는지에 대한 근거가 어디에도 없다. Flash-first 경제학, G1 목표, M1 exit criteria, §7 라우팅 표가 전부 이 두 문자열 위에 서 있다. **조치:** provider contract에 실측한 `/models` 응답을 인용해 model id를 pin하고, 티어가 다른 이름/구조면 §7과 spec 20을 먼저 고친다. 그 전까지 문서 전반에서 두 id를 `TBD(provider contract)`로 표기.

2. **Provider contract 문서 부재** — `docs/architecture/` 또는 `docs/adr/`에 파일 없음.
   streaming event 형태, thinking/effort 필드명, `usage`의 cache hit/miss 필드, cancellation, retry, rate limit, error taxonomy가 미상인 상태에서 spec 10(캐시 증거)·15(repair)·30(effort)이 "ready-for-impl"이 될 수 있는 경로가 없다. **조치:** `docs/adr/0004-deepseek-provider-contract.md`를 G2보다 **먼저** 머지. G2의 전제조건으로 명시.

3. **게이트에 원장과 검증자가 없다** — `HARNESS_PHILOSOPHY.md:270-286`, `AGENTS.md:47`.
   G0–G6가 지금 green인지 red인지 기록되는 곳이 없고, 누가 뒤집는지도 없다. §12는 PR 작성자에게 "어느 게이트를 green이라 가정하느냐"를 **자기신고**시킨다. CI는 `8dd1f40`에서 삭제됐고 재도입은 금지되어 있다. 반증: G1은 "`crates/` 스캐폴딩 전 필수"인데 `crates/`는 ADR 없이 이미 존재한다 — 게이트는 사후에 쓰였고 아무도 감사하지 않는다. **조치:** `docs/GATES.md` 상태 원장(게이트 · 상태 · 근거 커밋/PR · 뒤집은 사람) 추가. "process-police CI 금지"는 title lint 금지이지 **artifact 존재 검사 금지**가 아님을 `AGENTS.md`에 분리 명시하고, spec 파일 존재·status 행 검사만 하는 최소 job은 허용.

4. **"ready-for-impl" 정의의 `(golden or manual)` 탈출구** — `HARNESS_PHILOSOPHY.md:284`.
   spec 10의 존재 이유가 golden byte fixture인데, "test plan: 프리픽스를 수동으로 눈으로 확인"이 문자 그대로 G2를 통과시킨다. spec 45(stale snippet race)와 90(permission bypass)은 수동 검증이 원리적으로 불가능하다. **조치:** spec **10 / 15 / 45 / 50 / 90**은 자동화된 golden + negative 테스트를 필수로 지정하고, `manual` 허용은 UX 계열(30 UX, 110)로 한정.

5. **캐시 증거의 탈출구 3개가 philosophy와 정면충돌** — `PRD-v1.md:37` `(or proxy)`, `PRD-v1.md:110` `when API reports it`, `PRD-v1.md:150` `hit proxy`, `MILESTONES.md:60` `telemetry **or** golden prefix equality`.
   `HARNESS_PHILOSOPHY.md:144`는 "'intent to reuse' is not acceptance"라고 못박았다. 지금 상태로는 golden byte 동등성만 보이고 provider가 캐시를 실제로 재사용했는지는 한 번도 확인하지 않은 채 M1 exit이 가능하다. byte-stability는 캐시 히트의 **필요조건**이지 충분조건이 아니며(최소 토큰 임계·블록 granularity·서버측 정책), 어느 문서도 이 구분을 하지 않는다. **조치:** PRD·MILESTONES에서 `or proxy` / `or golden prefix equality`를 삭제하고 `AND`로 바꾼다. provider가 캐시 필드를 보고하지 않는다면 그 사실 자체를 provider contract에 기록하고 대체 증거(동일 프롬프트 2회 latency/과금 델타)를 정의한다.

6. **snippet 상태 × L3 병렬성의 일관성 소유자가 없다** *(신규)* — `HARNESS_PHILOSOPHY.md:80-88` vs `:197-212`, spec 50/60.
   snippet은 `(path, range, version)`의 **session-local** 상태다. spec 50은 한 턴에 다중 edit을, spec 60은 자체 컨텍스트를 가진 write worker를 허용한다. 미정의: snippet store가 세션 단위인가 에이전트 단위인가 / worker가 F를 수정하면 부모의 F snippet은 언제 어떻게 무효화되는가 / worktree 격리(M4) 하에서 snippet의 path는 어느 트리 기준인가. §3 충돌표는 edit **스키마** vs 속도만 다루고 **상태 일관성**은 다루지 않는다 — L1×L3 충돌인데 표에 행이 없다. **조치:** §3에 "snippet 상태 일관성" 행 추가(L1 승), spec 45가 소유권 모델을, spec 60이 전파/무효화를 정의.

7. **shell이 파일을 바꿨을 때 snippet 무효화 규칙이 없다** — `HARNESS_PHILOSOPHY.md:100`.
   Bypass law는 bash 파일 변경의 **허가**만 다루고 **결과**를 다루지 않는다. `sed -i`, `prettier --write`, `git checkout`, `npm install`, 빌드 산출물 — 전부 outstanding snippet의 version을 무효화한다. 무효화가 없으면 pillar A의 version check가 조용히 거짓 통과한다. **조치:** spec 45에 "bash 실행 후 워크스페이스 mtime/hash 재검증 → 영향받은 snippet 강제 만료" 규칙과, 만료 시 모델에게 반환할 오류 형태를 명시.

8. **모델의 side-effect 자기신고가 authoritative인지 advisory인지 미명시** — `HARNESS_PHILOSOPHY.md:171`.
   "bash requires the model to **declare** side effects; policy decides"는 정책의 입력이 모델 출력이라는 뜻이다. 적대적 상황이 아니어도 `find … -delete`가 서브셸/파이프 안에 있으면 오신고는 정상 실패 모드다. 신고가 유일한 입력이면 policy는 장식이다. **조치:** §4.1/§4.4에 "declaration은 advisory이며, 독립적인 명령 정적 분류기가 authoritative. 둘이 불일치하면 더 위험한 쪽으로 fail-close"를 명문화하고 spec 90의 필수 요소로 지정.

9. **`write`의 "explicit flag" 주체가 미정** — `HARNESS_PHILOSOPHY.md:99`.
   overwrite flag를 모델이 세팅할 수 있으면 pillar A는 한 필드로 무력화된다. **조치:** "flag는 사용자/정책만 설정 가능하며 모델 tool argument로 노출되지 않는다"를 Bypass law 표에 못박거나, 반대로 모델 노출을 허용하되 기존 파일에 대해서는 snippet version 검사를 동일하게 강제한다고 명시.

## Major (P1)

- **`/resume`·`/fork`(spec 100, M5) vs session-local snippet** — `HARNESS_PHILOSOPHY.md:82`, `:127-128`. 몇 시간 뒤 resume 시 snippet_id가 유효한가? fork하면 두 세션이 같은 경로에 대해 version 카운터가 갈라진다. 정의가 없으면 (a) 전부 만료시켜 재읽기 비용/캐시 손실을 감수하거나 (b) stale을 조용히 허용해 pillar A를 포기하거나 둘 중 하나로 귀결된다. spec 100 착수 **전에** spec 45가 이를 답해야 한다.

- **에이전트가 자기 stable prefix의 입력 파일을 편집할 수 있다** — prefix에 `AGENTS.md`/project standing instructions와 skills index가 들어가는데(`HARNESS_PHILOSOPHY.md:118`, `:156`), M2 dogfood 항목이 "이 저장소를 에이전트로 수정"이다. 자기 참조 무효화 규칙이 없다. spec 120은 소유자가 `All`인데(`00-overview.md:27`, `HARNESS_PHILOSOPHY.md:266`) **소유자 "All"은 소유자 없음**이다.

- **MCP/skills 동적 마운트 → epoch 정책 부재** — `HARNESS_PHILOSOPHY.md:92`(dynamic MCP) vs `:130`(mid-session 스키마 재작성 금지). `/mcp` 연결이나 디스크 상의 skill 추가가 stable prefix를 바꾼다. 미정의: epoch bump가 사용자에게 보이는가, 비용이 추정되는가, 턴 중간에 금지되는가, skills hot-reload는 아예 §4.3에 언급조차 없다. 게다가 이 기능(MCP)은 M5로 가장 늦게 오므로 M1의 invariant에 역압력이 걸리지 않는다.

- **병렬 실행 × permission "ask"의 승인 세만틱 부재** — §3 충돌표는 "side-effect declaration still required"로 답했는데, 어려운 부분은 declaration이 아니라 **approval**이다. N개 병렬 bash가 각각 ask면 프롬프트가 직렬화되는가, 배치되는가, 하나의 거부가 이미 실행된 형제 호출을 어떻게 처리하는가. 표의 그 행은 해결의 외관만 만든다.

- **제품 논지의 반증 장치가 M6 "should have"의 "expansion"** — `HARNESS_PHILOSOPHY.md:34`가 "quality per dollar"와 "wall-clock"에서 이긴다고 주장하는데, 정량 벤치는 M6+(`PRD-v1.md:196`)이고 목록상 "Adversarial acceptance suite **expansion**"(`PRD-v1.md:115`) — 존재하지 않는 suite의 확장이다. task corpus·baseline·cost model·latency 정의가 없고, "Grok Build 대비 wall-clock"은 다른 모델·다른 가격의 비통제 비교다. 현 상태의 논지는 반증 불가능하다.

- **"minimum" vs "full" permissions 구분이 정의되지 않음** — `PRD-v1.md:98,99`, `MILESTONES.md:80`. M2 "minimum"이 "path scopes + bash side-effect declare + ask/deny"인데 이건 사실상 전부다. 그러면 M3 "full polish"는 무엇인가. 구분이 없으므로 M2는 무엇을 만들든 minimum이라 부를 수 있다.

- **compaction 미정의인데 J3가 이미 약속** — `PRD-v1.md:151`("compaction … does not thrash the stable prefix") vs `PRD-v1.md:227`(open decision #4). long-session 경제성 주장의 핵심 변수가 미결정 상태로 남아 있고 어느 spec에도 배정되지 않았다(10에도 100에도 명시 없음).

- **DeepSeek 캐시 메커니즘 자체에 대한 가정이 기재되지 않음** — 자동 서버측 prefix caching인지, 최소 토큰 임계가 있는지, 블록 granularity가 있는지. byte-stability가 필요조건일 뿐이라는 사실이 어디에도 없어, spec 10이 "바이트만 맞추면 끝"으로 구현될 위험이 구조적으로 열려 있다.

## Minor (P2)

- **문서 우선순위 모순**: `HARNESS_PHILOSOPHY.md:7`은 "ADR이 philosophy의 지정 섹션을 supersede한다"고 하고, `SOURCES.md:105-112`는 `HARNESS_PHILOSOPHY > VISION+NON_GOALS > adr/`로 philosophy를 ADR 위에 둔다. 개정 경로가 자기모순.
- **status 어휘 미정의**: `TODO` / `Draft` / `ready-for-impl` 사이의 중간 상태도, 행을 뒤집는 주체도 없음. Codex P2 지적 미수정.
- **upstream 여전히 floating**: `deepcode-cli.md:5`는 "prefer pinning a commit SHA in future ADRs"로 문제를 **인정만** 하고 pin하지 않았다. 설계 근거가 upstream 편집으로 조용히 바뀔 수 있음.
- **toolchain 미결정이라면서 사실상 결정됨**: `crates/` 존재, `HARNESS_PHILOSOPHY.md:205` "Native-speed local tools", `REPO_LAYOUT.md:39-52`의 Grok식 패키지 표 — Rust를 세 곳에서 전제한다. ADR을 "열린 결정"이라 부르는 건 형식.
- **YOLO 금지가 "only mode"에만 걸림**: `NON_GOALS.md:14` "YOLO-**only** permissions", `PRD-v1.md:128` "as **only** mode". YOLO 모드 자체는 게이트 없이 암묵 허용된다.
- **`MILESTONES.md:12` "Snippet edit safety (when edit exists)"** — invariant에 조건절을 붙이면 invariant가 아니다.
- **UX 비계약성 유지**: `PRD-v1.md:173` "Exact names may differ; behavior is what ships against specs" — 그런데 interrupt/streaming thinking 표시/오류 표현을 다루는 spec이 없다. Codex P2 미수정.

## Missing before any runtime code

1. `docs/GATES.md` — G0–G6 상태 원장(상태·근거 PR·전환 주체). 게이트를 문장에서 검증 가능한 사실로 전환.
2. **Provider contract ADR** — 실측 model id pin, streaming event, thinking/effort 필드, `usage` 캐시 필드 유무, cancel/retry/rate-limit, error taxonomy. (P0-1, P0-2 동시 해소)
3. **Toolchain/config ADR** (G1) — 언어, 바이너리명, state dir, 프로젝트 config 경로, 자격증명 저장. `crates/` 기정사실 인정 또는 번복.
4. `HARNESS_PHILOSOPHY.md` 개정: `(golden or manual)` 탈출구 제거, §3에 "snippet 상태 일관성" 행 추가, declaration=advisory 명시, `write` flag 주체 확정, philosophy vs ADR 우선순위 확정.
5. `PRD-v1.md` / `MILESTONES.md` 개정: `or proxy`, `or golden prefix equality`, `when API reports it`, `(when edit exists)` 제거.
6. **Spec 10** — canonical serialization, epoch/무효화(스키마·skills·프로젝트 지시문·자기편집 포함), replay, compaction, worker prefix, golden fixture + provider 캐시 증거.
7. **Spec 15** — schema-aware repair, tool/result pairing 복구, 재시도 한도, "invalid side effect는 절대 dispatch 금지" 배리어, 감사 출력.
8. **Spec 20 / 30** — 라우팅 우선순위(사용자 > 라우터?), escalation 신호, 턴 예산, 모델 부재 시 fallback, Pro 사용의 가시성, effort 매핑.
9. **Spec 45** (40보다 먼저) — snippet identity/version/scope/ambiguity, 원자적 치환, **shell·외부 변경 시 강제 만료**, 병렬/subagent 하 소유권, resume/fork 거동, binary/generated 파일.
10. **Spec 90** (shell 활성화 전) — path 분류(symlink/traversal/nested workspace/worktree), 정적 명령 분류기 vs 모델 신고의 우선순위, 승인 수명, 병렬 승인 경합, deny 거동, 감사 스키마. "minimum"과 "full"의 경계를 문장으로 확정.
11. **Spec 40** — 최종 내장 툴 레지스트리, 인자 스키마, 출력 truncation, cancel, 금지 연산.
12. **Spec 50** — dispatch 순서, 취소, 타임아웃, 부분 실패, 편집 충돌, 승인 경합.
13. **Spec 120** — 소유자를 `All`이 아닌 실명으로 지정. 프로젝트 지시문 탐색·정렬·변경 무효화·prefix 상호작용.
14. **Adversarial acceptance suite v0** (M6가 아니라 M1과 함께) — golden prefix 바이트, replay pairing 복구, malformed tool JSON, stale snippet, ambiguous match, shell bypass, permission 경합, parallel 충돌, Flash/Pro 라우팅, 캐시 텔레메트리.
15. M4/M5 전 **Spec 60 / 80 / 100 / 110** — 특히 60은 worker template 동일성을 측정 가능한 금지 규칙으로, 80은 epoch rollover를 명시.

## Contradictions remaining

| A | B | 충돌 |
|---|---|---|
| `HARNESS_PHILOSOPHY.md:144` "'intent to reuse' is not acceptance" | `PRD-v1.md:37,110,150` · `MILESTONES.md:60` (`or proxy`, `or golden prefix equality`, `when API reports it`) | 캐시 수용 기준이 spine과 PRD에서 서로 다름 |
| `HARNESS_PHILOSOPHY.md:7` "ADR이 philosophy 섹션을 supersede" | `SOURCES.md:105-112` `philosophy > adr/` | 개정 권한의 방향이 반대 |
| `HARNESS_PHILOSOPHY.md:139-144` golden fixture 필수 | `HARNESS_PHILOSOPHY.md:284` ready-for-impl = "golden **or manual**" | 같은 문서 내에서 spec 10의 통과 기준이 두 개 |
| `HARNESS_PHILOSOPHY.md:270-286` 게이트 체계 | `AGENTS.md:47` artifact 검사 CI 금지 · `8dd1f40` CI 삭제 | 강제 수단 없는 강제 규칙 |
| `HARNESS_PHILOSOPHY.md:279` "G1 전 `crates/` 스캐폴딩 금지" | 저장소에 `crates/` 이미 존재(ADR 없음) | 게이트가 이미 위반된 상태로 선언됨 |
| `HARNESS_PHILOSOPHY.md:80-88` session-local snippet | `:197-212` subagent · spec 50 병렬 · M4 worktree | 상태 소유권/무효화 미정의, §3 충돌표에 해당 행 없음 |
| `HARNESS_PHILOSOPHY.md:92` dynamic MCP mount | `:130` mid-session 스키마 재작성 금지 | epoch 정책 없이 양립 불가 (Codex P1 미해소) |
| `HARNESS_PHILOSOPHY.md:171` "model declares side effects; policy decides" | pillar D의 안전 주장 | 정책 입력이 모델 출력이면 policy는 장식 — 정적 분류기 우선순위 미명시 |
| `PRD-v1.md:151` compaction이 prefix를 흔들지 않음 | `PRD-v1.md:227` compaction 전략 미결정 | 미정의 메커니즘에 대한 보장 |
| `PRD-v1.md:180` · `VISION.md:28-29` model id 확정형 | `PRD-v1.md:244` provider contract 미작성 | 미검증 문자열 위에 M1 전체가 서 있음 |
| `MILESTONES.md:9-14` invariant gates | `MILESTONES.md:12` `(when edit exists)` | 조건부 invariant |
| `NON_GOALS.md:14` · `PRD-v1.md:128` YOLO-**only** 금지 | pillar D | YOLO 모드 자체는 게이트 없이 허용됨 |
| `HARNESS_PHILOSOPHY.md:34` 비교 우위 주장 | `PRD-v1.md:196,115` 벤치는 M6+, suite는 "expansion" | 반증 장치 없이 선언된 논지 |
