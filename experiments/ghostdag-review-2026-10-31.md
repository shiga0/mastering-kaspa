# 第12章 GHOSTDAGの照合記録

- 確認日：2026-10-31（Asia/Tokyo）
- 照合対象コミット：`4edcbbccb449f84832afb1ca157904b35f37d006`
- 状態：主要規則はSource-checked。章全体のReviewed/Verified判定ではない。

## 読み取ったソース

- [consensus/src/processes/ghostdag/ordering.rs](https://github.com/kaspanet/rusty-kaspa/blob/4edcbbccb449f84832afb1ca157904b35f37d006/consensus/src/processes/ghostdag/ordering.rs) — SHA-256 `8b6716afbaddd2becd1910e8d80108747daaa97970669cb50c22ff0fca0dc47f`
- [consensus/src/processes/ghostdag/protocol.rs](https://github.com/kaspanet/rusty-kaspa/blob/4edcbbccb449f84832afb1ca157904b35f37d006/consensus/src/processes/ghostdag/protocol.rs) — SHA-256 `044b12f1941733ce45c5f3a5376e198ef2b4796f5b6f5bda3c4b670a18055e33`
- [consensus/src/model/stores/ghostdag.rs](https://github.com/kaspanet/rusty-kaspa/blob/4edcbbccb449f84832afb1ca157904b35f37d006/consensus/src/model/stores/ghostdag.rs) — SHA-256 `9d031f8e587fb7de2015b39ac694d30a6b0c7da0a2e6b5b74a28ce750d328bb4`
- [consensus/src/processes/ghostdag/mergeset.rs](https://github.com/kaspanet/rusty-kaspa/blob/4edcbbccb449f84832afb1ca157904b35f37d006/consensus/src/processes/ghostdag/mergeset.rs) — SHA-256 `8a86df5cf864a17c963be9cd1cf08961c040ea28e4684b238506ec0a51671122`

## 修正した説明

- 親はblue work、同値時はHashの比較で最大を選ぶ。
- selected parentを除くmerge set候補はblue work、hashの昇順で評価する。
- 保存するmergeset_bluesはselected parentを含む。
- blue候補自身と、影響を受ける既存blueの双方でanticoneの上限を確認する。
- score/workの加算対象と、通常ブロック・Genesis・ORIGINの違いを記載した。
- 通常ブロックの固定された親選択と、更新されるvirtual側のビューを区別した。

## 教材の実行検証

- 環境：macOS / Darwin arm64、Rust 1.90.0。
- 対象：`examples/dag-relations/`（今回の未コミット変更）。
- 実行したコマンド：

```bash
cargo test --manifest-path examples/dag-relations/Cargo.toml --locked
cargo fmt --manifest-path examples/dag-relations/Cargo.toml --check
cargo clippy --manifest-path examples/dag-relations/Cargo.toml --locked --all-targets -- -D warnings
```

5件のテスト、書式検査、Clippyが成功。並行ブロックを逆順に追加してもPastとAnticoneが一致することを含む。

## 未確認事項

原論文の証明との完全照合、公式実装を実行した分類結果の差分テスト、メインネットの稼働リリース、Virtual Stateの最終的な取引処理順、DevnetでのE2E検証は未実施。DAG解析器のテスト成功はGHOSTDAG実装の検証を意味しない。
