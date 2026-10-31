# Full DAAの窓・non-DAA集合・DAA score比較

基準版v1.0.1の `FullWindowManager::block_daa_window` を小さな合成DAGで実行し、全祖先を列挙するPythonモデルと比較する。公式GHOSTDAGで得たメタデータを入力に使うため、GHOSTDAG分類自体の独立検証は別の `../ghostdag/` に分ける。

固定コミットは `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。環境はmacOS ARM64、Rust 1.90.0、Python 3.11、Apple clang 17.0.0。公式Cargo.lockを維持し、取得済みのソースアーカイブとビルド領域を再利用できる。初回は依存取得、C/C++ビルド環境と数GBの空き容量が必要。他のOS/CPUは未確認。

## 再実行

リポジトリのルートから実行する。

```bash
curl -fL https://codeload.github.com/kaspanet/rusty-kaspa/tar.gz/fcd9c28f9b211b9872a0c90a8c94f79466ffb063 -o /tmp/kaspa-ghostdag-v101.tar.gz
python3 experiments/daa-window/run.py --archive /tmp/kaspa-ghostdag-v101.tar.gz --work-dir /tmp/kaspa-ghostdag-build
```

依存取得済みなら `--offline` を追加する。固定アーカイブのSHA-256・既存ソース検査には `../ghostdag/run.py`、target算術には `../daa/oracle.py` を共用する。ビルド領域は公開リポジトリ外に限定する。実行期限は30分、Cargoの並列数は2。公開用集計JSONは成功時の標準出力へ出し、生ビルドログを保存しない。

## 比較

入力は `vectors.json`。8種類のDAG × k=0/1/2 × 窓サイズ0/1/2/4/8の120ケースを、次の6条件で実行する。

- キャッシュなし、正しいFullキャッシュ、無視すべきSampledキャッシュの3通り。
- 親配列の通常順と反転順の2通り。

1条件あたり1950ブロック、計11700回で同一の結果となり、Pythonモデルとも一致した。

| 比較対象 | Pythonモデルの計算 |
|---|---|
| 親集合 | 各親の祖先を調べ、他の親の祖先となる親を除外 |
| 窓 | 全祖先からGenesisを除き、公式メタデータの(work, hash)が大きい順で指定件数を選ぶ |
| merge set | 親の過去の和集合からselected parentの過去を除き、selected parentを加える |
| non-DAA | merge set中で窓の最小(work, hash)より小さい要素。窓が空ならmerge set全体 |
| DAA score | selected parentのモデル側DAA score＋merge set中のnon-DAA以外の件数 |
| bits | 窓サイズ2以上で、構築した窓を前回のPython整数モデルへ渡して比較 |

窓サイズが正のケースでは、ORIGINに到達して標本が不足する場合に `InsufficientDaaWindowSize(0)` を返すことも576回のアサーションで確認した。これは関数の境界検査で、Pruning後の実IBDではない。

結果は [observed-2026-10-31.json](observed-2026-10-31.json)、詳細は [検証記録](../daa-window-comparison-2026-10-31.md)。各ケースの出力はダイジェストで記録する。入力とコードから再生成できる。

## 限界

Full側のみを直接呼ぶ。Sampled窓構築、DualWindowManagerの切り替え、実ネットワークやPruning後IBDは未検証。到達可能性はメモリ内DFS、ヘッダーは合成値で、PoW・前段検証・実際のDAA再計算による有効ブロック列を作る試験ではない。

GHOSTDAGのwork・score・選択親は公式出力をモデルへの入力とする条件付き比較。公式GHOSTDAGの不具合すべてをこのモデルが発見するという意味ではない。親配列の順を変えているが、並行ブロックの登録順・受信順は変えていない。

窓サイズ0/1は窓とDAA scoreの境界検査だけに使い、bits算術は呼ばない。これらを実ノードの推奨設定として扱わない。鍵・シード・資金・採掘・ノード起動は不要。
