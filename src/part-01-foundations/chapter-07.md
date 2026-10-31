# Chapter 07 — Introduction to BlockDAG（BlockDAG入門）

> **Mastering Kaspa — Part I: Foundations**  
> 基礎概説・DAG教材を確認：2026-10-31。対象は[本書の基準版](../baseline.md)を参照。

## 1. はじめに

Bitcoinのブロックチェーンは、Genesis以外の各ブロックが一つの親ブロックを参照し、分岐を含む候補から採用チェーンを選ぶ。一方、現実のP2Pネットワークには通信遅延があり、複数のマイナーが互いの最新ブロックを知らないまま並行してブロックを生成することがある。

**BlockDAG（Block Directed Acyclic Graph）**は、ブロック間の参照関係を有向非巡回グラフとして保持する構造である。Kaspaでは複数の親を参照するブロックを許容し、並行して生成されたブロックをDAGに取り込む。これをコンセンサスとして利用するには、GHOSTDAGなどの順序付け規則が別途必要になる。

## 2. 学習目標

この章を終えると、次を説明できるようになる。

1. グラフ・有向辺・非巡回性の意味
2. BlockchainとBlockDAGの構造上の違い
3. 親・祖先・子孫・過去・未来・Anticone（比較不能なブロックの集合）の意味
4. 並行ブロックが発生する理由
5. DAG構造とコンセンサスアルゴリズムの役割の違い
6. Rustで簡単なDAGを構築・検証する方法

## 3. DAGの基礎

グラフは頂点（vertex）と辺（edge）からなる。BlockDAGでは、頂点がブロック、辺が親ブロックへの参照に対応する。

- **Directed（有向）**: 参照に向きがある。
- **Acyclic（非巡回）**: 参照をたどって元のブロックへ戻る循環がない。
- **Graph（グラフ）**: 一本の列ではなく、分岐と合流を表現できる。

```text
            G (Genesis)
            |
            A
           / \
          B   C
           \ /
            D
           / \
          E   F
           \ /
            H
```

上図は理解用の図である。辺を「親から子へ」と描いているが、ブロックが実際に保持する参照は通常「子から親」へ向かう。DはBとCを親として参照する。

### 3.1 親・祖先・子孫

- **Parents（親）**: ブロックが直接参照するブロック。
- **Ancestors / Past（祖先・過去）**: 親参照を繰り返して到達できるブロックの集合。本章では対象自身を含めない。
- **Descendants / Future（子孫・未来）**: そのブロックを祖先として持つブロックの集合。本章では対象自身を含めない。
- **Anticone**: あるブロック自身と、そのPast・Futureのどれにも属さない、既知のブロックの集合。上図ではBのAnticoneは `{C}`、EのAnticoneは `{F}` である。

Past・Futureは時計の過去・未来ではなく、参照関係で決まる。Futureに属するブロックも、説明対象のDAGでは既に観測済みの場合がある。

Anticoneと、集合内のどの二つも比較不能な**反鎖（antichain）**は異なる。例えばBとCが並行し、XがCだけを親とするDAGでは、CとXは両方ともBのAnticoneに入る。しかしCはXの祖先なので、このAnticone自体は反鎖ではない。

また、[基準版のReachability API](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/model/services/reachability.rs)は自身も祖先とみなす規約を使う。本章の「自身を除くPast」とAPIの祖先判定は、同じ条件式へ無条件に置き換えない。

「並行」は厳密には同じ瞬間に生成されたという意味ではなく、**DAGの参照関係から前後関係を確定できない**ことを指す。

## 4. BlockchainとBlockDAG

### 4.1 Blockchain

```text
    Genesis
       |
       A
       |
       B
      / \
     C   C'
     |
     D
```

第4節も上を親、下を子として配置し、実際のハッシュ参照は子から親へ向かう。従来型のチェーンでは、競合したブロックの一部が主チェーンから外れる。Bitcoinでは有効なチェーンの累積作業量を基準にチェーン選択が行われる。

### 4.2 BlockDAG

```text
    Genesis
       |
       A
      / \
     B   C
      \ /
       D
```

DがBとCを参照することで、両方のブロックがDAG内に残る。ただし、**DAGに記録されることと、そこに含まれる全トランザクションが有効になることは別**である。競合するUTXOの支出などは、コンセンサスによる順序と検証規則で解決する。

| 観点 | Blockchain | BlockDAG |
|---|---|---|
| 合意で扱う構造 | 採用チェーン（候補全体には分岐もある） | 複数親の有向非巡回グラフ |
| 親参照 | 原則一つ | 複数を参照可能 |
| 並行ブロック | 主チェーンから外れることがある | DAGに保持できる |
| 合意の中心課題 | チェーン選択 | ブロックの分類・順序付け |
| 実装 | 比較的単純 | 参照関係・順序付けが複雑 |

## 5. なぜ並行ブロックが発生するのか

P2Pネットワークでは、ブロックが生成されてから各ノードへ伝わるまでに時間がかかる。

1. マイナーXがブロックBを発見する。
2. BがマイナーYへ伝わる前に、YがブロックCを発見する。
3. BとCは共通の親Aを参照している。
4. 後続ブロックDはBとCの両方を認識し、参照できる。

この現象は不正行為がなくても起こる。ブロック生成間隔を短くすると、通信遅延と生成間隔が近づき、並行ブロックが生じやすくなる。

ただし、BlockDAGにすれば無条件にスループットを上げられるわけではない。ブロックサイズ、ネットワーク帯域、検証処理、状態更新、攻撃耐性なども制約となる。

## 6. DAGの構造だけでは合意できない

DAGは**データの関係を表す構造**であり、トランザクションの有効性や最終的な順序を自動的に決めるものではない。

たとえば同じUTXOを使うトランザクションT1とT2が、並行ブロックBとCにそれぞれ含まれていたとする。

```text
            A
           / \
     B: T1   C: T2
           \ /
            D
```

両トランザクションを同時に有効とすることはできない。すべての正直なノードが一貫した結果に到達するには、ブロックの順序付けとトランザクション検証が必要である。

Kaspaでは、この問題に取り組むために**GHOSTDAG**を使用する。GHOSTDAGは選択親（selected parent）を基準とした構造や、blue/redの分類などを利用してブロックの順序を導く。詳細は[第12章](../part-02-consensus/chapter-12.md)で扱う。

> **注意:** blueは「常に正しい」、redは「不正」という意味ではない。また、GHOSTDAGの順序とトランザクションの有効性は同一概念ではない。

## 7. KaspaのBlockDAGを理解するための用語

| 用語 | 概要 |
|---|---|
| Block | PoWとトランザクション等を含むデータ単位 |
| Parent | 直接参照される先行ブロック |
| Tip | 既知のDAG内で子を持たない先端ブロック |
| Past | 参照を遡って到達できるブロック集合 |
| Future | あるブロックを祖先とするブロック集合 |
| Anticone | 対象ブロックと祖先・子孫関係にない、対象自身を除く既知ブロックの集合 |
| Selected Parent | コンセンサス規則に従って選択される親 |
| blue / red | GHOSTDAGによる分類 |
| Virtual Block | 選ばれた先端群を基に現在の状態を計算する仮想的な概念。採掘・配信される実ブロックではない |

## 8. Rustで最小のBlockDAGを作る

ここではKaspaの本番実装ではなく、**親参照と祖先探索を学ぶための小さなモデル**を作る。

### 8.1 新規プロジェクト

本書のリポジトリ外の作業用ディレクトリで実行する。Rust 1.90.0で確認しており、標準ライブラリ以外の依存はない。

```bash
cargo new --edition 2021 minimal-blockdag
cd minimal-blockdag
```

### 8.2 `src/main.rs`

```rust
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Block {
    id: &'static str,
    parents: Vec<&'static str>,
}

#[derive(Default)]
struct BlockDag {
    blocks: HashMap<&'static str, Block>,
}

impl BlockDag {
    fn add(&mut self, block: Block) -> Result<(), String> {
        if self.blocks.contains_key(block.id) {
            return Err(format!("duplicate block: {}", block.id));
        }
        if block.parents.iter().any(|p| *p == block.id) {
            return Err("self-reference is not allowed".into());
        }
        if block.parents.iter().any(|p| !self.blocks.contains_key(p)) {
            return Err("unknown parent".into());
        }
        if block.parents.iter().copied().collect::<HashSet<_>>().len() != block.parents.len() {
            return Err("duplicate parent".into());
        }
        self.blocks.insert(block.id, block);
        Ok(())
    }

    fn past(&self, id: &str) -> Result<HashSet<&'static str>, String> {
        let block = self
            .blocks
            .get(id)
            .ok_or_else(|| format!("unknown block: {id}"))?;
        let mut visited = HashSet::new();
        let mut stack = block.parents.clone();
        while let Some(current) = stack.pop() {
            if visited.insert(current) {
                stack.extend(self.blocks[current].parents.iter().copied());
            }
        }
        Ok(visited)
    }

    fn in_anticone(&self, a: &str, b: &str) -> Result<bool, String> {
        let a_past = self.past(a)?;
        let b_past = self.past(b)?;
        Ok(a != b && !a_past.contains(b) && !b_past.contains(a))
    }
}

fn main() {
    let mut dag = BlockDag::default();
    dag.add(Block {
        id: "G",
        parents: vec![],
    })
    .unwrap();
    dag.add(Block {
        id: "A",
        parents: vec!["G"],
    })
    .unwrap();
    dag.add(Block {
        id: "B",
        parents: vec!["A"],
    })
    .unwrap();
    dag.add(Block {
        id: "C",
        parents: vec!["A"],
    })
    .unwrap();
    dag.add(Block {
        id: "D",
        parents: vec!["B", "C"],
    })
    .unwrap();

    assert!(dag.in_anticone("B", "C").unwrap());
    let mut past: Vec<_> = dag.past("D").unwrap().into_iter().collect();
    past.sort_unstable();
    println!("D's past: {past:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> BlockDag {
        let mut dag = BlockDag::default();
        for (id, parents) in [
            ("G", vec![]),
            ("A", vec!["G"]),
            ("B", vec!["A"]),
            ("C", vec!["A"]),
            ("D", vec!["B", "C"]),
        ] {
            dag.add(Block { id, parents }).unwrap();
        }
        dag
    }

    #[test]
    fn past_is_transitive_and_excludes_self() {
        let dag = sample();
        assert_eq!(dag.past("D").unwrap(), HashSet::from(["G", "A", "B", "C"]));
        assert!(dag.past("G").unwrap().is_empty());
        assert!(!dag.in_anticone("B", "D").unwrap());
        assert!(!dag.in_anticone("D", "B").unwrap());
        assert!(!dag.in_anticone("B", "B").unwrap());
    }

    #[test]
    fn an_anticone_need_not_be_an_antichain() {
        let mut dag = sample();
        dag.add(Block {
            id: "X",
            parents: vec!["C"],
        })
        .unwrap();
        assert!(dag.in_anticone("B", "C").unwrap());
        assert!(dag.in_anticone("C", "B").unwrap());
        assert!(dag.in_anticone("B", "X").unwrap());
        assert!(dag.past("X").unwrap().contains("C"));
    }

    #[test]
    fn invalid_insertions_leave_the_dag_unchanged() {
        let mut dag = sample();
        let before = dag.blocks.clone();
        for block in [
            Block {
                id: "A",
                parents: vec!["D"],
            },
            Block {
                id: "X",
                parents: vec!["X"],
            },
            Block {
                id: "X",
                parents: vec!["Y"],
            },
            Block {
                id: "X",
                parents: vec!["G", "G"],
            },
        ] {
            assert!(dag.add(block).is_err());
            assert_eq!(dag.blocks, before);
        }
    }

    #[test]
    fn unknown_queries_return_errors() {
        let dag = sample();
        assert!(dag.past("missing").is_err());
        assert!(dag.in_anticone("B", "missing").is_err());
        assert!(dag.in_anticone("missing", "B").is_err());
        assert!(dag.in_anticone("missing", "missing").is_err());
    }

    #[test]
    fn parallel_insertion_order_preserves_relations() {
        let first = sample();
        let mut second = BlockDag::default();
        for (id, parents) in [
            ("G", vec![]),
            ("A", vec!["G"]),
            ("C", vec!["A"]),
            ("B", vec!["A"]),
            ("D", vec!["C", "B"]),
        ] {
            second.add(Block { id, parents }).unwrap();
        }
        for a in ["G", "A", "B", "C", "D"] {
            assert_eq!(first.past(a).unwrap(), second.past(a).unwrap());
            for b in ["G", "A", "B", "C", "D"] {
                assert_eq!(first.in_anticone(a, b), second.in_anticone(a, b));
            }
        }
    }
}
```

### 8.3 実行

```bash
cargo run
cargo test
```

期待する標準出力：

```text
D's past: ["A", "B", "C", "G"]
```

HashSetの走査順には依存せず、表示する前に辞書順で並べている。この表示順はコンセンサス上のブロック順序ではない。

この実装は、登録済みの親だけを参照でき、登録後に親を変更しないことで循環を作れなくしている。将来のブロックを親に指定して後から埋める方式ではない。未知のブロックの照会はエラーで返す。

文字列IDは暗号学的ハッシュではなく、複数の親なしブロックも許す一般的なDAGモデルである。KaspaのGenesis・接続性・親選択の全規則は再現していない。[第12章のDAG教材](../part-02-consensus/chapter-12.md)では集合の解析をさらに扱う。**PoW、ハッシュ、ネットワーク通信、GHOSTDAG、UTXO、実際のKaspaブロック検証は実装していない。** コード例は教材用であり、Kaspaノードとして使用できない。

## 9. 設計上のトレードオフ

BlockDAGでは、並行ブロックを保持できる一方、次の課題が生じる。

- 複数親の参照整合性とブロック検証
- DAGの過去・未来・anticoneを扱う計算量
- 共通順序とUTXO状態の導出
- ノード同期、ストレージ、Pruning
- ネットワーク遅延や攻撃的ブロック生成への耐性

したがって、BlockDAGの評価には**構造、アルゴリズム、ネットワーク条件、実装コスト**を一体で考える必要がある。

## 10. 理解度チェック

**Q1. DAGが「非巡回」であるとはどういう意味か。**  
A. 有向辺をたどって出発点へ戻る閉路が存在しないこと。

**Q2. BとCが互いのanticoneにあるとはどういう意味か。**  
A. BとCが異なるブロックで、BがCの祖先でも、CがBの祖先でもないこと。

**Q3. BlockDAGにブロックが記録されれば、その中の全トランザクションが有効になるか。**\
A. ならない。コンセンサスの順序とトランザクション検証が必要。

**Q4. 並行ブロックはなぜ発生するか。**  
A. ブロック生成とP2Pネットワークの情報伝播に時間差があるため。

**Q5. KaspaでGHOSTDAGが必要なのはなぜか。**  
A. DAGに存在するブロックの分類と順序付けを行い、一貫したコンセンサス処理につなげるため。

## 11. まとめ

- Blockchainは主として一本のチェーンを形成するが、BlockDAGはブロックの並行関係をグラフとして保持する。
- DAGの「過去」「未来」「anticone」は、ブロック間の因果的な参照関係を理解する鍵である。
- BlockDAGはデータ構造であり、それだけではトランザクションの競合を解決できない。
- KaspaではGHOSTDAGがブロックの分類・順序付けを担う。
- 高頻度ブロック生成には、コンセンサスだけでなく通信・検証・ストレージ面の制約もある。

**次章:** [Chapter 08 — From Blockchain to Kaspa（ブロックチェーンからKaspaへ）](chapter-08.md)

## 参考資料

1. [Kaspa公式サイト](https://kaspa.org/)
2. [Kaspa公式ドキュメント](https://docs.kaspa.org/)
3. [rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)
4. [PHANTOM and GHOSTDAG: A Scalable Generalization of Nakamoto Consensus](https://eprint.iacr.org/2018/104)
5. [Bitcoin: A Peer-to-Peer Electronic Cash System](https://bitcoin.org/bitcoin.pdf)

---

**確認範囲（2026-10-31）:** 集合の定義・図・基準版APIの祖先判定規約を確認し、本文DAG教材の実行と5テストを確認した。Kaspaのブロック検証やGHOSTDAGの実行比較ではない。記録はリポジトリの `experiments/chapter-07-review-2026-10-31.md` を参照。
