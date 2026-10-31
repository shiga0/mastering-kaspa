# SUMMARY順レビュー 03：「技術レビューの進捗」

確認日：2026-10-31。対象は `src/SUMMARY.md` の3ページ目 `src/review-status.md`。進捗の記述を、目次・検証記録・集計JSON・workflow設定と照合した。今回プロトコルの検証を新たに実行したものではない。

## 見つかった食い違い

- 目次順のレビューへ移行後も、末尾の「次のレビュー順」が重要章優先の古い並びになっていた。
- 「G-001〜G-010は未実行」という一括表示が、G-005/G-006の実行記録・付録Gと食い違っていた。
- 「継続的なCI検査は2教材が対象」という説明には、追加済みのSQLite観測ストアテストとHTMLビルドが含まれていなかった。
- 第16章・付録A/Dの「異常系は未完了」は、所定の拒否や成功応答喪失の試験済み範囲との区別が不明瞭だった。
- 全ページ表には、この進捗ページ自身がなく、SUMMARYとの対応が48/49ページになっていた。
- Devnetの実測結果が長い1項目へ追記され、検証範囲・限界・記録の対応が読み取りにくかった。

## 修正した構成

1. 目次順の確認状況を冒頭の表へ整理。はじめに・基準版・本ページを確認済みとし、次をエコシステムマップとした。
2. ページを通読して確認する作業、限定範囲の資料照合、個別の実行検証を分けて説明した。Source-checkedの欄と実行結果を分け、章全体のVerifiedへ自動的に広げない。
3. 主な実測を対象別の表へ整理し、それぞれの限界を併記した。Pruningは前提の照合と未達の確認までとして記載した。
4. ページ表をSUMMARYの全49ページと同じ順に整理。本ページを追加し、ソース照合欄の未着手を「未照合」と表記した。
5. 第14/15章の教材3/4テストなど、記録にある範囲を補い、第22/23/28章や付録A/Dの一律な説明を各ページに関連する内容へ改めた。
6. 記録の参照先を1つの表に集約した。初期の記録と後続検証を併せて読むよう案内した。
7. CIの設定対象を列挙し、GitHub上の実行確認や、手動の公式関数比較・Docker実習と区別した。

## 照合した記録

| 内容 | 根拠 |
|---|---|
| ページ数と順番 | `src/SUMMARY.md`、存在する各Markdownファイル |
| 判定の意味 | `src/appendices/appendix-h.md` の検証状態の定義。付録全体のレビューを完了としたものではない |
| G-005/G-006の表示 | `src/appendices/appendix-g.md` の実験ID表とG.10/G.11 |
| GHOSTDAGの照合・実行比較 | `ghostdag-baseline-review-2026-10-31.md`、`ghostdag-comparison-2026-10-31.md`、`ghostdag/observed-2026-10-31.json` |
| DAAの27/120/288ケース | DAA各検証記録、`daa/`・`daa-window/`・`daa-transition/` の集計JSON |
| 本体検証2件と第11/13/20章教材 | `core-validation-review-2026-10-31.md`、対応するsources JSON |
| 第14/15章教材3/4テスト | `consensus-state-review-2026-10-31.md` |
| 第16/24章の初期ソース照合 | `signing-wallet-review-2026-10-31.md` と後続のDevnet送金記録 |
| 送金の所定異常系 | `devnet-send-checks-2026-10-31.md`、`devnet-send-retry-2026-10-31.md` と対応する公開JSON |
| 2ノード・分断・報酬UTXO消失 | `devnet-two-node-2026-10-31.md`、`devnet-partition-2026-10-31.md`、`devnet-reorg-observer-2026-10-31.md` |
| SQLite観測・再照会 | 通知・live reconciliationの記録、`devnet/test_utxo_store.py` |
| Pruning未達 | `devnet-pruning-review-2026-10-31.md`、`devnet/pruning-observed-2026-10-31.json` |
| CI設定 | `.github/workflows/deploy.yml`、`scripts/check-book-examples.py`、両Rust教材の構成 |

ファイル名だけの記録は `experiments/` からの相対パス。古い記録の当時の未実施項目を削除したり、集計JSONを書き換えたりしていない。G-005/G-006以外のIDと既存の実験の対応・完了条件の照合は、付録の順番レビューでも確認する。

## 残る範囲と次のページ

個々の主張を再度すべて証明したり、既存の実行を再実施したりした記録ではない。根拠のない章全体のReviewed/Verified判定、GitHub上のジョブ成功、現在のMainnet状態の確認は追加していない。

次の対象はSUMMARYの4ページ目 `src/ecosystem-map.md`。関連ページの古い表記は、それぞれの順番レビューで既存記録との対応を確認する。

## 修正後の確認

全49ページの順・ラベル・参照先がSUMMARYと一致した。記録ファイルの存在、集計JSONの24/27/120/288ケース・公式テスト2件・教材5/5/4件、Pruning未達、SQLiteテスト7件とworkflow設定も照合した。mdBookビルド、生成HTML 53ページの内部リンク・アンカー、既存2教材の本文一致、差分の空白検査は成功。公開候補の秘密情報・ローカルホームパスのパターン検査で検出なし。
