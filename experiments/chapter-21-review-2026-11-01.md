# SUMMARY順レビュー 25：第21章「Pruning・ストレージ・ノード同期」

確認日：2026-11-01。基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。本文全体・図・教材・外部参照と実習導線を確認した。実Pruning・Pruning後IBDは今回実行していない。

## 本文・固定ソースの確認

- 主要経路の照合と目次順の本文確認、実際の削減・Pruning後同期を区別した。過去Devnetの5475ブロック/blue scoreとGenesisのままのpointは `devnet-pruning-review-2026-10-31.md` の記録で、今回の観測ではない。
- UTXOの未使用と成熟などを満たして使用可能であることを区別した。Pruning Point時点という表現も、そのヘッダーの過去UTXO文脈と対象ブロック自身の取引の適用を区別する説明へ補足した。
- masterの定数/設定リンクを固定SHAへ変更。参照名をNEW_PRUNING_DURATION等へ合わせた。未照合のProtocol v11チャンクIBDの断定は削除し、基準版のflowを読む案内へ置き換えた。
- IBDは共有履歴からの同期とDownloadHeadersProofを分ける。毎回proof/UTXOを再取得する状態機械ではない。
- headers proof経路はstaging consensusを作り、proof/ヘッダー/point/時刻/UTXOの処理後にcommitし、エラーでcancelする。
- import_pruning_point_utxo_setはMuHash一致を確認した後、Pruning用UTXOの位置を保存し、VirtualのUTXO集合をクリアして分割コピーし、その後で対象ブロックの通常取引を検証する。単一WriteBatchで全処理を差し替える関数ではない。関数内部の更新と外側staging、公開境界、電源断の耐久性を分けて説明した。
- 同じ未信頼ピアからUTXO集合とハッシュを受け取って一致するだけでは、比較先ヘッダーの正当性を確立できないと補った。
- 固定ソースの再利用、作業場所、依存の版、ログ等の公開範囲、次章と参考文献を整理した。

## 外部参照

[KIP-0006](https://github.com/kaspanet/kips/blob/master/kip-0006.md)のPoChMは、取引等の所属証明の文脈として扱い、Pruning Proofそのものと同一視しない。

[KIP-0015](https://github.com/kaspanet/kips/blob/master/kip-0015.md)の題名を文書に合わせ、Canonical Transaction Ordering and SelectedParent Accepted Transactions Commitmentへ訂正した。文書のStatusと、対象版/ネットワークでの有効化は別の証拠を要する。

[Issue #1100](https://github.com/kaspanet/rusty-kaspa/issues/1100)は追加の敵対的試験と防御に関する提案であり、固定版で試験が完了したとの証拠には数えない。Issue本文の自己申告を本書で独立に実測した結果へ読み替えない。

これらの書誌・文脈を確認したもので、全提案の証明・現行採用状況を全面照合していない。

## 教材の修正

SHA-256教材はMuHashではない。domain/versionの固定タグ（ASCII `mastering-kaspa/snapshot-lab/v1` と末尾NUL1バイト）を付け、BTreeMapのキー順に、u64 little-endianのキー長、UTF-8キー、u64 little-endianの金額をハッシュする独自形式とした。

空のOutpoint文字列と、max_key_bytesを超えるキーを拒否する。件数・キー長はVecの受信/確保後の検査で、ネットワーク受信前の資源上限を実装したことにはならない。文字列の型は本当のOutpointでもなく、金額以外のScript・Coinbase・DAA情報も再現しない。

expectedは呼び出し元が信頼できる文脈から用意すべき値であり、この関数自体はproofやヘッダーを検証しない。Errでのメモリ内の集合維持と、永続化/クラッシュの保証は別である。

## 実行した検査

本文のCargo.tomlとRustブロックを一時Cargo libプロジェクトへ抽出。Rust/Cargo 1.90.0、macOS ARM64、edition 2021、sha2を=0.10.9に固定。キャッシュ済み依存を用いてcargo test --offline、cargo fmt --check、cargo clippy --offline --all-targets -- -D warningsが成功した。rustfmtの結果を本文へ反映した。

今回解決した依存はsha2 0.10.9、digest 0.10.7、block-buffer 0.10.4、crypto-common 0.1.7、generic-array 0.14.7、typenum 1.20.1、version_check 0.9.5、cpufeatures 0.2.17、libc 0.2.191、cfg-if 1.0.5。一時Cargo.lockのSHA-256は `ec7915f459eb85a3ed43501d0655b7bfb05ead2e849972e5d85062bde1f9ff64`。一時プロジェクト・lock・生成物は公開リポジトリへ追加していない。

8テスト成功：

1. 一致するスナップショットへ置き換える。
2. コミットメント不一致で旧集合を保つ。
3. 重複Outpointを拒否する。
4. 件数超過を拒否する。
5. 固定バイト列の独立なSHA-256既知値と一致する。
6. 2件の受信順を反転した初回インポートと、通常順での再インポートで同じ集合になる。
7. 空キー・キー長超過・重複・件数超過・不一致という全5拒否理由で、非空の旧集合全体が変わらない。
8. UTF-8の「あ」は2バイト上限で拒否し3バイトで受理。期待コミットメントが空集合に対応する場合は上限0で空集合へ置換できる。

5〜8が追加テスト。独立期待値はPython hashlibへ、固定タグに続けてhex `0600000000000000`、ASCII `tx-a:0`、hex `6400000000000000` を渡して得た。SHA-256は `145c04ecfe647a47605439ee7633cf31cc92742e6de83a1289ef3665dfaf2aa2`。Rust側で計算した値をそのまま期待値として保存したものではない。

これは教材の独自バイト形式の検査で、公式MuHashや敵対的Pruningインポートの実行試験ではない。cargo newから新規環境で手入力した検査とも区別する。CI同期対象・examplesフォルダーは追加していない。

## ソースの識別情報と限界

以下は今回、入口・該当処理を読んだファイルの識別情報で、全行・全異常系の監査完了を表さない。

| ファイル | SHA-256 |
|---|---|
| [consensus/src/pipeline/virtual_processor/processor.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/virtual_processor/processor.rs) | `e1e92b1925016cba6e032b245e42e5e5e41775e4e41ab10f663110e44617511e` |
| [consensus/src/pipeline/pruning_processor/processor.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/pruning_processor/processor.rs) | `e5b7c2bd6307514b44537bc3e3a719515032692fa2dc3252368b6be9d0204e73` |
| [consensus/src/processes/pruning_proof/validate.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/pruning_proof/validate.rs) | `b94f22b79919669322c558405dc4502e5f0538998bf3fbb466adeb7810126e15` |
| [consensus/src/processes/pruning_proof/apply.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/pruning_proof/apply.rs) | `3c585dd462b9dd34ea3520e18e33d83e7a697ae91d42bdf312264d19571ca9b1` |
| [protocol/flows/src/v5/ibd/flow.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/protocol/flows/src/v5/ibd/flow.rs) | `fc25e488f5db06f147d34ac0d409f6ad37c6607547901de4840c3bfe38b043c8` |

Pruning条件の既存照合と実測の限界は上記のdevnet-pruning-review記録を維持する。全保持規則、staging commit/cancelの障害注入、永続化の耐久性、実際の削減と削減後の新規同期は未検証。今回は鍵生成・ノード起動・採掘・送金も行っていない。

次はSUMMARYの26ページ目、第22章 `src/part-04-development/chapter-22.md`。

## 統合・公開物の確認

既存の本文とRustソースの同期対象2件が一致。mdbook build成功、生成HTML53ページの内部ファイル・アンカーのリンク切れ0件。SUMMARYとreview-statusの49項目は名称・順序一致。git diff --checkも成功した。

追跡済み・追加候補171ファイルへの既存の秘密情報パターン検査は検出0件。gitleaks/trufflehogは未導入であり、秘密情報が一切ないとの保証ではない。新しいexamplesフォルダー、生成物、秘密鍵、個人パスは追加していない。
