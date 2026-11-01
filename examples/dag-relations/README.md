# 第12章：DAGの関係を調べる

Past・Anticone・k-cluster条件を調べる教材です。GHOSTDAGの分類や親選択は実装していません。既知の親のみ登録でき、孤児ブロックやPruningには対応していません。空ID・重複ID・自己参照・重複親・未知の親を拒否し、拒否時は既存の関係を変更しません。

`is_k_cluster` は全要素が既知かを先に検査します。空集合は条件を満たします。最大の集合やblue集合を選ぶ機能はありません。

2026-11-01の改訂後は8テスト成功。記録は `experiments/chapter-12-review-2026-11-01.md`。Anticone内の要素同士に祖先・子孫関係がある例、未知の要素を含む集合のエラー、拒否時の状態維持を含みます。

```bash
cd examples/dag-relations
cargo test --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
```
