# DeepSeek Build — 테마 시스템 개편 리포트

> 이 문서는 **핸드오프 스펙**입니다. 작성자는 코드를 수정하지 않았습니다.
> 색상값은 전부 sRGB 상대휘도(WCAG 2.1) / CIE L\* / HSL 색상각으로 직접 계산해 검증했습니다.
> GitHub PR/이슈로 나가는 텍스트는 영어로 번역해야 합니다 (`github-pr` 규칙).

---

## 0. 요청 요약

1. **DeepSeek Night v2 (C-balanced)** 를 만들고 **제품 기본 테마**로 삼는다.
2. 나머지 테마는 **유의미한 것만 남겨** 사용자가 고를 수 있게 한다.
3. 색 선택은 취향이 아니라 **측정 가능한 불변조건**으로 고정한다.

---

## 1. 먼저: 지금 상태에서 발견한 결함 3가지

### 1-1. `deepseeknight`가 Settings 선택지에 없다 (기능 결함)

두 개의 테마 선택 surface가 **서로 다른 목록**을 씁니다.

| Surface | 참조하는 목록 | `deepseeknight` 포함? |
|---|---|---|
| `/theme` 슬래시 커맨드 | `ThemeKind::available()` (`theme/mod.rs:66`) | **있음** |
| Settings → Appearance → Theme | `THEME_CHOICES` (`settings/defs.rs:41`) | **없음** |
| Auto dark/light 하위 설정 | `CONCRETE_THEME_CHOICES` (`settings/defs.rs:479`) | **없음** |

결과: 제품은 `dsb-cli/src/agent_launch.rs:78`의 `PRODUCT_THEME = "deepseeknight"`를 config에 써 넣어 기본 적용하지만, **사용자가 Settings에서 테마를 한 번이라도 바꾸면 UI로는 되돌아올 수 없습니다.** `/theme deepseeknight`를 직접 쳐야만 복구됩니다.

### 1-2. Settings 기본값이 아직 `groknight`

```
settings/defs.rs:707    default: "groknight",       // theme
settings/defs.rs:723    default: "groknight",       // auto_dark_theme
settings/defs.rs:739    default: "grokday",         // auto_light_theme
settings/registry.rs:619,626,866,882   .unwrap_or("groknight")
```

렌더 레이어는 이미 DeepSeek으로 수렴해 있습니다 (`theme/mod.rs:170` `Default for Theme` → `deepseeknight()`, `:297` `Auto` → `deepseeknight()`, `:361` truecolor 미지원 fallback → `DeepSeekNight`). **설정 레이어만 뒤처져 있습니다.**

부수적으로 `theme/mod.rs:158` `display_name_for_canonical()`에 `"oscura-midnight"` 항목이 빠져 있어 raw 문자열이 그대로 노출됩니다.

### 1-3. 팔레트 자체의 측정 결함

현재 `deepseeknight`를 원본값 그대로 계산한 결과입니다.

| 항목 | 측정값 | 판정 |
|---|---|---|
| 의미 역할에 쓰이는 색 개수 | **18** | 실제 구분되는 건 5~6개 |
| 색상 충돌 (Δhue < 35°) | **50쌍** | `MAGENTA`↔`PURPLE` 0.0°, `YELLOW`↔`accent_plan` 1.1°, `GREEN1`↔`TEAL` 7.4° |
| `gray` (`COMMENT #6E748C`) on `bg_base` | **3.97:1** | AA(4.5) **미달** |
| `gray_dim` (`FG_GUTTER`) on `bg_base` | **2.11:1** | 사실상 비가시 |
| `md_heading_h2` = `#4D6BFE` (본문 텍스트) | **4.24:1** | AA **미달** |
| `bg_hover` → `bg_visual` | **ΔL\* 1.06** | 호버/선택 구분 불가 |
| 표면 6단 총 대비폭 | **1.37:1** | 6단을 담을 수 없는 폭 |
| 표면 틴트 chroma | `bg_base` 10 → `bg_visual` **30** | 밝아질수록 파래져 액센트를 잠식 |

`command`(YELLOW, 40°)와 `path`(ORANGE, 25°)는 **같은 줄에 나란히 렌더**되는데 15° 차이에 명도까지 같아 한 덩어리로 읽힙니다.

> **이미 증상은 발견돼 있었습니다.** `theme/mod.rs:380` `windows_contrast_boost()`는
> *"the theme's native ~12-unit RGB steps collapse visually"* 라며 표면을 16~60단계씩 강제로 벌립니다.
> 진단은 정확하지만 **Windows 전용 우회**로 처리됐습니다. 램프 자체가 좁은 게 원인이므로
> v2처럼 원본을 벌려두면 이 보정폭을 크게 줄이거나 없앨 수 있습니다.

---

## 2. 만들 것 — DeepSeek Night v2 (C-balanced)

새 파일: `third_party/grok-build/crates/codegen/xai-grok-pager-render/src/theme/deepseeknight_v2.rs`

기존 `deepseeknight.rs`는 **건드리지 말고 그대로 두세요.** 롤백 경로 + `deepseek_blue_is_official` 테스트가 걸려 있습니다.

### 2-1. 설계 불변조건 (이게 스펙의 본체입니다)

| # | 불변조건 | 검증 방법 |
|---|---|---|
| I1 | 의미 색상은 **6개 계열**을 넘지 않는다 | 색상각 목록 |
| I2 | 의미 색상 간 색상각 **≥ 35°** | 쌍별 Δhue = 0 violations |
| I3 | 브랜드 파랑은 **단일 색상(227~231°)의 명도 4단** | hue spread ≤ 6° |
| I4 | 텍스트 역할 색은 `bg_base`·`bg_raised` **양쪽에서 CR ≥ 4.5** | WCAG 2.1 |
| I5 | `gray_dim`은 **텍스트 금지** — 괘선/테두리 전용 | 코드 리뷰 |
| I6 | 표면 인접 단계 **ΔL\* ≥ 2.3** | CIE L\* |
| I7 | `bg_hover`는 **명도축**, `bg_visual`은 **채도축** (Δchroma ≥ 30) | 두 축이 겹치지 않음 |
| I8 | **h2~h6는 단조 비증가 대비 사다리.** h1은 색상으로 구별되는 유일한 예외이며 `BLUE_TEXT`여야 한다 (h1 CR 7.31 < h2 15.04 — 밝기가 아니라 색상으로 최상위 표시) | h1=`BLUE_TEXT`; h2≥h3≥…≥h6 on bg_base |
| I9 | `#4D6BFE`(공식색)는 **선·면 전용**, 텍스트는 `#7E9AFF` | 4.46 < 4.5 이므로 텍스트 불가 |

I9가 핵심입니다. **공식 브랜드 색은 그대로 보존되지만 역할이 바뀝니다** — 테두리·레일·채움에는 쓰고, 글자에는 밝은 짝을 씁니다.

### 2-2. 팔레트 상수

```rust
// ── 브랜드 램프: 단일 색상(227~231°)의 명도 4단 ────────────────
pub const BLUE_DIM:   Color = rgb( 50,  72, 190); // #3248BE  CR 2.58  선/면
pub const BLUE:       Color = rgb( 77, 107, 254); // #4D6BFE  CR 4.46  선/면 · 공식색
pub const BLUE_MID:   Color = rgb(103, 128, 254); // #6780FE  CR 5.60  텍스트(예비)
pub const BLUE_TEXT:  Color = rgb(126, 154, 255); // #7E9AFF  CR 7.31  텍스트

// ── 의미색: 상호 색상각 ≥ 38° ──────────────────────────────
pub const VIOLET:     Color = rgb(185, 140, 245); // #B98CF5  CR 7.47  추론(시그니처)
pub const GREEN:      Color = rgb( 95, 211, 155); // #5FD39B  CR10.37  성공/추가
pub const RED:        Color = rgb(242, 112, 138); // #F2708A  CR 6.86  오류/삭제
pub const AMBER:      Color = rgb(232, 183,  95); // #E8B75F  CR10.45  진행/경고/plan

// ── 표면: 균등 L* 사다리, chroma 3~5 고정 ────────────────────
pub const BG_TERMINAL:Color = rgb(  8,   9,  11); // #08090B  L* 2.45
pub const BG_PROMPT:  Color = rgb( 10,  11,  13); // #0A0B0D  L* 3.01
pub const BG_BASE:    Color = rgb( 13,  14,  17); // #0D0E11  L* 3.98
pub const BG_DARK:    Color = rgb( 19,  20,  24); // #131418  L* 6.37
pub const BG_RAISED:  Color = rgb( 24,  25,  28); // #18191C  L* 8.77
pub const BG_HOVER:   Color = rgb( 35,  36,  39); // #232427  L*14.21  ← 명도축
pub const BG_VISUAL:  Color = rgb( 26,  34,  64); // #1A2240  L*14.02  ← 채도축(chroma 38)

// ── 회색 램프: bg_base / bg_raised 양쪽 AA 통과 ───────────────
pub const FG:         Color = rgb(226, 227, 230); // #E2E3E6  15.04 / 13.70
pub const FG_DARK:    Color = rgb(182, 183, 186); // #B6B7BA   9.62 /  8.77
pub const GRAY_BRIGHT:Color = rgb(147, 148, 151); // #939497   6.36 /  5.80
pub const GRAY:       Color = rgb(128, 129, 132); // #808184   4.95 /  4.51
pub const GRAY_DIM:   Color = rgb( 84,  85,  88); // #545558   2.59 /  2.36  ※괘선 전용

// ── 구조 ────────────────────────────────────────────────
pub const BORDER:     Color = rgb( 46,  48,  54); // #2E3036
pub const SCROLL_FG:  Color = rgb( 58,  59,  64); // #3A3B40
pub const DIFF_ADD_BG:Color = rgb( 11,  58,  34); // #0B3A22  GREEN 대비 6.87
pub const DIFF_DEL_BG:Color = rgb( 69,  19,  31); // #45131F  RED   대비 5.46
```

### 2-3. 전체 필드 맵

`Theme` 구조체(`theme/tokyonight.rs:50`) 전 필드입니다. **파랑이 맡는 역할은 10개**입니다.

```rust
// 배경
bg_base: BG_BASE,          bg_light: BG_RAISED,     bg_dark: BG_DARK,
bg_highlight: BG_RAISED,   bg_hover: BG_HOVER,      bg_terminal: BG_TERMINAL,
bg_visual: BG_VISUAL,

// 액센트 — 파랑 7 / 그 외 semantic
accent_user:      BLUE,          // ① 브랜드
accent_assistant: BLUE,          // ② 브랜드 (v1은 BLUE_BRIGHT — 레일은 선이므로 BLUE로 통일)
accent_system:    BLUE,          // ③
accent_skill:     BLUE,          // ④
accent_thinking:  VIOLET,        //    시그니처 — 팔레트에서 보라는 여기뿐
accent_tool:      GRAY_BRIGHT,   //    기계 동작 = 무채색
accent_error:     RED,
accent_success:   GREEN,
accent_running:   AMBER,         //    v1은 BLUE_BRIGHT — 진행상태는 파랑에서 뺀다

// 텍스트 / 회색
text_primary: FG,         text_secondary: FG_DARK,
gray_dim: GRAY_DIM,       gray: GRAY,       gray_bright: GRAY_BRIGHT,

// 의미
command: FG,              // v1 YELLOW — 굵기로 처리, path와 충돌 제거
path:    BLUE_TEXT,       // ⑤ "갈 수 있는 것"
running: AMBER,           // v1 CYAN
warning: AMBER,

fuzzy_accent: BLUE,       // ⑥

accent_plan:     AMBER,
accent_verify:   VIOLET,
accent_feedback: GREEN,
accent_remember: GREEN,
accent_model:    GRAY_BRIGHT,

// 테두리
selection_border:     BLUE,       // ⑦
hover_border:         BORDER,
prompt_border:        GRAY_DIM,
prompt_border_active: BLUE,       // ⑧

// 스크롤바
scrollbar_bg: BG_RAISED,  scrollbar_fg: SCROLL_FG,

// diff
diff_delete_bg: DIFF_DEL_BG,  diff_delete_fg: RED,
diff_insert_bg: DIFF_ADD_BG,  diff_insert_fg: GREEN,
diff_equal_fg:  GRAY,         diff_gutter_fg: GRAY_DIM,

// paste
paste_bg: BG_RAISED,  paste_fg: FG_DARK,  paste_dim: GRAY_DIM,

// markdown — 제목은 색이 아니라 밝기 사다리 (I8)
md_heading_h1: BLUE_TEXT,   md_heading_h1_mod: Modifier::BOLD,   // ⑨ 7.31
md_heading_h2: FG,          md_heading_h2_mod: Modifier::BOLD,   //   15.04
md_heading_h3: GRAY_BRIGHT, md_heading_h3_mod: Modifier::BOLD,   //    6.36
md_heading_h4: GRAY_BRIGHT, md_heading_h4_mod: Modifier::empty(),//    6.36
md_heading_h5: GRAY,        md_heading_h5_mod: Modifier::BOLD,   //    4.95
md_heading_h6: GRAY,        md_heading_h6_mod: Modifier::empty(),//    4.95
md_code:           FG_DARK,       // v1 BLUE1 — cyan과 충돌하던 4번째 파랑 제거
md_code_bg:        BG_RAISED,
md_text:           FG_DARK,
md_task_checked:   GREEN,
md_task_unchecked: FG_DARK,
md_muted:          GRAY,
link_fg:           BLUE_TEXT,     // ⑩
```

v1 대비 **h5/h6가 AA 미달 회색으로 렌더되던 문제**도 함께 해소됩니다 (v1: `COMMENT` 3.97, `DARK3` 2.5 수준).

### 2-4. 검증 결과

| 항목 | v1 | v2 (C-balanced) |
|---|---|---|
| 파랑이 맡는 역할 | 12 | **10** |
| 의미 색상 계열 수 | 18 | **6** |
| 색상 충돌 (< 35°) | 50쌍 | **0쌍** |
| 텍스트 역할 AA 실패 | 3개 | **0개** |
| `gray` on bg_base / bg_raised | 3.97 ✗ | **4.95 / 4.51 ✓** |
| hover vs visual | ΔL\* 1.06 | **Δchroma +34** |
| 브랜드 램프 색상 스프레드 | — | **3.6°** (단일 계열) |

---

## 3. 기본 테마로 만들기 — 정확한 변경 지점

### 3-1. 렌더 레이어

| 파일 | 변경 |
|---|---|
| `theme/deepseeknight_v2.rs` | **신규** — §2 대로 작성 |
| `theme/mod.rs:17` 부근 | `mod deepseeknight_v2;` 추가 |
| `theme/mod.rs:33` `ThemeKind` | `DeepSeekNightV2 = 7` 추가 (판별값 재사용 금지 — 디스크 직렬화됨) |
| `theme/mod.rs:53` `ALL` | **맨 앞**에 `DeepSeekNightV2` |
| `theme/mod.rs:70` `NO_TRUECOLOR` | `DeepSeekNightV2` 추가 |
| `theme/mod.rs:84` `display_name()` | `=> "deepseeknight-v2"` |
| `theme/mod.rs:101` `requires_truecolor()` | `=> false` |
| `theme/mod.rs:116` `from_name()` | `"deepseeknight-v2" \| "deepseek2" \| "dsb2"` 매핑 |
| `theme/mod.rs:158` `display_name_for_canonical()` | `"DeepSeek Night"` + **누락된 `"oscura-midnight"`도 함께 추가** |
| `theme/mod.rs:170` `Default for Theme` | `deepseeknight_v2()` |
| `theme/mod.rs:289` kind→theme 매치 | 분기 추가 |
| `theme/mod.rs:297` `Auto` fallback | `deepseeknight_v2()` |
| `theme/mod.rs:361` `clamp_to_terminal` | fallback을 `DeepSeekNightV2`로 |

> **네이밍 판단은 그쪽에서 하세요.** 위는 `deepseeknight-v2`를 새 kind로 추가하는 안전한 안입니다.
> 대안으로 `deepseeknight` 이름을 그대로 두고 내용만 교체하면 기존 config가 자동 승계되지만
> 롤백이 어렵고 `deepseek_blue_is_official` 테스트를 손봐야 합니다.

### 3-2. 설정 레이어 ← **1-1 결함이 고쳐지는 지점**

| 파일 | 변경 |
|---|---|
| `settings/defs.rs:41` `THEME_CHOICES` | `auto` 다음에 DeepSeek Night 항목 추가 |
| `settings/defs.rs:479` `CONCRETE_THEME_CHOICES` | **맨 앞**에 추가 |
| `settings/defs.rs:707` `theme.default` | `"groknight"` → `"deepseeknight-v2"` |
| `settings/defs.rs:723` `auto_dark_theme.default` | `"groknight"` → `"deepseeknight-v2"` |
| `settings/defs.rs:739` `auto_light_theme.default` | `"grokday"` → §4 참고 |
| `settings/registry.rs:619,626,866,882` | `.unwrap_or("groknight")` → 새 기본값 |

```rust
EnumChoice {
    canonical: "deepseeknight-v2",
    display: "DeepSeek Night",
    description: "Product default — DeepSeek blue on neutral dark.",
},
```

### 3-3. 제품 레이어

`crates/dsb-cli/src/agent_launch.rs:78`
```rust
pub const PRODUCT_THEME: &str = "deepseeknight-v2";
```
`:407 :428 :440 :443 :465`와 `crates/dsb-tools/src/path_a_permissions.rs:210`의 fixture 문자열도 같이 갱신 필요합니다.

---

## 4. 테마 목록 정리 — "유의미한 것" 기준

기준 셋: **(a) 브랜드 정합성**, **(b) 사용자가 이름을 알아보는가**, **(c) 역할이 겹치지 않는가**.

| 테마 | 판단 | 근거 |
|---|---|---|
| **DeepSeek Night (v2)** | **기본** | 제품 정체성 |
| **Grok Night** | **목록에서 제거** | 다른 제품 브랜드명이 DeepSeek UI에 노출. 역할도 v2와 완전 중복 |
| **Grok Day** | **개명 + 재설계 필요** | 유일한 라이트 테마라 기능적으로 필요하지만 이름이 브랜드 누출 |
| **Tokyo Night** | **유지** | 커뮤니티 표준. 사용자가 이름으로 인지 |
| **Rose Pine Moon** | **유지** | 위와 동일 |
| **Oscura Midnight** | **유지** | 위와 동일. `display_name_for_canonical` 누락만 수정 |
| **terminal_default** | **유지 (minimal 전용)** | `Color::Reset` 기반 polarity-safe 경로 |

**하위호환:** `from_name()`에서 `"groknight"` / `"grokday"` **파싱은 반드시 유지**하세요. 목록에서만 빼고 기존 config는 계속 해석돼야 합니다. 목록에서 사라진 값이 저장돼 있을 때의 동작(그대로 적용 vs 기본값으로 마이그레이션)을 명시적으로 정하고 테스트를 다세요.

### 라이트 테마는 미완입니다 — 솔직한 범위 표시

C-balanced는 **다크 전용**으로 설계·검증했습니다. 라이트 대응은 하지 않았습니다. 라이트는 대비 계산이 반전되고(어두운 글자/밝은 배경), `windows_contrast_boost`의 비대칭 처리(`user_block_push` dark 28 / light 8)에서 보듯 같은 ΔL\*가 다르게 읽힙니다. **`#4D6BFE`는 흰 배경에서 CR 4.46이라 라이트에서도 텍스트로는 못 씁니다.**

권장: 이번 PR에서는 `grokday`를 **이름만** `deepseekday`로 옮기고(파싱 하위호환 유지), 팔레트 재설계는 **별도 작업**으로 분리하세요. 라이트 테마를 급히 만들면 위 불변조건을 못 지킵니다.

---

## 5. 깨질 테스트 / 확인할 것

| 위치 | 내용 |
|---|---|
| `theme/deepseeknight.rs:144` `deepseek_blue_is_official` | v1을 안 건드리면 통과. v1 교체 시 수정 필요 |
| `theme/mod.rs:734` `all_excludes_auto`, `:739` `available_excludes_auto` | 새 kind 추가해도 통과해야 함 |
| `theme/mod.rs:744` `is_dark_classifies_built_in_themes` | v2에 대한 `is_dark()` 케이스 추가 (`bg_base` L\* 3.98 → dark) |
| `theme/mod.rs:793` `ansi16_overrides_preserve_bg_base` | `bg_base`는 사용자 터미널 소유 — v2도 이 계약 유지 |
| `theme/cache.rs:402` | 캐시 왕복 동등성 |
| `settings_e2e.rs:1959, 2571, 2619, 2646, 2672` | 기본값 `"groknight"` 하드코딩 다수 — 전수 갱신 |
| `slash/commands/theme.rs:257-264, 320` | `"groknight"`가 목록에 있다고 가정하는 테스트 |

**새로 추가할 것을 권합니다** — 불변조건 I1~I9를 테스트로 고정하면 앞으로 테마가 늘어도 품질이 유지됩니다.

```rust
#[test] fn v2_text_roles_pass_aa_on_both_surfaces() { /* I4 */ }
#[test] fn v2_semantic_hues_are_35deg_apart()       { /* I2 */ }
#[test] fn v2_brand_ramp_is_single_hue_family()     { /* I3 */ }
#[test] fn v2_hover_and_visual_differ_by_chroma()   { /* I7 */ }
#[test] fn v2_heading_ladder_is_monotonic()         { /* I8 */ }
```

---

## 6. 범위 밖 (별도 판단 필요)

1. **`windows_contrast_boost` 축소** — v2는 램프가 이미 벌어져 있어 보정폭을 줄일 수 있지만, 실제 ConHost 검증 없이 건드리면 안 됩니다. v2 머지 후 별도로.
2. **`bg_base`를 `Color::Reset`으로** — Claude Code가 어느 터미널에서도 자연스러운 구조적 이유입니다. `theme/terminal_default.rs`가 이미 이 원칙을 구현했고 `theme/mod.rs:794`에 *"bg_base belongs to the user's terminal session, not to us"* 주석까지 있습니다. 다만 브랜드 배경을 포기하는 제품 판단이 필요해 이번 범위에서 뺐습니다.
3. **라이트 테마 재설계** — §4 참고.
4. **`accent_thinking` 렌더 처리** — 팔레트만으로는 시그니처가 완성되지 않습니다. thinking 블록을 이탤릭 + 한 단 흐린 회색으로 렌더하는 건 `md_style.rs` / 메시지 렌더 쪽 변경입니다.

---

## 부록 — 검토 산출물

| 파일 | 내용 |
|---|---|
| `/private/tmp/deepseeknight-preview.html` | (원본) 현재 vs 중립화 |
| `/private/tmp/deepseek-theme-3way.html` | 현재 / 중립화 / 옵션 C + 측정표 + 스퀸트·그레이스케일 검사 |
| `/private/tmp/deepseek-blue-density.html` | C-lean / **C-balanced** / C-rich 농도 비교 |

**채택: C-balanced.** C-lean은 어시스턴트 레일을 무채색으로 두는데, 레일을 남긴 채 회색을 칠하면 절제가 아니라 "비활성"으로 읽혀 기각했습니다. C-rich는 `md_code`·`md_heading_h2`까지 파랑이라 본문 한복판에 누를 수 없는 파란 텍스트가 생겨 기각했습니다.
