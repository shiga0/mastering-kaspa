# 第27章：最小Kaspa型ノード

第27章27.6〜27.12のコードをまとめた、標準ライブラリのみで動く教材です。
GHOSTDAG・PoW・署名・実際のP2P通信は実装していません。
ブロック受信順で状態が変わり得るため、分散コンセンサスとしては成立せず、Kaspaネットワークには接続できません。

リポジトリのルートから実行します。

```bash
cd examples/minimal-node
cargo run --locked
cargo test --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
```

実行結果：

```text
blocks: 2
utxos: 2
mempool: 0
```

16テストでUTXO更新、ブロック途中の失敗時の全状態保持、ID再利用・自己参照・出力衝突、算術overflow、未知/複数の親、Mempoolの部分予約防止を確認します。
27.13/27.14の基本ケースはテスト内の関数呼び出しで再現し、親の再送と競合ブロックの受信順による不一致を確認します。実ネットワークの通信は実装していません。

使用済みID履歴はメモリに保持します。署名・Script・mass・容量制限・Mempoolの完全な再検証・永続化・reorgは未実装です。最新の記録は `experiments/chapter-27-review-2026-11-01.md` を参照してください。
