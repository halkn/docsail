# ADR-0003: Node.js およびブラウザ依存を避ける

- Status: Accepted
- Date: 2026-07-18

## Context

DocSail は WSL を含む開発環境で、追加セットアップの少ない単一バイナリとして使える必要がある。Mermaid の一般的な描画手段には Node.js やブラウザを必要とするものがある。

## Decision

Node.js、`mmdc`、Chromium、Puppeteer、WebView、外部ブラウザ、外部画像変換コマンドへ依存しない。Mermaid を実装する場合も Pure Rust で SVG、RGBA pixel buffer、Terminal graphics protocol まで扱う経路を検討する。

## Consequences

導入・運用負荷と外部プロセスへの依存を抑えられる。一方、Mermaid の互換性と描画品質は Rust の Backend に制約されるため、採用前の Spike が必要となる。
