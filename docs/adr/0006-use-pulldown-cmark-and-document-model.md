# ADR-0006: `pulldown-cmark` と独自 Document Model を使用する

- Status: Accepted
- Date: 2026-07-18

## Context

DocSail v0.1.0 は GFM の見出し、リスト、Task List、引用、コード、表、リンク、画像プレースホルダー、水平線、Autolink を Terminal に表示する。表示層が parser のイベントや AST に直接依存すると、Terminal 向けの折り返しやスタイルを変える際に parser の API が境界を越える。

## Decision

Markdown parser は `pulldown-cmark` 0.13.4 を採用し、次の実装 Issue で依存に追加する。CommonMark の pull parser であり、GFM の表・打消し・Task List を Option で有効化できるイベント API を持つ。ライセンスは MIT、最低 Rust 版は 1.71.1、リポジトリは `raphlinus/pulldown-cmark` である。

2026-07-18 に crates.io の `cargo info` で各候補の最新版を確認した。`pulldown-cmark` は 0.13.4、`comrak` は 0.54.0、`markdown` は 1.0.0 が最新版として公開され、いずれも公開リポジトリとドキュメントを持つ。この確認時点で `pulldown-cmark` は必要な GFM 拡張を小さい API 面で提供しており、採用可能と判断した。

比較対象の `comrak` は GFM 対応の AST を提供するが、既定 feature が CLI と syntax highlighting を含み、最低 Rust 版は 1.85、ライセンスは BSD-2-Clause である。`markdown` は MIT の AST parser だが、DocSail が必要とする GFM 表現への変換を小さく保つには `pulldown-cmark` のイベント API が適する。

parser のイベントは `markdown::Document` へ変換する。Document Model は Block と Inline のみで構成し、Ratatui の型・Terminal 幅・色を含めない。これにより parser と表示を独立に差し替え・テストできる。

## Consequences

Markdown 実装は `pulldown-cmark` のイベントから独自 Document Model への変換を必要とする。一方で表示層は parser 固有の型に依存せず、GFM 構文ごとの描画を段階的に追加できる。
