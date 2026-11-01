# SUMMARY順レビュー 21：第17章「Mempoolとトランザクション伝播」

確認日：2026-11-01。基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。本文全体・図・教材・実習導線を確認し、主要な受付・保留・伝播・ブロック後処理を固定ソースと照合した。全規則や公式関数の実行検証ではない。

## ソースで確認し本文へ反映したこと

- RPC入口はunsafe_rpcとrequest.allow_orphanの両方で孤児の許可を決める。要求のフラグだけでは許可されない。
- 通常RPCはPriority::HighとRbfPolicy::Forbidden。置換用RPCはHigh/Orphan::Forbidden/RbfPolicy::Mandatory。P2P tx relayはLow/Allowed/Allowedを使う。Highは保持・期限・再伝播の扱いで、名前だけで採掘の選択優先度を意味しない。
- RBFの制約はfee/massの閾値を使い、Mandatoryでは競合がちょうど1件であることも要求する。全RBF条件・閾値を実行比較したものではない。
- MiningManagerは読み取りロック下の事前検査・入力補完、ロック外のコンセンサスAPIの検証、書き込みロック下の事後再検査・登録を分ける。事後の重複等の確認が必要になる。
- transaction_pool、orphan_pool、accepted_transactionsを区別。コンセンサス規則の検査はMempool登録前にも行うが、保持したことでUTXOを消費するわけではない。
- RejectMissingOutpointでOrphan::Allowedなら孤児プールへの追加を試み、Forbiddenでは拒否する。親の出力情報の補完と、孤児の解決後の再検証を確認。親子をMempoolに置けることと同一ブロックでの親子取引の許可を区別する。
- tx relayはInvTransactions、RequestTransactions、本体/TransactionNotFoundを扱い、要求IDと受信本体のIDを確認してバッチ登録へ渡す。v5という配置名と交渉したプロトコル番号を同一視しない。
- FlowContextはVirtual処理を待ってからブロックの取引をMiningManagerへ渡す。handle_new_block_transactionsはCoinbaseを除く取引を使って候補・競合の削除、最近のIDの記録、孤児の再評価等を行う。accepted_transactionsという内部名を、Virtual受理や不可逆な確定の独立証拠にしない。
- 公式のpost_validate_and_insert_transactionには、RBFで既存候補を除いた後の容量制限で新規取引が拒否され、先の削除が残る例外のコメントがある。教材の原子性を、公式Mempoolの全受付失敗へ広げない。

## 教材の修正・実行

空の文字列IDを拒否するEmptyIdを追加。取引にPartialEq/Eqを付け、map全体を比較するテストを可能にした。追加は全拒否理由の集合維持・未知ID削除・削除する取引だけの入力予約を解放する3テスト。

本文のRustブロックを一時Cargo libプロジェクトへ抽出し、rustfmtの結果を本文へ反映。Rust/Cargo 1.90.0、macOS ARM64、edition 2021、外部依存なしでcargo test --offline、cargo fmt --check、cargo clippy --offline --all-targets -- -D warningsを実行し成功した。

8テスト成功：

1. 最初の取引を保持する。
2. 同じIDを二重登録しない。
3. 競合する取引を拒否する。
4. 同一取引内の重複入力を拒否する。
5. 削除で入力予約を解放し、別候補を登録できる。
6. 空ID・空入力・既存IDの別内容・重複入力・後ろの入力の競合という5入力で、取引と予約の両map全体が不変となる。
7. 未知IDの削除で他の予約が変わらない。
8. 1件だけ削除しても他の取引の予約は残り、重複削除も無害である。解放した入力を別候補へ予約できる。

同じ文字列IDの別内容もDuplicateTransactionとして拒否するが、暗号学的なID/内容の一致は検証しない。削除でモデル上の予約が空いても、実際のコンセンサスUTXOが使用可能になったという意味ではない。

RBF・孤児・容量/期限・UTXO・署名・mass・ネットワーク伝播・永続化・並行アクセスをこのモデルは実装しない。Errでのメモリ内の集合維持と、プロセス停止や電源断の耐久性は別である。cargo newから新規環境で手入力した試験でもない。

## 導線・記録の整理

lib/editionとリポジトリ外の作業場所を明記。ソース探索は第1章の固定版を再利用する。概念図の矢印・状態名を公式の実行順と同一視せず、送信応答喪失の成否不明、Mempoolの削除と確定の区別も補った。

既存のDevnet Mempool照会・署名変更/競合の拒否は `devnet-send-checks-2026-10-31.md`、送信成功応答の喪失からの照会・同一取引の再送は `devnet-send-retry-2026-10-31.md` を参照した。今回は再実行・鍵生成・署名・採掘・送金・ノード起動を行っていない。

## 固定ソースの識別情報

以下は入口・該当関数・関連箇所を読んだファイルの識別情報で、全行・全分岐の監査完了を表さない。

| ファイル | SHA-256 |
|---|---|
| [rpc/service/src/service.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/rpc/service/src/service.rs) | `b07acf50884a1d4c0361f0227952293012a7af6cf49663be5d8b0aad022ad43f` |
| [mining/src/manager.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/mining/src/manager.rs) | `b839ccb5b70879b0b91ab63de47f61d085e2358361fa6fe6b19147d53640c348` |
| [mining/src/mempool/mod.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/mining/src/mempool/mod.rs) | `d16e17f455ecd23e545c4b057c71914c41d4cb9f1c161513e93459ac251bc085` |
| [mining/src/mempool/validate_and_insert_transaction.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/mining/src/mempool/validate_and_insert_transaction.rs) | `ff6f285b5971c110191736fbda6d4fd1519e7c3643e990bc493c9e95e923cbd8` |
| [mining/src/mempool/populate_entries_and_try_validate.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/mining/src/mempool/populate_entries_and_try_validate.rs) | `4773d659b4838000627dfa557c72802f5b3dfc288da4e36cf525cf2a3de5dff1` |
| [mining/src/mempool/replace_by_fee.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/mining/src/mempool/replace_by_fee.rs) | `b9697e20dac975fed3128b5c074dd18b32b7461865422ce34ab18c01cb8c9c69` |
| [mining/src/mempool/handle_new_block_transactions.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/mining/src/mempool/handle_new_block_transactions.rs) | `f6a3be6cc718f8cee415aea88f8446f03394018d8cf5e6bd5e6285d62c6eb67d` |
| [protocol/flows/src/flow_context.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/protocol/flows/src/flow_context.rs) | `6f1ed18826f7d9c261ee8bb65e6602f919bfbdca5314673fb733776a4e737014` |
| [protocol/flows/src/v5/txrelay/flow.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/protocol/flows/src/v5/txrelay/flow.rs) | `44ad5fedf47a2a4013997c7c7d8821bf5722ea052659b9c770ef956a698719d0` |
| [protocol/flows/src/flowcontext/transactions.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/protocol/flows/src/flowcontext/transactions.rs) | `1f94d340d1c1bac00c5cb393011ea044bdce418e5d9ec10ce3aba87982e6c74f` |

固定版Mempoolのオンライン参照はCache missとなったため、既存の固定ソースアーカイブを読んだ。

全RBF条件・孤児の上限/期限・全再編成時の再投入・全ネットワークの有効設定・取引選択の分布・公式関数との差分実行・現在のMainnetとの一致は未検証。ソースのコメントに記された例外を、今回実測した例外としては扱わない。新しいexamplesフォルダー・生成物を追加せず、CI同期対象も変更していない。

次はSUMMARYの22ページ目、第18章 `src/part-03-transactions/chapter-18.md`。

## 統合・公開物の確認

既存の本文とRustソースの同期対象2件が一致。mdbook build成功、生成HTML53ページの内部ファイル・アンカーのリンク切れ0件。SUMMARYとreview-statusの49項目は名称・順序一致。git diff --checkでは変更した冒頭行の末尾空白を検出し、改行表現を修正後に成功した。

追跡済み・追加候補167ファイルへの既存の秘密情報パターン検査は検出0件。gitleaks/trufflehogは未導入であり、秘密情報が一切ないとの保証ではない。新しいexamplesフォルダー、生成物、秘密鍵、個人パスは追加していない。
