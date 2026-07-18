# DocSail repository guidance

- ユーザー向けの報告は日本語で簡潔に行う。
- Rust標準の `cargo fmt`、Clippy、test を検証に使用する。
- DocSail は単一 crate から開始し、過剰な抽象化や早期の複数 crate 化を避ける。
- 設計図には D2 を使用する。Mermaid は製品のテスト対象であるため、設計図には使用しない。
- Node.js、Chromium、Puppeteer、`mmdc`、WebView、外部画像変換コマンドを追加しない。
- Mermaid の Pure Rust Backend は `merman` を第一候補として Spike で評価する。正式採用前に実装を特定 Backend へ密結合させない。
- 社内文書、実在する社内 URL・組織名、認証情報、個人情報、ローカル絶対パスを fixture や文書へ含めない。
- 依存を追加する前に既存構成・標準ライブラリで代替できないか確認し、追加理由・代替案・ライセンスを確認する。
- 変更は Issue 単位で小さく保つ。1 Issue は原則として 1 PR で完了できる大きさにする。
- Git diff は Hunk へ委譲する。v0.1.0 では Mermaid 描画、Azure DevOps 固有構文の網羅、複数 crate 化を行わない。
