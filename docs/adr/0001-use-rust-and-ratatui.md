# ADR-0001: Rust と Ratatui を採用する

- Status: Accepted
- Date: 2026-07-18

## Context

DocSail は WSL、Linux、macOS で使える Terminal-native の Markdown ワークスペースビューアである。起動速度、リソース効率、単一バイナリでの配布しやすさ、Terminal UI とファイル監視への適性が求められる。

## Decision

実装言語に Rust を、TUI フレームワークに Ratatui を採用する。Terminal 入出力は Crossterm を候補として、実装開始時に最新安定版・互換性・保守状況・ライセンスを確認して選定する。初期構成は単一 crate とする。

## Consequences

ネイティブ配布とクロスプラットフォーム対応を進めやすい。一方で依存の API・ライセンスを追加時に確認し、TUI の状態・イベント処理をテスト可能な単位へ分離する必要がある。
