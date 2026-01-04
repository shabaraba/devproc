# devproc - 開発プロセスマネージャー

開発者向けのプロセス管理ツール。TUIとCLIの両方のインターフェースを提供し、開発サーバーやビルドタスクの起動・監視・終了を統一的に管理します。

## 特徴

- 🚀 **簡単なプロセス管理**: 開発サーバーやビルドタスクを名前で管理
- 📊 **リアルタイム監視**: CPU・メモリ使用率をリアルタイム表示
- 🎯 **スクリプト自動検出**: package.jsonのスクリプトを自動検出
- 💻 **TUI/CLI両対応**: 対話的なTUIと自動化可能なCLI
- 🏷️ **自動命名**: プロセスに識別しやすい名前を自動付与

## インストール

```bash
cargo install --path .
```

または、ビルドして使用:

```bash
cargo build --release
```

## 使い方

### TUI（ターミナルUI）モード

引数なしで実行するとTUIが起動します：

```bash
devproc
```

**キーバインド:**
- `j`/`k` または `↑`/`↓`: 項目の選択
- `Tab`: セクション切り替え
- `Enter`: スクリプト実行 / プロセス詳細
- `d`: プロセス終了
- `q`: 終了

### CLIコマンド

#### プロセス一覧表示

```bash
# テーブル形式
devproc list

# JSON形式
devproc list --format json
```

#### スクリプト一覧表示

```bash
devproc scripts

# 特定ディレクトリのスクリプト
devproc scripts --dir /path/to/project
```

#### スクリプト実行

```bash
# 基本的な実行
devproc start dev

# ポート指定
devproc start dev -p 3001

# カスタム名指定
devproc start dev -n my-dev-server

# 追加引数
devproc start dev -- --watch
```

#### 任意のコマンド実行

```bash
devproc run "npm run dev"
devproc run "python -m http.server" -n web-server
```

#### プロセス終了

```bash
# 名前で終了
devproc kill my-dev-server

# PIDで終了
devproc kill 12345

# シグナル指定
devproc kill my-dev-server --signal 9
```

#### コマンド履歴

```bash
# 履歴表示
devproc history

# 履歴件数制限
devproc history --limit 10

# 履歴クリア
devproc history --clear
```

#### 設定管理

```bash
# 設定表示
devproc config

# 設定ファイルを編集
devproc config --edit
```

#### MCPサーバー

```bash
# MCPサーバーとして起動（AIアシスタント連携用）
devproc mcp
```

## 実装済み機能

### Phase 1: MVP
✅ プロセス一覧表示（sysinfo使用）
✅ package.jsonスクリプト検出
✅ スクリプト実行（名前自動付与）
✅ プロセスkill
✅ 基本TUI（メイン画面のみ）
✅ 基本CLI（list, start, kill コマンド）

### Phase 2: 機能拡充
✅ 詳細画面（TUIで`i`または`Enter`で表示）
✅ build.gradle/build.gradle.kts対応
✅ Makefile対応
✅ Cargo.toml対応
✅ コマンド履歴機能
✅ 設定ファイル対応（~/.config/devproc/config.toml）
✅ フィルタ機能（TUIで`/`キーで起動）

### Phase 3: 高度な機能
✅ MCPサーバー（AIアシスタント連携）

## TUIキーバインド

- `/`: フィルタモード開始
- `Esc`: フィルタ解除 / 画面を戻る
- `i` または `Enter`: プロセス詳細表示（Processesセクション）
- `Enter`: スクリプト実行（Scriptsセクション）
- `j`/`k` または `↑`/`↓`: 項目の選択
- `Tab`: セクション切り替え
- `d`: プロセス終了
- `q`: 終了

## 対応ビルドツール

- **Node.js**: package.json
- **Gradle**: build.gradle / build.gradle.kts
- **Make**: Makefile
- **Rust**: Cargo.toml

## ライセンス

MIT License - 詳細はLICENSEファイルをご覧ください。