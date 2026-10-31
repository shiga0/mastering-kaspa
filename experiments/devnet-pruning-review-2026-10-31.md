# Devnet：Pruning条件の照合と前提検査

確認日：2026-10-31。参照ソース：v1.0.1 / `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。既存Dockerイメージとソースのバイナリ一致は従来どおり未証明。

## 固定ソース

- [consensus/core/src/config/params.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/params.rs) — SHA-256 `9b9a88195e530888de6543004a241c136c142a0bf606e27de4c527901b17ffea`
- [consensus/src/processes/pruning.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/pruning.rs) — SHA-256 `3d42fb8ad501ce5ae3318519c3ec1123d8796508485f9ece806b2c532aff6e7f`
- [consensus/src/pipeline/pruning_processor/processor.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/pruning_processor/processor.rs) — SHA-256 `e5b7c2bd6307514b44537bc3e3a719515032692fa2dc3252368b6be9d0204e73`
- [consensus/src/processes/pruning_proof/validate.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/pruning_proof/validate.rs) — SHA-256 `b94f22b79919669322c558405dc4502e5f0538998bf3fbb466adeb7810126e15`
- [consensus/src/processes/pruning_proof/apply.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/pruning_proof/apply.rs) — SHA-256 `3c585dd462b9dd34ea3520e18e33d83e7a697ae91d42bdf312264d19571ca9b1`
- [consensus/src/pipeline/virtual_processor/processor.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/virtual_processor/processor.rs) — SHA-256 `e1e92b1925016cba6e032b245e42e5e5e41775e4e41ab10f663110e44617511e`

## ソースで確認したこと

- DevnetはCrescendo無効。従来pruning depth 185798、finality depth 86400、目標1000ms。
- point選択はblue scoreによる深さとfinality score区間を使う。ブロック数やDAA scoreだけの閾値ではない。
- point更新、pruning用UTXO更新、データ削減は別段階。archivalならデータ削減を行わない。
- Pruning Proofの検証・適用と、受信UTXOのMuHash/ヘッダーcommitment一致検査は別の経路。

## 実測

`python3 experiments/devnet/pruning-check.py --require-advanced` を実行。5475ブロック、sink blue score 4835。Pruning pointはDevnet Genesisのまま。想定どおり終了コード2で未達を返した。結果：`devnet/pruning-observed-2026-10-31.json`。採掘は実行していない。

pointが動いていても検査スクリプトはhistoryRemovalVerifiedとpruningProofImportVerifiedをtrueにはしない。両者の実行証拠は別に必要。

## 続けるための条件と完了判定

1. 同じ基準版・ネットワークでPruning条件を満たしたデータセットを準備し、出所・SHA・生成条件・所要時間・ディスク量を記録する。長期の生成試験なら時間・CPU・ディスクの上限と中止/再開手順を先に固定する。
2. archival無効を確認し、point移動と実際の削減ログ・古い対象の照会結果を対応づける。単なる照会失敗を削減の証拠にせず、事前に存在を確認した対象を使う。
3. 削減済みノードを再起動し、現在UTXO/状態を照合する。
4. 別の新規データ領域のノードを接続し、Pruning Proof/UTXO転送の経路を通った証拠と、同期後の状態一致を記録する。通常の短いDAGの伝播では代替しない。
5. 不正な証明・不一致UTXO集合の拒否は、隔離した対象版テストとして実行する。

今回この1〜5は未実施。Pruningの完了をROADMAPに付けない。深さを小さくした改変ビルドやSimnetで検証する場合は別の教材実験として扱い、無変更Devnetの動作証拠へ流用しない。
