# 第27章 最小ノードの検証記録

- 実施日：2026-10-31（Asia/Tokyo）
- 環境：macOS / Darwin arm64
- Rust：rustc 1.90.0 (1159e78c4 2025-09-14)
- 対象：`examples/minimal-node/`（今回の未コミット変更）
- ソース `src/main.rs` のSHA-256：`066a464004b583dd29d64f5ce25d4a1aad0d1d4a206d67191ffcf81766e3d422`
- 依存関係：標準ライブラリのみ。Cargo.lockを収録。

リポジトリのルートで実行したコマンド：

```bash
cargo run --manifest-path examples/minimal-node/Cargo.toml --locked
cargo test --manifest-path examples/minimal-node/Cargo.toml --locked
cargo fmt --manifest-path examples/minimal-node/Cargo.toml --check
cargo clippy --manifest-path examples/minimal-node/Cargo.toml --locked --all-targets -- -D warnings
```

実測結果：

```text
blocks: 2
utxos: 2
mempool: 0

test result: ok. 6 passed; 0 failed
```

書式検査とClippyも終了コード0で成功した。
第27章27.6〜27.12の掲載コードは、収録サンプルと同じ内容に揃えている。

これは教育用モデルの単体検証であり、GHOSTDAG、Kaspa互換性、Devnet接続、ネットワーク収束、永続化は検証対象に含まない。
付録GのG-001〜G-010とは別の試験であり、それらの未実行状態は変更していない。
