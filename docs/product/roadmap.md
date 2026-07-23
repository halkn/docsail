# DocSail ロードマップ

## v0.1.0 — Workspace Viewer

日常的に試用できる最小の縦切りを完成させる。完了条件は、`docsail .` で Markdown ファイルツリーとプレビューを表示し、スクロール、ファイル更新の反映、resize 後の継続利用、日本語 Markdown の実用的な閲覧ができること。

実装単位と依存関係は [v0.1.0 Issue 計画](v0.1.0-issue-plan.md) を参照する。

## v0.2.0 — Navigation and Azure Wiki

- 相対 Markdown リンクと見出しアンカー
- Back / Forward と TOC overlay
- Azure DevOps Wiki の `.order` によるページ順序
- ファイル名ファジー検索とページ内検索

## v0.3.0 — Mermaid

- Mermaid fenced block の検出
- `merman` の評価 Spike と Pure Rust Backend の決定
- SVG 生成、`resvg` による RGBA 変換、`ratatui-image` による Terminal 内表示
- Terminal capability detection、非対応時の fallback、source / rendered 切替、メモリキャッシュ、resize 時の再 raster

## v0.4.0 以降の候補

- broken link と見出しアンカーの検証
- Azure DevOps Wiki 互換性診断
- Git 変更ファイルの状態表示と Hunk 連携
- Mermaid 対応図種の拡張、ディスクキャッシュ
- 設定ファイルとキーバインド設定
