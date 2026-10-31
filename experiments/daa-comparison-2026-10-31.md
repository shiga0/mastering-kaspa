# 第13章：公式DAA算術の数値比較

実施日：2026-10-31。基準版v1.0.1、コミット `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。macOS ARM64、Rust 1.90.0、Python 3.11、Apple clang 17.0.0。前回のGHOSTDAG比較と同じSHA-256固定ソースアーカイブ、変更していない公式Cargo.lockを使用した。

**後続検査：** 同日の `daa-window-comparison-2026-10-31.md` でFull側の窓構築・non-DAA集合・DAA scoreを120ケース比較した。本記録の範囲外の記述は算術検査単体の範囲を示す。Sampled窓とDual切り替えは引き続き未検証。

## 方法

`FullDifficultyManager::calculate_difficulty_bits` と `SampledDifficultyManager::calculate_difficulty_bits` に、明示したメモリ内ヘッダー・窓・設定を渡すRust exampleを実行した。Python側はtargetの展開とcompact変換を任意精度整数で別実装し、途中の整数除算を含めて期待値を計算する。公式側は期待bitsを読み取らない。

入力・保存済み期待値は `experiments/daa/vectors.json`、再実行入口は `experiments/daa/run.py`、モデルは `experiments/daa/oracle.py`、公式APIを呼ぶ側は `experiments/daa/compare.rs`。手順は `experiments/daa/README.md`。集計は `experiments/daa/observed-2026-10-31.json` に記録する。集計JSONにソースアーカイブ・Cargo.lock・各ハーネスファイルのSHA-256を含めた。

## 結果

27ケースが、Python整数モデル・保存済み期待bits・公式計算器で一致した。各ケースでheapへの挿入順も反転し、計54回の公式計算が成功。通常順と反転順は同じbitsを返した。

| 代表ケース | 公式と一致したbits（16進） |
|---|---|
| Full、標本不足 | `207fffff`（Genesis bits） |
| Full、目標ペース | `1e7fffff` |
| Full、全時刻同一 | `1d0aec33` |
| Full、同時刻でwork順をhash順と逆にした入力 | `1f049713` |
| Full、同時刻かつwork同点でhash順を使う入力 | `1e222ade` |
| Full/Sampled、途中の整数平均が丸められる入力 | `01020000`（target=2） |
| Sampled、標本不足かつ祖父DAAがactivation直前 | `1f04ffff`（目標時間比で補正） |
| Sampled、同条件で祖父DAAがactivationと一致/直後 | `1e7fffff`（selected parentのbitsを継承） |

途中丸めの例は、最小時刻の要素を除いたtargetが1と2、時間比が2になるようにした。実装どおり先に整数平均すると `floor((1+2)/2) × 2 = 2`。平均化を最後へまとめた計算では3になり、同じ式のつもりでも演算順が異なることを検出できる。

他に必要件数ちょうど、速い/遅い時刻、1ms幅、u64上限近くの時刻、最大target制限、最小時刻の外れ値除外、compact変換、sample rateの違いを含む。最初の25ケースの成功後、同時刻でのwork優先をhash順と区別し、途中平均の丸めがbitsにも現れるようケースを強化した。公開結果はその最終27ケースのもの。

公式コードには既知のlifetime表記の警告があるが、改変せずビルド・実行に成功した。公式ログ・ビルド成果物・ソース一式を公開リポジトリへ追加していない。

## 根拠と限界

- [difficulty.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/difficulty.rs)：今回直接呼んだFull/Sampled計算、標本不足時の分岐、時刻・整数targetの処理。
- [window.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/window.rs)：窓の構築とDualWindowManagerの選択は今回の実行範囲外。

窓の正しい構築、non-DAA集合とDAA score、ヘッダーの前段検証、PoW、各ネットワークの有効設定、全compact値、攻撃耐性は検証していない。窓の挿入順を変えた試験であり、ブロックの受信順やDAG登録順を変えた検証ではない。

Sampledの標本不足ケースは、選択親が移行後かをその親（今回の候補から見た祖父）のDAA scoreで判定する内部処理の境界を検査している。DualWindowManagerの外側の切り替えを通った証拠ではない。

第13章と付録GのG-006に、算術部分の実行証拠として反映する。章全体やDAA全体のVerifiedにはしない。鍵・シード・実資産・採掘は使用していない。

## 収録後の確認

Rustハーネスの書式、Python構文、既存2教材の本文一致、mdBookビルド、生成HTML 53ページの内部リンク・アンカー、差分の空白検査は成功。途中平均を最後の除算へまとめるとtargetが2から3へ変わること、work優先とhash優先を区別できる期待値も確認した。公開候補の秘密情報・ローカルホームパスのパターン検査で検出なし。
