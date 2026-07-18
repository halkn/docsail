# DocSail のビジョン

## プロダクトの目的

DocSail は、Markdown ドキュメントワークスペースを Terminal 内で巡回・閲覧するネイティブ TUI である。コーディングエージェントや開発者が Markdown を編集した後、エディタやブラウザを開かずに、ワークスペース全体を確認できるようにする。

製品名、リポジトリ名、Cargo package 名、実行バイナリ名はすべて `docsail` とする。

## 提供する価値

- ワークスペース内の Markdown を探索し、ファイルツリーで移動できる。
- 選択したドキュメントを Terminal 幅に合わせて読みやすくプレビューできる。
- ファイル更新を検知し、閲覧中の内容へ反映できる。
- WSL、Linux、macOS で扱いやすい単一バイナリを目指す。

## 製品の位置づけ

DocSail は静的サイトジェネレーター、ブラウザエミュレーター、Git diff ツールではない。サイト固有のビルド設定や CSS を解釈せず、Markdown ディレクトリを直接閲覧する。Git diff は Hunk へ委譲する。

## 将来像

将来はリンク移動、TOC、Azure DevOps Wiki のナビゲーション補助、Mermaid の Terminal 内表示を提供する。Mermaid は Node.js やブラウザへ依存せず、Pure Rust の描画パイプラインで実現することを目指す。
