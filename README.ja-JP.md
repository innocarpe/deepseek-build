<div align="center">

[English](README.md) · [简体中文](README.zh-CN.md) · **日本語** · [한국어](README.ko-KR.md)

<!-- 一時的な hero 画像の出典: deepseek-ai/DeepSeek-V2 figures/logo.svg（DeepSeek-V3 でも使用）。 -->
<a href="https://github.com/deepseek-ai/DeepSeek-V3">
  <img src="assets/deepseek-logo.svg" width="60%" alt="DeepSeek logo">
</a>

<h1>DeepSeek Build</h1>

<p><strong>DeepSeek ネイティブなコーディング。Grok 級の実行。</strong></p>

<p>
  DeepSeek モデルを中心に設計された、安全な編集・キャッシュ認識セッション・
  並列実行を備えたフルスクリーンのターミナルコーディングエージェントです。
</p>

<p>
  <a href="https://github.com/innocarpe/deepseek-build/releases"><img alt="GitHub release" src="https://img.shields.io/github/v/release/innocarpe/deepseek-build?style=flat-square&label=release"></a>
  <a href="https://www.npmjs.com/package/@innocarpe/deepseek-build"><img alt="npm version" src="https://img.shields.io/npm/v/%40innocarpe%2Fdeepseek-build?style=flat-square&label=npm"></a>
  <a href="LICENSE"><img alt="Apache 2.0 license" src="https://img.shields.io/badge/license-Apache--2.0-blue?style=flat-square"></a>
</p>

<p>
  <a href="#クイックスタート">クイックスタート</a> ·
  <a href="#なぜ-deepseek-build-か">なぜ DeepSeek Build か</a> ·
  <a href="#仕組み">仕組み</a> ·
  <a href="#deepseek-harness">DeepSeek Harness</a> ·
  <a href="#ドキュメント">ドキュメント</a> ·
  <a href="#コントリビューション">コントリビューション</a>
</p>

</div>

<p align="center">
  <img src="assets/dsb-welcome.jpg" alt="DeepSeek Build のウェルカム画面 — dsb で開くフルスクリーンの DeepSeek エージェント TUI" width="85%">
</p>

## クイックスタート

npm からインストールし、DeepSeek API キーを追加して TUI を開きます。

npm 12.0.0 以降は依存パッケージの install スクリプトを既定で実行しません。
下のフラグがないと、成功したように見えてエージェントは入りません。
npm 11 以前はフラグがあってもなくてもインストールされます。

```bash
npm install -g --allow-scripts=@innocarpe/deepseek-build @innocarpe/deepseek-build
deepseek-build setup
deepseek-build
```

以後のグローバルインストールを一度だけ許可すれば、普通のコマンドで足ります:

```bash
npm config set allow-scripts=@innocarpe/deepseek-build --location=user
npm install -g @innocarpe/deepseek-build
```

レジストリ版は Node.js 18 以上が必要で、対応するリリースアセットがあれば
プリビルドバイナリを使用します。この経路では Rust は不要です。プラットフォーム
とソースフォールバックの詳細は [npm インストールガイド](docs/user-guide/05-npm.md) を
参照してください。

`deepseek-build` がプライマリコマンドです。`dsb` は同じ動作を持つ完全サポートの
短いエイリアスで、完全なセマンティックバージョンを持ちます:

```bash
deepseek-build --version
dsb --version
```

インストーラが製品 bin ディレクトリが `PATH` にないと報告した場合は、起動前に
追加してください:

```bash
export PATH="$HOME/.deepseek-build/bin:$PATH"
```

## なぜ DeepSeek Build か

| 能力 | 意味 |
| --- | --- |
| **DeepSeek ネイティブ** | DeepSeek API デフォルト、Flash/Pro ルーティング、reasoning effort、DeepSeek ブランドの TUI。 |
| **安全な編集** | バージョン束縛のスニペット編集とフェイルクローズのワークスペース権限。サイレントな全ファイル置換はしません。 |
| **長セッションの経済性** | セッションの変化はキャッシュ済み履歴を書き換えず、その後ろに追加されます。キャッシュミスは動いた組み立て済みドキュメントを名指しし、セッションは累計キャッシュ合計を記録し、セッションログと食い違うリクエストはそのターンを失敗させます。 |
| **スループット** | 並列ツール、バックグラウンドシェルジョブ、サブエージェント、オプトインの worktree を安全層・キャッシュ層の下で実行します。 |
| **永続セッション** | 直近のフルスクリーンセッションを再開するか、保存済みセッションに直接移動します。 |

その結果は、Grok 由来の実行エンジンの速さを保ちながら、DeepSeek 固有のコスト・
編集・権限ルールを製品経路に組み込んだコーディングエージェントです。

## 日常的な使い方

```bash
# フルスクリーン TUI を開く
deepseek-build

# 直近のフルスクリーンセッションを再開
deepseek-build --resume

# 非対話ターンを 1 回実行
deepseek-build run "Explain the architecture of this repository."

# 信頼済みのローカルコーディングプロファイルを使用
deepseek-build --dogfood
```

`--dogfood` は現在のワークスペース内の書き込みを許可し、ポリシー下でのシェル
実行を有効にします。ワークスペース外の書き込み・削除は引き続き拒否されます。

短いコマンドを使う場合は、例の `deepseek-build` を `dsb` に置き換えてください。

幅が 60 列以下の場合、送信済みプロンプトは 1 行に折りたたまれ、装飾用の `❯`
と空白パディング行が省かれます。それより広いウィンドウは現行の 3 行予算を
維持します。フルスクリーン画面の下部には設定可能なステータス行があります。
公式 DeepSeek API では、DeepSeek V4.1 Flash（`deepseek-flash`）が添付画像を
直接受け取り、テキスト専用モデルは画像をワイヤに載せずディスク上のフォール
バックを使います。

## 認証と設定

対話型セットアップは API キーを `0600` 権限で
`~/.deepseek-build/credentials.json` に保存します:

```bash
deepseek-build setup
deepseek-build auth status
deepseek-build auth logout
```

CI など非対話環境では `DEEPSEEK_API_KEY` を設定してください。環境変数は
資格情報ファイルより優先されます。製品設定・資格情報・セッション・ユーザースキルは
デフォルトで `~/.deepseek-build/` 配下に置かれます。

## ソースからのビルド

ソースインストールはコントリビュータと未サポートのリリースプラットフォーム向けです。
Rust 1.94 以上と `protoc` または DotSlash が必要で、初回のエージェントビルドには
数分かかることがあります。

```bash
git clone https://github.com/innocarpe/deepseek-build.git
cd deepseek-build
./scripts/install.sh

deepseek-build --version
dsb --version
```

このチェックアウトで `npm install` を実行しても `deepseek-build` と `dsb` は
インストールされません。プリビルトのダウンロードもコンパイルもしません。ここでは
`./scripts/install.sh` を使い、レジストリからは
`npm install -g --allow-scripts=@innocarpe/deepseek-build @innocarpe/deepseek-build`
です。npm 12 はこのフラグがないとエージェントをインストールしません。

Cargo とカスタムプレフィックスのオプションは
[インストールガイド](docs/user-guide/01-install.md) を参照してください。

## 仕組み

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

3 つのレイヤーには明確な所有権があります。より高スループットの機構は、その下の
編集・権限・キャッシュ契約を迂回できません。

| レイヤー | 出典 | 担当 |
| --- | --- | --- |
| **L1** | [Deep Code CLI](https://github.com/lessweb/deepcode-cli) | スニペット安全編集、スキルをコンテキストとして扱う方式、副作用権限。 |
| **L2** | [Reasonix](https://github.com/esengine/DeepSeek-Reasonix) | 安定プレフィックスの経済性、Flash/Pro 動作、ツールコール修復。 |
| **L3** | [Grok Build](https://github.com/xai-org/grok-build) | ベースランタイム、TUI、並列ツール、サブエージェント、バックグラウンド処理、worktree。 |

DeepSeek 公式ハーネスを読んでも、このマップにレイヤーは増えていません。
append ルールとリクエスト/ログ検査はその証拠から来たもので、次の節で
説明します。

規範的な競合ルールは [harness 哲学](docs/architecture/HARNESS_PHILOSOPHY.md) に、
完全なシステム構成図は [SYSTEM_ARCHITECTURE.md](docs/architecture/SYSTEM_ARCHITECTURE.md) にあります。

## DeepSeek Harness

[dsh](https://github.com/deepseek-ai/deepseek-harness) は DeepSeek の公式
オープンソースハーネスです。本製品のアーキテクチャではなく、4 つ目の
レイヤーでもありません。DeepSeek Build は Rust のフルスクリーン TUI のままで、
編集は Deep Code、キャッシュ契約は Reasonix、実行は Grok Build が所有します。
dsh はあるファミリーの振る舞いの証拠です — セッションが変わってもキャッシュ
済みのプレフィックスは生き残り、セッションログと食い違うリクエストは送られ
ません。

| セッション中に変わるもの | 製品の動作 |
| --- | --- |
| 安定した system 本文の後からの変更 | 以前の system メッセージはバイト単位でそのまま残り、新しい本文はその後ろに追加されます。モデルが現在の本文として扱うのは最新の system メッセージです。 |
| その本文内の tools、skills、environment、project instructions | 同じく追加です。ワイヤ上の `tools` 配列は毎リクエストで全リストです。ツール追加・削除専用の履歴メッセージはありません — Chat Completions にそのフィールドがないためです。 |
| キャッシュミス | そのターンは、どの組み立て済みドキュメントが動いたかを `prefix_change=` で示します。エポックが変わり、すべてのドキュメントハッシュが一致する場合は、原因をでっち上げずに `unattributed` と表示します。 |
| セッションのキャッシュ | キャッシュフィールドを持つ応答が一度でもあれば、以後の各ターンが `cache_session=` を 1 行記録します。hit/miss はトークン合計です。キャッシュフィールドのない応答は `unreported` で、miss には数えません。`deepseek-build run` と REPL はこの合計をセッションに保存し、resume が続きから数えます。フルスクリーンのステータスチップは依然として最後のターンの比率です。フルスクリーンのカウンターはセッションファイルに書き込まれず、新しいプロセスは 0 から始まります。 |
| ログと食い違うリクエスト | サンプリング前にターンが失敗します。シリアライズできない項目もフェイルクローズします。 |
| deny | 後からの allow は deny を置き換えません。 |

意図的に取り入れなかったもの: plugin host、agent teams、sandbox escalation、
request-series bookkeeping（`initial` / `resume` / `change`）、そして
Anthropic Messages トランスポートです。本製品は DeepSeek Chat Completions を
話します（[ADR 0005](docs/adr/0005-deepseek-provider-contract.md)）。

すでに存在するため再移植しなかったもの: 巨大なツール結果の spill（先頭・末尾と
残りのパス）、ウォームプレフィックスに合わせた compaction、dsh より厳しい
snippet staleness。これらは dsh から新たに取り入れた機能ではありません。

詳しくは: [dsh リサーチノート](docs/research/dsh-deepseek-harness.md) ·
[今回の作業での変更](docs/product/CHANGELIST_6_1_0.md) ·
[設計ソース](docs/product/SOURCES.md)。

## ドキュメント

| まずここから | 用途 |
| --- | --- |
| [ユーザーガイド](docs/user-guide/README.md) | インストール、セットアップ、日常利用、機能インデックス全体。 |
| [初回セットアップ](docs/user-guide/00-setup.md) | API キー、資格情報の優先順位、ヘッドレスセットアップ。 |
| [セッション](docs/user-guide/03-sessions.md) | フルスクリーン再開とラインモードセッションの保存。 |
| [権限](docs/user-guide/08-permissions.md) | 対話型確認、ヘッドレス拒否、ワークスペース境界。 |
| [サブエージェント](docs/user-guide/11-subagents.md) · [バックグラウンドタスク](docs/user-guide/12-background-tasks.md) · [Worktree](docs/user-guide/13-worktrees.md) | L3 実行面。 |
| [既知の制限](docs/product/KNOWN_LIMITS.md) | 現在のパッケージング、ライブスモーク、プラットフォーム境界。 |
| [製品 SSOT](docs/product/SSOT.md) | 製品ドキュメントが衝突したときの優先規則。 |

## 開発

```bash
cargo build -p dsb-cli
cargo test --workspace
./scripts/check-semver.sh
./scripts/test-owner-bar.sh
```

ルート Rust ワークスペースが製品 crates をカバーします。日常チェックでは vendor
全体への Cargo 実行を避けてください。owner-bar スクリプトは有界な製品経路を使用します。

crate マップは [crates/README.md](crates/README.md)、リポジトリマップとドキュメント
の所有権は [docs/README.md](docs/README.md) を参照してください。

## コントリビューション

変更前に [CONTRIBUTING.md](CONTRIBUTING.md) をお読みください。すべての意味ある
作業は、アトミックな Conventional Commit、既存の kind ラベル、誠実なテスト証拠、
[PR 作成ガイド](docs/contributing/pr-body-standard.md) のレビュー記述を備えた
焦点を絞った PR として取り込まれます。

## ライセンス

DeepSeek Build は [Apache License 2.0](LICENSE) の下で提供されます。ベンダリング
されたコードとサードパーティコードは元のライセンスを保持します。詳細は
[NOTICE](NOTICE) を参照してください。
