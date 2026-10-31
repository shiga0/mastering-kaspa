# 公式GHOSTDAGとの小規模DAG比較

基準コミット `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` の `GhostdagManager::ghostdag` を直接実行し、別に実装した集合の全探索モデルと比較する。第12章の `dag-relations` 自体は分類を実装していないため、この検証ハーネスを分けている。

確認環境：macOS ARM64、Rust 1.90.0、Python 3.11、Apple clang 17.0.0。公式のCargo.lockを変更せず使い、依存の取得には初回ネットワーク接続が必要。RocksDBなどのネイティブ依存を含むためC/C++ビルド環境が必要で、他のOS/CPUでの実行は未確認。

## 実行

リポジトリのルートから実行する。公式ソースとビルド成果物はリポジトリ外へ置く。初回のライブラリビルドは実測約4分、example用依存の追加ビルドも別途必要だった。生成物は数GBになるため、空き容量を確認する。

```bash
curl -fL https://codeload.github.com/kaspanet/rusty-kaspa/tar.gz/fcd9c28f9b211b9872a0c90a8c94f79466ffb063 -o /tmp/kaspa-ghostdag-v101.tar.gz
python3 experiments/ghostdag/run.py --archive /tmp/kaspa-ghostdag-v101.tar.gz --work-dir /tmp/kaspa-ghostdag-build
```

スクリプトはアーカイブのSHA-256を検査してから展開し、既存のソースがある場合も元アーカイブと一致することを確認する。公式workspaceの `consensus/examples/book_ghostdag_compare.rs` にこのリポジトリのハーネスを追加し、`cargo run --locked --jobs 2` で実行する。公式アルゴリズム・パラメータ定義・Cargo.lockは書き換えない。

依存の取得済み環境では `--offline` を追加できる。全体の実行期限は30分、Cargoのビルド並列数は2。標準エラーはビルド診断、標準出力は比較成功時の集計JSON。公開用JSONは各ケースの出力ダイジェストを持ち、ユーザー名・作業パス・生ビルドログを含めない。

## 比較するもの

- selected parent、順序付きmergeset_blues/reds、blue score、blue work。
- 固定k=0/1/2/18、異なるbits、work同点でのhash比較。
- 親配列を反転しても同じ結果になること。
- selected parentのDAA scoreがactivationの前・一致・後となるケース。
- Genesis/ORIGINの初期値、上位レベル用のwork下限。

24ケース、各親配列順で712ブロック（計1424回の通常ブロックの比較）。固定したLCGシードによる16ケースも含む。生成時に祖先関係のある親を取り除き、親集合をanticoneにしている。実測結果は [observed-2026-10-31.json](observed-2026-10-31.json)、詳細は [検証記録](../ghostdag-comparison-2026-10-31.md)。

## 比較モデルの範囲

公式側は公式のMemoryGhostdagStoreを使う。ヘッダー・親参照はこのハーネスのメモリ内アダプターで提供し、到達可能性は双方とも同じDAGのDFSで判断する。集合モデルはselected parentのblue past全体を保持し、候補を加えたときの各blueのanticoneを直接数える。公式の分類関数・anticoneキャッシュ・score/work計算関数をモデル側から呼ばない。work算術は使用する3種類のbitsだけに限定した既知値である。

拒否理由の件数はモデル側の判定であり、公式関数の内部経路を計測した値ではない。パラメータとヘッダーは合成試験用であり、Devnet既定設定をそのまま動かした試験ではない。

これは小規模な合成DAGの差分テスト。公式のReachability実装、PoW、ヘッダー検証、DAAの計算、ノード間同期、永続化、取引処理順、任意の大規模DAGにおける同値性は対象外。並行ブロックの登録順の入れ替えも今回の比較には含めていない。秘密鍵・シード・資金・ノードの起動は不要。
