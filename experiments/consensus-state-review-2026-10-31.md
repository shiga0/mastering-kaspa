# 第14・15章：基準版ソースの照合記録

確認日：2026-10-31。対象：v1.0.1 / `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。Source-checkedの範囲のみを記録し、公式ノードの実行や差分テストの成功とは扱わない。

## 読んだ固定ソース

- [consensus/src/pipeline/virtual_processor/utxo_validation.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/virtual_processor/utxo_validation.rs) — SHA-256 `06eaec070d3829452b2584aabd58289f5c7c6dac6b7711c91fd8c272d08ea246`
- [consensus/src/pipeline/virtual_processor/processor.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/virtual_processor/processor.rs) — SHA-256 `e1e92b1925016cba6e032b245e42e5e5e41775e4e41ab10f663110e44617511e`
- [consensus/core/src/tx.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/tx.rs) — SHA-256 `52c5fe7980c80ced32861b6ba54a6374581d5fc5d8bd738ce8e380ab04289f65`
- [consensus/src/model/stores/virtual_state.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/model/stores/virtual_state.rs) — SHA-256 `6e629c93767cc3a14faf6ec8bd6051ac1137effaf848897cf40c7bcc3b40a164`
- [consensus/src/processes/transaction_validator/tx_validation_in_utxo_context.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/transaction_validator/tx_validation_in_utxo_context.rs) — SHA-256 `43c445db82fbd046cd2cbcdb90646ea84d5df732985163483ff7f8a004b25ce7`
- [consensus/core/src/utxo/utxo_diff.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/utxo/utxo_diff.rs) — SHA-256 `578a0cd5b2a76cb43b341f2ffda6a149fab2d99cafcb935a20bd879131975d10`
- [consensus/src/model/stores/ghostdag.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/model/stores/ghostdag.rs) — SHA-256 `c23fa656f30227291ee75e17cb0634c71a7a02dcea17fdba435ceef61cea1e18`
- [consensus/src/pipeline/body_processor/body_validation_in_isolation.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/body_processor/body_validation_in_isolation.rs) — SHA-256 `a292d8eaa6a5748dceba7f93a20b0a93cbe0469d915b4e6e8df689ca0755110a`

## 確認と修正

- selected parentを先頭に、blueとredを統合した残りのmerge setをblue work/hashの昇順で処理する。
- selected parentのCoinbaseを先に反映し、通常トランザクションをブロックごとの合成viewで検証する。
- 同一ブロック内の二重消費・親子トランザクションの拒否が並列検証の前提となる。
- Crescendo前後で受理IDのコミットメント用の並びに条件分岐がある。
- virtual親は全tipsの無条件採用ではない。
- virtualの逆差分・chain経路での差分合成と、UTXO/state/selected chainのWriteBatch保存を確認。
- Transaction/Outpoint/UtxoEntryの型とCoinbase成熟・金額・sequence・Script等の検証経路を確認。

## 未確認

公式実装との実行時の差分検証、クラッシュ耐久性、全subnetwork規則、報酬生成・ハッシュ・署名の完全照合、章全体の説明と証明の完全レビューは未実施。

## 第14章の既存教材の実行

Rust 1.90.0、edition 2021で掲載Rustコードを一時ファイルへ抽出し、`rustc --test` で実行。これは教育用モデルのテストであり公式実装との互換性検証ではない。

```text
running 3 tests
test tests::failed_transaction_does_not_mutate_state ... ok
test tests::repeated_input_is_rejected ... ok
test tests::first_spend_wins_in_given_order ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 第15章の既存教材の実行

Rust 1.90.0、edition 2021で掲載Rustコードを一時ファイルへ抽出し、`rustc --test` で実行。これは教育用モデルのテストであり公式実装との互換性検証ではない。

```text
running 4 tests
test tests::duplicate_inputs_are_rejected ... ok
test tests::double_spend_is_rejected ... ok
test tests::failed_transaction_does_not_mutate_state ... ok
test tests::transfer_consumes_input_and_creates_outputs ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
