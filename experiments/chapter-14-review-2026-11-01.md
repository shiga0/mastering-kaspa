# SUMMARY順レビュー 18：第14章「コンセンサス順序と仮想状態」

確認日：2026-11-01。基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。本文全体・図・教材・実習導線・既存の照合記録を確認した。

## 本文の修正

- GHOSTDAGの計算はヘッダー検証内で使うことを補い、責務図を本体/UTXO検証完了後の実行順と混同しないようにした。
- 親子の配置と参照方向、親選択の(work, hash)比較、selected parentを含むmerge setの扱いを前章までと整合させた。
- 受理IDの並びのCrescendo切り替えがpov_daa_scoreを使うことを明記。GHOSTDAG/DAAのselected parentのスコアと機械的に同一視しない。
- VirtualStateがUTXO集合だけの名前ではなく、親群、GHOSTDAGデータ、DAA score、past median time、差分、受理ID等を持つと説明した。
- 親集合と共通規則が同じ既存ブロックのselected parentと、新たな先端で変わり得るVirtualのselected parentを区別した。
- 基準状態と差分からのUTXO更新を説明し、毎回Genesisから再実行することを前提としないようにした。
- 教材の逐次的な依存取引と、実Kaspaの同一ブロック内の親子取引の禁止を区別した。モデルの返すErrでの状態維持と、電源断や永続化の耐久性も区別した。
- ソース探索は第1章の固定版を再利用。関連章をリンクにし、編集者向けの古い作業メモを今回の範囲と未確認事項へ置き換えた。

## 教材の修正・実行

本文のRust教材は、暗号学的なID・署名・GHOSTDAG・実際の報酬や手数料を実装しないUTXOの逐次更新モデルである。

空ID・空入力・空出力をInvalidStructureとして拒否する。出力数をu32へ変換できるか保存前に確認し、無条件のasによる切り詰めを避けた。出力番号は確認済みの範囲から作る。

既存の現在UTXOとの出力衝突検査は維持した。ただし過去に使われ、すべて消費された文字列IDの再利用まで検出する履歴管理ではない。再送時の冪等な応答や、取引IDのハッシュ検証も実装しない。

本文のRustブロックを一時Cargo binプロジェクトへ抽出し、rustfmtの結果を本文へ反映。Rust/Cargo 1.90.0、macOS ARM64、edition 2021、外部依存なしでcargo test --offline、cargo fmt --check、cargo clippy --offline --all-targets -- -D warnings、cargo run --offlineを実行した。

7テスト成功：

1. 指定された順序で最初の消費が成功し、競合する消費はMissingInputとなる。
2. 金額不足で元の入力が残る。
3. 重複した入力を拒否する。
4. 内容を変えず同じ2取引の順序を反転すると、残る出力も受理された取引に対応して変わる。
5. 空ID/空入力/空出力、後ろの入力の欠落、入力合計/出力合計のオーバーフロー、金額不足、重複入力、既存出力の衝突という9入力で、拒否時に集合全体が不変となる。
6. 依存先の出力がなければ失敗し、生成元を適用後に再試行すると成功する。
7. 独立した2取引の順序交換で最終集合が一致する。

4〜7が追加のテスト。書式・Clippyも成功。出力は次と完全一致した。

```text
tx1: Ok(())
tx2: Err(MissingInput)
remaining UTXOs: 2
```

出力数のu32上限を超える入力は巨大な配列を要するため実行していない。Errでのメモリ内の状態維持を確認したもので、panic、メモリ不足、並行更新、クラッシュ・電源断の耐久性の保証ではない。cargo newから新規環境で手入力した試験とも区別する。

新しいexamplesフォルダーや一時プロジェクトの生成物は追加せず、CI同期対象も変更していない。

## 固定ソースと既存の証拠

今回読んだ主な箇所：

| ファイル | SHA-256 |
|---|---|
| [consensus/src/model/stores/ghostdag.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/model/stores/ghostdag.rs) | `c23fa656f30227291ee75e17cb0634c71a7a02dcea17fdba435ceef61cea1e18` |
| [consensus/src/model/stores/virtual_state.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/model/stores/virtual_state.rs) | `6e629c93767cc3a14faf6ec8bd6051ac1137effaf848897cf40c7bcc3b40a164` |
| [consensus/src/pipeline/virtual_processor/utxo_validation.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/virtual_processor/utxo_validation.rs) | `06eaec070d3829452b2584aabd58289f5c7c6dac6b7711c91fd8c272d08ea246` |
| [consensus/src/pipeline/virtual_processor/processor.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/virtual_processor/processor.rs) | `e1e92b1925016cba6e032b245e42e5e5e41775e4e41ab10f663110e44617511e` |

GhostdagDataの合意順序、VirtualStateのフィールド、calculate_utxo_stateのselected parentと残りのmerge setの処理・受理IDのactivation条件、commit_virtual_stateの同一WriteBatchへの反映を確認した。ソースのSHA-256は既存の `consensus-state-review-2026-10-31.md` に記録したものと一致した。

オンラインで固定版VirtualStateのソースを参照する試行はCache missとなったため、既存の固定ソースアーカイブから確認した。旧教材3テストと主要経路の照合は上記の既存記録を参照し、今回再実行した公式テストとしては扱わない。

ページの通読と教材検証を、公式実装との実行時差分、報酬生成・全Script/署名・全例外規則、クラッシュ耐久性、現在のMainnetとの一致へ一般化しない。

次はPart III、SUMMARYの19ページ目、第15章 `src/part-03-transactions/chapter-15.md`。Part II第9〜14章の本文確認は一区切りだが、全技術の検証完了ではない。

## 統合・公開物の確認

既存の本文とRustソースの同期対象2件が一致。mdbook build成功、生成HTML53ページの内部ファイル・アンカーのリンク切れ0件。SUMMARYとreview-statusの49項目は名称・順序一致。git diff --checkも成功した。

追跡済み・追加候補164ファイルへの既存の秘密情報パターン検査は検出0件。gitleaks/trufflehogは未導入であり、秘密情報が一切ないとの保証ではない。新しいexamplesフォルダー、生成物、秘密鍵、個人パスは追加していない。
