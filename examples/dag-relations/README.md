# 第12章：DAGの関係を調べる

Past・Anticone・k-cluster条件を調べる教材です。GHOSTDAGの分類や親選択は実装していません。既知の親のみ登録でき、孤児ブロックやPruningには対応していません。

```bash
cd examples/dag-relations
cargo test --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
```
