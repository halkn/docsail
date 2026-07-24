# DocSail

DocSail は、Markdown ドキュメントワークスペースを Terminal 内で巡回・閲覧するネイティブ TUI です。Markdown を編集した後に、エディタやブラウザを開かずワークスペース全体を確認できるようにします。

静的サイトジェネレーターやブラウザエミュレーターではなく、Markdown ディレクトリを直接閲覧します。Git diff は Hunk へ委譲します。

## 状態

DocSail は開発中です。v0.3.0 では、flowchart と sequenceDiagram の Mermaid fenced block を Terminal 内へ描画します。`m` で Mermaid の描画表示とソース表示を切り替えられます。画像 protocol を透過できない端末または multiplexer では Mermaid ソースを表示します。

## 目標

`docsail .` で Markdown ワークスペースを開きます。

- WSL、Linux、macOS で扱いやすい単一バイナリを目指す
- GitHub Flavored Markdown（GFM）を優先して表示する
- ファイル更新、Terminal resize、日本語・Unicode 幅に対応する
- `Enter` で最初の内部 Markdown リンクを開き、`[` / `]` で履歴を移動する
- `t` で TOC、`f` でファイル検索、`/` で本文検索を開く
- `m` で現在文書の Mermaid を描画表示とソース表示で切り替える
- Node.js、ブラウザ、WebView、外部画像変換コマンドへ依存しない

対象範囲と除外項目は [スコープ](docs/product/scope.md)、実装順は [ロードマップ](docs/product/roadmap.md) を参照してください。

## 使い方

`PATH` にはファイルまたはディレクトリを指定できます。指定しない場合は、カレントディレクトリの `docs/` を優先して開きます。

次のように起動します。

```console
$ docsail .
$ docsail docs/
$ docsail README.md
```

ファイルツリーでは `j` / `k` または矢印キーで行を選択します。ディレクトリは `Enter`、ページ配下のツリーは `Space` で折りたたみ・展開できます。選択行はペイン内へ自動スクロールします。`Tab` でプレビューへフォーカスを移し、同じキーでスクロールできます。`q` で終了します。

プレビューの確認には [GFM プレビュー確認](docs/gfm-preview-test.md) を利用できます。対応範囲と制限は [スコープ](docs/product/scope.md) を参照してください。

## インストール

GitHub Release で配布するバイナリは、mise の GitHub バックエンドからインストールできます。最新の安定版をグローバルに有効化するには、次を実行します。

```console
$ mise use -g github:halkn/docsail@latest
```

特定の版を使う場合は、`latest` をバージョン番号に置き換えます。

```console
$ mise use -g github:halkn/docsail@0.2.0
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
