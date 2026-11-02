# 執筆・技術レビューの管理資料

本書の執筆者・メンテナー向けに、付録Hから移した管理形式・テンプレートをまとめます。
読者向けの参考文献・判定基準・変更履歴は [付録H](../src/appendices/appendix-h.md)、
投稿方法は [CONTRIBUTING](../CONTRIBUTING.md) を参照してください。

以下は管理方法の参考資料です。従来のチェックリストは2026-11-01のレビュー時点の記録であり、
未チェック欄を現在の必須作業へ追加するものではありません。現在の作業範囲は [ROADMAP](../ROADMAP.md) に従います。

## 変更履歴の設計

書籍の変更を「文章修正」と「技術的な意味の変更」に分ける。

| 変更種別 | 例 | 必要な確認 |
|---|---|---|
| Editorial | 誤字、見出し、表現 | 編集レビュー |
| Clarification | 用語・図解の改善 | 内容の整合性 |
| Technical correction | 誤った数式・コード修正 | 一次資料とテスト |
| Version update | Rust/Kaspaリリース更新 | 差分と互換性 |
| Protocol update | KIPの有効化・コンセンサス変更 | ネットワーク別確認 |
| Experiment update | 実測値・再現手順更新 | ログ・環境・証拠 |

## Gitによる変更履歴の追跡

```bash
# 書籍リポジトリで実行
git status --short
git log --oneline --decorate -n 15

# 特定章の履歴
git log --follow -- src/part-05-advanced/chapter-34.md

# 差分確認
git diff -- src/appendices/appendix-h.md

# 以下はレビュー後に必要な場合のコミット例（今回自動実行していない）
git add src/appendices/appendix-h.md
git commit -m "docs: add bibliography and review policy"
```

公開後はGitタグで書籍版を固定すると、読者が同じ内容を再参照しやすくなる。

## リリースノートの雛形

```markdown
# Mastering Kaspa v0.x.y

公開日: YYYY-MM-DD
対象 Kaspa ノード: <version + commit>
対象 Rust: <rustc --version>

## Added
- ...

## Changed
- ...

## Fixed
- ...

## Known limitations
- 実行していないコード例
- 未確認のKIP/ネットワーク有効化条件

## Verification
- cargo test: <結果>
- mdbook build: <結果>
- Devnet: <実験ID>
```

## ソース・実験・章の追跡可能性

```text
       論文 / KIP / 公式仕様
                 │
                 ▼
         固定コミットの実装
                 │
                 ▼
            テスト・実験
                 │
                 ▼
         章・節・図・コード
                 │
                 ▼
           レビュー記録
                 │
                 ▼
           公開リリース
```

最低限の対応表：

| Claim ID | 主張 | Source ID | Code commit | Experiment ID | Review |
|---|---|---|---|---|---|
| CLM-001 | GHOSTDAGの特定の性質 | P-02 | 未記入 | G-005 | Unverified |
| CLM-002 | UTXO二重消費の拒否 | K-03 | 未記入 | G-003 | Unverified |
| CLM-003 | ZK検証機能の存在 | K-05/K-04 | 未記入 | 未記入 | Unverified |

この表は**管理形式の例**であり、検証済みの主張一覧ではない。

## 技術レビューIssueのテンプレート

```markdown
## 対象
- Chapter / Section:
- Claim ID:

## 問題
- 現在の記述:
- 疑義・再現条件:

## 一次資料
- 論文/KIP:
- Commit/Tag:
- 関連テスト:

## 対応
- [ ] 記述修正
- [ ] コード修正
- [ ] 実験追加
- [ ] 参照リンク修正

## 判定
- Unverified / Literature-checked / Source-checked / Verified
```

## 執筆と技術レビューの分離

**執筆**では、概念を理解できるようにする。**技術レビュー**では、その説明が資料と実装に照らして正しいかを確認する。二つの作業を混ぜると、読みやすいが誤った説明が残りやすい。

推奨する流れ：

1. 章の目的と読者の前提知識を定義。
2. 一次資料を収集して版を固定。
3. 概念図と本文を作成。
4. 数式・コード・数値に出典IDを付ける。
5. コードを実行し、実験IDを付ける。
6. 別の視点でレビュー。
7. 修正履歴を残して公開。

## 出版前の確認リスト

今回の本文/リンク確認と、全技術の検証・ライセンス全面監査・公開作業は別。チェックは明示した範囲だけを示す。

- [x] 導入4ページ・全36章・Argent補足・付録A〜Hの49ページがSUMMARY順に対応する。
- [x] ローカルmdBook 0.5.4でHTMLビルドが成功する。
- [x] ビルドした53 HTMLのローカルリンク/アンカー検査で不整合0（外部URLの全件確認ではない）。
- [ ] 主要な数式・定数・KIP状態に一次資料がある。
- [ ] ソースコード参照はタグ・コミットで固定されている。
- [ ] Rustコードはコンパイル・テスト結果を記録している。
- [ ] Devnetの実験結果に環境情報がある。
- [ ] 研究段階の構想をメインネット実装済みと混同していない。
- [ ] サードパーティのコード・図のライセンスを確認した。
- [ ] 変更履歴と既知の制限事項を公開している。

## mdBookへの組み込み

ファイルの配置：

```text
mastering-kaspa/
└── src/
    ├── SUMMARY.md
    └── appendices/
        └── appendix-h.md
```

`src/SUMMARY.md` の例：

```markdown
- [Appendix H — 参考文献・変更履歴](appendices/appendix-h.md)
```

ビルド：

```bash
mdbook build
mdbook serve --hostname 127.0.0.1 --port 3000
```

## 継続的なメンテナンス

Kaspaの実装・KIP・SDKは更新される。公開後も、少なくとも**新しい主要リリース時**に関連章を見直す。更新対象は「値の変更」だけではなく、用語、互換性、セキュリティ前提、RPC応答、テスト手順なども含む。

自動化できる部分：リンクチェック、Markdown lint、Rustテスト、mdBookビルド、参照タグの存在確認。自動化しにくい部分：論文の仮定、コンセンサスの意味、脅威モデル、実際のネットワーク有効化条件。

