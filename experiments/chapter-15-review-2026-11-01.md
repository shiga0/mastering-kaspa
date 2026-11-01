# SUMMARY順レビュー 19：第15章「UTXOモデルとトランザクション処理」

確認日：2026-11-01。基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。本文全体・図・数値例・教材・実習導線・既存のソース照合記録を確認した。

## 本文の修正

- 変更出力は必須ではなく、入力と全出力の差額が手数料になると補った。理解度チェックと「差額」の説明も合わせた。数値例を実際の必要手数料と誤解しない説明を追加。
- KAS/sompiの比率に固定版定数へのリンクを付けた。UTXO選択の例には手数料も賄えることが前提と補足。
- 未使用と即時に使用可能を区別し、Coinbase成熟に必要な3つのスコアの役割を示した。u64のオーバーフロー検査だけで公式のMAX_SOMPI検査を再現したことにならないと明記した。
- 型・ID・Scriptの全面照合を第16章で完了すると予告する表現を、同章の確認範囲への案内へ変更。
- 取引のライフサイクルは代表的な経路とし、すべての段階を各ノードが必ず観測するわけではないと説明した。
- Mempoolの保持とコンセンサス受理・決済確定を区別。Virtualの更新図は旧差分の巻き戻しと再評価を含む表現にした。
- Script検証に参照UTXOの情報が必要なため、責務図で入力解決を先に示した。厳密な公式関数の呼び出し順とは区別した。
- 固定版のソース探索は第1章の取得済みソースを再利用。lib/edition、リポジトリ外の作業場所、前後章・主要経路へのリンクを整理。

## 教材の修正

以前のinsert_genesisは同じOutpointを上書きできた。初期状態を作る補助と分かるinsert_initial_outputへ改名し、空ID・既存出力をエラーとして、失敗時に変更しないようにした。通常取引のapplyも空IDを拒否する。

出力番号は既存コードですべて検査してから更新していたため、後半のasが単独で危険な切り詰めを起こしていたわけではない。今回、その検査済みという前提をtry_from/expectで明示した。

genesis・100・ownerは教材用の入力で、実KaspaのGenesis報酬や署名を再現しない。ownerは単なる文字列で、支出権限を検証しない。衝突検査は現在のUTXO集合だけで、消費済みIDの履歴管理や再送への冪等な応答は持たない。

## 実行した検査

本文のRustブロックを一時Cargo libプロジェクトへ抽出し、rustfmtの結果を本文へ反映。Rust/Cargo 1.90.0、macOS ARM64、edition 2021、外部依存なしでcargo test --offline、cargo fmt --check、cargo clippy --offline --all-targets -- -D warningsを実行し、成功した。

7テスト成功：

1. 入力を消費し、送金先・変更出力を作り、手数料5を返す。
2. 同じ入力を使う後続取引をMissingInputで拒否する。
3. 金額超過の取引で集合全体が変わらない。
4. 重複入力を拒否する。
5. 初期出力の重複・空IDを拒否し、既存集合を保つ。
6. 空ID・空入力・空出力・重複入力・後ろの入力の欠落・入力合計/出力合計のオーバーフロー・既存出力の衝突・金額超過という9入力で、エラーと集合全体の不変性を確認する。
7. 依存先の出力がなければ失敗し、生成元の適用後に成功する。手数料は5と10で、残る出力35と50、合計手数料15の和が初期値100になる。残るOutpoint・owner・金額も集合全体で一致する。

5〜7が追加テスト。依存取引の逐次適用は教材の動作で、同一Kaspaブロック内の親子取引を許す説明ではない。原子性は通常のErrでのメモリ内の状態維持であり、panic・電源断・永続化の耐久性ではない。

出力番号の範囲を超える巨大配列、公式MAX_SOMPI・成熟・mass・Script・署名・並行更新を今回の実行検査へ含めない。cargo newから新規環境で手入力した試験でもない。新しいexamplesフォルダーや生成物は追加せず、CI同期対象も変更していない。

## 固定ソースの確認

| ファイル | SHA-256 |
|---|---|
| [consensus/core/src/constants.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/constants.rs) | `7039d7e758d65d1848b9c27ebdd1cbdc31eb06be12f52f40914921e7c1298123` |
| [consensus/core/src/tx.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/tx.rs) | `52c5fe7980c80ced32861b6ba54a6374581d5fc5d8bd738ce8e380ab04289f65` |
| [consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs) | `f518e6ec878645b6b563c2be6528b4bef0154960f19d6f9d40093f340b7a0ddc` |
| [consensus/src/processes/transaction_validator/tx_validation_in_utxo_context.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/transaction_validator/tx_validation_in_utxo_context.rs) | `43c445db82fbd046cd2cbcdb90646ea84d5df732985163483ff7f8a004b25ce7` |

constants.rsのSOMPI_PER_KASPA/MAX_SOMPI、tx.rsのUtxoEntry、文脈非依存の検査入口、UTXO文脈の金額・成熟・Script・Mempool用feerate条件の境界を再確認した。

固定版constants.rsをオンライン参照する試行はCache missとなったため、既存の固定ソースアーカイブで確認した。tx.rsとUTXO文脈のソースは `consensus-state-review-2026-10-31.md` に記録済みのSHA-256と一致する。旧教材4テストと主要な状態処理は同記録を参照する。

今回、公式実装との実行時差分、全subnetwork・全Script、署名、報酬生成、クラッシュ耐久性、現在のMainnetとの一致は検証していない。ページの通読と教材の実行を、全規則の検証と一括りにしない。

次はSUMMARYの20ページ目、第16章 `src/part-03-transactions/chapter-16.md`。

## 統合・公開物の確認

既存の本文とRustソースの同期対象2件が一致。mdbook build成功、生成HTML53ページの内部ファイル・アンカーのリンク切れ0件。SUMMARYとreview-statusの49項目は名称・順序一致。git diff --checkも成功した。

追跡済み・追加候補165ファイルへの既存の秘密情報パターン検査は検出0件。gitleaks/trufflehogは未導入であり、秘密情報が一切ないとの保証ではない。新しいexamplesフォルダー、生成物、秘密鍵、個人パスは追加していない。
