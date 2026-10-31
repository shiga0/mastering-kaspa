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

テストは正常なUTXO更新、失敗時の状態保持、未知の親、複数親、Mempool競合、重複入力を検証します。
27.13以降の伝播・受信順の実験は発展課題です。
