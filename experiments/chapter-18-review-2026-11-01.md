# SUMMARY順レビュー 22：第18章「マイニングとブロックテンプレート構築」

確認日：2026-11-01。基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。本文全体・図・教材・実習導線を読み、主要なテンプレート生成経路と照合した。全規則・全経路の監査や公式関数との実行時差分ではない。

## 固定ソースと本文の修正

- get_block_template_callは報酬先の接頭辞を検査し、Scriptとextra dataをMinerDataへまとめる。応答のblockとis_syncedを区別する。
- MiningManagerはキャッシュ利用・miner data変更時の複製変更・新規生成を分ける。変更ではCoinbase payload、必要な出力、Merkle root、時刻、ヘッダーの再計算等を行う。候補の無効取引をMempoolから除いて再試行する経路もある。
- 最初の取引候補の選択はVirtualのread lock取得前で、取得後のUTXO viewで検証する。すべてのMempoolとVirtualを単一のロックで固定するとの説明にしない。
- 通常取引はready frontierから選ぶ。合計mass等によりTakeAllSelector、事前サンプリングしたSequenceSelector、RebalancingWeightedTransactionSelectorへ分岐する。後者等は乱数を使うため、同じ状態から常に同一テンプレートを出すことはコンセンサス上の要件ではない。
- builder.rsの冒頭の長いコメントは入力の年齢やfee/kilobyteを説明するが、実際のbuild_block_template本体はコンセンサスAPIへ委譲する。コメントだけから現在の選択方式を説明せず、manager→ready frontier→selectorを追う。
- 同一ブロック内の親子取引を順番に詰める説明を修正した。Mempoolに親子を保持することと、同じブロックに親子を同梱することは別である。
- build_block_template_from_virtual_stateはVirtualの親群からレベル別の親を構築し、bits、DAA/blue score、blue work、コミットメント等をヘッダーへ使う。Coinbaseを先頭へ追加し、本体Merkle rootを計算する。
- 時刻はpast median time + 1とunix_nowの大きい方、初期nonceは0。CoinbaseはVirtualのmerge setの報酬情報・non-DAA集合等から構築し、現在の候補の通常取引のfeeを単純に同じ候補のCoinbaseへ足す説明と区別した。
- submit_block_callには同期状態やallow_non_daa_blocksに応じた受付判定がある。古い仕事、ローカルの受付、コンセンサス無効、プールのシェア受付を一括りにしない。
- 図の生成方向と実際の参照方向、PoW値とブロック識別子を区別。固定版の再利用、lib/edition、リポジトリ外の作業場所と次章への導線も整理した。

## 教材の修正

以前は数値IDが同じでも入力が異なる候補を二重に採用できた。同一内容の同一IDは1件へまとめ、別内容があるIDはその全候補を除外する教材独自の方針へ変更した。暗号学的なID/内容の整合性検査を代用するものではない。

選択はfee/massの降順、同率でID昇順を維持。比較にはu128の交差積を使い、最大のu64同士の積も表現できる。貪欲選択であり、最大手数料となる組み合わせの最適性は保証しない。候補順の不変性と公式の確率的な選択は別の性質である。

parent_idsは入力をそのまま返し、親の妥当性や選択を検査しない。total_massは通常取引の申告massだけでCoinbaseを含まず、空の選択結果も実ノードのCoinbaseなしブロックを示さない。

## 教材の実行

本文のRustブロックを一時Cargo libプロジェクトへ抽出し、rustfmtの結果を本文へ反映。Rust/Cargo 1.90.0、macOS ARM64、edition 2021、外部依存なしでcargo test --offline、cargo fmt --check、cargo clippy --offline --all-targets -- -D warningsを実行し成功した。

8テスト成功：

1. mass上限を超えない取引を選ぶ。
2. 同じ入力の競合では教材の優先度が高い方を選ぶ。
3. 同一取引内の重複入力を除く。
4. 候補なしでは通常取引の選択結果と合計massが空になる。
5. 同一IDの同一内容を1件へまとめ、別内容の同一IDはすべて除く。
6. 同率・重複・同一IDの別内容を含む候補を通常順・反転順・回転順で与えても、Template全体が一致する。
7. max_mass=0、mass=0、空入力、上限ちょうど、u64::MAXからの加算オーバーフローを確認する。
8. 過大massや重複入力で不採用となった候補が入力を予約せず、後の有効な候補を選べる。

5〜8が追加のテスト。公式selectorの確率分布やノード・ネットワーク・PoWを実行したものではない。cargo newから新規環境で手入力した試験でもない。

## ソースの識別情報

以下は入口・該当関数を読んだファイルのハッシュで、全行・全分岐の照合完了を表さない。

| ファイル | SHA-256 |
|---|---|
| [rpc/service/src/service.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/rpc/service/src/service.rs) | `b07acf50884a1d4c0361f0227952293012a7af6cf49663be5d8b0aad022ad43f` |
| [mining/src/manager.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/mining/src/manager.rs) | `b839ccb5b70879b0b91ab63de47f61d085e2358361fa6fe6b19147d53640c348` |
| [mining/src/block_template/builder.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/mining/src/block_template/builder.rs) | `bd56382a974fa6c7a162ed1681118b42b83ce71a77e85822dba26670d527808d` |
| [mining/src/block_template/selector.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/mining/src/block_template/selector.rs) | `2ef0e923209fa0eb5a143bc82c17743557edb7010043c7ea71a1f74a5da2c40c` |
| [mining/src/mempool/model/transactions_pool.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/mining/src/mempool/model/transactions_pool.rs) | `7b7d610f5fa97cd4e92b67c49668c522ef5f8b29ab19983d454bce0604156f9e` |
| [mining/src/mempool/model/frontier.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/mining/src/mempool/model/frontier.rs) | `b0955564917c6dcf360e3646c02c96d5433dc13a3e05773dab3c733397555f92` |
| [consensus/src/pipeline/virtual_processor/processor.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/virtual_processor/processor.rs) | `e1e92b1925016cba6e032b245e42e5e5e41775e4e41ab10f663110e44617511e` |

固定版builder.rsのオンライン参照はCache missとなったため、既存の固定ソースアーカイブを読んだ。

## 限界と次の対象

Coinbaseの全報酬計算、全選択器の分布・gas/massの全条件、キャッシュ/更新競合、PoWの既知値、全ネットワークの有効設定、公式関数との差分実行、現行Mainnetとの一致は未検証。今回は鍵生成・採掘・ノード起動・署名・送金も行っていない。ソースのコメントやAPIの名称を、実行済みの検証結果に数えない。

新しいexamplesフォルダー・生成物は公開物へ追加せず、CI同期対象も変更していない。

次はSUMMARYの23ページ目、第19章 `src/part-03-transactions/chapter-19.md`。

## 統合・公開物の確認

既存の本文とRustソースの同期対象2件が一致。mdbook build成功、生成HTML53ページの内部ファイル・アンカーのリンク切れ0件。SUMMARYとreview-statusの49項目は名称・順序一致。git diff --checkも成功した。

追跡済み・追加候補168ファイルへの既存の秘密情報パターン検査は検出0件。gitleaks/trufflehogは未導入であり、秘密情報が一切ないとの保証ではない。新しいexamplesフォルダー、生成物、秘密鍵、個人パスは追加していない。
