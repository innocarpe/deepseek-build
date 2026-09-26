<div align="center">

[English](README.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja-JP.md) · **한국어**

<!-- 임시 hero 이미지 출처: deepseek-ai/DeepSeek-V2 figures/logo.svg (DeepSeek-V3에서도 사용). -->
<a href="https://github.com/deepseek-ai/DeepSeek-V3">
  <img src="assets/deepseek-logo.svg" width="60%" alt="DeepSeek logo">
</a>

<h1>DeepSeek Build</h1>

<p><strong>DeepSeek 네이티브 코딩. Grok급 실행.</strong></p>

<p>
  DeepSeek 모델을 중심으로 설계된, 안전한 편집·캐시 인식 세션·병렬 실행을 갖춘
  풀스크린 터미널 코딩 에이전트입니다.
</p>

<p>
  <a href="https://github.com/innocarpe/deepseek-build/releases"><img alt="GitHub release" src="https://img.shields.io/github/v/release/innocarpe/deepseek-build?style=flat-square&label=release"></a>
  <a href="https://www.npmjs.com/package/@innocarpe/deepseek-build"><img alt="npm version" src="https://img.shields.io/npm/v/%40innocarpe%2Fdeepseek-build?style=flat-square&label=npm"></a>
  <a href="LICENSE"><img alt="Apache 2.0 license" src="https://img.shields.io/badge/license-Apache--2.0-blue?style=flat-square"></a>
</p>

<p>
  <a href="#빠른-시작">빠른 시작</a> ·
  <a href="#왜-deepseek-build인가">왜 DeepSeek Build인가</a> ·
  <a href="#작동-방식">작동 방식</a> ·
  <a href="#deepseek-harness">DeepSeek Harness</a> ·
  <a href="#문서">문서</a> ·
  <a href="#기여하기">기여하기</a>
</p>

</div>

<p align="center">
  <img src="assets/dsb-welcome.jpg" alt="DeepSeek Build 웰컴 화면 — dsb로 여는 풀스크린 DeepSeek 에이전트 TUI" width="85%">
</p>

## 빠른 시작

npm에서 설치하고 DeepSeek API 키를 추가한 뒤 TUI를 엽니다.

npm 12.0.0 이상은 의존성 install 스크립트를 기본으로 막습니다. 아래 플래그가
없으면 설치가 성공한 것처럼 보여도 에이전트는 깔리지 않습니다. npm 11 이하는
플래그가 있어도 없어도 설치됩니다.

```bash
npm install -g --allow-scripts=@innocarpe/deepseek-build @innocarpe/deepseek-build
deepseek-build setup
deepseek-build
```

이후 전역 설치를 한 번만 허용해 두면 평범한 명령으로 충분합니다:

```bash
npm config set allow-scripts=@innocarpe/deepseek-build --location=user
npm install -g @innocarpe/deepseek-build
```

레지스트리 설치에는 Node.js 18 이상이 필요하며, 일치하는 릴리스 asset이 있으면
프리빌트 바이너리를 사용합니다. 이 경로에는 Rust가 필요 없습니다. 플랫폼 및
소스 폴백에 대한 자세한 내용은 [npm 설치 가이드](docs/user-guide/05-npm.md)를
참고하세요.

`deepseek-build`가 기본 명령입니다. `dsb`는 동일한 동작을 가진 정식 지원
단축 별칭이며, 전체 시맨틱 버전을 가집니다:

```bash
deepseek-build --version
dsb --version
```

설치 프로그램이 제품 bin 디렉터리가 `PATH`에 없다고 보고하면 실행 전에 추가하세요:

```bash
export PATH="$HOME/.deepseek-build/bin:$PATH"
```

## 왜 DeepSeek Build인가

| 기능 | 의미 |
| --- | --- |
| **DeepSeek 네이티브** | DeepSeek API 기본값, Flash/Pro 라우팅, reasoning effort, DeepSeek 브랜드 TUI. |
| **안전한 편집** | 버전 바인딩 스니펫 편집과 fail-closed 워크스페이스 권한 — 무언의 전체 파일 교체 대신. |
| **긴 세션 경제성** | 세션 변경은 캐시된 히스토리를 다시 쓰는 대신 그 뒤에 붙습니다. 캐시 미스는 움직인 조립 문서를 말하고, 세션은 누적 캐시 합계를 기록하며, 세션 로그와 어긋난 요청은 그 턴을 실패시킵니다. |
| **월클록 스루풋** | 병렬 도구, 백그라운드 셸 작업, 서브에이전트, 옵트인 워크트리를 안전·캐시 레이어 아래에서 실행. |
| **지속 세션** | 가장 최근 풀스크린 세션을 재개하거나 저장된 세션으로 바로 이동. |

결과는 Grok 파생 실행 엔진의 속도를 유지하면서, DeepSeek 특유의 비용·편집·권한
규칙을 제품 경로에 포함한 코딩 에이전트입니다.

## 일상 사용

```bash
# 풀스크린 TUI 열기
deepseek-build

# 가장 최근 풀스크린 세션 재개
deepseek-build --resume

# 비대화형 턴 1회 실행
deepseek-build run "Explain the architecture of this repository."

# 신뢰할 수 있는 로컬 코딩 프로필 사용
deepseek-build --dogfood
```

`--dogfood`는 현재 워크스페이스 내 쓰기를 허용하고 정책 하에서 셸 실행을
활성화합니다. 워크스페이스 밖 쓰기·삭제는 계속 거부됩니다.

짧은 명령을 쓰려면 예시의 `deepseek-build`를 `dsb`로 바꾸면 됩니다.

폭이 60열 이하이면 제출된 프롬프트가 한 줄로 접히고, 장식용 `❯`와 빈 패딩
줄이 빠집니다. 그보다 넓은 창은 지금의 세 줄 예산을 유지합니다. 같은 폰 폭에서는
잔액, 캐시, 모델, 권한이 입력 상자 아래 한 줄에 모입니다. 비용과 캐시는 왼쪽,
모델과 권한은 오른쪽입니다. 더 넓은 창은 모델을 상자 테두리에, 잔액은 그 아래
줄에 둡니다. 풀스크린 화면
하단에는 설정 가능한 상태 줄이 있습니다. 공식 DeepSeek API에서 DeepSeek V4.1
Flash(`deepseek-flash`)는 첨부 이미지를 직접 받고, 텍스트 전용 모델은 이미지를
와이어에 싣지 않고 디스크 fallback을 씁니다.

## 인증 및 설정

대화형 셋업은 API 키를 `0600` 권한으로 `~/.deepseek-build/credentials.json`에
저장합니다:

```bash
deepseek-build setup
deepseek-build auth status
deepseek-build auth logout
```

CI 등 비대화형 환경에서는 `DEEPSEEK_API_KEY`를 설정하세요. 환경 변수가 자격증명
파일보다 우선합니다. 제품 설정·자격증명·세션·사용자 스킬은 기본적으로
`~/.deepseek-build/` 아래에 위치합니다.

## 소스에서 빌드

소스 설치는 기여자와 미지원 릴리스 플랫폼을 위한 것입니다. Rust 1.94 이상과
`protoc` 또는 DotSlash가 필요하며, 첫 에이전트 빌드는 수 분 걸릴 수 있습니다.

```bash
git clone https://github.com/innocarpe/deepseek-build.git
cd deepseek-build
./scripts/install.sh

deepseek-build --version
dsb --version
```

이 체크아웃에서 `npm install` 은 `deepseek-build` 나 `dsb` 를 설치하지 않습니다.
프리빌트를 받지 않고, 컴파일도 하지 않습니다. 여기서는 `./scripts/install.sh` 를
쓰고, 레지스트리 패키지는
`npm install -g --allow-scripts=@innocarpe/deepseek-build @innocarpe/deepseek-build`
입니다. npm 12는 이 플래그 없이 에이전트를 설치하지 않습니다.

Cargo 및 커스텀 프리픽스 옵션은 [설치 가이드](docs/user-guide/01-install.md)를
참고하세요.

## 작동 방식

```text
deepseek-build | dsb
        │
        ▼
product launcher ── auth · config · model routing
        │
        ▼
deepseek-build-agent ── full-screen TUI · tools · sessions
        │
        ▼
DeepSeek API
```

세 레이어는 명시적 소유권을 가집니다. 더 높은 스루풋의 메커니즘은 그 아래의
편집·권한·캐시 계약을 우회할 수 없습니다.

| 레이어 | 소스 | 담당 |
| --- | --- | --- |
| **L1** | [Deep Code CLI](https://github.com/lessweb/deepcode-cli) | 스니펫 안전 편집, 스킬-인-컨텍스트, 부작용 권한. |
| **L2** | [Reasonix](https://github.com/esengine/DeepSeek-Reasonix) | 안정 프리픽스 경제성, Flash/Pro 동작, 툴콜 복구. |
| **L3** | [Grok Build](https://github.com/xai-org/grok-build) | 베이스 런타임, TUI, 병렬 도구, 서브에이전트, 백그라운드 작업, 워크트리. |

DeepSeek의 공식 하네스를 읽은 것은 이 지도에 레이어를 더하지 않았습니다.
append 규칙과 요청/로그 검사는 그 증거에서 왔으며, 다음 절에서 다룹니다.

규범적 충돌 규칙은 [harness 철학](docs/architecture/HARNESS_PHILOSOPHY.md)에,
전체 시스템 구성도는 [SYSTEM_ARCHITECTURE.md](docs/architecture/SYSTEM_ARCHITECTURE.md)에
있습니다.

## DeepSeek Harness

[dsh](https://github.com/deepseek-ai/deepseek-harness)는 DeepSeek의 공식
오픈소스 하네스입니다. 이 제품의 아키텍처가 아니며 네 번째 레이어도 아닙니다.
DeepSeek Build는 Rust 풀스크린 TUI로 남고, 편집은 Deep Code, 캐시 계약은
Reasonix, 실행은 Grok Build가 그대로 소유합니다. dsh는 한 가족의 동작에 대한
증거입니다 — 세션이 바뀌어도 캐시된 접두는 살아 있고, 세션 로그와 어긋난
요청은 보내지 않습니다.

| 세션에서 바뀌는 것 | 제품이 하는 일 |
| --- | --- |
| 안정 시스템 본문의 나중 변경 | 앞선 system 메시지는 바이트 그대로 남고, 새 본문이 그 뒤에 붙습니다. 모델이 현재 본문으로 취급하는 것은 가장 최근 system 메시지입니다. |
| 그 본문 안의 tools, skills, environment, project instructions | 같은 append입니다. 와이어의 `tools` 배열은 매 요청의 전체 목록입니다. tool 추가/삭제 전용 히스토리 메시지는 없습니다 — Chat Completions에 그 필드가 없습니다. |
| 캐시 미스 | 턴이 어떤 조립 문서가 움직였는지 `prefix_change=`로 말합니다. 에폭이 달라졌는데 문서 해시가 전부 같으면 원인을 만들지 않고 `unattributed`라고 말합니다. |
| 세션의 캐시 | 캐시 필드를 가진 응답이 한 번이라도 있으면, 이후 모든 턴이 `cache_session=` 한 줄을 남깁니다. hit/miss는 토큰 합입니다. 캐시 필드가 없는 응답은 `unreported`이고 miss로 세지 않습니다. `deepseek-build run`과 REPL은 이 합계를 세션에 저장해서 resume가 이어서 셉니다. 풀스크린 상태 칩은 여전히 마지막 턴의 비율이고, 풀스크린 카운터는 세션 파일에 쓰지 않으며 새 프로세스는 0에서 시작합니다. |
| 로그와 다른 요청 | 샘플링 전에 턴이 실패합니다. 직렬화할 수 없는 항목도 실패로 닫습니다. |
| deny | 나중 allow가 deny를 바꾸지 않습니다. |

일부러 가져오지 않은 것: plugin host, agent teams, sandbox escalation,
request-series bookkeeping(`initial` / `resume` / `change`), Anthropic Messages
트랜스포트입니다. 이 제품은 DeepSeek Chat Completions를 말합니다
([ADR 0005](docs/adr/0005-deepseek-provider-contract.md)).

이미 있어서 다시 포팅하지 않은 것: 큰 도구 결과의 spill(앞/뒤와 나머지 경로),
따뜻한 접두에 맞춘 compaction, dsh보다 엄격한 snippet staleness입니다. 이것들은
dsh에서 새로 가져온 기능이 아닙니다.

더 읽기: [dsh 리서치 노트](docs/research/dsh-deepseek-harness.md) ·
[이 작업이 바꾼 것](docs/product/CHANGELIST_6_1_0.md) ·
[디자인 소스](docs/product/SOURCES.md).

## 문서

| 여기서 시작 | 용도 |
| --- | --- |
| [사용자 가이드](docs/user-guide/README.md) | 설치, 셋업, 일상 사용, 전체 기능 인덱스. |
| [첫 실행 셋업](docs/user-guide/00-setup.md) | API 키, 자격증명 우선순위, 헤드리스 셋업. |
| [세션](docs/user-guide/03-sessions.md) | 풀스크린 재개와 라인 모드 세션 저장. |
| [권한](docs/user-guide/08-permissions.md) | 대화형 질문, 헤드리스 거부, 워크스페이스 경계. |
| [서브에이전트](docs/user-guide/11-subagents.md) · [백그라운드 작업](docs/user-guide/12-background-tasks.md) · [워크트리](docs/user-guide/13-worktrees.md) | L3 실행 표면. |
| [알려진 제한](docs/product/KNOWN_LIMITS.md) | 현재 패키징, 라이브 스모크, 플랫폼 경계. |
| [제품 SSOT](docs/product/SSOT.md) | 제품 문서 충돌 시 무엇이 우선하는지. |

## 개발

```bash
cargo build -p dsb-cli
cargo test --workspace
./scripts/check-semver.sh
./scripts/test-owner-bar.sh
```

루트 Rust 워크스페이스가 제품 크레이트를 커버합니다. 일상 체크에서는 vendor 전체
Cargo 실행을 피하세요. owner-bar 스크립트는 경계가 있는 제품 경로를 사용합니다.

크레이트 맵은 [crates/README.md](crates/README.md), 리포지토리 맵과 문서 소유권은
[docs/README.md](docs/README.md)를 참고하세요.

## 기여하기

변경 전에 [CONTRIBUTING.md](CONTRIBUTING.md)를 읽어주세요. 모든 의미 있는 작업은
원자적 Conventional Commit, 기존 kind 라벨, 정직한 테스트 증거,
[PR 작성 가이드](docs/contributing/pr-body-standard.md)의 리뷰 서술을 갖춘
집중된 PR로 들어옵니다.

## 라이선스

DeepSeek Build는 [Apache License 2.0](LICENSE)으로 제공됩니다. 벤더 및
서드파티 코드는 원래 라이선스를 유지합니다. 자세한 내용은 [NOTICE](NOTICE)를
참고하세요.
