# DocSail のスコープ

## 対象

- 一般的な Markdown ドキュメントディレクトリ、`docs/` ディレクトリ、リポジトリ直下の Markdown ファイル
- Azure DevOps Wiki の Git リポジトリ
- GitHub Flavored Markdown（GFM）と Azure DevOps Wiki Markdown

各基盤は通常の Markdown ディレクトリとして読める範囲で扱う。Azure DevOps API には接続しない。

## v0.1.0 に含めるもの

`docsail .` で起動し、Markdown のファイルツリーと選択中のプレビューを表示する最小の縦切りを提供する。

- ファイルまたはディレクトリの起動引数と、引数なし時のワークスペース検出（`docs/` を優先）
- `.gitignore` を考慮した Markdown の再帰探索
- 左にファイルツリー、右にプレビューを置く 2 ペイン TUI
- キーボードによる選択、プレビューのスクロール、ペインのフォーカス切替、再読み込み、終了
- ファイル変更監視、Terminal resize、終了時の Terminal 状態復元
- GFM の主要構文（見出し、段落、強調、打消し、ネストしたリストと Task List、ネストした引用、コード、表、リンク、画像プレースホルダー、水平線、角括弧付き Autolink）
- 日本語・Unicode の表示幅と Terminal 幅に応じた折り返し

Mermaid fenced block は通常のコードブロックとして表示する。

HTML は描画せず、リテラルとして表示する。見出しは Terminal で階層を判別できるよう、レベルごとの色と罫線・接頭辞で表示する。

確認用の入力例は [`docs/gfm-preview-test.md`](../gfm-preview-test.md) に置く。

## v0.1.0 から除外するもの

- Mermaid 図形描画、Terminal 画像表示、外部 SVG/PNG 変換
- Git diff、Rendered diff、Git 履歴
- 全文検索、リンク遷移、Back/Forward、TOC、broken link 診断
- Azure DevOps Wiki の `.order` および固有構文の対応
- D2、数式、HTML/CSS の完全再現、Plugin、設定・テーマシステム
- MkDocs、Docusaurus、Sphinx、GitHub Wiki 固有ナビゲーション、MDX、reStructuredText の個別解釈

## 現在の制限

- GFM の裸 URL、`www.`、メールアドレスによる拡張 Autolink はリンクとして装飾しない。
- 表セル内ではインライン装飾をプレーンテキストとして表示する。セル内の hard break はサポートしない。

## 実装上の制約

- Rust、Ratatui、Crossterm を用いる単一 crate とする。
- Node.js、Chromium、Puppeteer、`mmdc`、WebView、外部ブラウザ必須化を行わない。
- 早期の複数 crate 化、過剰な抽象化、不要な非同期ランタイムを避ける。`tokio` は明確な必要性が生じるまで追加しない。
