# Chapter 07 — Introduction to BlockDAG（BlockDAG入門）

> **Mastering Kaspa — Part I: Foundations**  
> 原稿バージョン: v0.1（学習用初稿）

## 1. はじめに

Bitcoinのブロックチェーンは、各ブロックが原則として一つの親ブロックを参照することで、一本の履歴を構成する。一方、現実のP2Pネットワークには通信遅延があり、複数のマイナーが互いの最新ブロックを知らないまま並行してブロックを生成することがある。

**BlockDAG（Block Directed Acyclic Graph）**は、ブロック間の参照関係を有向非巡回グラフとして保持する構造である。Kaspaでは複数の親を参照するブロックを許容し、並行して生成されたブロックをDAGに取り込む。これをコンセンサスとして利用するには、GHOSTDAGなどの順序付け規則が別途必要になる。

## 2. 学習目標

この章を終えると、次を説明できるようになる。

1. グラフ・有向辺・非巡回性の意味
2. BlockchainとBlockDAGの構造上の違い
3. 親・祖先・子孫・過去・未来・反鎖（anticone）の意味
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
- **Ancestors / Past（祖先・過去）**: 親参照を繰り返して到達できるブロック。
- **Descendants / Future（子孫・未来）**: そのブロックを祖先として持つブロック。
- **Anticone（反鎖に属する関係）**: 二つのブロックのどちらも他方の過去に含まれない関係。たとえば上図のBとCは互いの過去に含まれない。

「並行」は厳密には同じ瞬間に生成されたという意味ではなく、**DAGの参照関係から前後関係を確定できない**ことを指す。

## 4. BlockchainとBlockDAG

### 4.1 Blockchain

```text
Genesis -> A -> B -> C -> D
                  \
                   C'  (競合するブロック)
```

従来型のチェーンでは、競合したブロックの一部が主チェーンから外れる。Bitcoinでは有効なチェーンの累積作業量を基準にチェーン選択が行われる。

### 4.2 BlockDAG

```text
Genesis -> A -> B ----> D
              \       /
               -> C --
```

DがBとCを参照することで、両方のブロックがDAG内に残る。ただし、**DAGに記録されることと、そこに含まれる全トランザクションが有効になることは別**である。競合するUTXOの支出などは、コンセンサスによる順序と検証規則で解決する。

| 観点 | Blockchain | BlockDAG |
|---|---|---|
| 基本構造 | チェーン | 有向非巡回グラフ |
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

DAGは**データの関係を表す構造**であり、取引の有効性や最終的な順序を自動的に決めるものではない。

たとえば同じUTXOを使う取引T1とT2が、並行ブロックBとCにそれぞれ含まれていたとする。

```text
            A
           / \
     B: T1   C: T2
           \ /
            D
```

両取引を同時に有効とすることはできない。すべての正直なノードが一貫した結果に到達するには、ブロックの順序付けとトランザクション検証が必要である。

Kaspaでは、この問題に取り組むために**GHOSTDAG**を使用する。GHOSTDAGは選択親（selected parent）を基準とした構造や、blue/redの分類などを利用してブロックの順序を導く。詳細は第12章で扱う。

> **注意:** blueは「常に正しい」、redは「不正」という意味ではない。また、GHOSTDAGの順序とトランザクションの有効性は同一概念ではない。

## 7. KaspaのBlockDAGを理解するための用語

| 用語 | 概要 |
|---|---|
| Block | PoWとトランザクション等を含むデータ単位 |
| Parent | 直接参照される先行ブロック |
| Tip | 既知のDAG内で子を持たない先端ブロック |
| Past | 参照を遡って到達できるブロック集合 |
| Future | あるブロックを祖先とするブロック集合 |
| Anticone | 互いに祖先・子孫関係にないブロック集合 |
| Selected Parent | コンセンサス規則に従って選択される親 |
| Blue / Red | GHOSTDAGによる分類 |
| Virtual Block | 複数のDAG先端を基に現在の合意状態を扱う仮想的な概念 |

## 8. Rustで最小のBlockDAGを作る

ここではKaspaの本番実装ではなく、**親参照と祖先探索を学ぶための小さなモデル**を作る。

### 8.1 新規プロジェクト

```bash
cargo new minimal-blockdag
cd minimal-blockdag
```

### 8.2 `src/main.rs`

```rust
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
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
        if block.parents.iter().copied().collect::<HashSet<_>>().len()
            != block.parents.len()
        {
            return Err("duplicate parent".into());
        }
        self.blocks.insert(block.id, block);
        Ok(())
    }

    fn past(&self, id: &str) -> HashSet<&str> {
        let mut visited = HashSet::new();
        let mut stack: Vec<&str> = self.blocks[id].parents.clone();
        while let Some(current) = stack.pop() {
            if visited.insert(current) {
                stack.extend(self.blocks[current].parents.iter().copied());
            }
        }
        visited
    }

    fn in_anticone(&self, a: &str, b: &str) -> bool {
        a != b && !self.past(a).contains(b) && !self.past(b).contains(a)
    }
}

fn main() {
    let mut dag = BlockDag::default();
    dag.add(Block { id: "G", parents: vec![] }).unwrap();
    dag.add(Block { id: "A", parents: vec!["G"] }).unwrap();
    dag.add(Block { id: "B", parents: vec!["A"] }).unwrap();
    dag.add(Block { id: "C", parents: vec!["A"] }).unwrap();
    dag.add(Block { id: "D", parents: vec!["B", "C"] }).unwrap();

    assert!(dag.in_anticone("B", "C"));
    assert!(dag.past("D").contains("G"));
    println!("D's past: {:?}", dag.past("D"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallel_blocks_are_in_each_others_anticone() {
        let mut dag = BlockDag::default();
        dag.add(Block { id: "G", parents: vec![] }).unwrap();
        dag.add(Block { id: "B", parents: vec!["G"] }).unwrap();
        dag.add(Block { id: "C", parents: vec!["G"] }).unwrap();
        assert!(dag.in_anticone("B", "C"));
    }

    #[test]
    fn unknown_parent_is_rejected() {
        let mut dag = BlockDag::default();
        assert!(dag.add(Block { id: "X", parents: vec!["Y"] }).is_err());
    }
}
```

### 8.3 実行

```bash
cargo run
cargo test
```

この実装は、親を先に登録することで循環を作れない単純な構造としている。**PoW、ハッシュ、ネットワーク通信、GHOSTDAG、UTXO、実際のKaspaブロック検証は実装していない。** コード例は教材用であり、Kaspaノードとして使用できない。

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
A. BがCの祖先でも、CがBの祖先でもないこと。

**Q3. BlockDAGにブロックが記録されれば、その中の全取引が有効になるか。**  
A. ならない。コンセンサスの順序とトランザクション検証が必要。

**Q4. 並行ブロックはなぜ発生するか。**  
A. ブロック生成とP2Pネットワークの情報伝播に時間差があるため。

**Q5. KaspaでGHOSTDAGが必要なのはなぜか。**  
A. DAGに存在するブロックの分類と順序付けを行い、一貫したコンセンサス処理につなげるため。

## 11. まとめ

- Blockchainは主として一本のチェーンを形成するが、BlockDAGはブロックの並行関係をグラフとして保持する。
- DAGの「過去」「未来」「anticone」は、ブロック間の因果的な参照関係を理解する鍵である。
- BlockDAGはデータ構造であり、それだけでは取引の競合を解決できない。
- KaspaではGHOSTDAGがブロックの分類・順序付けを担う。
- 高頻度ブロック生成には、コンセンサスだけでなく通信・検証・ストレージ面の制約もある。

**次章:** Chapter 08 — From Blockchain to Kaspa（ブロックチェーンからKaspaへ）

## 参考資料

1. [Kaspa公式サイト](https://kaspa.org/)
2. [Kaspa公式ドキュメント](https://docs.kaspa.org/)
3. [rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)
4. [PHANTOM and GHOSTDAG: A Scalable Generalization of Nakamoto Consensus](https://eprint.iacr.org/2018/104)
5. [Bitcoin: A Peer-to-Peer Electronic Cash System](https://bitcoin.org/bitcoin.pdf)

---

**原稿ステータス:** v0.1。技術的な説明とコード例は学習用。正式公開前に対象バージョンのKaspa実装・論文との照合、およびRustコードの実行確認を行う。
