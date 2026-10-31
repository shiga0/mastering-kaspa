# Sampled DAA窓とDual切り替えの比較

基準版v1.0.1の `DualWindowManager` で窓・non-DAA集合・DAA score・bitsを求め、対応するFull/Sampled計算器の直接呼び出しと、Pythonモデルの両方に比較する。

固定コミットは `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。確認環境はmacOS ARM64、Rust 1.90.0、Python 3.11、Apple clang 17.0.0。公式Cargo.lockを変更せず使い、初回は依存取得とC/C++ビルド環境・数GBの空き容量を要する。他のOS/CPUは未確認。

## 再実行

リポジトリのルートから実行する。前回のアーカイブとビルド領域を再利用できる。

```bash
curl -fL https://codeload.github.com/kaspanet/rusty-kaspa/tar.gz/fcd9c28f9b211b9872a0c90a8c94f79466ffb063 -o /tmp/kaspa-ghostdag-v101.tar.gz
python3 experiments/daa-transition/run.py --archive /tmp/kaspa-ghostdag-v101.tar.gz --work-dir /tmp/kaspa-ghostdag-build
```

依存取得済みなら `--offline` を付ける。アーカイブのSHA-256・既存ソースの検査は `../ghostdag/run.py`、target算術は `../daa/oracle.py` を共用する。公開リポジトリ外へ展開し、Cargoを2並列・30分の上限で実行する。公式コードを変更せずexampleを追加する。標準出力は成功時の集計JSON、生ログは公開結果へ混ぜない。

## 比較条件と結果

8種類の合成DAG × k=0/2 × Sampled窓サイズ2/4 × sample rate=1/2/3 × activation=0/5/9の288ケース。Full窓サイズ4、目標間隔は移行前1000ms・移行後100msの合成設定。

キャッシュなし・正しいキャッシュ・要求するoriginと逆のキャッシュの3条件と、親配列2順で実行する。異種キャッシュは要求ごとに選択親のキーへ挿入する。過去のキャッシュのタグを無条件に反転すると、移行時に次の要求と同じoriginになり得るためである。

1条件あたり4680ブロック、Dualで計28080回、対応する直接呼び出しも28080回。Pythonモデル、窓とnon-DAAの全要素、DAA score、bits、各キャッシュ条件と親順の出力がすべて一致した。

| 1条件あたりの境界・サンプリング入力 | 件数 |
|---|---:|
| 選択親DAAがactivation未満 | 2136 |
| activationと一致 | 566 |
| activationより大きい | 1978 |
| 直前（activation−1、未満の内数） | 240 |
| 直後（activation＋1、超過の内数） | 351 |
| 候補自身のDAAで選ぶとモードが変わる | 324 |
| Sampled側で、現在のmerge setに属し、non-DAAではないが窓に入らない要素 | 1772 |

最後の値はブロックごとの出現回数であり、全体で一意なブロック数ではない。入力は `vectors.json`、集計は [observed-2026-10-31.json](observed-2026-10-31.json)、詳細は [検証記録](../daa-transition-comparison-2026-10-31.md)。

## モデルと限界

公式GHOSTDAGからwork・blue score・selected parentを供給する条件付き検査。Python側ではDAGの過去をmerge setに分割し、activation、blue scoreの閾値、merge set内のDAA indexから標本を列挙して、最後にwork/hash順で窓を選ぶ。公式のキャッシュや早期終了をモデルへ移植しない。DAA scoreはモデル側で積み上げる。

窓に含まれないこととnon-DAAであることは同義ではない。サンプリングやactivationによる除外と、DAA加算からの除外を区別する。

固定した少数の合成設定での関数検証であり、Devnet既定設定や現在のMainnet、全フォーク値の検証ではない。到達可能性はDFS、ヘッダーは合成値。公式Reachability、PoW、ヘッダー検証、中央値時刻の窓、実際のPruning後IBD・同期用coverの生成、DB永続化は対象外。親配列は反転するが、DAGへの登録順は変えない。鍵・シード・採掘・ノード起動は不要。
