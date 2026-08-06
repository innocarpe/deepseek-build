# DeepSeek Build 기획 문서 적대적 리뷰 — 야간 자율 루프 관점

읽은 것: `MASTER_PLAN.md`, `ULTRAGOAL_CHAIN.md`, `ULTRAGOAL_PR_PLANNING.md`, `ULTRAGOAL_PROMPT_OVERNIGHT.md`, `ULTRAGOAL_PROMPT_COLD_START_0x.md`, `RELEASE_TRAIN_0x.md`, `prd/PRD-wave-A~D`, `GATES.md`, `specs/00-overview.md`, `SYSTEM_ARCHITECTURE.md`, `AGENTS.md`, `contributing/pull-requests.md`·`releases.md`, `.omc/ultragoal/plans/*`, 그리고 실제 저장소 상태(`Cargo.toml`, `.github/`, `scripts/`).

리뷰 중 `main`이 `136f793 feat(dogfood): ship 0.4.0 (#23)`으로 이동하고 G004가 complete로 바뀌었습니다. 아래 판단은 그 상태 기준입니다.

---

## 1. Verdict

| 목표 | 판정 |
|------|------|
| "야간 루프 → PR 잘 쪼개서 머지 → **npm install** → `dsb`로 실코딩" (문자 그대로) | **FAIL** |
| "야간 루프 → 0.5.0/0.6.0 PR 정상 분할·머지 → **소스 설치**로 dogfood 가능" | **PASS_WITH_FIXES** |

FAIL 근거는 감이 아니라 구조입니다.
- npm 패키지 설계가 **문서 어디에도 없고**, `SYSTEM_ARCHITECTURE.md:264`가 "npm binary download strategy"를 **아직 열린 설계 항목**으로 명시합니다. 저장소에 `package.json`은 존재하지 않습니다.
- `ULTRAGOAL_PROMPT_OVERNIGHT.md:67`이 publish identity 부재 시 "명령만 문서화하고 다른 일 계속"이라고 **명시적으로 종결 불가**를 인정합니다. 즉 야간 루프는 설계상 npm 앞에서 멈춥니다.
- 그 전에 0.5.0에서 **G6 게이트 데드락**(§2 H1)에 먼저 걸립니다.

풀 비전(Wave B~D)까지의 격차는 그보다 훨씬 큽니다: 파싱 가능한 테마 스펙 없음(`MASTER_PLAN.md:190` "구현 PR 착지 시 작성"), 스펙 40/50/60/70/80/100/110 전부 TODO(`specs/00-overview.md:17-26`), CI 없음.

---

## 2. 야간 PR 머지 루프를 깨뜨릴 치명적 구멍

### H1. G6 데드락 — Wave A가 자기 게이트에 막힌다 (최우선)
- `GATES.md:15`: **G6 = 스펙 70, 80, 100, 110** 묶음, **red**.
- `specs/00-overview.md:21,24`: 70 = Skills, **100 = Sessions**.
- 그런데 Wave A 스토리 `0.5.0 Sessions`(`goals.json` G005), `0.6.0 skills index min`(G006)이 바로 그 기능입니다.
- `ULTRAGOAL_PROMPT_COLD_START_0x.md:26`은 "G4–G6는 스펙 생기기 전까지 red 유지, 함부로 뒤집지 말 것", `AGENTS.md:86`은 "GATES 갱신 없이 green 주장 금지".
- **부분 플립 개념이 없습니다.** 스펙 100만 써도 G6는 못 켭니다(70/80/110 필요). 결과: 야간 에이전트는 ① 게이트 위반 후 진행, ② 정지, ③ Wave B 스펙 4개를 Wave A에서 몰아 쓰기 중 하나를 **자의로** 고릅니다. 셋 다 계획 붕괴입니다.
- 게다가 G005/G006 목표 문자열에 **스펙 작성 단위가 아예 없습니다** — PR 유닛 플랜을 쓰라고 해도 재료가 없습니다.

### H2. npm이 "inventable failure" (§5에서 상술)

### H3. 외부 검증자 부재 — self-merge 루프에 falsifier가 없다
`.github/workflows/`에는 README뿐이고 product CI가 0입니다. `pull-requests.md:328-337`은 solo self-merge를 허용, `AGENTS.md:90-96`은 "빌드/테스트할 게 생기면 진짜 CI 허용"이라 해놓고 Wave A에 CI 유닛이 없습니다(하든은 `MASTER_PLAN.md:112`의 0.15.0). 즉 밤새 머지되는 모든 증거가 **에이전트 자기 신고**입니다. `main`이 깨져도 몇 시간 뒤 스토리까지 오염됩니다. 이건 그들이 금지한 "process-police CI"가 아니라 그들 문서가 **허용한** product CI입니다.

### H4. 실패 사다리(failure ladder)가 없다
`ULTRAGOAL_PROMPT_OVERNIGHT.md:65-68`의 STOP 조건은 단 2개(secret 부재, 제품 포크 결정). 정의 안 된 것: 테스트 3연속 red, rebase 충돌, `gh` 인증 만료/브랜치 보호 거부, DeepSeek API 쿼터 소진, **`omc` 바이너리 부재**(전 체인이 `omc ultragoal`에 의존하는데 폴백 없음). `goals.json`에 `attempt` 필드는 있지만 재시도 상한도 `blocked` 전이 규칙도 문서에 없습니다 → 무한 재시도 위험.

### H5. 자율 종료 기준에 사람이 박혀 있다
`prd/PRD-wave-A-dogfood.md:38` "documented residual gaps **with owner accept**", §52-55 metrics는 전부 정성적. `RELEASE_TRAIN_0x.md:97` "**0.4.0 이후 owner가 재평가**"는 `ULTRAGOAL_PROMPT_OVERNIGHT.md:55` "웨이브 끝나면 즉시 다음, idle 금지"와 정면 충돌 — **하필 지금(0.4.0 머지 직후) 루프가 도달한 라인**입니다.

### H6. 상태 중복 → 드리프트가 이미 발생 중
버전이 `Cargo.toml` + `MASTER_PLAN.md:71,138,241` + `RELEASE_TRAIN_0x.md:30,118` + `PRD-wave-A:43-50` + `user-guide/01-install.md:3,44-46` + README에 중복. `releases.md:34-41` 릴리스 체크리스트는 이 중 MASTER_PLAN/PRD/user-guide를 **빠뜨립니다**. 실증: #23 머지 후 `RELEASE_TRAIN`은 갱신됐지만 **"one board"라는 `MASTER_PLAN`은 §2 "expect 0.3.0+", §4 `- [ ] 0.4.0`, §8 로그에 0.4.0 행 없음** — 보드가 한 릴리스 만에 거짓말을 시작했습니다.

### H7. SSOT가 세 개
`MASTER_PLAN.md:9` "This is the one board" vs `ULTRAGOAL_PROMPT_COLD_START_0x.md:87` "RELEASE_TRAIN_0x.md **(SSOT for versions + dogfood DoD)**" vs `goals.json` 원장. 우선순위 규칙이 없고, 하필 이 셋이 0.8.0/0.9.0에서 서로 다릅니다(§4).

### H8. 병렬은 선언일 뿐, 운용 프로토콜이 없다
`ULTRAGOAL_PR_PLANNING.md:138`은 "두 병렬 에이전트에 disjoint 유닛 할당"을 말하지만 **클레임 파일/소유권 레지스트리/워크트리 지시/중재자가 없습니다**. 반면 `MASTER_PLAN.md:196`은 "세션당 웨이브 플랜 하나". 게다가 Wave A는 모든 스토리가 SemVer를 올리고 **진행 로그 테이블(`RELEASE_TRAIN §7`, `MASTER_PLAN §8`)을 공유 편집**하므로 실질 병렬도 ≈ 0인데, 병렬 금지 파일 목록에는 `Cargo.toml/Cargo.lock`만 있습니다(`ULTRAGOAL_PR_PLANNING.md:70`).

### H9. Wave A 종료 조건이 Wave B 기능에 의존
`RELEASE_TRAIN_0x.md:34`는 dogfood-usable 잔여 갭으로 "interactive ask"를 꼽는데, 인터랙티브 권한 UX는 `PRD-wave-B-native.md:46`의 **0.9.0**입니다. `login`/`auth status` 폴리시(같은 줄 §32)는 **어떤 스토리에도 배정돼 있지 않습니다**.

---

## 3. PR 유닛 / 순차·병렬 / 원자 커밋 / 스태킹 규칙은 에이전트가 쓸 만큼 운용적인가

**단일 에이전트 기준: 예(이 문서 세트에서 가장 강한 부분). 다중 에이전트 기준: 아니오.**

작동하는 것 — `ULTRAGOAL_PR_PLANNING.md:36-44`의 유닛 템플릿은 필드가 구체적(Intent/Touches/Depends/Parallelizable/SemVer/Tests), §2.2의 병렬 4조건은 fail-close, §4 안티패턴 표와 §6 체크리스트는 기계적으로 검사 가능, `pull-requests.md:104-117`의 split 결정 트리에 정량 임계(≈600 LOC / 12파일)까지 있습니다.

빠진 것:
1. **인스턴스화된 예제 0개.** 0.5.0 Sessions에 대한 실제 유닛 플랜 예시가 없습니다. 에이전트는 템플릿보다 완성 예제를 훨씬 잘 복제합니다.
2. **PR 플랜의 거처가 잘못됨.** `ULTRAGOAL_PR_PLANNING.md:28`은 "첫 PR 본문에" 쓰라고 하는데, squash 후엔 저장소에 남지 않고 콜드 스타트 재개 세션이 못 찾습니다. → `.omc/ultragoal/plans/<plan>/pr-plans/<goal-id>.md`로 강제해야 합니다.
3. **충돌 복구 플레이북 없음.** "rebase stack after main moves"(§2.4 규칙 3)뿐, 충돌 시 명령·포기 기준·retarget 실패 처리 없음.
4. **병렬 소유권 프로토콜 없음**(H8).
5. **공유 가변 문서(진행 로그 2곳)가 직렬화 목록에 없음**(H8).

---

## 4. RELEASE_TRAIN(0.8 병렬) vs MASTER_PLAN(Wave A는 0.7 종료) 모순

이건 단순 불일치가 아니라 **한 파일 내부에서도 자가당착**입니다.

| 출처 | 0.8.0의 의미 | 병렬 도입 시점 |
|------|--------------|----------------|
| `RELEASE_TRAIN_0x.md:69` | "Later waves — Waves B–D 참조" | (Wave A 아님) |
| `RELEASE_TRAIN_0x.md:94-95` (같은 파일 §5 `dogfood-0x` 매핑) | **Parallel = 0.8.0**, Harden = 0.9.0 | **0.8.0** |
| `MASTER_PLAN.md:97,124` + `PRD-wave-B-native.md:45` | Spec 40 + tool surface (Wave B) | — |
| `MASTER_PLAN.md:106` + `PRD-wave-C-throughput.md:44` | — | **0.12.0** |
| `.omc/.../native-0x/goals.json:12-14` | Spec 40 (Wave B) | — |

즉 **0.8.0에 3가지 정의, 병렬에 2가지 버전(0.8.0 vs 0.12.0), 0.9.0에 2가지 정의(Harden vs Permissions+Theme)**. 야간 에이전트가 `RELEASE_TRAIN`을 "SSOT"라고 들은 상태(`COLD_START_0x:87`)에서 §5 표를 읽으면 **G4 없이 0.8.0 병렬을 시도**할 수 있고, 이는 `MASTER_PLAN.md:200`·`PRD-wave-C:50`의 최우선 금지사항 위반입니다.

추가 축: 마일스톤 장부도 어긋납니다. `MILESTONES.md:74-78`의 M2 exit criteria에 병렬·백그라운드 셸이 포함돼 있는데, `RELEASE_TRAIN_0x.md:64-65`는 0.3.0을 "M2 core (minus parallel)", 0.4.0을 "**M2 dogfood exit**"으로 선언합니다 — 종료 조건을 미충족한 채 마일스톤을 닫은 셈입니다.

**조치:** `RELEASE_TRAIN §5`의 Parallel/Harden 두 행을 삭제하거나 `~~superseded → MASTER_PLAN §3~~`로 표기 + 파일 상단에 "0.8.0 이상은 이 문서 범위 밖" 명시 + SSOT 우선순위 1줄(예: `goals.json` > `MASTER_PLAN` > wave PRD > `RELEASE_TRAIN`).

---

## 5. npm 0.7.0 준비도 — 명세 충분한가, 아니면 inventable failure

**inventable failure입니다.** 지금 명시된 전부는 다음 3줄뿐입니다:
- `COLD_START_0x:153-155` "package.json에 bin 두 개, 버전 일치, publish는 owner-gated"
- `releases.md:19-25` 동일 3줄
- `SYSTEM_ARCHITECTURE.md:229-237` 다이어그램 + **`:264` "npm binary download strategy = 열린 항목"**

결정되지 않은 것(전부 에이전트가 밤에 발명하게 됨):

| 항목 | 상태 |
|------|------|
| 패키지 이름/스코프 (`deepseek-build`? `@innocarpe/deepseek-build`?) | 미결정, ADR 없음 |
| 배포 전략: 플랫폼별 `optionalDependencies` vs postinstall 다운로드 vs napi vs `cargo install` 래퍼 | 미결정 |
| 플랫폼 매트릭스 (darwin-arm64/x64, linux-x64/arm64, windows) | 미결정 (`PRD-wave-D:20`은 macOS+Linux만) |
| 바이너리 호스팅(GitHub Releases) + 릴리스 워크플로 | **존재하지 않음** (workflows 비어 있음) |
| 체크섬/서명 검증 | 없음 |
| Rust 없는 환경에서 `npm i -g` 성립 여부 | 미검토 (현 install.sh는 로컬 빌드 전제) |
| dist-tag(0.y.z를 `latest`로?), `--access public`, 2FA, NPM_TOKEN | 없음 |
| `files`/`.npmignore`/`engines`/`repository` | 없음 |
| 이름 선점 실패 시 폴백 | 없음 |

`npm publish`는 **외부 공개·되돌리기 어려운 행위**인데(72시간 unpublish 창), 문서는 이름조차 정해주지 않습니다. 야간 에이전트가 임의 이름으로 퍼블리시하면 사실상 복구 불가입니다.

**권장 설계(그대로 문서화하면 야간 실행 가능):** ADR 0007에서 이름·스코프 확정 → tag push 시 CI가 4플랫폼 바이너리 빌드 → GitHub Release 업로드 → npm 래퍼는 `optionalDependencies` 플랫폼 패키지 + 체크섬 검증 → **에이전트의 DoD는 `npm pack` + `npm i -g ./*.tgz` + 양쪽 `--version` 일치 스모크까지**, 실제 `npm publish`는 사람 게이트. 이렇게 쪼개야 "0.7.0 완료"가 야간에 **검증 가능한 상태**로 닫힙니다.

---

## 6. dogfood-usable 기준 vs 실제로 빠진 엔지니어링 단계

먼저 기준 자체가 불일치합니다. `RELEASE_TRAIN §3`(1~7번)을 문자 그대로 읽으면 **0.4.0은 이미 전부 충족**입니다(install.sh ✓, credentials ✓, chat ✓, read/edit/write/grep/bash ✓, `--dogfood` ✓, README 스모크 ✓, SemVer ✓). 그런데 같은 파일 `:34`는 "sessions, search comfort, npm, interactive ask가 남았다"고 합니다 — **§3에 들어 있지도 않은 항목들**입니다. 이 틈으로 에이전트는 Wave A를 조기 종료하거나, 반대로 수용 기준 없는 "comfort" 작업을 무한히 할 수 있습니다.

어떤 스토리에도 배정되지 않은 실제 작업:

| 빠진 단계 | 근거 |
|-----------|------|
| **자동 E2E 스모크 스크립트**(install→auth→chat→edit→검증, exit code) | `scripts/`에 없음; §3-6은 "README 문서 스모크"뿐 |
| 스펙 100 + **세션 로드 시 tool-pair 복구**(spec 15 연계) | `COLD_START_0x:141`이 요구, 스펙 없음 |
| 스킬 인덱스의 **캐시 에폭 영향 분석**(spec 10) | 안정 프리픽스 변경 = 에폭 변경(`SYSTEM_ARCHITECTURE:130-132`), 유닛 없음 |
| `login` / `auth status` UX | `RELEASE_TRAIN:32`가 결함으로 인정, 스토리 없음 |
| 인터랙티브 권한 ask | Wave B 0.9.0인데 Wave A 갭으로 계산됨 (H9) |
| 야간 라이브 API **비용/레이트 리밋 가드** | 어디에도 없음 |
| 0.5/0.6용 user-guide 페이지 | `docs/user-guide/`에 01, 02만 존재 |
| 업그레이드/언인스톤 경로, CHANGELOG 파일 | `releases.md:40` "파일이 있으면"으로 회피, 파일 없음 |

---

## 7. 문서 수정 Top 7 (심각도 순)

1. **G6를 분해하고 Wave A 스토리에 스펙 유닛을 넣는다.** `GATES.md:15`를 G6a(100 sessions)/G6b(70 skills)/G6c(80 mcp)/G6d(110 plan)으로 분리, `goals.json` G005를 "spec 100 → feat sessions → docs" 3유닛으로 재정의. **이거 없으면 오늘 밤 0.5.0에서 멈춥니다.**
2. **ADR 0007 npm 패키징 + 0.7.0 PR 유닛 플랜을 미리 쓴다.** 이름/전략/플랫폼/호스팅/체크섬/dist-tag 확정, 에이전트 DoD를 `npm pack` 로컬 설치 스모크로 못 박고 publish는 사람 게이트로 분리(§5).
3. **`ULTRAGOAL_PROMPT_OVERNIGHT.md`에 실패 사다리를 추가한다.** 시도 상한(3), `--status blocked --evidence` 체크포인트 의무화, 충돌/인증/쿼터/`omc` 부재 각각의 분기, "막히면 독립 유닛으로 이동" 규칙.
4. **0.5.0 이전에 최소 product CI 유닛을 넣는다.** `cargo build/test --workspace` + `check-semver.sh` + install 스모크(ubuntu, macos). 그들 문서(`AGENTS.md:90-96`, `workflows/README`)가 이미 허용하는 범주이며, self-merge 루프의 유일한 외부 falsifier가 됩니다.
5. **0.8.0/0.9.0/병렬 모순 제거 + SSOT 우선순위 명문화.** `RELEASE_TRAIN_0x.md:94-95` 두 행 삭제/무효 표기, 파일 상단에 범위 한정, `MASTER_PLAN.md:9`와 `COLD_START_0x:87`의 SSOT 충돌 해소(§4).
6. **dogfood-usable을 기계 검증 가능하게 만든다.** `scripts/smoke-dogfood.sh`(exit code) 신설 + `RELEASE_TRAIN §3`을 그 스크립트 항목과 1:1로 맞춤 + `PRD-wave-A:38`의 "owner accept"에 야간 폴백 정의(잔여 갭을 `blocked` 라벨 이슈로 열고 진행).
7. **버전 상태 단일화.** `releases.md`에 "릴리스 시 갱신할 파일 전체 목록"(MASTER_PLAN §2/§4/§8, RELEASE_TRAIN §2/§7, PRD-wave-A 표, user-guide, README) 추가 + 드리프트 검사 스크립트, 그리고 **지금 당장 `MASTER_PLAN.md:71,138,241`의 0.4.0 반영**.

보너스(8, 9): PR 플랜 산출물을 `.omc/ultragoal/plans/<plan>/pr-plans/<goal-id>.md`로 강제, 병렬 에이전트용 유닛 클레임 프로토콜 + 공유 진행 로그 파일을 직렬화 대상에 추가.

---

## 8. 이미 강한 것 (립서비스 아님)

- **`ULTRAGOAL_PR_PLANNING.md`** — 유닛 필드 템플릿(§2.1), 병렬 4조건(§2.2), 원자 커밋 Do/Don't 표(§2.3), 스태킹 5규칙 + 바텀업 머지(§2.4), 안티패턴 fail-close 표(§4), 첫 편집 전 체크리스트(§6). 사람 팀 대부분보다 낫습니다.
- **process-police CI 거부 + "진짜 CI는 이런 것"까지 적어둔 것**(`.github/workflows/README.md`) — 녹색 체크 연극을 구조적으로 차단.
- **게이트 원장의 감사 가능성**(`GATES.md:6-23`) — 증거 PR·플립 주체 기록, "critical 스펙은 **자동** 테스트 필수" 규칙(rule 2)까지 있음.
- **Wave ↔ plan-id ↔ PRD ↔ 콜드스타트 프롬프트 1:1 매핑**, 그리고 그게 **실제로 돌아간 흔적**(ledger에 PR #18/#19/#23 증거 문자열 포함).
- **SemVer fail-close가 문서가 아니라 스크립트**(`scripts/check-semver.sh` + ADR 0006 금지형 표) — 에이전트가 `0.5`를 쓸 확률을 실제로 낮춤.
- **듀얼 CLI 불변식**이 AGENTS/ADR/릴리스/유저가이드 전 계층에 반복 — 소실되기 어려움.
- **정직한 자기 라벨링** — `RELEASE_TRAIN_0x.md:34` "Honest label for 0.4.0: dogfood **proof**". 과대선언 억제 문화가 문서에 박혀 있음.
- **L1/L2/L3 충돌 우선순위**(`AGENTS.md:43-50`)와 각 PRD의 "Failure if" 절 — Wave C에서 속도 때문에 안전 계약을 깨는 흔한 붕괴를 선제 차단.

---

**한 줄 요약:** 프로세스 규율(PR 분할·커밋·게이트·SemVer)은 상위 1% 수준인데, **실행 종결 경로**(G6 분해, npm 설계, 실패 사다리, 검증 가능한 DoD)가 비어 있습니다. 위 1~4번만 오늘 반영하면 오늘 밤 루프는 0.6.0까지 안전하게 굴러가고, 5~7번까지 반영하면 0.7.0을 "사람이 `npm publish` 한 줄만 치면 끝나는 상태"로 닫을 수 있습니다.
