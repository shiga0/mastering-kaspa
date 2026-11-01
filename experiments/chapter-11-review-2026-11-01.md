# SUMMARY順レビュー 15：第11章「ブロック処理と検証」

確認日：2026-11-01。対象は `src/part-02-consensus/chapter-11.md`。既存の主要経路照合を踏まえ、本文全体の説明・図・教材・判定範囲を確認した。

## 修正内容

- 冒頭に通読確認日と基準版を示し、ページの確認と全経路の検証を区別した。
- ヘッダーフィールドの親はレベル別であること、Nonceの検査はブロック識別子のハッシュではなくPoW値との比較であることを整理。Pruning pointの項目を補った。
- Timestampのミリ秒単位と、Hash Merkle RootのトランザクションハッシュとAccepted ID Merkle Rootの受理ID列の違いを補足した。教材用ステートマシンを公式の実行順と読み違えない説明も追加した。
- 既知状態は単に処理済みではなく、ヘッダー・本体・UTXOの各段階を分ける説明へ変更した。
- Virtualは規則で選ぶ親群を使い、全tipsの無条件採用や実ブロックの採掘とは異なると補った。
- BlockStatus::is_validはStatusInvalid以外でtrueとなるため、検証完了の確認と混同しないようにした。StatusUTXOValidも全TXの受理や不可逆な決済確定ではない。
- I1の「無効」は永続的なStatusInvalidと対応させ、本体の不一致などすべての処理エラーがそれを意味するわけではないと整理した。
- 教材は同じIDの同一データの再送と、表現上の内容の衝突を区別するConflictingIdを追加。単一段階で保存する教材の方針を、公式のHeaderOnlyからの本体再取得と同一視しないようにした。
- 親到着後の再試行と、全6種類の拒否時に集合全体が不変であるテストを追加。Cargoのlib/editionとリポジトリ外の作業場所を明記した。
- エラー分類の演習ではNoOpを加え、入力拒否・再取得・永続無効化・ピア評価を分けて考えるよう修正した。
- ソース探索・次章リンク・教材実行数を更新し、以前の公式テスト2件と今回の教材7テストを分けて記載した。

## 固定ソースの再確認

基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。主に以下の定義・処理箇所を読んだ。

| ソース | SHA-256 |
|---|---|
| [consensus/core/src/header.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/header.rs) | `3d627075136dba04ae9eb2e48db638f07c9501075d1108e4fb6aadf4f6654c37` |
| [consensus/core/src/blockstatus.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/blockstatus.rs) | `4b9ad49593a6f5831b020916e9b58d75b48ef121b76f06a370a6d942d8dd97b3` |
| [consensus/src/pipeline/header_processor/processor.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/header_processor/processor.rs) | `27d5e5964e41ad4fd47df07a7c02fe5506386a5f91b13d3288578638cfba443c` |
| [consensus/src/pipeline/header_processor/post_pow_validation.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/header_processor/post_pow_validation.rs) | `bbdbf1dba38e3a17418beebe6dbb690b18796e55b91dc92b3cd744ffd2579de6` |
| [consensus/src/pipeline/body_processor/processor.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/body_processor/processor.rs) | `5158abde7814277ddab817737b8577073803ea4e0f43faf8c1d05d410c0947c1` |
| [consensus/src/pipeline/body_processor/body_validation_in_isolation.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/body_processor/body_validation_in_isolation.rs) | `a292d8eaa6a5748dceba7f93a20b0a93cbe0469d915b4e6e8df689ca0755110a` |
| [consensus/core/src/merkle.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/merkle.rs) | `5169ec85adb9116d5cd5862ada0482243ff4f523a1e33062f0910b7591183e7a` |

Headerのparents_by_level・pruning_point、BlockStatusの定義とis_valid、通常ヘッダーの処理順、post_pow_validationのPruning point検査、process_bodyのエラー別の保存方針、本体Merkle検査の入口を再確認した。Merkle葉にトランザクションハッシュを使う定義も確認した。

GitHub経由で固定版blockstatus.rsを参照する試行はCache missとなったため、既存の固定版ソースアーカイブから確認した。既存の照合対象ファイルの識別情報・主要経路・公式本体テスト2件は `core-validation-review-2026-10-31.md` と対応JSONを参照。今回それらを再ビルド・再実行したものではない。

## 改訂教材の実行

本文のRustブロックを一時Cargo libプロジェクトへ抽出。Rust/Cargo 1.90.0、macOS ARM64、edition 2021、外部依存なしでcargo test --offlineとcargo fmt --checkを実行した。

7テスト成功：

1. 根・並行した子・複数親のブロックを登録する。
2. 親不足では登録されない。
3. 同一データの再送でAlreadyKnownとなり、件数が増えない。
4. 自己参照・重複親を拒否する。
5. サイズ制限を超える申告を拒否する。
6. 親不足の子を、親到着後に再試行して登録・再送できる。
7. 空ID、自己参照、重複親、過大サイズ、親不足、同一IDの内容衝突という6種類の拒否で、既存の集合全体が変わらない。

本文のコードを検査したもので、別実装の試験ではない。一時プロジェクト・生成物を公開物へ追加せず、CI同期対象も変更していない。cargo newからの手順を新規環境で通した試験とは区別する。

## 限界

教材のID・payload_sizeは呼び出し側が指定した値で、実データのハッシュや実際の受信サイズの検証ではない。保留キュー・自動的な親の取得・全親上限・永続化・クラッシュ・PoW・署名・GHOSTDAG・全UTXOの規則を実装していない。

ページを通読したことと、プロトコル全体・全異常系を検証したことは別である。既存のチェックリストの未完項目は全条件・全経路の確認を含むため、一括で完了にはしていない。新たな実ノードの起動・通信・採掘・送金も行っていない。

次はSUMMARYの16ページ目、第12章 `src/part-02-consensus/chapter-12.md`。

## 統合・公開物の確認

mdbook build成功。生成HTML53ページの内部ファイル・アンカーのリンク切れ0件。SUMMARYとreview-statusの49項目は名称・順序一致。既存の本文とRustソースの同期対象2件は一致し、git diff --checkも成功した。

追跡済み・追加候補161ファイルへの既存の秘密情報パターン検査は検出0件。gitleaks/trufflehogは未導入であり、秘密情報が一切ないとの保証ではない。新しい教材フォルダーや生成物、個人パスは追加していない。
