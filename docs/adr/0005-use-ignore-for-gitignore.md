# ADR-0005: `.gitignore` の解釈に `ignore` crate を使用する

- Status: Accepted
- Date: 2026-07-18

## Context

DocSail は Markdown ワークスペースの探索結果にリポジトリの `.gitignore` を反映する必要がある。`.gitignore` には glob、ディレクトリ規則、否定パターン、階層ごとの設定がある。

## Decision

`ignore` crate を使用する。標準ライブラリの `std::fs` は再帰走査のみを提供し、`.gitignore` 規則の解釈を提供しないため、独自実装は採用しない。`ignore` は ripgrep と同じプロジェクトで保守され、`.gitignore` を含む ignore file の走査・照合を提供する。採用バージョンは 0.4.30、ライセンスは `Unlicense OR MIT` である。

グローバル ignore、`.ignore`、Git の exclude はワークスペースの再現性を損なうため適用しない。起動対象が `docs/` のようなリポジトリのサブディレクトリであってもリポジトリの `.gitignore` を反映できるよう、起動対象とその親ディレクトリの `.gitignore` を適用する。

## Consequences

Git の規則に近い探索結果を少ない実装量で得られる。一方で、探索機能は `ignore` crate へ依存する。
