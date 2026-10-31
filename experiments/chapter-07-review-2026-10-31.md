# SUMMARY順レビュー 11：第7章「BlockDAG入門」

確認日：2026-10-31。対象は `src/part-01-foundations/chapter-07.md`。定義・図・本文DAG教材を確認した。

## 修正内容

- Bitcoinは候補全体に分岐を持ち、その中から採用チェーンを選ぶという説明に揃えた。
- Past/Futureは参照関係の集合とし、対象自身を除く規約を明記。時計の過去・未来ではないことを補った。
- Anticoneと反鎖（antichain）の混同を修正。あるブロックに対して比較不能な集合でも、その集合内のブロック同士には祖先・子孫関係があり得る。Cとその子Xが、並行するBのAnticoneに共に属する反例を説明・テストした。
- 基準版Reachability APIの自己包含の祖先判定と、教材の自己を除くPastを区別した。
- チェーンとDAGの図は上を親・下を子へ揃え、実参照が子から親であることを説明した。
- 教材のpastは存在しないIDでHashMapの添字アクセスによりpanicする実装だったため、Resultでエラーを返すようにした。in_anticoneも両IDを検査してから判定し、未知ID同士を単にfalseとしないようにした。
- HashSetの任意の走査順をそのまま表示していた箇所を、表示用に整列する形へ変更。表示の辞書順をコンセンサス順序と混同しない説明を補った。
- 登録済みの親だけを参照し、登録後は親を変更しないという非巡回性の前提を明示。複数の親なしブロックを許す一般的な教材であり、KaspaのGenesisや親選択の規則は再現していないことも明記した。
- 第12章の既存DAG教材へ導線を追加した。既存教材を変更・再収録したものではない。

## 出典と範囲

- [PHANTOM and GHOSTDAG](https://eprint.iacr.org/2018/104)：ePrintレコード・要旨を確認。PDFは403で取得できず、今回本文の全定義や安全性証明を確認したものとは扱わない。
- [Rust HashSetの資料](https://doc.rust-lang.org/std/collections/struct.HashSet.html)：集合型の一般的な挙動の参考。1.90.0の固定URLは取得できなかったため、取得できた標準資料と、Rust 1.90.0での実行を区別する。
- 既存の `examples/dag-relations/src/lib.rs`：自身を除くPast、Anticone、登録順の教材との説明の対応を確認。公式実装と同一という意味ではない。
- 基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` の[ReachabilityService](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/model/services/reachability.rs)：祖先判定に自身を含む規約のコメントを確認。

ReachabilityファイルのSHA-256：`737aeed155f7d35692fbc8a08eda9c10889bb7279ec4a48ce7ebd1f4d02414c2`。

集合の定義と反例は教材のグラフとして検査した。公式Reachabilityのデータ構造やクエリの全経路を検証した記録ではない。

## 教材の実行

本文のRustブロックを一時ファイルへ抽出。Rust 1.90.0、macOS ARM64、edition 2021、標準ライブラリのみで通常実行・rustc --test・rustfmt --checkを確認した。

標準出力は本文と一致した。

```text
D's past: ["A", "B", "C", "G"]
```

5テスト成功：

1. Pastの正確な集合、推移的祖先、自己除外、Genesisの空集合、祖先・子孫や同一IDをAnticoneとしないこと。
2. 対象BのAnticoneにCとその子Xが共に入る反例。B/Cの対称性も確認。
3. 重複ID・自己参照・未知の親・重複親を拒否し、拒否の前後でDAG全体が同一であること。
4. 未知IDのPast、片側・両側とも未知IDのAnticone照会をエラーとすること。
5. 並行ブロックの登録順と親配列順を逆にしても、5ブロックのPastと全25組のAnticone判定が一致すること。

実行・テストしたコードは最終本文から抽出したもの。一時的なコード・バイナリは公開物へ重複収録せず、CI同期検査の対象も増やしていない。cargo new/run/testの新規環境での手順全体は今回別途実行していない。

## 残る範囲

PoW、暗号学的なID、P2P、GHOSTDAG、UTXO、Pruning、巨大DAG・悪意ある入力への耐性、公式実装の差分比較は教材の対象外。任意のグラフを後から編集して閉路を検出する実装でもない。

次はSUMMARYの12ページ目、第8章 `src/part-01-foundations/chapter-08.md`。

## 本文反映後の確認

mdBookビルド、生成HTML 53ページの内部リンク・アンカー、SUMMARY全49ページと進捗表の順・ラベル、既存2教材の本文一致、差分の空白検査は成功。公開候補157ファイルの秘密情報・個人ホームパスのパターン検査で検出なし。新たな実ノードの起動・採掘・送金は行っていない。
