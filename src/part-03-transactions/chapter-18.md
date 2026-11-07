# Chapter 18 — Mining & Block Template Construction（マイニングとブロックテンプレート構築）

<p class="chapter-part">Part III — Transactions, Mining &amp; Security（トランザクション・マイニング・セキュリティ）</p>

> この章のRustコードは教育用モデルであり、Kaspaメインネットのブロック生成・PoWを再現するものではない。

この章の教材は実ノードの採掘結果を再現するものではない。

## この章で学ぶこと

本章では、Kaspaのマイニングを「ハッシュを計算する作業」だけではなく、**ノードがコンセンサスに従ってブロック候補を構築し、マイナーがPoWを探索し、ネットワークがその結果を検証する一連の処理**として理解する。

- ノード・マイナー・マイニングプールの責務の違い
- BlockDAGの親選択とブロックテンプレート
- Mempoolからのトランザクション選択、Coinbase、報酬
- Target、Nonce、PoW、難易度調整の関係
- 新ブロック受信時のテンプレート更新と競合
- `rusty-kaspa` を使った実装調査とテスト設計

---

## 18.1 マイニングの位置づけ

KaspaはProof of Work（PoW）を使用する。マイナーは有効なブロック候補に対してPoWの条件を満たす値を探索し、発見したブロックをネットワークに伝播する。ノードは、マイナーの主張を信用せず、受け取ったブロックを独立に検証する。

```text
Wallet ──取引送信──▶ Node / Mempool
                         │
                         ▼
                Block Template Builder
                         │
            親・取引・報酬・難易度
                         ▼
                    Miner / PoW
                         │
                 発見したブロック
                         ▼
                  P2P Network
                         │
                         ▼
             Independent Validation
                         │
                         ▼
                  BlockDAG / State
```

重要なのは、**マイニングとコンセンサスは同じものではない**という点である。PoWはブロックの生成に計算コストを課す。一方、BlockDAG内のブロックの関係、GHOSTDAGの順序、トランザクションの受理はコンセンサス規則によって決まる。

## 18.2 ノード、マイナー、プールの責務

| コンポーネント | 主な責務 | 信頼してはいけないもの |
|---|---|---|
| フルノード | DAG・UTXO・コンセンサス規則の検証、P2P通信 | 外部から届くブロック・トランザクション |
| テンプレート生成器 | 現在のDAG・Mempool・ルールから候補を作る | 古い状態や無効なトランザクション |
| マイナー | テンプレートに対してPoWを探索する | テンプレートが常に最新・有効であるという前提 |
| マイニングプール | 参加者への仕事の配布、シェア集計、報酬配分 | ワーカーが提出するシェア |

プールのシェア難易度は、ネットワークのブロック受理難易度とは異なる場合がある。**プールがシェアを受理しても、ネットワークがブロックを受理したことにはならない。**

## 18.3 Bitcoinのブロック生成とKaspaの違い

Bitcoinでは一般に直前のチェーン先端を参照するブロック候補を作る。KaspaはBlockDAGを採用しており、複数の親ブロックを参照できる。

```text
Bitcoin（概念図）
  A ──▶ B ──▶ C ──▶ [新ブロック候補]

Kaspa（概念図）
        ┌──▶ B ──┐
  A ────┤         ├──▶ [新ブロック候補]
        └──▶ C ──┘
```

図の矢印は親から子への生成関係を示し、実際のヘッダー参照は子から親へ向かう。この図は構造の違いを示すための簡略化である。実際のKaspaの親集合の選択には、DAGの状態、親候補の制約、コンセンサス規則が関与する。単に「見えている先端を全部親にする」わけではない。

## 18.4 ブロックテンプレートとは

ブロックテンプレートは、マイナーがPoWを探索するための**ブロック候補の材料**である。概念的には次の情報を含む。

| 項目 | 意味 | 注意点 |
|---|---|---|
| Parents | 参照する親ブロック | BlockDAGの関係とプロトコル制約に従う |
| Version | ブロック形式・規則のバージョン | 受理可能な値はネットワーク規則による |
| Timestamp | ブロック時刻 | 許容範囲などの検証を受ける |
| Bits / Target | PoWの閾値を表す情報 | 正しい難易度計算が必要 |
| Nonce | PoW探索時に変更する値 | 値を変えるだけで有効性は保証されない |
| Transactions | ブロックに含めるトランザクション | 依存関係、重複支出、手数料などを考慮 |
| Hash commitments | トランザクション等に対するコミットメント | 正式なフィールド・ハッシュ計算は実装仕様を参照 |
| Coinbase | ブロック報酬に関するトランザクション | 通常の送金トランザクションとは異なるルール |

### 18.4.1 基準版の生成経路

[`get_block_template_call`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/rpc/service/src/service.rs) は報酬先アドレスのネットワーク接頭辞を確認し、Scriptとextra dataをMinerDataとしてMiningManagerへ渡す。応答の `block` と `is_synced` は別の項目で、候補が返ったことだけで同期済みと判断しない。

[`MiningManager`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/mining/src/manager.rs) はキャッシュ利用と新規生成を分ける。キャッシュのminer dataが違う場合は複製を変更し、Coinbase payload、必要な報酬出力、Merkle root、時刻等を更新する。新規生成で候補取引が無効ならMempoolから除いて再試行する経路もある。キャッシュを変更する処理と、Virtualを更新する処理は同一ではない。

**テンプレートは確定済みブロックではない。** 親ブロックやMempoolの変化に応じて、更新方針に従った再利用・変更・再生成が必要になる。

### 18.4.2 依存関係を先に見る

```text
Consensus State ──┐
                  ├──▶ Template Builder ──▶ Mining Work
DAG Tips ─────────┤
                  │
Mempool ──────────┤
                  │
Network Params ───┘
```

テンプレート生成器は、参照する複数の状態の整合性を保つ必要がある。基準版は最初の候補選択をVirtualのread lock取得前に行い、取得後のUTXO viewで検証する。MempoolとVirtualのすべてを一度に固定する単一スナップショットという意味ではない。親だけ新しく、難易度やトランザクション選択だけ古いといった不整合を防ぐ設計が重要になる。

## 18.5 親ブロックの選択

BlockDAGでは、並行して発見された複数のブロックが存在する。新しいブロックは、コンセンサス規則に従って適切な親集合を参照する。

ここで区別したい概念は次のとおり。

- **Tip**：現在知られているDAGの末端ブロック。選択対象の候補になりうる。
- **Selected parent**：GHOSTDAGの規則に基づいて選択される特別な親。
- **Parents**：新しいブロックが直接参照する親集合。
- **Past / Anticone**：ブロック間の到達可能性から定まる関係。

親集合の決定は、単なるP2P処理ではなく、コンセンサスに関係する操作である。[第12章](../part-02-consensus/chapter-12.md)のGHOSTDAG、[第14章](../part-02-consensus/chapter-14.md)のVirtual Stateと合わせて読むと理解しやすい。

基準版の [`build_block_template_from_virtual_state`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/virtual_processor/processor.rs) はVirtualの親群からレベル別の親を構築し、同じVirtualのbits、DAA score、blue score/work、UTXO commitment、受理IDのコミットメント等をヘッダーへ入れる。Coinbaseを先頭へ追加し、本体のHash Merkle Rootも計算する。時刻はpast median time + 1とローカル時刻の大きい方、初期nonceは0とする。

## 18.6 Mempoolからのトランザクション選択

ブロックテンプレート生成器は、Mempoolに存在するトランザクションを無条件に詰め込むわけではない。少なくとも以下の観点がある。

1. トランザクションの構造・署名・スクリプト・UTXOに関する検証
2. 同一UTXOの二重消費を防ぐこと
3. 同一ブロック内に親子トランザクションを含めないこと
4. ブロックのサイズ・質量（mass）等の制限
5. 手数料とブロック内でのトランザクション選択方針
6. 新しいVirtual Stateに対して再評価が必要なトランザクション

Kaspaの**mass**は単純なバイト数と同義ではない。正確な計算式や制限値は、対象ネットワークとノード実装のバージョンを確認する。

### 18.6.1 基準版の候補選択

Mempoolの [`ready frontier`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/mining/src/mempool/model/frontier.rs) は、候補の合計massとブロックの制限に応じて、全件選択、事前サンプリングした列、重み付き選択器を使い分ける。[`RebalancingWeightedTransactionSelector`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/mining/src/block_template/selector.rs) は乱数を使う。したがって、同じ状態でテンプレートの取引集合が常に同一になることは、コンセンサス上の要件ではない。

選ばれた取引はVirtualのUTXO viewで検証される。Mempoolに親子を保持できても、単に親を先に並べて同じブロックへ入れる方式ではない（[第15章](chapter-15.md)）。なお、builder.rs冒頭には入力の年齢やfee/kilobyteを述べる長いコメントがあるが、本文では実際のselectorの呼び出しと本体を基準に読む。

### 18.6.2 トランザクション選択の不変条件

```text
選択済み取引集合 S について：

- 同じOutpointを二度消費しない
- 各取引が適用時点で有効である
- 同一ブロック内の出力を同じブロックの別取引から消費しない
- ブロック全体の制約を超えない
- 失敗した取引を含めない
```

実際の選択ポリシーは、コンセンサス規則とローカルなMempoolポリシーを分けて考える必要がある。**ポリシーに反するトランザクション**と**コンセンサス上無効なトランザクション**は同義ではない。

## 18.7 Coinbaseとマイニング報酬

Coinbaseは新しい通貨発行やマイナーへの報酬に関わる特殊なトランザクションである。通常の送金のように既存UTXOを消費して価値を移転するトランザクションとは役割が異なる。

Kaspaでは、複数のブロックが並行して存在するBlockDAG構造と報酬の扱いを理解する必要がある。BitcoinのCoinbaseの説明をそのままKaspaへ当てはめてはいけない。

報酬を考える際は、次を分ける。

- **Subsidy**：プロトコルに従って新規発行される部分
- **Transaction fees**：トランザクション処理に伴う手数料
- **報酬の適格性・計算**：Kaspa固有のコンセンサス規則に依存
- **プールからの支払い**：プール運営者の会計・分配ポリシーに依存

基準版の候補生成は、VirtualのGHOSTDAGデータ、merge setの報酬情報とnon-DAA集合を `expected_coinbase_transaction` へ渡す。現在の候補に新しく入れた通常取引の手数料を、その候補のCoinbaseへ単純に足すBitcoin型の説明とは区別する。報酬の詳細と適格性は[第19章](chapter-19.md)で扱い、本章では全計算の照合完了とはしない。

## 18.8 PoW：ハッシュ探索とTarget

PoWでは、ブロックのヘッダー等から規定された手順で算出した値が、コンセンサスで定めるTarget条件を満たす必要がある。

```text
Block Template
     │
     ▼
Header / PoW Input
     │
     ├── nonce = 0 ──▶ hash ──▶ 条件不成立
     ├── nonce = 1 ──▶ hash ──▶ 条件不成立
     ├── nonce = 2 ──▶ hash ──▶ 条件不成立
     └── ...       ──▶ hash ──▶ 条件成立
                                    │
                                    ▼
                              Found Block
```

一般化した説明として、ハッシュ出力が十分に一様で、Targetが整数の閾値なら、1回の独立した試行が成功する確率はTargetとハッシュ空間の大きさの比に概ね対応する。実際のKaspaのPoWには固有のアルゴリズムとデータのエンコードがあり、**通常のSHA-256をヘッダーにかけるだけではKaspaのPoWにはならない**。

### 18.8.1 難易度とハッシュレート

- **Targetが厳しい**：成功する試行の割合が低い。
- **ネットワークハッシュレートが高い**：単位時間当たりの試行回数が多い。
- **DAA**：観測されたブロック履歴等から、ネットワークのブロック生成ペースを制御する。

ブロック識別子のハッシュとPoW値は別の計算である。難易度調整の詳細は[第13章](../part-02-consensus/chapter-13.md)を参照。

## 18.9 ブロックを見つけた後の処理

```text
PoW Solution
    │
    ▼
Block Submission
    │
    ▼
Local Validation
    │
    ├── invalid ──▶ reject / diagnose
    │
    ▼
BlockDAG Integration
    │
    ▼
P2P Propagation
    │
    ▼
Other Nodes Validate Independently
```

マイナーがPoW条件を満たす値を発見しても、親参照、時刻、難易度、トランザクション、コミットメントなどの規則に違反すれば、そのブロックは拒否されうる。**PoWの成立はブロック有効性の必要条件の一部であって、十分条件ではない。**

## 18.10 テンプレート更新と競合

マイニング中にもネットワークは動き続ける。

- 新しいブロックが届く
- DAGのTip集合が変化する
- 難易度・Virtual Stateが更新される
- 新しい手数料付きトランザクションがMempoolへ入る
- 古いトランザクションが削除・再評価される

したがって、テンプレートには**鮮度**の概念が必要になる。新しい親集合に基づく仕事が発行された後、古い仕事に対する結果が到着することもある。古い仕事をどう扱うかは、ローカルノードの受理規則とプール側のシェア管理を分けて検討する。

基準版の `submit_block_call` は同期状態や `allow_non_daa_blocks` に応じた古さの検査も行う。ローカルの提出受付条件、コンセンサス規則、プールのシェア受付は別であり、古い仕事を一律にコンセンサス無効とみなさない。

### 18.10.1 競合のタイムライン

```text
時刻      Node A                     Node B
 t0       Tip = X                    Tip = X
 t1       Xを親とする候補A           Xを親とする候補B
 t2       ブロックAを発見            探索中
 t3       Aを伝播 ────────────────▶ Aを受信
 t4       次のテンプレートを生成      候補Bを更新
```

BlockDAGでは並行して生成されたブロックを構造上保持できる。ただし、**並行ブロックの全トランザクションが必ず受理されるわけではない**。コンセンサス順序とUTXOの競合解決が必要である。

## 18.11 マイニングプールの基本構造

```text
                  Pool Coordinator
                  /      |       \
                 /       |        \
             Worker A  Worker B  Worker C
                 \       |        /
                  \      |       /
                 Shares / Solutions
                          │
                          ▼
                  Pool Accounting
```

プールは参加者に仕事を分配し、提出されたシェアを検証する。ネットワーク難易度を満たす解が発見された場合は、ブロックとして提出できる可能性がある。

プールが導入する追加の信頼境界は、**仕事の内容を誰が決めるか**と**報酬を誰が分配するか**である。ネットワークの分散性を考える際、ハッシュレートだけでなくテンプレート構築権限の集中も観察対象となる。

## 18.12 セキュリティと失敗モード

| 失敗・攻撃 | 想定される影響 | 防御・確認ポイント |
|---|---|---|
| 無効なテンプレート | 計算資源の浪費 | ノード側の検証、パラメータ整合性 |
| 古いテンプレート | 競合増加、報酬機会の低下 | DAG更新通知、テンプレート再生成 |
| 偽のシェア | プール資源の消費 | ワーカー単位の検証・制限 |
| 不正なCoinbase | ブロック拒否 | コンセンサスルールの検証 |
| Mempool汚染 | 候補生成の遅延・負荷増大 | 入力検証、リソース制限 |
| Eclipse攻撃 | 古いDAG情報に基づく採掘 | ピア多様性、監視、接続管理 |
| DoS | テンプレート生成・配信停止 | レート制限、バックプレッシャー |

ハッシュレートが高いことと、単一の運営主体が支配的でないことは別の指標である。

## 18.13 Rustによる最小ブロックテンプレートモデル

以下は**コンセンサス非互換**の学習用コード。実際のKaspaでは署名・UTXO・mass・親選択・報酬・PoWを正式なルールで処理する必要がある。ここでは、テンプレート生成時の「取引内の入力重複と、取引間の入力競合の排除」と「容量制約」に絞る。

本書とは別の作業用ディレクトリで作成し、`src/lib.rs` に配置する。外部依存はなく、Rust 1.90.0で確認した。

```bash
cargo new --lib --edition 2021 template-lab
cd template-lab
```

<details class="code-details">
<summary>ブロックテンプレートモデルの実装とテストを見る</summary>

```rust
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tx {
    pub id: u64,
    pub inputs: Vec<u64>, // 学習用Outpoint ID
    pub fee: u64,
    pub mass: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Template {
    pub parent_ids: Vec<u64>,
    pub transactions: Vec<Tx>,
    pub total_mass: u64,
}

pub fn build_template(parent_ids: Vec<u64>, candidates: Vec<Tx>, max_mass: u64) -> Template {
    // 同一データの再送は1件へまとめる。同じIDの別内容は全候補を除外する。
    let mut by_id: HashMap<u64, Tx> = HashMap::new();
    let mut ambiguous_ids = HashSet::new();
    for tx in candidates {
        if let Some(previous) = by_id.get(&tx.id) {
            if previous != &tx {
                ambiguous_ids.insert(tx.id);
            }
        } else {
            by_id.insert(tx.id, tx);
        }
    }
    let mut candidates: Vec<_> = by_id
        .into_values()
        .filter(|tx| !ambiguous_ids.contains(&tx.id))
        .collect();

    // 教育用の単純な優先度。実際のKaspaの選択方式ではない。
    candidates.sort_by(|a, b| {
        let lhs = (a.fee as u128) * (b.mass.max(1) as u128);
        let rhs = (b.fee as u128) * (a.mass.max(1) as u128);
        rhs.cmp(&lhs).then_with(|| a.id.cmp(&b.id))
    });

    let mut spent = HashSet::new();
    let mut selected = Vec::new();
    let mut total_mass = 0_u64;

    for tx in candidates {
        if tx.mass == 0 || tx.inputs.is_empty() {
            continue;
        }
        let unique_inputs: HashSet<_> = tx.inputs.iter().copied().collect();
        if unique_inputs.len() != tx.inputs.len() {
            continue;
        }
        if unique_inputs.iter().any(|input| spent.contains(input)) {
            continue;
        }
        let Some(next_mass) = total_mass.checked_add(tx.mass) else {
            continue;
        };
        if next_mass > max_mass {
            continue;
        }

        spent.extend(unique_inputs);
        total_mass = next_mass;
        selected.push(tx);
    }

    Template {
        parent_ids,
        transactions: selected,
        total_mass,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx(id: u64, input: u64, fee: u64, mass: u64) -> Tx {
        Tx {
            id,
            inputs: vec![input],
            fee,
            mass,
        }
    }

    #[test]
    fn selects_transactions_within_mass_limit() {
        let result = build_template(vec![100, 101], vec![tx(1, 10, 10, 4), tx(2, 20, 20, 5)], 8);
        assert_eq!(result.parent_ids, vec![100, 101]);
        assert_eq!(result.transactions.len(), 1);
        assert!(result.total_mass <= 8);
    }

    #[test]
    fn rejects_conflicting_inputs() {
        let result = build_template(vec![100], vec![tx(1, 10, 10, 2), tx(2, 10, 20, 2)], 10);
        assert_eq!(result.transactions.len(), 1);
        assert_eq!(result.transactions[0].id, 2);
    }

    #[test]
    fn rejects_duplicate_inputs_inside_one_transaction() {
        let bad = Tx {
            id: 1,
            inputs: vec![7, 7],
            fee: 100,
            mass: 1,
        };
        let result = build_template(vec![100], vec![bad], 10);
        assert!(result.transactions.is_empty());
    }

    #[test]
    fn duplicate_ids_are_handled_before_selection() {
        let first = tx(1, 10, 10, 1);
        let result = build_template(
            vec![100],
            vec![
                first.clone(),
                first,
                tx(2, 20, 9, 1),
                tx(3, 30, 8, 1),
                tx(3, 31, 8, 1),
            ],
            10,
        );
        assert_eq!(
            result.transactions.iter().map(|t| t.id).collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert_eq!(result.total_mass, 2);
    }

    #[test]
    fn candidate_reordering_preserves_the_template() {
        let candidates = vec![
            tx(1, 10, 2, 2),
            tx(2, 20, 3, 3),
            tx(1, 10, 2, 2),
            tx(3, 30, 1, 1),
            tx(3, 31, 1, 1),
        ];
        let expected = build_template(vec![100], candidates.clone(), 5);
        let mut reversed = candidates.clone();
        reversed.reverse();
        let mut rotated = candidates;
        rotated.rotate_left(2);
        assert_eq!(build_template(vec![100], reversed, 5), expected);
        assert_eq!(build_template(vec![100], rotated, 5), expected);
        assert_eq!(
            expected
                .transactions
                .iter()
                .map(|t| t.id)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    #[test]
    fn mass_boundaries_and_invalid_shapes() {
        assert!(build_template(vec![100], vec![tx(1, 10, 1, 1)], 0)
            .transactions
            .is_empty());
        assert!(build_template(
            vec![100],
            vec![
                tx(1, 10, 1, 0),
                Tx {
                    id: 2,
                    inputs: vec![],
                    fee: 1,
                    mass: 1
                }
            ],
            10
        )
        .transactions
        .is_empty());
        let exact = build_template(vec![100], vec![tx(1, 10, 4, 4), tx(2, 20, 5, 5)], 9);
        assert_eq!((exact.transactions.len(), exact.total_mass), (2, 9));
        let overflow = build_template(
            vec![100],
            vec![tx(1, 10, u64::MAX, u64::MAX), tx(2, 20, 0, 1)],
            u64::MAX,
        );
        assert_eq!(
            (overflow.transactions.len(), overflow.total_mass),
            (1, u64::MAX)
        );
    }

    #[test]
    fn skipped_candidates_do_not_reserve_inputs() {
        let oversized = tx(1, 10, 100, 11);
        let duplicate_input = Tx {
            id: 2,
            inputs: vec![10, 10],
            fee: 50,
            mass: 1,
        };
        let valid = tx(3, 10, 1, 1);
        let result = build_template(
            vec![100],
            vec![oversized, duplicate_input, valid.clone()],
            10,
        );
        assert_eq!(result.transactions, vec![valid]);
        assert_eq!(result.total_mass, 1);
    }

    #[test]
    fn empty_candidates_produce_empty_template() {
        let result = build_template(vec![100], vec![], 10);
        assert_eq!(result.total_mass, 0);
        assert!(result.transactions.is_empty());
    }
}
```

</details>

**実行手順：**

```bash
cargo test
```

本文から抽出した教材は8テスト成功（`experiments/chapter-18-review-2026-11-01.md`）。同じIDの同一データは1件にまとめ、別内容ならそのIDの全候補を除外する。これは任意の数値IDを受け取る教材の方針であり、実ノードのハッシュ照合の代用ではない。

教材の選択はfee/massの降順、同率ではID昇順。u128の交差積で比較するため浮動小数点は使わない。候補順を変えても同じ結果になるが、最大手数料となる組み合わせを保証する最適化ではない。

> **モデルの限界**：入力の存在確認、署名、Coinbase、親子取引、UTXOセット、DAG状態、手数料の実際の計算、Kaspaのmass規則は実装していない。特に`inputs: Vec<u64>`は本来のOutpointを代用したものである。parent_idsは入力をそのまま返し、親の存在・重複・数・選択規則は検査しない。total_massは通常取引の申告massだけを合計し、Coinbaseの負荷を含まない。空の選択結果も実KaspaのCoinbaseなしブロックを示さない。

## 18.14 ステートマシンで理解する

テンプレートのライフサイクルを単純化すると、次のようになる。

```text
[No Template]
      │ build
      ▼
[Ready]
      │ distribute
      ▼
[Mining]
  ├── solution ──▶ [Submitted] ──▶ [Validated / Rejected]
  └── DAG update ──▶ [Stale] ──▶ [Rebuild]
```

**不変条件**：

- マイニング仕事は、生成時点の親・難易度・トランザクションコミットメントと結び付く。
- 候補生成に失敗してもコンセンサス状態を変更しない。
- 受理されるブロックは、独立した検証を通過しなければならない。
- テンプレート生成とネットワーク上の確定状態を混同しない。

## 18.15 DDD・C4で責務を整理する

| 境界 | 入力 | 出力 | 主な不変条件 |
|---|---|---|---|
| Consensus | DAG・トランザクション・パラメータ | 検証結果・状態 | 同じ履歴から決定的な結果 |
| Mempool | 未確定トランザクション | 候補トランザクション集合 | ローカルポリシー・競合管理 |
| Template Builder | DAG状態・Mempool | 候補ブロック | コンセンサス制約を満たす候補 |
| Miner | PoW仕事 | 候補解 | 指定された仕事の条件を満たす探索 |
| P2P | ブロック・メッセージ | 伝播・受信 | 未検証データを信用しない |

設計上の質問は「どの関数が仕事をしているか」だけではなく、**どのコンポーネントがそのルールの正しさに責任を持つか**である。

## 18.16 `rusty-kaspa` の調査手順

### 実際のノードとマイナーを動かす

本章のテンプレートモデルを読んだ後は、[第28章28.6](../part-04-development/chapter-28.md#286-ローカル環境の準備)で環境を準備し、[既存Devnet実習の「短時間採掘と再起動後の状態比較」](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/devnet/README.md#短時間採掘と再起動後の状態比較)でノードとCPUマイナーを起動する。起動・採掘停止・再起動前後の比較・終了のコマンドを、その手順にまとめている。ノードはブロック候補を用意し、別プロセスのマイナーがPoWを探索する構成を、実際に動かして観察できる。

見るポイントは、採掘後のブロック数・virtual DAA score・報酬UTXOと、採掘停止後にノードだけを再起動したときの状態保持である。UTXO通知も受けたい場合は、[28.6.4の公式SDK監視サンプル](../part-04-development/chapter-28.md#2864-公式sdkを使った最小監視サンプル)を使う。

この実習は既存マイナーを使った隔離Devnetでの動作確認であり、本章のRustモデルを実マイナーへ接続するものではない。テンプレートの内部フィールドや取引選択を読むには、次の固定ソースを追う。

### 固定ソースの入口

公式リポジトリ：[kaspanet/rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)

実装の構成はリリースによって変わりうるため、まず対象コミットを固定する。

```bash
# 第1章で取得した固定版の親ディレクトリから実行する
cd rusty-kaspa-v1.0.1
git rev-parse HEAD
grep -n -E 'get_block_template_call|submit_block_call' rpc/service/src/service.rs
grep -n -E 'get_block_template|build_selector' mining/src/manager.rs
grep -n -E 'build_selector|TakeAllSelector|SequenceSelector' mining/src/mempool/model/frontier.rs
grep -n -E 'build_block_template|expected_coinbase_transaction' consensus/src/pipeline/virtual_processor/processor.rs
```

未取得なら[第1章](../part-01-foundations/chapter-01.md)の手順を使い、HEADを本章冒頭の完全SHAと照合する。検索結果を起点に、次の呼び出し関係を追う。

1. RPC等からブロックテンプレートを要求する入口
2. DAGの親・難易度を取得する処理
3. Mempoolからトランザクションを選択する処理
4. Coinbaseやコミットメントを構築する処理
5. テンプレートをマイナーへ返す処理
6. 解を受け取りブロック検証へ渡す処理

**調査記録テンプレート**：

```text
対象コミット:
呼び出し入口:
テンプレート構築関数:
親選択の責務:
取引選択の責務:
Coinbase構築の責務:
PoW検証の責務:
関連テスト:
未確認事項:
```

検索結果やコメントだけでなく、実際に選ばれる処理と本体を追う。

## 18.17 観測・障害調査

マイニングが期待どおり動かない場合、まずどの境界で止まっているかを分ける。

| 観測症状 | 最初に調べる対象 |
|---|---|
| テンプレートが返らない | ノード同期状態、RPC、コンセンサス初期化 |
| テンプレート更新が遅い | DAG更新イベント、キャッシュ、キュー |
| シェアが大量に拒否される | 仕事ID、難易度、Nonceのエンコード、時刻 |
| ブロック提出後に拒否される | PoW以外のヘッダー・トランザクション・Coinbase検証 |
| Mempoolトランザクションが選ばれない | mass制限、依存関係、手数料、競合 |
| ノード間で候補が違う | DAGの見え方、同期遅延、ローカルポリシー |

記録すべき項目の例：テンプレート生成時刻、参照親、DAGの状態識別子、トランザクション件数、合計mass、難易度情報、提出結果、拒否理由。

## 18.18 BitcoinとKaspaの比較

| 観点 | Bitcoin | Kaspa |
|---|---|---|
| ブロックの構造 | 競合する枝から採用チェーンを選ぶ | BlockDAG |
| 親参照 | 通常は直前ブロック1つ | 複数の親を参照できる |
| 並行生成ブロック | 主チェーン外の競合ブロックが生じる | DAG上に並行ブロックを表現できる |
| 順序決定 | チェーン順序 | GHOSTDAGに基づくコンセンサス順序 |
| PoW | Bitcoin固有のPoW | Kaspa固有のPoW |
| テンプレート | チェーン先端に依存 | DAG状態・親集合に依存 |
| トランザクションの受理 | チェーン上の検証・状態遷移 | DAGの順序とVirtual Stateに依存 |

**「BlockDAGだからPoWが不要」ではない。** データ構造とSybil耐性の仕組みは異なる設計レイヤーに属する。

## 18.19 実践課題

<details class="exercise-details">
<summary>実践課題を見る</summary>

### 課題A：テンプレート更新をモデル化する

第18.13節の`Template`に`generation: u64`を追加し、DAG更新時に世代番号を増やす。古い世代の仕事に対する解を識別するテストを書く。

### 課題B：トランザクション依存関係を追加する

親トランザクションが作る出力を子トランザクションが消費するケースをモデル化する。単純な手数料順ソートでは不十分になる理由を説明する。Kaspaの同一ブロック内では親子を同梱せず、先行ブロックの出力として使用可能になるまで子を待たせる設計と、一般的な親子同梱モデルを区別する。

### 課題C：障害注入

「テンプレート構築中に新しいブロックを受信した」ケースを想定し、スナップショットの整合性を守る設計案を2つ比較する。

### 課題D：ソースコード調査

対象コミットを固定し、テンプレート生成の入口からブロック提出までを呼び出しグラフにする。確認できた事実と推測を別欄に記録する。

</details>

## 18.20 理解度チェック

<details class="quiz-answers">
<summary>理解度チェックを見る</summary>

1. フルノードとマイナーの責務はどう違うか。
2. ブロックテンプレートを更新する必要があるのはなぜか。
3. BlockDAGの親集合は、Bitcoinの前ブロック参照とどう違うか。
4. Mempoolに入っているトランザクションが必ずブロックに入るわけではない理由は何か。
5. プールのシェア受理とネットワークのブロック受理は何が違うか。
6. PoW条件を満たしてもブロックが拒否されるのはどのような場合か。
7. Coinbaseと通常の送金トランザクションの役割はどう違うか。
8. テンプレート生成器が依存する状態を3つ挙げよ。
9. massとバイトサイズを同一視してはいけないのはなぜか。
10. マイニングの集中化を評価するとき、ハッシュレート以外に何を観察するべきか。

**解答例**

1. フルノードは共有規則でデータを検証し状態を維持する。マイナーはテンプレート等から候補を作りPoWを探索する。マイナーの提出もノードの検証を受ける。
2. 親・Virtual状態・Mempool候補・時刻や規則の文脈等が変わるため。状態に応じて再利用・修正・再生成する。
3. Bitcoinの単一の前ブロック参照に対し、BlockDAGでは複数親を参照して並行履歴を取り込める。親選択にもコンセンサス規則がある。
4. 競合・依存関係、VirtualのUTXO view、mass制限、候補選択方針等で選外や再検証失敗が起きるため。基準版では同一ブロック内の親子取引もそのまま詰め込まない。
5. シェアはプールが貢献を測るための条件、ブロック受理はネットワークのPoWを含む合意規則への適合である。シェア成功がブロック発見とは限らない。
6. 期待難易度の不一致、親・時刻・形式・コミットメント等の違反や、不正な取引を含む場合など。PoWだけで全規則は満たせない。
7. Coinbaseはプロトコル上の報酬処理を担い、通常送金は既存UTXOを支出する。Coinbaseの特別な構造・報酬規則を通常送金に一般化しない。
8. 一例はVirtualの合意状態、親候補となるDAG情報、Mempoolの取引候補である。ネットワークパラメータ等も参照する。
9. massは検証・計算やストレージ等の資源負担を評価する尺度で、単なるシリアライズ後のバイト数ではないから。式は対象版と有効化文脈で確認する。
10. テンプレート作成・取引選択の主体、プール運用者、機材所有者、接続・地域や運用基盤の偏り等を分けて観察する。

</details>

## 18.21 まとめ

<details class="summary-details">
<summary>まとめを見る</summary>

本章の核心は、**マイニングをPoW探索だけに還元しないこと**である。

Kaspaでは、BlockDAGの状態に基づく親選択、Mempoolからのトランザクション選択、Coinbase、難易度、PoW探索、提出後の独立検証が一つの処理系としてつながる。しかし、それぞれの責務と信頼境界は異なる。テンプレートは一時点の候補にすぎず、ネットワーク上の最終的な受理はコンセンサスが決める。

</details>

[次章「Chapter 19 — Mining Economics & Network Security」](chapter-19.md)では、報酬・費用・ハッシュレート・マイニングの集中化がネットワークの安全性にどう影響するかを扱う。

## 18.22 参考資料

<details class="reference-details">
<summary>参考資料を見る</summary>

- [Kaspa公式サイト](https://kaspa.org/)
- [Kaspa Wiki](https://wiki.kaspa.org/en/home)
- [rusty-kaspa（公式Rustノード実装）](https://github.com/kaspanet/rusty-kaspa)
- [Kaspa Improvement Proposals（KIPs）](https://github.com/kaspanet/kips)
- [Bitcoin Developer Reference — Block Chain](https://developer.bitcoin.org/reference/block_chain.html)
- [Bitcoin Whitepaper](https://bitcoin.org/bitcoin.pdf)

</details>

<nav class="chapter-next" aria-label="次のページ">

[次章：Chapter 19 — Mining Economics & Network Security（マイニング経済とネットワークセキュリティ）](chapter-19.md)

</nav>

<details class="verification-details">
<summary>検証状況を見る</summary>

[対象バージョン](../baseline.md)・[このページのレビュー記録](../review-status.md#review-chapter-18)

### 技術レビュー時の確認項目

- [x] 対象コミットを固定し、現在のMainnetや実測Devnetとの同一性へ一般化していないか
- [ ] PoW方式・テンプレートフィールド・Coinbaseの説明を現行実装で確認したか
- [ ] ブロック報酬の適格性と手数料の扱いをソースで確認したか
- [x] 概念図と主要な実装経路・選択ポリシーを区別したか
- [x] 教育用Rustコードの独自方針と未実装の条件を明記したか

主要経路のソース照合と教材8テストは `experiments/chapter-18-review-2026-11-01.md`。全選択器の分布・キャッシュ/更新競合・報酬計算・PoWの既知値・公式関数との差分実行・実際の採掘は今回検証していない。

</details>
