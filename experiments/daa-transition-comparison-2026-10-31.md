# 第13章：Sampled窓構築とDual切り替えの実行比較

実施日：2026-10-31。基準版v1.0.1、コミット `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。macOS ARM64、Rust 1.90.0、Python 3.11、Apple clang 17.0.0。既存のSHA-256固定ソースアーカイブと、変更していない公式Cargo.lockを使った。

## 方法

公式GHOSTDAGへ合成DAGを入力し、得られたメタデータを `DualWindowManager::block_daa_window` とbits計算へ渡した。同じ入力を、選択親のDAA scoreから選ぶFull/Sampled計算器にも直接渡し、全要素と結果を照合する。

Python側は公式GHOSTDAGのwork・blue score・選択親を入力として使う。過去のブロックをmerge setへ分割し、各文脈のactivation・blue scoreの閾値・DAA indexに基づいて標本を列挙し、最後にwork/hash順で窓を選ぶ。公式のキャッシュ探索・早期終了をそのまま移植したものではない。DAA scoreはモデル側で過去から累積計算する。

入力とコードは `experiments/daa-transition/`、実行入口は `run.py`、手順は `README.md`、集計は `observed-2026-10-31.json`。公式ソースアーカイブ・Cargo.lock・ハーネスと共用コードのSHA-256を集計に記録する。

## 結果

8種類のDAG × k=0/2 × Sampled窓2/4 × sample rate=1/2/3 × activation=0/5/9の288ケース。Full窓4、移行前後の目標間隔1000ms/100msは今回の合成設定であり、Devnet既定値を再現したものではない。

各ケースをキャッシュ3条件と親配列2順で検査した。1条件4680ブロック、Dual計28080回、直接呼び出しも28080回。窓・non-DAA集合の全要素、DAA score、bits、窓サイズ・sample rateの選択が一致。Pythonモデルとも一致した。

| 選択親の境界等（1条件あたり） | 観測件数 |
|---|---:|
| activationより前 | 2136 |
| activationと一致 | 566 |
| activationより後 | 1978 |
| 直前：activation−1 | 240（前の内数） |
| 直後：activation＋1 | 351（後の内数） |
| 候補自身のDAA scoreで選ぶと異なるモードになる入力 | 324 |
| 現在merge set中でnon-DAAではないがSampled窓にない要素 | 1772 |

最後の値はケース/処理ブロックごとの出現数で、一意なブロック数ではない。Sampled窓に含まれないものを一律にDAA加算対象外とすると誤ることを確認した。

キャッシュ条件は、保存なし・正しいorigin付きの窓・要求と逆のoriginの空窓。異種キャッシュは要求時点で選択親のキーへ入れ、Full/Sampledの移行で偶然同じタグになることを避けた。異種キャッシュを無視して再構築でき、正しいキャッシュから継承する場合とも結果が一致した。

最初の比較と、未使用import除去・直前直後件数を追加した最終版がともに成功した。公開結果は最終版。公式コードにあるlifetime表記の警告は改変していない。生ログ・ビルド成果物・ソース一式・DBは公開リポジトリへ追加していない。

## 本文へ反映する区別

- Dualの切り替えは選択親のDAA scoreであり、候補自身のDAA scoreではない。
- Sampled窓では、候補ブロックが有効化後かだけでなく、標本となるブロック自身と、それを取り込む文脈の有効化も考慮する。
- non-DAAはblue scoreの閾値で判定し、sample rateによる間引きとは区別する。
- DAA indexはnon-DAAを除いたmerge setの順序に基づく。activationのフィルタやキャッシュのoriginを無視して窓だけを継承しない。

根拠：[window.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/window.rs)、[difficulty.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/difficulty.rs)、[block_window_cache.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/model/stores/block_window_cache.rs)。

## 限界

有限の合成DAG・フォーク設定・sample rateを対象にした関数検査。GHOSTDAG分類・公式Reachabilityは今回の独立検証範囲外。到達可能性はDFSで提供し、ヘッダーはPoW・前段検証を通した実ブロックではない。

中央値時刻の窓、全ネットワークの実際の有効化条件、全sample rate、trusted情報の不足/ORIGIN、Pruning後IBD、同期用cover生成、DB永続化・並行処理は未検証。親配列の順は反転したが、DAG登録順を変えた試験ではない。研究論文・攻撃モデルの正しさを保証せず、第13章全体のVerifiedにはしない。秘密鍵・シード・資産・採掘は不要。

## 収録後の検査

Rust書式、Python構文、既存2教材の本文一致、mdBookビルド、生成HTML 53ページの内部リンク・アンカー、差分の空白検査は成功。集計の各件数と実行したコード・共用ファイルのSHA-256も一致。公開候補の秘密情報・ローカルホームパスのパターン検査で検出なし。
