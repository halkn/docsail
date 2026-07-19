# DocSail

DocSail は、Markdown ドキュメントワークスペースを Terminal 内で巡回・閲覧するネイティブ TUI です。Markdown を編集した後に、エディタやブラウザを開かずワークスペース全体を確認できるようにします。

静的サイトジェネレーターやブラウザエミュレーターではなく、Markdown ディレクトリを直接閲覧します。Git diff は Hunk へ委譲します。

## 状態

DocSail は開発中です。v0.1.0 では、ファイルツリーと Markdown プレビューを表示する最小の TUI を提供します。

## 目標

v0.1.0 では、`docsail .` で Markdown のファイルツリーと選択中のプレビューを表示できる最小の縦切りを提供します。

- WSL、Linux、macOS で扱いやすい単一バイナリを目指す
- GitHub Flavored Markdown（GFM）を優先して表示する
- ファイル更新、Terminal resize、日本語・Unicode 幅に対応する
- Node.js、ブラウザ、WebView、外部画像変換コマンドへ依存しない

対象範囲と除外項目は [スコープ](docs/product/scope.md)、実装順は [v0.1.0 Issue 計画](docs/product/v0.1.0-issue-plan.md) を参照してください。

## 使い方

`PATH` にはファイルまたはディレクトリを指定できます。指定しない場合は、カレントディレクトリの `docs/` を優先して開きます。

次のように起動します。

```console
$ docsail .
$ docsail docs/
$ docsail README.md
```

ファイルツリーでは `j` / `k` または矢印キーでファイルを選択します。`Tab` でプレビューへフォーカスを移し、同じキーでスクロールできます。`q` で終了します。

プレビューの確認には [GFM プレビュー確認](docs/gfm-preview-test.md) を利用できます。対応範囲と制限は [スコープ](docs/product/scope.md) を参照してください。

## インストール

GitHub Release で配布するバイナリは、mise の GitHub バックエンドからインストールできます。最新の安定版をグローバルに有効化するには、次を実行します。

```console
$ mise use -g github:halkn/docsail@latest
```

特定の版を使う場合は、`latest` をバージョン番号に置き換えます。

```console
$ mise use -g github:halkn/docsail@0.1.0
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
