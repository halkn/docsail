# ADR-0002: GFM と Azure DevOps Wiki を対象にする

- Status: Accepted
- Date: 2026-07-18

## Context

DocSail は一般的な Markdown ドキュメントと Azure DevOps Wiki の Git リポジトリを閲覧対象とする。個別の静的サイトジェネレーターを解釈すると、設定・ビルド・CSS の再現までスコープが拡大する。

## Decision

Markdown 方言は GFM と Azure DevOps Wiki Markdown を対象とする。v0.1.0 では GFM の主要構文を優先し、Azure DevOps Wiki の `.order` と固有構文の対応は v0.2.0 以降とする。MkDocs、Docusaurus、Sphinx、GitHub Wiki、MDX、reStructuredText の個別対応は行わない。

## Consequences

通常の Markdown ディレクトリを幅広く閲覧でき、主利用対象の Azure DevOps Wiki へ段階的に対応できる。一方、各基盤固有のナビゲーションやビルド結果の再現は保証しない。
