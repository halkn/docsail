# DocSail

DocSail は、Markdown ドキュメントワークスペースを Terminal 内で巡回・閲覧するネイティブ TUI です。Markdown を編集した後に、エディタやブラウザを開かずワークスペース全体を確認できるようにします。

静的サイトジェネレーターやブラウザエミュレーターではなく、Markdown ディレクトリを直接閲覧します。Git diff は Hunk へ委譲します。

## 状態

DocSail は開発中です。現在は Rust プロジェクトと `docsail [PATH]` の基本 CLI を初期化した段階であり、ファイルツリーと Markdown プレビューの TUI はまだ実装されていません。

## 目標

v0.1.0 では、`docsail .` で Markdown のファイルツリーと選択中のプレビューを表示できる最小の縦切りを提供します。

- WSL、Linux、macOS で扱いやすい単一バイナリを目指す
- GitHub Flavored Markdown（GFM）を優先して表示する
- ファイル更新、Terminal resize、日本語・Unicode 幅に対応する
- Node.js、ブラウザ、WebView、外部画像変換コマンドへ依存しない

対象範囲と除外項目は [スコープ](docs/product/scope.md)、実装順は [v0.1.0 Issue 計画](docs/product/v0.1.0-issue-plan.md) を参照してください。

## 使い方

現時点では CLI の引数解析のみを提供しています。

```console
$ cargo run -- --help
Usage: docsail [PATH]
```

`PATH` にはファイルまたはディレクトリを指定できます。ワークスペースの解決と TUI 表示は今後のリリースで追加します。

完成時には、次のような起動を想定しています。

```console
$ docsail .
$ docsail docs/
$ docsail README.md
```

## 開発

Rust の安定版ツールチェーンを用意して、リポジトリ直下で実行します。

```console
$ cargo fmt --check
$ cargo clippy -- -D warnings
$ cargo test
```

## ドキュメント

- [ビジョン](docs/product/vision.md)
- [ロードマップ](docs/product/roadmap.md)
- [スコープ](docs/product/scope.md)
- [アーキテクチャ判断記録（ADR）](docs/adr/)

## ライセンス

[MIT](LICENSE)
