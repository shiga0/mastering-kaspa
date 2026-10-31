# 公式DAA計算器との整数比較

基準版v1.0.1の `FullDifficultyManager::calculate_difficulty_bits` と `SampledDifficultyManager::calculate_difficulty_bits` を直接実行する。期待値はPythonの任意精度整数で計算し、公式の整数・compact target補助関数をモデル側から呼ばない。

固定コミットは `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。確認環境はmacOS ARM64、Rust 1.90.0、Python 3.11、Apple clang 17.0.0。公式Cargo.lockを変更せず使う。C/C++のビルド環境が必要で、初回は依存取得と数GBのビルド領域を要する。他のOS/CPUは未確認。

## 再実行

リポジトリのルートで実行する。GHOSTDAG比較で取得した同じアーカイブ・ビルド領域を再利用できる。

```bash
curl -fL https://codeload.github.com/kaspanet/rusty-kaspa/tar.gz/fcd9c28f9b211b9872a0c90a8c94f79466ffb063 -o /tmp/kaspa-ghostdag-v101.tar.gz
python3 experiments/daa/run.py --archive /tmp/kaspa-ghostdag-v101.tar.gz --work-dir /tmp/kaspa-ghostdag-build
```

依存取得済みなら `--offline` を追加する。アーカイブのSHA-256照合と既存ソースの内容確認は `../ghostdag/run.py` の処理を共用する。この処理はGHOSTDAGのexampleも配置するが、今回はDAAのexampleだけを実行する。公式アルゴリズムやCargo.lockは変更せず、ビルド領域は公開リポジトリ外に限定する。

`vectors.json` の期待bitsをPythonの `oracle.py` で再計算してから、公式workspaceへ `compare.rs` をexampleとして追加し、Cargoを最大2並列・30分の期限で実行する。公式側は期待bitsを読み取るフィールドを持たず、ヘッダーと窓・設定値からbitsを計算する。診断は標準エラー、公開可能な集計JSONは成功時の標準出力。生の公式ログは集計へ混ぜない。

## 確認した範囲

27ケースで公式bits・整数モデル・保存済み期待bitsが一致。各ケースで窓への挿入順を反転し、計54回の公式計算結果も一致した。

- 空の窓・必要件数未満・必要件数ちょうど。
- 目標ペース・速い/遅いペース・最小時刻の外れ値除外。
- 全timestamp同一・1msの幅・u64上限近くのtimestamp。
- 異なるtarget・途中の整数平均の丸め・compact表現の符号ビット補正。
- 同じtimestampのblue work順、workも同点の場合のhash順。
- Full/Sampledの分母の差、sample rate 2/3/40。
- Sampledの標本不足時：Genesis、selected parentが移行前/境界/移行後となる分岐とtarget制限。

入力は `vectors.json`、結果は [observed-2026-10-31.json](observed-2026-10-31.json)、経緯は [検証記録](../daa-comparison-2026-10-31.md)。

## 対象外

窓は合成値を直接渡しており、`DualWindowManager` の分岐や実際のDAA window構築・抽出を通さない。Sampledへの移行状態も明示したメモリ内ヘッダーと祖父のDAA scoreで用意する。ネットワークごとの既定設定をそのまま実行した試験ではない。

non-DAA集合・DAA score算出、PoW、ヘッダー検証、ネットワークでの有効化、任意のbitsの妥当性、攻撃耐性はこの比較に含めない。特に未来時刻や難易度の範囲などを検証する前段を省くため、u64上限のtimestampをブロックとして受理することを示すものではない。秘密鍵・シード・採掘・ノード起動は不要。
