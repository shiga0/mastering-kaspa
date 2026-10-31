# 第11・13・20章：検証パイプライン・DAA・Finalityの主要経路レビュー

確認日：2026-10-31。基準版はv1.0.1、コミット `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。ソースは前回のGHOSTDAG実行比較と同じ固定アーカイブで、公式コードは改変していない。18ファイルのSHA-256と教材・公式テストの結果は `core-validation-review-sources-2026-10-31.json` に記録する。

**後続の実行検証：** 同日の `daa-comparison-2026-10-31.md` でFull/Sampled算術27ケースを実行比較した。本記録の「DAA数値差分は範囲外」は主要経路レビュー時点の範囲を示す。

## 第11章で確認・修正したこと

- 通常ヘッダーは、既知状態 → 単独検査（申告bitsへのPoWを含む）→ 親関係 → GHOSTDAG → pre_pow/post_pow検査 → 保存、と進む。本体より先にGHOSTDAG計算・ヘッダー保存が行われる。
- `pre_pow_validation` は期待DAA score/bitsなどを検査するが、名前に反して申告bitsへのPoW検査より後。単独検査と履歴依存の難易度検査を区別する。
- 本体単独検査でHash Merkle Root、Coinbase、mass、TX重複、同一ブロック内の二重消費・連鎖消費を確認する。文脈検査は親の本体・Coinbase制約・lock time等であり、過去UTXOに対する検証全体の完了ではない。
- ヘッダー保存は `StatusHeaderOnly`、本体保存は `StatusUTXOPendingVerification`。UTXO Commitment/Accepted ID Merkle Rootの一致は `verify_expected_utxo_state` で別に検査する。
- 本体側の `MissingParents`・`BadMerkleRoot`・`PrunedBlock` ではブロック全体を `StatusInvalid` と記録しない。正しい本体の再取得や依存の到着を許す。ヘッダー側も失敗した段階によって無効状態を保存する扱いが異なる。
- v5 relay flowの親不足は `process_orphan` と再投入の経路を持つ。コンセンサスのエラーとP2Pでの取得・再試行を分ける。
- `block_task` と `virtual_state_task`、通常とtrusted、Genesisを区別した。HeaderOnlyとDisqualifiedを単純なValid/Invalidへ畳み込まない。

## 第13章で確認・修正したこと

- `DualWindowManager` はselected parentのDAA scoreとCrescendo activationでFull/Sampledを切り替える。基準版Devnet既定はCrescendo無効なのでFull側。
- Full窓はGHOSTDAG・親チェーン・merge setから構築し、単なる受信順や選択親チェーンのブロック数ではない。DAA scoreはselected parentの値にmerge setのnon-DAA以外の件数を加える。
- Fullのmin window size未満ではGenesis bitsを返す。通常計算では最小timestampの要素を1つ除き、残ったtargetを整数平均し、除く前の時間差（最低1ms）と目標時間を使って新targetを求める。
- 同一timestampでのblue work/hashによる順序、Uint320の整数演算順とcompact bitsへの変換、最大targetによる制限を明記。Sampled側のsample rateとactivation直後の別処理をFullと分けた。
- 掲載Rust教材は相対difficultyをf64で更新する独自モデル。「targetを更新」という説明を直し、倍率の0.25〜4制限を公式Kaspaの規則として読ませない。

## 第20章で確認・修正したこと

- `calc_finality_point` はblue scoreによる深さとselected parent chainを使い、適用depthはselected parentのDAA scoreで選ぶ。depth増加時も過去の点から進める設計で、DAA scoreの単純な引き算ではない。
- 基準版Devnet既定はprior finality depth 86400、pruning depth 185798、Crescendo無効。Crescendoの `NEW_FINALITY_DURATION=43200`秒と `NEW_PRUNING_DURATION=108000`秒をそのままDevnetの待ち時間として示していた記述を修正。
- Crescendoのfinality depthはBPS×duration、pruning depthはduration由来の値と安全性の下限式の大きい方。単に「30時間でPruning」としない。
- `virtual_finality_point` とsink候補のchain祖先条件・UTXO評価を追跡した。finality違反候補をVirtual chainから除外することと、ブロックを一律にInvalidにすることを区別した。
- Pruning point一覧に対するfinality検査は、ソースに浅い違反のfalse negativeの可能性が記されている。この一関数であらゆる履歴改変を検出できるとは主張しない。
- master参照を固定し、入金確認ポリシーや現在のMainnetとの混同を避けた。

## 一次資料

- [consensus/src/consensus/mod.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/consensus/mod.rs)
- [consensus/src/pipeline/header_processor/processor.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/header_processor/processor.rs)
- [consensus/src/pipeline/header_processor/pre_ghostdag_validation.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/header_processor/pre_ghostdag_validation.rs)
- [consensus/src/pipeline/header_processor/pre_pow_validation.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/header_processor/pre_pow_validation.rs)
- [consensus/src/pipeline/header_processor/post_pow_validation.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/header_processor/post_pow_validation.rs)
- [consensus/src/pipeline/body_processor/processor.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/body_processor/processor.rs)
- [consensus/src/pipeline/body_processor/body_validation_in_isolation.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/body_processor/body_validation_in_isolation.rs)
- [consensus/src/pipeline/body_processor/body_validation_in_context.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/body_processor/body_validation_in_context.rs)
- [consensus/src/pipeline/virtual_processor/processor.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/virtual_processor/processor.rs)
- [consensus/src/processes/difficulty.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/difficulty.rs)
- [consensus/src/processes/window.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/window.rs)
- [consensus/src/processes/block_depth.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/block_depth.rs)
- [consensus/core/src/blockstatus.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/blockstatus.rs)
- [consensus/core/src/config/params.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/params.rs)
- [consensus/core/src/config/constants.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/constants.rs)
- [consensus/core/src/config/bps.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/bps.rs)
- [protocol/flows/src/v5/blockrelay/flow.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/protocol/flows/src/v5/blockrelay/flow.rs)
- [consensus/src/pipeline/virtual_processor/utxo_validation.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/virtual_processor/utxo_validation.rs)

## 実行した検査

環境はmacOS ARM64、Rust 1.90.0。公式workspaceのCargo.lockを維持した。公式本体検証テスト2件は次の形で個別に実行した。

```bash
cargo test --locked --offline --jobs 2 -p kaspa-consensus --lib pipeline::body_processor::body_validation_in_isolation::tests::validate_body_in_isolation_test -- --exact
cargo test --locked --offline --jobs 2 -p kaspa-consensus --lib pipeline::body_processor::body_validation_in_isolation::tests::merkle_root_missing_parents_known_invalid_test -- --exact
```

各コマンドで1件成功、45件はfilter対象外。最初は本体単独検査の複数の拒否条件を確認する公式テスト。後者はPoWを無効にした公式TestConsensusで、BadMerkleRoot/MissingParentsが再投入時にKnownInvalidへ変わらず、対応するエラーを返すことを検査する。PoW込みE2Eや全公式テストの成功として数えない。

本文のRustコードは各章の唯一のrustコードブロックを抽出し、一時ファイルに対して次を実行した。

```bash
rustc --edition 2021 --test snippet.rs -o snippet-tests
./snippet-tests
```

第11章5件、第13章5件、第20章4件、計14件成功。snippetのSHA-256もJSONに保存した。これは教材の検査であり、公式DAA/Finalityの数値差分テストや攻撃確率の検証ではない。既存のGHOSTDAG差分比較とも別の試験である。

## 未確認事項

P2Pメッセージ境界・DoS上限・全ピアペナルティ、全ネットワークとフォーク境界、クラッシュ時の永続化・並行処理、DAAの実数値ベクトル差分、公式Finality経路の実行時境界検査、攻撃者モデルに対する安全性の証明、Pruning後IBDは今回の範囲外。trusted block/Genesisについては通常経路との分岐を確認した範囲にとどまり、全trusted検証とPruning Proofの信頼条件を完了とはしない。Bitcoin比較と全参考文献も全面照合していない。

第11・13・20章は**主要経路のみSource-checked**とし、章全体のReviewed/Verifiedにはしない。秘密鍵・シードを教材・記録へ追加せず、公式TestConsensusの一時DBやビルド成果物・ログも公開対象へ追加しない。

## 収録後の確認

mdBookビルド、生成HTML 53ページの内部リンク・アンカー、既存2教材の本文一致、差分の空白検査は成功。公開候補の秘密情報・ローカルホームパスのパターン検査で検出なし。Rust教材のコードは変更していない。
