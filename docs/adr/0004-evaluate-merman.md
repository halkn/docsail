# ADR-0004: `merman` を Mermaid Backend 候補として評価する

- Status: Accepted
- Date: 2026-07-18

## Context

将来の DocSail は Mermaid fenced block を Terminal 内で表示することを目指す。ただし v0.1.0 には Mermaid 図形描画を含めず、Node.js やブラウザにも依存しない。

## Decision

Pure Rust Mermaid Backend として `merman` を採用する。v0.3.0 の Spike では flowchart と sequenceDiagram の SVG・PNG 出力、日本語ラベルを含むラスタ化、`ratatui-image` による画像 protocol との統合を確認した。半ブロック表示は図中の文字が実用的な解像度にならないため、画像 protocol を透過できない端末または multiplexer では Mermaid ソースを表示する。

採用後もアプリケーション全体を特定 Backend の API へ直接密結合させず、描画要求・結果・エラーを表す小さな境界を設ける。代替 Backend と Mermaid ソース表示 fallback を可能にする。

## Consequences

`merman` は MIT OR Apache-2.0 であり、DocSail の MIT ライセンスと両立する。描画境界を維持するため、将来の Backend 差し替えと Mermaid ソース表示 fallback を可能にする。v0.3.0 の品質保証対象は flowchart と sequenceDiagram に限る。Herdr 経由の描画は、macOS の Ghostty で `experimental.kitty_graphics = true` を設定した場合だけサポートする。Windows の Herdr における Kitty graphics は未検証である。
