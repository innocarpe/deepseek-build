# DeepSeek Night v2 (C-balanced) Implementation Plan

> **For agentic workers:** 이 계획은 `omc ultragoal`로 실행하도록 작성됐습니다.
> 스토리 단위는 아래 G001~G007이며, 각 스토리는 독립적으로 테스트 가능하고
> 리뷰어가 개별적으로 반려할 수 있는 경계로 나눴습니다.
> 스텝은 체크박스(`- [ ]`)로 추적합니다.

> **Status — superseded historical plan:** 이 계획은 v2를 제품 기본으로 삼던
> 당시의 설계와 실행 순서를 기록합니다. 아래 v2-default 지시와 예제는 모두
> historical intent이며 현재 계약이 우선합니다. 현재 계약은 classic `deepseeknight`가
> 제품/runtime/config 기본이고, `deepseeknight-v2`는 선택 가능한 첫 번째 picker
> alternate이며, `DeepSeekNightNeutral`과 `GrokNight`은 parser/config 호환성만
> 유지하고 picker에서 숨기는 것입니다.

**[역사적 목표 — superseded] Goal:** DeepSeek Build의 기본 색 테마를 측정 검증된
DeepSeek Night v2(C-balanced)로 교체하고, 테마 선택 경로의 기존 결함을 함께
수정한다.

**[역사적 아키텍처 — superseded] Architecture:** 기존 `deepseeknight.rs`는
**손대지 않고** 새 파일 `deepseeknight_v2.rs`를 추가한 뒤, `ThemeKind`에 새
variant를 등록한다. 그다음 렌더 레이어 기본값 → 설정 레이어 기본값 → 제품
레이어 상수 순으로 전환한다. 마지막에 픽커 목록을 정리한다. 각 단계는 앞
단계가 없어도 컴파일되며, 되돌릴 때도 역순으로 한 스토리씩 되돌릴 수 있다.

**Tech Stack:** Rust 2021, `ratatui::style::{Color, Modifier}`, `cargo test`

**근거 문서:** `docs/product/THEME_V2_REPORT.md` (측정값·결함 분석·필드 맵의 출처)

---

## Global Constraints

- 기존 `theme/deepseeknight.rs`는 **수정 금지**. 롤백 경로이며 `deepseek_blue_is_official` 테스트가 걸려 있다.
- `ThemeKind`의 판별값(discriminant)은 **디스크에 직렬화**된다. 기존 값 재사용 금지, 새 값은 `7`.
- `from_name()`의 기존 문자열 매핑은 **절대 제거하지 않는다**. 픽커 목록에서만 뺀다. 사용자 config에 저장된 값이 계속 해석돼야 한다.
- 공식 브랜드 색 `#4D6BFE`(`rgb(77, 107, 254)`)는 값이 바뀌지 않는다. **역할만** 선·테두리·채움 전용으로 제한된다.
- GitHub로 나가는 텍스트(PR 제목/본문/코멘트)는 **영어**. 커밋 메시지는 Conventional Commits.
- 모든 스토리는 `cargo test` 전체 통과 후 커밋한다.

---

## File Structure

| 파일 | 책임 | 스토리 |
|---|---|---|
| `third_party/grok-build/crates/codegen/xai-grok-pager-render/src/theme/deepseeknight_v2.rs` | **신규** — v2 팔레트 상수 + `Theme::deepseeknight_v2()` + 불변조건 테스트 | G001 |
| `.../src/theme/mod.rs` | `ThemeKind` 등록, 이름 매핑, 기본값, 픽커 목록 | G002·G003·G005 |
| `.../xai-grok-pager/src/settings/defs.rs` | 설정 선택지·기본값 | G004·G005 |
| `.../xai-grok-pager/src/settings/registry.rs` | 설정 fallback 값 | G004 |
| `.../xai-grok-pager/tests/settings_e2e.rs` | 설정 E2E 기대값 | G004·G005 |
| `.../xai-grok-pager/src/slash/commands/theme.rs` | `/theme` 커맨드 테스트 | G005 |
| `crates/dsb-cli/src/agent_launch.rs` | `PRODUCT_THEME` + fixture | G006 |
| `crates/dsb-tools/src/path_a_permissions.rs` | fixture 문자열 | G006 |
| `CHANGELOG.md` | 릴리스 노트 | G007 |

---

## 설계 불변조건 (테스트로 고정한다)

| # | 불변조건 | 테스트 |
|---|---|---|
| I1 | 의미 역할은 **선언된 팔레트 상수만** 사용 | `semantic_roles_only_use_declared_palette` |
| I2 | 의미색 상호 색상각 **≥ 35°** | `semantic_hues_are_at_least_35_degrees_apart` |
| I3 | 브랜드 파랑은 **단일 색상 계열** (스프레드 ≤ 6°) | `brand_ramp_is_a_single_hue_family` |
| I4 | 텍스트 역할은 `bg_base`·`bg_raised` 양쪽에서 **CR ≥ 4.5** | `text_roles_pass_aa_on_base_and_raised` |
| I5 | `gray_dim`은 텍스트 역할에 **등장 금지** | `gray_dim_is_never_a_text_role` |
| I6 | 표면 사다리 인접 단계 **ΔL\* ≥ 2.3** | `surface_ladder_steps_are_visible` |
| I7 | `bg_hover`=명도축 / `bg_visual`=채도축 (Δchroma ≥ 30) | `hover_and_visual_separate_on_different_axes` |
| I8 | **`h2`~`h6`는 단조 비증가 대비 사다리. `h1`은 색상으로 구별되는 유일한 예외이며 `BLUE_TEXT`여야 한다.** | `heading_ladder_is_non_increasing_below_h1` |
| I9 | `#4D6BFE`(CR 4.46 < 4.5)는 **텍스트 역할 사용 금지** | `official_blue_is_never_a_text_role` |

> **I8 주의:** `docs/product/THEME_V2_REPORT.md`는 이 규칙을 "제목 전체가 단조 감소"로 느슨하게 적었으나 **부정확하다.**
> 실제 값은 h1 `7.31` → h2 `15.04`로 올라간다. h1은 밝기가 아니라 **색상**으로 최상위임을 표시하기 때문이다.
> 위 표의 문구가 정본이다.

---

## ultragoal 실행

계획 저장 후 이 저장소 루트에서:

```bash
omc ultragoal create-goals \
  --brief-file docs/superpowers/plans/2026-08-07-deepseek-theme-v2.md \
  --auto-plan-id \
  --goal "Palette::Add deepseeknight_v2 palette module with invariant tests, wired to nothing" \
  --goal "Register::Register ThemeKind::DeepSeekNightV2 so the theme is selectable, defaults unchanged" \
  --goal "RenderDefault::Switch render-layer defaults and fallbacks to DeepSeekNightV2" \
  --goal "Settings::Add v2 to settings choices and flip settings defaults, fixing the missing-choice defect" \
  --goal "Curate::Remove superseded themes from picker lists while keeping name parsing for back-compat" \
  --goal "Product::Update PRODUCT_THEME and all fixture strings" \
  --goal "Verify::Full suite, changelog, and docs sync"
```

이후 `omc ultragoal complete-goals`로 스토리를 하나씩 진행한다.

---

## Task G001: 팔레트 모듈 + 불변조건 테스트

배선 없이 파일만 추가한다. 이 스토리 종료 시점에 v2는 **어디에서도 참조되지 않는다.** 따라서 기존 동작에 대한 위험이 0이며, 불변조건 테스트가 스펙 전체를 코드로 고정한다.

**Files:**
- Create: `third_party/grok-build/crates/codegen/xai-grok-pager-render/src/theme/deepseeknight_v2.rs`
- Modify: `third_party/grok-build/crates/codegen/xai-grok-pager-render/src/theme/mod.rs` (`mod` 선언 1줄만)

**Interfaces:**
- Consumes: `super::tokyonight::Theme` (구조체 정의는 `theme/tokyonight.rs:50`)
- Produces:
  - `Theme::deepseeknight_v2() -> Theme` (const fn)
  - `pub const DEEPSEEK_BLUE_V2: Color` — 공식색 재노출
  - 모듈 내부 `palette` 상수: `BLUE_DIM BLUE BLUE_MID BLUE_TEXT VIOLET GREEN RED AMBER BG_TERMINAL BG_PROMPT BG_BASE BG_DARK BG_RAISED BG_HOVER BG_VISUAL FG FG_DARK GRAY_BRIGHT GRAY GRAY_DIM BORDER SCROLL_FG DIFF_ADD_BG DIFF_DEL_BG`

- [ ] **Step 1: `mod` 선언 추가**

`theme/mod.rs`의 `mod deepseeknight;`(17행) 바로 아래에 추가:

```rust
mod deepseeknight_v2;
```

- [ ] **Step 2: 실패하는 테스트부터 작성**

`theme/deepseeknight_v2.rs`를 만들고 **테스트 모듈만** 먼저 넣는다:

```rust
//! DeepSeekNight v2 — measured C-balanced palette.
//!
//! Design invariants I1..I9 are enforced by the tests at the bottom of this
//! file. See `docs/product/THEME_V2_REPORT.md` for the measurements and
//! `docs/superpowers/plans/2026-08-07-deepseek-theme-v2.md` for the rules.

use ratatui::style::{Color, Modifier};

use super::tokyonight::Theme;

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- colorimetry helpers -------------------------------------------

    fn rgb_of(c: Color) -> (f64, f64, f64) {
        match c {
            Color::Rgb(r, g, b) => (r as f64, g as f64, b as f64),
            other => panic!("expected Color::Rgb, got {other:?}"),
        }
    }

    fn lin(v: f64) -> f64 {
        let v = v / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }

    fn luminance(c: Color) -> f64 {
        let (r, g, b) = rgb_of(c);
        0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b)
    }

    fn contrast(a: Color, b: Color) -> f64 {
        let (la, lb) = (luminance(a), luminance(b));
        let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
        (hi + 0.05) / (lo + 0.05)
    }

    fn l_star(c: Color) -> f64 {
        let y = luminance(c);
        let d = 6.0 / 29.0;
        let f = if y > d * d * d {
            y.cbrt()
        } else {
            y / (3.0 * d * d) + 4.0 / 29.0
        };
        116.0 * f - 16.0
    }

    fn hue_deg(c: Color) -> f64 {
        let (r, g, b) = rgb_of(c);
        let (r, g, b) = (r / 255.0, g / 255.0, b / 255.0);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        if delta == 0.0 {
            return 0.0;
        }
        let h = if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * ((b - r) / delta + 2.0)
        } else {
            60.0 * ((r - g) / delta + 4.0)
        };
        if h < 0.0 { h + 360.0 } else { h }
    }

    fn hue_gap(a: Color, b: Color) -> f64 {
        let d = (hue_deg(a) - hue_deg(b)).abs();
        d.min(360.0 - d)
    }

    fn chroma(c: Color) -> f64 {
        let (r, g, b) = rgb_of(c);
        r.max(g).max(b) - r.min(g).min(b)
    }

    /// Every role that renders as **glyphs**. Lines, borders and fills are
    /// deliberately excluded — they are allowed to sit below the AA floor.
    fn text_roles(t: &Theme) -> Vec<(&'static str, Color)> {
        vec![
            ("text_primary", t.text_primary),
            ("text_secondary", t.text_secondary),
            ("gray_bright", t.gray_bright),
            ("gray", t.gray),
            ("command", t.command),
            ("path", t.path),
            ("running", t.running),
            ("warning", t.warning),
            ("accent_thinking", t.accent_thinking),
            ("accent_tool", t.accent_tool),
            ("accent_error", t.accent_error),
            ("accent_success", t.accent_success),
            ("accent_running", t.accent_running),
            ("accent_model", t.accent_model),
            ("accent_plan", t.accent_plan),
            ("accent_verify", t.accent_verify),
            ("accent_remember", t.accent_remember),
            ("md_heading_h1", t.md_heading_h1),
            ("md_heading_h2", t.md_heading_h2),
            ("md_heading_h3", t.md_heading_h3),
            ("md_heading_h4", t.md_heading_h4),
            ("md_heading_h5", t.md_heading_h5),
            ("md_heading_h6", t.md_heading_h6),
            ("md_code", t.md_code),
            ("md_text", t.md_text),
            ("md_muted", t.md_muted),
            ("md_task_checked", t.md_task_checked),
            ("md_task_unchecked", t.md_task_unchecked),
            ("link_fg", t.link_fg),
            ("paste_fg", t.paste_fg),
            ("diff_equal_fg", t.diff_equal_fg),
        ]
    }

    // ---- I4 -------------------------------------------------------------

    #[test]
    fn text_roles_pass_aa_on_base_and_raised() {
        let t = Theme::deepseeknight_v2();
        for (sname, surface) in [("bg_base", BG_BASE), ("bg_raised", BG_RAISED)] {
            for (rname, c) in text_roles(&t) {
                let cr = contrast(c, surface);
                assert!(cr >= 4.5, "{rname} on {sname}: {cr:.2}:1 < 4.5 (AA)");
            }
        }
    }

    // ---- I9 -------------------------------------------------------------

    #[test]
    fn official_blue_is_never_a_text_role() {
        // #4D6BFE measures 4.46:1 on bg_base — below the AA floor. It is a
        // line/border/fill color only; glyphs use BLUE_TEXT.
        let t = Theme::deepseeknight_v2();
        for (rname, c) in text_roles(&t) {
            assert_ne!(c, BLUE, "{rname} must not use the line-only official blue");
        }
    }

    // ---- I5 -------------------------------------------------------------

    #[test]
    fn gray_dim_is_never_a_text_role() {
        let t = Theme::deepseeknight_v2();
        for (rname, c) in text_roles(&t) {
            assert_ne!(c, GRAY_DIM, "{rname} must not use rule-only GRAY_DIM");
        }
    }

    // ---- I1 -------------------------------------------------------------

    #[test]
    fn semantic_roles_only_use_declared_palette() {
        let t = Theme::deepseeknight_v2();
        let allowed = [
            BLUE_DIM, BLUE, BLUE_MID, BLUE_TEXT, VIOLET, GREEN, RED, AMBER,
            FG, FG_DARK, GRAY_BRIGHT, GRAY, GRAY_DIM, BORDER,
        ];
        let mut roles = text_roles(&t);
        roles.extend([
            ("accent_user", t.accent_user),
            ("accent_assistant", t.accent_assistant),
            ("accent_system", t.accent_system),
            ("accent_skill", t.accent_skill),
            ("fuzzy_accent", t.fuzzy_accent),
            ("selection_border", t.selection_border),
            ("hover_border", t.hover_border),
            ("prompt_border", t.prompt_border),
            ("prompt_border_active", t.prompt_border_active),
            ("diff_insert_fg", t.diff_insert_fg),
            ("diff_delete_fg", t.diff_delete_fg),
            ("diff_gutter_fg", t.diff_gutter_fg),
            ("paste_dim", t.paste_dim),
        ]);
        for (rname, c) in roles {
            assert!(
                allowed.contains(&c),
                "{rname} uses an undeclared color {c:?}; add it to the palette \
                 or reuse an existing token"
            );
        }
    }

    // ---- I2 -------------------------------------------------------------

    #[test]
    fn semantic_hues_are_at_least_35_degrees_apart() {
        let sem = [
            ("BLUE_TEXT", BLUE_TEXT),
            ("VIOLET", VIOLET),
            ("GREEN", GREEN),
            ("RED", RED),
            ("AMBER", AMBER),
        ];
        for i in 0..sem.len() {
            for j in (i + 1)..sem.len() {
                let gap = hue_gap(sem[i].1, sem[j].1);
                assert!(
                    gap >= 35.0,
                    "{} vs {}: hue gap {gap:.1} deg < 35",
                    sem[i].0,
                    sem[j].0
                );
            }
        }
    }

    // ---- I3 -------------------------------------------------------------

    #[test]
    fn brand_ramp_is_a_single_hue_family() {
        let ramp = [BLUE_DIM, BLUE, BLUE_MID, BLUE_TEXT];
        let hues: Vec<f64> = ramp.iter().map(|c| hue_deg(*c)).collect();
        let max = hues.iter().cloned().fold(f64::MIN, f64::max);
        let min = hues.iter().cloned().fold(f64::MAX, f64::min);
        assert!(
            max - min <= 6.0,
            "brand ramp hue spread {:.1} deg > 6 — it is no longer one family",
            max - min
        );
    }

    // ---- I6 -------------------------------------------------------------

    #[test]
    fn surface_ladder_steps_are_visible() {
        let ladder = [BG_BASE, BG_DARK, BG_RAISED, BG_HOVER];
        for pair in ladder.windows(2) {
            let step = l_star(pair[1]) - l_star(pair[0]);
            assert!(step >= 2.3, "surface step dL* {step:.2} < 2.3");
        }
    }

    // ---- I7 -------------------------------------------------------------

    #[test]
    fn hover_and_visual_separate_on_different_axes() {
        let t = Theme::deepseeknight_v2();
        let dc = chroma(t.bg_visual) - chroma(t.bg_hover);
        assert!(dc >= 30.0, "bg_visual vs bg_hover chroma delta {dc:.0} < 30");
        assert!(
            hue_gap(t.bg_visual, BLUE) <= 6.0,
            "the selection tint must sit in the brand hue family"
        );
    }

    // ---- I8 -------------------------------------------------------------

    #[test]
    fn heading_ladder_is_non_increasing_below_h1() {
        let t = Theme::deepseeknight_v2();
        // h1 is the single hue-marked heading; it is NOT part of the
        // luminance ladder (BLUE_TEXT 7.31 sits below FG 15.04 on purpose).
        assert_eq!(t.md_heading_h1, BLUE_TEXT, "h1 must be the brand text blue");
        let tail = [
            ("h2", t.md_heading_h2),
            ("h3", t.md_heading_h3),
            ("h4", t.md_heading_h4),
            ("h5", t.md_heading_h5),
            ("h6", t.md_heading_h6),
        ];
        for pair in tail.windows(2) {
            let (an, a) = pair[0];
            let (bn, b) = pair[1];
            assert!(
                contrast(a, BG_BASE) >= contrast(b, BG_BASE) - 1e-9,
                "{an} ({:.2}) must not be dimmer than {bn} ({:.2})",
                contrast(a, BG_BASE),
                contrast(b, BG_BASE)
            );
        }
    }

    // ---- composite surfaces --------------------------------------------

    #[test]
    fn text_reads_on_selection_and_diff_surfaces() {
        let t = Theme::deepseeknight_v2();
        for (label, fg, bg) in [
            ("primary on selection", t.text_primary, t.bg_visual),
            ("primary on hover", t.text_primary, t.bg_hover),
            ("diff insert", t.diff_insert_fg, t.diff_insert_bg),
            ("diff delete", t.diff_delete_fg, t.diff_delete_bg),
        ] {
            let cr = contrast(fg, bg);
            assert!(cr >= 4.5, "{label}: {cr:.2}:1 < 4.5");
        }
    }

    #[test]
    fn official_blue_value_is_unchanged() {
        assert!(matches!(BLUE, Color::Rgb(77, 107, 254)));
        assert!(matches!(DEEPSEEK_BLUE_V2, Color::Rgb(77, 107, 254)));
    }
}
```

- [ ] **Step 3: 컴파일 실패 확인**

Run: `cargo test -p xai-grok-pager-render deepseeknight_v2`
Expected: FAIL — `cannot find value BG_BASE in this scope`, `no function or associated item named deepseeknight_v2`

- [ ] **Step 4: 팔레트 상수 추가**

Step 2에서 만든 파일의 `const fn rgb` 아래, `#[cfg(test)]` 위에 삽입:

```rust
/// Official DeepSeek product accent — value unchanged from v1.
/// v2 restricts it to lines, borders and fills (4.46:1 is below the AA floor).
pub const DEEPSEEK_BLUE_V2: Color = rgb(77, 107, 254);

// Brand ramp — one hue family (227-231 deg), four lightness steps.
const BLUE_DIM: Color = rgb(50, 72, 190); //  #3248BE  2.58:1  line/fill
const BLUE: Color = rgb(77, 107, 254); //     #4D6BFE  4.46:1  line/fill, official
const BLUE_MID: Color = rgb(103, 128, 254); // #6780FE  5.60:1  text (reserved)
const BLUE_TEXT: Color = rgb(126, 154, 255); // #7E9AFF  7.31:1  text

// Semantic colors — mutually >= 38 deg apart.
const VIOLET: Color = rgb(185, 140, 245); //  #B98CF5  7.47:1  reasoning
const GREEN: Color = rgb(95, 211, 155); //    #5FD39B 10.37:1  success/insert
const RED: Color = rgb(242, 112, 138); //     #F2708A  6.86:1  error/delete
const AMBER: Color = rgb(232, 183, 95); //    #E8B75F 10.45:1  running/warning

// Surfaces — even L* ladder, chroma held at 3-5.
const BG_TERMINAL: Color = rgb(8, 9, 11); //  #08090B  L*  2.45
const BG_PROMPT: Color = rgb(10, 11, 13); //  #0A0B0D  L*  3.01
const BG_BASE: Color = rgb(13, 14, 17); //    #0D0E11  L*  3.98
const BG_DARK: Color = rgb(19, 20, 24); //    #131418  L*  6.37
const BG_RAISED: Color = rgb(24, 25, 28); //  #18191C  L*  8.77
const BG_HOVER: Color = rgb(35, 36, 39); //   #232427  L* 14.21  lightness axis
const BG_VISUAL: Color = rgb(26, 34, 64); //  #1A2240  L* 14.02  chroma axis (38)

// Gray ramp — AA on both bg_base and bg_raised.
const FG: Color = rgb(226, 227, 230); //      #E2E3E6  15.04 / 13.70
const FG_DARK: Color = rgb(182, 183, 186); // #B6B7BA   9.62 /  8.77
const GRAY_BRIGHT: Color = rgb(147, 148, 151); // #939497  6.36 /  5.80
const GRAY: Color = rgb(128, 129, 132); //    #808184   4.95 /  4.51
const GRAY_DIM: Color = rgb(84, 85, 88); //   #545558   2.59 /  2.36  RULES ONLY

// Structure.
const BORDER: Color = rgb(46, 48, 54); //     #2E3036
const SCROLL_FG: Color = rgb(58, 59, 64); //  #3A3B40
const DIFF_ADD_BG: Color = rgb(11, 58, 34); // #0B3A22  GREEN reads 6.87:1
const DIFF_DEL_BG: Color = rgb(69, 19, 31); // #45131F  RED reads 5.46:1
```

- [ ] **Step 5: `Theme::deepseeknight_v2()` 구현**

상수 블록 아래, `#[cfg(test)]` 위에 삽입:

```rust
impl Theme {
    /// Historical DeepSeek Build v2 palette proposal (not the current
    /// product default) — measured C-balanced palette.
    ///
    /// Blue carries 10 roles; all of them are identity or "you can go here"
    /// affordances. Everything else is the gray ramp plus four semantic hues.
    pub const fn deepseeknight_v2() -> Self {
        Self {
            // Backgrounds
            bg_base: BG_BASE,
            bg_light: BG_RAISED,
            bg_dark: BG_DARK,
            bg_highlight: BG_RAISED,
            bg_hover: BG_HOVER,
            bg_terminal: BG_TERMINAL,
            bg_visual: BG_VISUAL,

            // Accents — blue marks identity, violet marks reasoning
            accent_user: BLUE,
            accent_assistant: BLUE,
            accent_system: BLUE,
            accent_skill: BLUE,
            accent_thinking: VIOLET,
            accent_tool: GRAY_BRIGHT,
            accent_error: RED,
            accent_success: GREEN,
            accent_running: AMBER,

            // Text
            text_primary: FG,
            text_secondary: FG_DARK,
            gray_dim: GRAY_DIM,
            gray: GRAY,
            gray_bright: GRAY_BRIGHT,

            // Semantic
            command: FG,
            path: BLUE_TEXT,
            running: AMBER,
            warning: AMBER,

            fuzzy_accent: BLUE,

            accent_plan: AMBER,
            accent_verify: VIOLET,
            accent_remember: GREEN,
            accent_model: GRAY_BRIGHT,

            // Borders
            selection_border: BLUE,
            hover_border: BORDER,
            prompt_border: GRAY_DIM,
            prompt_border_active: BLUE,

            // Scrollbar
            scrollbar_bg: BG_RAISED,
            scrollbar_fg: SCROLL_FG,

            // Diff
            diff_delete_bg: DIFF_DEL_BG,
            diff_delete_fg: RED,
            diff_insert_bg: DIFF_ADD_BG,
            diff_insert_fg: GREEN,
            diff_equal_fg: GRAY,
            diff_gutter_fg: GRAY_DIM,

            // Paste
            paste_bg: BG_RAISED,
            paste_fg: FG_DARK,
            paste_dim: GRAY_DIM,

            // Markdown — h1 is hue-marked, h2..h6 dim monotonically
            md_heading_h1: BLUE_TEXT,
            md_heading_h1_mod: Modifier::BOLD,
            md_heading_h2: FG,
            md_heading_h2_mod: Modifier::BOLD,
            md_heading_h3: GRAY_BRIGHT,
            md_heading_h3_mod: Modifier::BOLD,
            md_heading_h4: GRAY_BRIGHT,
            md_heading_h4_mod: Modifier::empty(),
            md_heading_h5: GRAY,
            md_heading_h5_mod: Modifier::BOLD,
            md_heading_h6: GRAY,
            md_heading_h6_mod: Modifier::empty(),
            md_code: FG_DARK,
            md_code_bg: BG_RAISED,
            md_text: FG_DARK,
            md_task_checked: GREEN,
            md_task_unchecked: FG_DARK,
            md_muted: GRAY,
            link_fg: BLUE_TEXT,
        }
    }
}
```

> `BLUE_DIM`·`BLUE_MID`·`BG_PROMPT`는 이 스토리에서 아직 쓰이지 않는다.
> `#[allow(dead_code)]`를 상수 블록 앞에 붙이거나, clippy가 통과하면 그대로 둔다.
> 필드 누락이 있으면 컴파일러가 `missing field` 에러로 알려준다 — 그때 `theme/tokyonight.rs:50`의 구조체 정의를 보고 채운다.

- [ ] **Step 6: 테스트 통과 확인**

Run: `cargo test -p xai-grok-pager-render deepseeknight_v2`
Expected: PASS — 10개 테스트 전부 통과

실패하면 색을 고치지 말고 **먼저 어느 불변조건이 깨졌는지 읽는다.** 실패 메시지에 측정값이 들어 있다.

- [ ] **Step 7: 기존 테스트 회귀 없음 확인**

Run: `cargo test -p xai-grok-pager-render`
Expected: PASS (v1 `deepseek_blue_is_official` 포함)

- [ ] **Step 8: 커밋**

```bash
git add third_party/grok-build/crates/codegen/xai-grok-pager-render/src/theme/deepseeknight_v2.rs \
        third_party/grok-build/crates/codegen/xai-grok-pager-render/src/theme/mod.rs
git commit -m "feat(theme): add DeepSeek Night v2 palette with measured invariants"
```

---

## Task G002: `ThemeKind::DeepSeekNightV2` 등록

이 스토리가 끝나면 v2를 **고를 수 있다.** 기본값은 아직 바뀌지 않는다.

**Files:**
- Modify: `third_party/grok-build/crates/codegen/xai-grok-pager-render/src/theme/mod.rs`

**Interfaces:**
- Consumes: `Theme::deepseeknight_v2()` (G001)
- Produces: `ThemeKind::DeepSeekNightV2`, canonical name `"deepseeknight-v2"`

- [ ] **Step 1: 실패하는 테스트 작성**

`theme/mod.rs`의 `mod tests` 안에 추가:

```rust
#[test]
fn v2_round_trips_through_name() {
    assert_eq!(
        ThemeKind::from_name("deepseeknight-v2"),
        Some(ThemeKind::DeepSeekNightV2)
    );
    assert_eq!(ThemeKind::from_name("dsb2"), Some(ThemeKind::DeepSeekNightV2));
    assert_eq!(ThemeKind::DeepSeekNightV2.display_name(), "deepseeknight-v2");
    assert_eq!(
        ThemeKind::from_name(ThemeKind::DeepSeekNightV2.display_name()),
        Some(ThemeKind::DeepSeekNightV2)
    );
}

#[test]
fn v2_is_listed_and_survives_256_color_terminals() {
    assert!(ThemeKind::ALL.contains(&ThemeKind::DeepSeekNightV2));
    assert!(!ThemeKind::DeepSeekNightV2.requires_truecolor());
}

#[test]
fn v2_is_dark() {
    assert!(Theme::deepseeknight_v2().is_dark());
}

#[test]
fn every_listed_theme_has_a_display_label() {
    // Guards the gap that left "oscura-midnight" rendering as a raw string.
    for kind in ThemeKind::ALL {
        let canonical = kind.display_name();
        let label = display_name_for_canonical(canonical);
        assert_ne!(
            label, canonical,
            "{canonical} has no human-readable label in display_name_for_canonical"
        );
    }
}
```

- [ ] **Step 2: 실패 확인**

Run: `cargo test -p xai-grok-pager-render theme::tests`
Expected: FAIL — `no variant named DeepSeekNightV2`

- [ ] **Step 3: enum variant 추가**

`theme/mod.rs:33` `ThemeKind`에 추가. **판별값 7 고정** (디스크 직렬화 대상):

```rust
    /// Historical v2 palette proposal (superseded); classic is the current
    /// DeepSeek Build product default.
    DeepSeekNightV2 = 7,
```

- [ ] **Step 4: 목록에 추가**

`ALL`(53행) — **맨 앞**에 넣어 픽커 최상단에 오게 한다:

```rust
    pub const ALL: &[ThemeKind] = &[
        ThemeKind::DeepSeekNightV2,
        ThemeKind::DeepSeekNight,
        ThemeKind::GrokNight,
        ThemeKind::GrokDay,
        ThemeKind::TokyoNight,
        ThemeKind::RosePineMoon,
        ThemeKind::OscuraMidnight,
    ];
```

`NO_TRUECOLOR`(70행) — 맨 앞에 `ThemeKind::DeepSeekNightV2,` 추가.

- [ ] **Step 5: 이름 매핑 추가**

`display_name()`(84행) match에:
```rust
            Self::DeepSeekNightV2 => "deepseeknight-v2",
```

`requires_truecolor()`(101행) match에:
```rust
            Self::DeepSeekNightV2 => false,
```

`from_name()`(116행) match에 — **기존 `"deepseeknight"` 팔은 그대로 둔다**:
```rust
            "deepseeknight-v2" | "deepseek-night-v2" | "deepseek2" | "dsb2" => {
                Some(Self::DeepSeekNightV2)
            }
```

`display_name_for_canonical()`(158행)에 두 줄 추가 — 두 번째는 기존 누락 수정:
```rust
        "deepseeknight-v2" => "DeepSeek Night",
        "oscura-midnight" => "Oscura Midnight",
```

- [ ] **Step 6: kind → theme 디스패치**

`theme/mod.rs:289` 부근 match에:
```rust
            ThemeKind::DeepSeekNightV2 => Self::deepseeknight_v2(),
```

- [ ] **Step 7: 테스트 통과 확인**

Run: `cargo test -p xai-grok-pager-render`
Expected: PASS

`all_excludes_auto`(734행)·`available_excludes_auto`(739행)도 통과해야 한다.

- [ ] **Step 8: 커밋**

```bash
git add third_party/grok-build/crates/codegen/xai-grok-pager-render/src/theme/mod.rs
git commit -m "feat(theme): register DeepSeekNightV2 kind and fix missing oscura label"
```

---

## Task G003: 렌더 레이어 기본값 전환 (역사적 v2-default 단계 — superseded)

여기서부터 **모든 사용자의 기본 화면이 바뀐다.** 앞 두 스토리와 리뷰 경계를 나눈 이유다.

**Files:**
- Modify: `third_party/grok-build/crates/codegen/xai-grok-pager-render/src/theme/mod.rs:170, 297, 361`

- [ ] **Step 1: 실패하는 테스트 작성**

> 다음 테스트와 fallback 예시는 v2-default 단계의 역사적 기대값이며
> superseded입니다. 현재 기본/fallback 계약은 classic `deepseeknight`이고,
> v2는 첫 번째 선택 가능 alternate입니다.

```rust
#[test]
fn default_theme_is_v2() {
    assert_eq!(Theme::default().bg_base, Theme::deepseeknight_v2().bg_base);
}

#[test]
fn non_truecolor_terminals_fall_back_to_v2() {
    // clamp_to_terminal must land on a theme that does not need truecolor.
    assert!(!ThemeKind::DeepSeekNightV2.requires_truecolor());
    assert!(ThemeKind::available().contains(&ThemeKind::DeepSeekNightV2));
}
```

- [ ] **Step 2: 실패 확인**

Run: `cargo test -p xai-grok-pager-render default_theme_is_v2`
Expected: FAIL — `bg_base` 불일치 (v1 `rgb(18,20,28)` vs v2 `rgb(13,14,17)`)

- [ ] **Step 3: 세 지점 교체**

`theme/mod.rs:170`:
```rust
impl Default for Theme {
    fn default() -> Self {
        Self::deepseeknight_v2()
    }
}
```

`theme/mod.rs:297` — `Auto` fallback:
```rust
            ThemeKind::Auto => Self::deepseeknight_v2(),
```

`theme/mod.rs:361` — `clamp_to_terminal` fallback:
```rust
            ThemeKind::DeepSeekNightV2
```

- [ ] **Step 4: 통과 확인**

Run: `cargo test -p xai-grok-pager-render`
Expected: PASS

`ansi16_overrides_preserve_bg_base`(793행)가 깨지면 **테스트가 아니라 구현을 의심한다** — `bg_base`는 사용자 터미널 소유라는 계약이 v2에서도 유지돼야 한다.

- [ ] **Step 5: 커밋**

```bash
git add third_party/grok-build/crates/codegen/xai-grok-pager-render/src/theme/mod.rs
git commit -m "feat(theme): make DeepSeek Night v2 the render-layer default"
```

---

## Task G004: 설정 레이어 — 선택지 추가 + 기본값 전환 (역사적 v2-default 단계 — superseded)

**리포트 §1-1의 결함이 여기서 고쳐진다.** 지금은 Settings 시트에서 DeepSeek 테마를 고를 수 없다.

**Files:**
- Modify: `.../xai-grok-pager/src/settings/defs.rs:41, 479, 707, 723, 739`
- Modify: `.../xai-grok-pager/src/settings/registry.rs:619, 626, 866, 882`
- Modify: `.../xai-grok-pager/tests/settings_e2e.rs`

- [ ] **Step 1: 실패하는 테스트 작성**

`tests/settings_e2e.rs`에 추가:

> 다음 설정 테스트와 기대값은 v2를 기본으로 전환하던 역사적 단계의 기록이며
> superseded입니다. 현재 `theme`/dark 기본은 classic `deepseeknight`이고,
> v2는 선택 가능하며 picker에서 첫 번째 concrete alternate입니다.

```rust
#[test]
fn theme_picker_offers_the_product_default() {
    // Regression: the product shipped deepseeknight via config injection while
    // the settings sheet never listed it, so a user who changed themes could
    // not get back without typing /theme.
    let reg = SettingsRegistry::new();
    let meta = reg.find("theme").expect("theme setting exists");
    let SettingKind::Enum { default, choices, .. } = &meta.kind else {
        panic!("theme must be an enum setting");
    };
    assert_eq!(*default, "deepseeknight-v2");
    assert!(
        choices.iter().any(|c| c.canonical == "deepseeknight-v2"),
        "the default must be selectable in the picker"
    );
}

#[test]
fn every_theme_default_is_present_in_its_own_choice_list() {
    let reg = SettingsRegistry::new();
    for key in ["theme", "auto_dark_theme", "auto_light_theme"] {
        let meta = reg.find(key).expect("setting exists");
        let SettingKind::Enum { default, choices, .. } = &meta.kind else {
            panic!("{key} must be an enum setting");
        };
        assert!(
            choices.iter().any(|c| c.canonical == *default),
            "{key}: default {default:?} is not in its own choice list"
        );
    }
}
```

> `SettingsRegistry::new()` / `find` / `SettingKind` 의 정확한 경로와 시그니처는
> `tests/settings_e2e.rs:2450` 부근의 기존 사용례(`reg.find("theme")`)를 그대로 따른다.

- [ ] **Step 2: 실패 확인**

Run: `cargo test -p xai-grok-pager --test settings_e2e theme_picker_offers`
Expected: FAIL — `assertion failed: left "groknight", right "deepseeknight-v2"`

- [ ] **Step 3: 선택지에 추가**

`defs.rs:41` `THEME_CHOICES` — `auto` 항목 **바로 다음**에:

```rust
    EnumChoice {
        canonical: "deepseeknight-v2",
        display: "DeepSeek Night",
        description: "Product default - DeepSeek blue on neutral dark.",
    },
```

`defs.rs:479` `CONCRETE_THEME_CHOICES` — **맨 앞**에 같은 블록.

- [ ] **Step 4: 기본값 전환**

`defs.rs:707` (`theme`) 및 `defs.rs:723` (`auto_dark_theme`):
```rust
                default: "deepseeknight-v2",
```
위 주석 `// Option<String> — None resolved to "groknight".`도 함께 갱신한다.

`defs.rs:739` (`auto_light_theme`)는 **`"grokday"` 그대로 둔다.** 라이트 테마는 이 계획의 범위 밖이다.

- [ ] **Step 5: registry fallback 전환**

`registry.rs`의 619·626·866·882행 `.unwrap_or("groknight")` 중
`theme`·`auto_dark_theme`에 해당하는 것만 `"deepseeknight-v2"`로 바꾼다.
`auto_light_theme` 경로는 `"grokday"`를 유지한다.
각 지점이 어느 키를 다루는지 앞뒤 10줄을 읽고 확인할 것 — 네 곳이 모두 같은 키가 아니다.

- [ ] **Step 6: 기존 E2E 기대값 갱신**

`settings_e2e.rs`의 `"groknight"` 하드코딩을 찾아 갱신:
```bash
grep -n '"groknight"' third_party/grok-build/crates/codegen/xai-grok-pager/tests/settings_e2e.rs
```
1959·2571·2619·2646·2672행 부근이 후보다. **테스트 의도를 읽고** 기본값을 확인하는 것인지, 임의의 유효 테마가 필요한 것인지 구분한다. 후자면 바꾸지 않는다.

- [ ] **Step 7: 통과 확인**

Run: `cargo test -p xai-grok-pager`
Expected: PASS

- [ ] **Step 8: 커밋**

```bash
git add third_party/grok-build/crates/codegen/xai-grok-pager/src/settings/ \
        third_party/grok-build/crates/codegen/xai-grok-pager/tests/settings_e2e.rs
git commit -m "fix(settings): list DeepSeek Night in the theme picker and default to v2"
```

---

## Task G005: 픽커 목록 정리 — 파싱 하위호환 유지 (역사적 결정 — superseded)

기준: **브랜드 정합성 / 사용자가 이름을 알아보는가 / 역할이 겹치지 않는가.**

| 테마 | 처리 | 근거 |
|---|---|---|
| DeepSeek Night v2 | **역사적 기본 (superseded)** | 당시 제품 정체성 제안; 현재는 첫 번째 picker alternate |
| DeepSeek Night (classic) | **현재 기본 + 선택 가능** | DSB 제품/runtime/config 기본 |
| DeepSeek Night (v1) | **역사적 결정 (superseded): 목록에서 제거** | v2로 대체됨. 같은 이름 두 개가 보이면 안 된다 |
| Grok Night | **목록에서 제거** | 타 제품 브랜드명 노출 + 역할이 v2와 완전 중복 |
| Grok Day | 유지 | 유일한 라이트 테마 — 기능적으로 필요 |
| Tokyo Night / Rose Pine Moon / Oscura Midnight | 유지 | 커뮤니티 표준, 사용자가 이름으로 인지 |

**Files:**
- Modify: `.../theme/mod.rs:53, 70` (`ALL`, `NO_TRUECOLOR`)
- Modify: `.../settings/defs.rs:41, 479`
- Modify: `.../slash/commands/theme.rs` (테스트)

- [ ] **Step 1: 하위호환 테스트부터 작성**

`theme/mod.rs`의 `mod tests`에:

> 아래 hidden-picker 테스트는 classic까지 숨기던 역사적 결정의 기록이며
> superseded입니다. 현재 classic `deepseeknight`는 `ALL`과 `NO_TRUECOLOR`에서
> 선택 가능하고, neutral/GrokNight만 compatibility-only로 숨깁니다.

```rust
#[test]
fn retired_theme_names_still_parse() {
    // Removed from the picker, but existing configs on disk must keep working.
    for name in ["groknight", "grok-night", "deepseeknight", "deepseek", "dsb"] {
        assert!(
            ThemeKind::from_name(name).is_some(),
            "{name} must still parse for back-compat"
        );
    }
}

#[test]
fn retired_themes_are_hidden_from_pickers() {
    assert!(!ThemeKind::ALL.contains(&ThemeKind::GrokNight));
    assert!(!ThemeKind::ALL.contains(&ThemeKind::DeepSeekNight));
}
```

- [ ] **Step 2: 실패 확인**

Run: `cargo test -p xai-grok-pager-render retired_themes_are_hidden`
Expected: FAIL

- [ ] **Step 3: `ALL` / `NO_TRUECOLOR` 정리**

```rust
    pub const ALL: &[ThemeKind] = &[
        ThemeKind::DeepSeekNightV2,
        ThemeKind::GrokDay,
        ThemeKind::TokyoNight,
        ThemeKind::RosePineMoon,
        ThemeKind::OscuraMidnight,
    ];
```

```rust
        const NO_TRUECOLOR: &[ThemeKind] = &[
            ThemeKind::DeepSeekNightV2,
            ThemeKind::GrokDay,
        ];
```

`display_name()`·`requires_truecolor()`·`from_name()`의 `GrokNight`/`DeepSeekNight` 팔은 **그대로 둔다.**

- [ ] **Step 4: 설정 선택지에서 제거**

`defs.rs:41` `THEME_CHOICES`와 `defs.rs:479` `CONCRETE_THEME_CHOICES`에서 `canonical: "groknight"` `EnumChoice` 블록을 삭제한다.

- [ ] **Step 5: 저장된 값이 목록에 없을 때의 동작을 고정**

`settings_e2e.rs`에 추가:

```rust
#[test]
fn a_retired_theme_stored_on_disk_still_applies() {
    // A user who picked Grok Night before it was retired keeps it until they
    // choose something else. We do not silently rewrite their config.
    assert_eq!(
        ThemeKind::from_name("groknight"),
        Some(ThemeKind::GrokNight)
    );
    assert!(Theme::groknight().is_dark());
}
```

> 이 동작(그대로 적용 vs 기본값으로 마이그레이션)은 **제품 판단**이다.
> 위는 "그대로 적용"을 택한 것이다. 반대로 가려면 이 테스트를 뒤집고
> 마이그레이션 코드를 추가해야 하며, 그건 별도 스토리로 분리한다.

- [ ] **Step 6: `/theme` 커맨드 테스트 수정**

`slash/commands/theme.rs:257-264`는 `"groknight"`가 목록에 있다고 가정한다.
해당 테스트의 대상을 `"deepseeknight-v2"`로 바꾼다. 320행 `cmd.run(&mut ctx, "groknight")`는
**파싱 경로 테스트이므로 그대로 둬도 통과해야 한다** — 통과하지 않으면 하위호환이 깨진 것이다.

- [ ] **Step 7: 통과 확인**

Run: `cargo test --workspace`
Expected: PASS

- [ ] **Step 8: 커밋**

```bash
git add third_party/grok-build/crates/codegen/xai-grok-pager-render/src/theme/mod.rs \
        third_party/grok-build/crates/codegen/xai-grok-pager/src/settings/defs.rs \
        third_party/grok-build/crates/codegen/xai-grok-pager/src/slash/commands/theme.rs \
        third_party/grok-build/crates/codegen/xai-grok-pager/tests/settings_e2e.rs
git commit -m "refactor(theme): retire superseded themes from pickers, keep name parsing"
```

---

## Task G006: 제품 레이어 상수 + fixture (역사적 v2 injection 단계 — superseded)

**Files:**
- Modify: `crates/dsb-cli/src/agent_launch.rs:78` 및 fixture `:407 :428 :440 :443 :465`
- Modify: `crates/dsb-tools/src/path_a_permissions.rs:210`

- [ ] **Step 1: 실패하는 테스트 확인**

Run: `cargo test -p dsb-cli agent_launch`
Expected: 아직 PASS (상수와 fixture가 서로 일치) — 이 스토리는 상수를 먼저 바꿔 실패를 만든다.

- [ ] **Step 2: 상수 교체**

`crates/dsb-cli/src/agent_launch.rs:78`:
> 아래 상수는 v2를 주입하던 역사적 제안이며 superseded입니다. 현재
> `PRODUCT_THEME`는 classic `deepseeknight`이고, v2는 picker alternate입니다.

```rust
pub const PRODUCT_THEME: &str = "deepseeknight-v2";
```

- [ ] **Step 3: 실패 확인**

Run: `cargo test -p dsb-cli`
Expected: FAIL — 407·428·440·443·465행의 fixture가 `"deepseeknight"`를 기대

- [ ] **Step 4: fixture 갱신**

해당 다섯 지점의 `deepseeknight` 문자열을 `deepseeknight-v2`로 바꾼다.
`crates/dsb-tools/src/path_a_permissions.rs:210`의 `theme = "deepseeknight"`도 같이 바꾼다.

가능하면 리터럴 대신 `PRODUCT_THEME` 상수를 참조하도록 고쳐 재발을 막는다.

- [ ] **Step 5: 통과 확인**

Run: `cargo test --workspace`
Expected: PASS

- [ ] **Step 6: 커밋**

```bash
git add crates/dsb-cli/src/agent_launch.rs crates/dsb-tools/src/path_a_permissions.rs
git commit -m "feat(cli): ship DeepSeek Night v2 as the injected product theme"
```

---

## Task G007: 전체 검증 + 문서

**Files:**
- Modify: `CHANGELOG.md`
- Modify: `docs/product/THEME_V2_REPORT.md` (I8 문구 정정)

- [ ] **Step 1: 전체 스위트**

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```
Expected: 전부 PASS

- [ ] **Step 2: 실물 확인**

`dsb`를 띄워 다음을 눈으로 확인한다. 계약 테스트가 못 잡는 부분이다.
- 기본 진입 화면이 v2인가
- `/theme`에 "DeepSeek Night"가 최상단에 있고 Grok Night가 없는가
- Settings → Appearance → Theme에서도 동일한가
- 행을 hover했을 때와 선택했을 때가 **다르게 보이는가** (I7의 실물 검증)
- 코드블록 안 흐린 텍스트가 읽히는가 (I4의 실물 검증)

- [ ] **Step 3: 리포트의 I8 문구 정정**

`docs/product/THEME_V2_REPORT.md`의 I8 행을 이 계획의 표현으로 교체한다.
현재 "단조 감소하는 밝기 사다리"는 h1에 대해 사실이 아니다 (h1 7.31 < h2 15.04).

- [ ] **Step 4: CHANGELOG 추가**

> 다음 changelog 초안은 v2-default 및 classic picker 제거를 전제로 한
> 역사적 문안이며 superseded입니다. 현재 동작 계약은 문서 상단의 classic
> default/V2 alternate/compatibility-only 규칙을 따릅니다.

```markdown
### Changed
- Default theme is now DeepSeek Night v2, a measured palette: 6 semantic hue
  families (down from 18), zero hue collisions (down from 50 colliding pairs),
  and every text role at WCAG AA or better on both the base and raised surfaces.
- Hover and selection now separate on different axes (lightness vs chroma)
  instead of both stepping in lightness, where they were indistinguishable.
- Grok Night and the v1 DeepSeek Night are no longer listed in the theme
  pickers. Existing configs naming them keep working.

### Fixed
- The theme settings sheet never listed the shipped product theme, so a user
  who switched themes could not switch back without the /theme command.
- "oscura-midnight" rendered as a raw identifier instead of "Oscura Midnight".
```

- [ ] **Step 5: 커밋**

```bash
git add CHANGELOG.md docs/product/THEME_V2_REPORT.md
git commit -m "docs: record the theme v2 switch and correct the heading-ladder rule"
```

- [ ] **Step 6: 최종 품질 게이트**

ultragoal의 마지막 스토리이므로 `ai-slop-cleaner` + `verification` + `$code-review` 증거를 모아
`--quality-gate-json`으로 전달한다. 리뷰가 깨끗하지 않으면 **complete로 표시하지 말고**
`record-review-blockers`로 블로커 스토리를 추가한다.

---

## 범위 밖 (별도 계획으로)

1. **라이트 테마 재설계 + Grok Day 개명** — 대비가 반전되어 같은 불변조건이 그대로 오지 않는다. `#4D6BFE`는 흰 배경에서도 4.46:1이라 라이트에서도 텍스트 불가. `auto_light_theme`는 이번 계획에서 `"grokday"`로 유지된다.
2. **`windows_contrast_boost` 축소** (`theme/mod.rs:380`) — v2는 램프가 이미 벌어져 있어 16~60단계 보정이 과할 수 있으나, 실제 ConHost 검증 없이 건드리지 않는다.
3. **`bg_base`를 `Color::Reset`으로** — 브랜드 배경을 포기하는 제품 판단이 필요하다. `theme/terminal_default.rs`에 선례가 있다.
4. **추론 스트림 렌더링** — `accent_thinking`에 색만 주는 것으로는 시그니처가 완성되지 않는다. thinking 블록을 이탤릭 + 한 단 흐린 회색으로 렌더하는 것은 `md_style.rs` / 메시지 렌더 변경이다.

---

## Self-Review 결과

- **스펙 커버리지:** 리포트 §3의 변경 지점 22곳이 G002·G003·G004·G006에 모두 매핑됨. §4의 목록 정리는 G005. §2의 팔레트·필드 맵은 G001. 누락 없음.
- **플레이스홀더:** 없음. 모든 코드 스텝에 실제 코드가 들어 있다. 단 G004 Step 5·6과 G005 Step 6은 "앞뒤를 읽고 판단"을 요구하는데, 대상 라인이 여러 키를 다루고 있어 기계적 치환이 위험하기 때문이다.
- **타입 일관성:** `Theme::deepseeknight_v2()`, `ThemeKind::DeepSeekNightV2`, canonical `"deepseeknight-v2"`가 G001~G006에서 동일하게 쓰인다. 팔레트 상수명도 G001 정의와 테스트에서 일치.
- **알려진 불일치 1건 정정:** 리포트의 I8 문구가 부정확했고, 이 계획의 표가 정본이다. G007 Step 3에서 리포트를 고친다.
