# ADR-0004: `merman` を Mermaid Backend 候補として評価する

- Status: Proposed
- Date: 2026-07-18

## Context

将来の DocSail は Mermaid fenced block を Terminal 内で表示することを目指す。ただし v0.1.0 には Mermaid 図形描画を含めず、Node.js やブラウザにも依存しない。

## Decision

Pure Rust Mermaid Backend の第一候補として `merman` を評価する。正式採用は v0.3.0 の Spike 後に決定する。評価では Mermaid.js 互換性、対応図種、SVG 出力品質、日本語ラベル、API 安定性、性能、ライセンス、保守状況、`resvg` との統合容易性を確認する。

採用後もアプリケーション全体を特定 Backend の API へ直接密結合させず、描画要求・結果・エラーを表す小さな境界を設ける。代替 Backend と Mermaid ソース表示 fallback を可能にする。

## Consequences

Mermaid 実装のリスクを MVP から切り離し、評価結果に基づく選定ができる。一方、v0.3.0 の着手前に実現可能性・ライセンス・Terminal 互換性を検証する Issue が必要となる。
