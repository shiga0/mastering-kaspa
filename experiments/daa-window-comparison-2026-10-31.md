# 第13章：Full DAA窓・non-DAA集合・DAA scoreの実行比較

実施日：2026-10-31。基準版v1.0.1、コミット `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。macOS ARM64、Rust 1.90.0、Python 3.11、Apple clang 17.0.0。GHOSTDAG・DAA算術比較と同じSHA-256固定アーカイブを使い、公式ソースとCargo.lockは改変していない。

**後続検査：** 同日の `daa-transition-comparison-2026-10-31.md` でSampled/Dualを288ケース比較した。本記録の対象外の説明はFull単体検査の範囲であり、全ネットワーク・Pruning後IBDは引き続き未検証。

## 対象と方法

公式GHOSTDAGに小さなDAGを与えてメタデータを作り、公式 `FullWindowManager::block_daa_window` で窓・non-DAA集合・DAA scoreを取得する。本書の基準版DevnetはFull側だが、今回の設定値とヘッダー自体は合成試験用である。

Python側は同じ入力DAGの全祖先を列挙し、Genesisを除いた要素を公式メタデータの(work, hash)で並べて窓を選ぶ。公式実装と同じselected parent chainをキャッシュ付きで辿るコードを移植したモデルではない。DAA scoreはモデル側で過去から積み上げる。

GHOSTDAGのwork・score・選択親は公式から供給するため、条件付きの窓/DAAロジック検査である。公式GHOSTDAGの分類を独立に検証する前回の比較とは範囲を分ける。

## 入力と結果

8種類のDAG：直鎖、並行枝、workが異なる枝、幅の広いfork、固定シードで生成した4種類。k=0/1/2、窓サイズ0/1/2/4/8を組み合わせた120ケース。各ケースをキャッシュ3条件（なし・Full・異種Sampled）×親配列2順で実行した。

| 対象 | 結果 |
|---|---|
| 1条件あたりのブロック | 1950 |
| 6条件の公式ブロック比較 | 11700回、すべて一致 |
| 窓の全要素 | Pythonの全祖先列挙モデルと一致 |
| merge set / non-DAA全要素 | モデルと一致 |
| DAA score | モデル側の累積計算と一致 |
| キャッシュ条件・親配列の違い | 全出力が一致 |
| ORIGINで必要な窓が不足する境界 | 576回で `InsufficientDaaWindowSize(0)` を確認 |
| 構築した窓からのbits | 窓サイズ2以上で前回の整数算術モデルと一致 |

non-DAA要素がある入力だけでなく、merge setがk+1のblue容量を超える入力も含めた。merge set全体とblueのみの集合を混同しない。これらは有限個の合成DAGでの結果であり、全DAGや攻撃耐性を証明しない。

実行入口は `experiments/daa-window/run.py`、入力は同ディレクトリの `vectors.json`、公式関数を呼ぶRustは `compare.rs`、別モデルは `oracle.py`。再実行手順は `README.md`、集計は `observed-2026-10-31.json`。アーカイブ・Cargo.lock・各ファイルと共用コードのSHA-256を集計に記録した。

初回はハーネスのGhostdagData import不足でコンパイル失敗。import修正後は成功し、不要メソッドを除いた最終版でも全比較が成功した。失敗を成功回数へ加えない。公式コードの既知のlifetime表記警告は改変せず扱った。ソース一式・ビルド成果物・生ログ・DBは公開対象に追加していない。

## 固定ソース

- [window.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/window.rs)：Fullの窓構築とキャッシュoriginの区別。
- [difficulty.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/difficulty.rs)：non-DAA集合・DAA score・bits計算。
- [block_window_cache.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/model/stores/block_window_cache.rs)：キャッシュがFull/Sampled originを区別する処理。
- [ghostdag/protocol.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/ghostdag/protocol.rs)：今回メタデータを供給した公式GHOSTDAG。

## 残る検証

Sampled窓構築、DualWindowManagerのフォーク切り替え、全ネットワーク、公式Reachability、PoW、前段のヘッダー検証、受信/登録順の変更、実際のPruning後IBDは対象外。ORIGIN境界のアサーションは、データ削減や証明付き同期が動いた証拠にはならない。

窓サイズ0/1は窓とDAA scoreだけを検査し、bits算術には使わない。ヘッダーのbitsや時刻は合成値で、前段の有効性検査を通った実ブロック列ではない。章全体のVerified判定は付けない。秘密鍵・シード・実資産・採掘は不要。

## 収録後の確認

Rustハーネスの書式、Python構文、既存2教材の本文一致、mdBookビルド、生成HTML 53ページの内部リンク・アンカー、差分の空白検査は成功。集計の件数とコード/共用ファイルのSHA-256も一致。公開候補の秘密情報・ローカルホームパスのパターン検査で検出なし。
