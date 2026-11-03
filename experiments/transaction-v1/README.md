# Transaction v1のハッシュ対象を確認する

第15章15.5.1の比較表を、公式関数で確かめる実験です。
ハッシュを再実装せず、固定した`kaspa-consensus-core`へ14種類の変更を与えます。
取引ID、完全な取引ハッシュ、`SIGHASH_ALL`の署名ハッシュを別々に比較します。

## 固定版と実行

- rusty-kaspa v2.1.0：`01b532e8b553523216471682649693af92f0fd16`。
- [公式ソースアーカイブ](https://codeload.github.com/kaspanet/rusty-kaspa/tar.gz/01b532e8b553523216471682649693af92f0fd16)のSHA-256：`f2c30553a1993245419ba225f53fd898806b0102e5d81a5889d7648d51db03ff`。
- Rust 1.94.0 / edition 2024、Python 3.11、C/C++ビルド環境。公式の宣言MSRVは1.91.0ですが、その最低版では実行していません。
- 本書v1.0.1のDevnet・Rust 1.90.0教材とは別の環境です。

本書リポジトリのルートで実行します。既に同じアーカイブを持っていれば、再取得せずhashを確認して使えます。

```bash
curl -fL https://codeload.github.com/kaspanet/rusty-kaspa/tar.gz/01b532e8b553523216471682649693af92f0fd16 \
  -o /tmp/mastering-kaspa-v2.1.0.tar.gz
rustup toolchain install 1.94.0 --profile minimal
python3 experiments/transaction-v1/run.py \
  --archive /tmp/mastering-kaspa-v2.1.0.tar.gz \
  --work-dir /tmp/mastering-kaspa-transaction-v1
```

初回は依存の取得と数分以上のビルドが必要です。依存取得済みなら`--offline`を追加できます。既定ツールチェーンは変更しません。

runnerはアーカイブのhashを検査して、専用work-dirの`node-v2.1.0`へ展開します。
既存の上流ファイルが一致しない場合は拒否します。
本書の`checks.rs`だけを追加のintegration testとしてコピーし、上流のCargo.tomlとCargo.lockを変更せず、`cargo +1.94.0 test --locked --jobs 2 -p kaspa-consensus-core --test book_tx_v1 -- --nocapture`を実行します。
対象workspaceの依存解決に、直接のハッシュ計算以外のGit依存の取得が必要になる場合もあります。
実行期限は30分です。work-dirは実験専用にし、既存の開発checkoutを指定しないでください。

## 結果と限界

2026-11-03、macOS/ARM64で14ケースすべてが期待と一致しました。[結果JSON](observed-2026-11-03.json)にソース・runner・上流lockfile・アーカイブのSHA-256を記録しています。

| 変更 | ID | 完全な取引ハッシュ | ALL署名ハッシュ |
|---|---|---|---|
| signature script、compute budget、storage mass（3件） | 不変 | 変化 | 不変 |
| 出力額、Script、ID、authorizing input、binding削除、参照outpoint、sequence、payload、locktime（9件） | 変化 | 変化 | 変化 |
| 参照UTXOの金額、Script（2件） | 不変 | 不変 | 変化 |

すべての差をassertで検査し、不一致なら実行は失敗します。
取引のIDキャッシュや、変更前の署名ハッシュのキャッシュを使い回さず、各変更について公式関数で計算し直します。

入力・出力はハッシュ対象の差を調べる合成データです。範囲外のauthorizing inputや偽のoutpointなど、取引全体として無効な変種も意図的に含めています。
ハッシュ関数はこれらの有効性を検査する関数ではありません。
実署名の作成・検証、Script VM、Genesis Proof、取引受理、資源上限・手数料ポリシーの試験ではなく、ノードへ送信もしません。
「同じ署名ハッシュになる」ことと「有効な取引として受理される」ことを区別してください。

固定ソースの`consensus/core/src/hashing/tx.rs`・`sighash.rs`と、`crypto/hashes/src/hashers.rs`が根拠です。
上流ハッシュ関連の5単体テストも別に実行しましたが、全公式テストの成功やMainnet稼働の実測とは扱いません。
