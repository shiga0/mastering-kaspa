# Finality pointの境界比較

基準版v1.0.1 / `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` の `BlockDepthManager::calc_finality_point` を、独立したselected-parent chain列挙モデルと比較する。公式ソースとCargo.lockは変更せず、公開リポジトリ外へexampleを追加する。

## 再実行

macOS ARM64、Rust 1.90.0、Python 3.11、Apple clangを使用。初回ビルドは数GBのディスクと数分以上を要する。Cargoは2並列・30分上限。他OS/CPUは未確認。

```bash
curl -fL https://codeload.github.com/kaspanet/rusty-kaspa/tar.gz/fcd9c28f9b211b9872a0c90a8c94f79466ffb063 -o /tmp/kaspa-finality-v101.tar.gz
python3 experiments/finality/run.py --archive /tmp/kaspa-finality-v101.tar.gz --work-dir /private/tmp/kaspa-finality-build --offline
```

`--offline` は依存取得済みの場合に使う。初回に依存がなければ外して取得する。既存のGHOSTDAG/DAA比較用work-dirを再利用してもよい。ソースアーカイブのSHA-256検査と安全な展開は `../ghostdag/run.py` を共用する。runnerはwork-dirを絶対パスへ解決するため、macOSの `/tmp` のシンボリックリンクにも対応する。

入力は `vectors.json`。`python3 experiments/finality/make_vectors.py` で同じ入力を再生成できる。ID 0はORIGIN、ID 1は合成Genesis。ノードのblue score・DAA score・selected parentを明示し、公式Reachabilityのメモリストアへ `inquirer::add_block` で登録する。GHOSTDAGによるこれらのメタデータの導出は今回の対象外。

## 比較する境界

- selected parentがORIGIN。
- candidate blue scoreがdepthの直前・一致・直後。
- pruning pointのblue score＋depthの直前・一致・直後。
- pruning pointがDAG上の祖先だがselected parent chain上にはない場合。
- 保存された直前の点がORIGINのときのpruning pointへのフォールバック。
- blue scoreが飛び飛びのchainで、目標scoreと一致する点、その前後。
- selected parentのDAAがactivationの直前・一致・直後。候補のblue scoreを固定して適用depthを比較する。
- depthが増加/減少する前後と、scoreに飛びのあるchainの連続計算。子の計算には公式関数が直前に返した点を保存して渡す。
- Devnetのprior depth 86400を使う合成score境界（長時間採掘なし）。

公式関数のforward iteratorをPythonへ写さず、祖先を親ポインタから列挙してscoreで選ぶ。数値条件を満たす点と以前の点のうち後方へ戻らない方を期待値とする。22件の単発境界は手で付けた期待IDとも比較する。必須分岐やactivation前後、連続計算が欠けた場合、runnerは成功にしない。

## 結果と限界

実測と本文への反映は [検証記録](../finality-comparison-2026-11-02.md) と `observed-2026-11-02.json` を参照。

これは合成メタデータを与えた関数単位の比較。公式Reachabilityの登録・照会は実行するが、PoW・ヘッダー検証・本体/UTXO検証・VirtualProcessorのsink候補拒否、Pruning point一覧検査、実際のPruning後IBD、フォークのネットワーク有効化、安全性証明は対象外。Devnetを改変して採掘した結果やMainnetの動作証拠として扱わない。鍵生成・ノード起動・採掘・送金は行わない。
