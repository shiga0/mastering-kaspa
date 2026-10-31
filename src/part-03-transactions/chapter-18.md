# Chapter 18 — Mining & Block Template Construction（マイニングとブロックテンプレート構築）

> **Mastering Kaspa / Part III — Transactions, Mining & Security**  
> 原稿ステータス：v0.1（技術レビュー前）  
> この章のRustコードは教育用モデルであり、Kaspaメインネットのブロック生成・PoWを再現するものではない。

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

この図は構造の違いを示すための簡略化である。実際のKaspaの親集合の選択には、DAGの状態、親候補の制約、コンセンサス規則が関与する。単に「見えている先端を全部親にする」わけではない。

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

**テンプレートは確定済みブロックではない。** 親ブロックやMempoolが変化すると、候補を作り直す必要がある。

### 18.4.1 依存関係を先に見る

```text
Consensus State ──┐
                  ├──▶ Template Builder ──▶ Mining Work
DAG Tips ─────────┤
                  │
Mempool ──────────┤
                  │
Network Params ───┘
```

テンプレート生成器は、複数の状態の**整合したスナップショット**を必要とする。親だけ新しく、難易度やトランザクション選択だけ古いといった不整合を防ぐ設計が重要になる。

## 18.5 親ブロックの選択

BlockDAGでは、並行して発見された複数のブロックが存在する。新しいブロックは、コンセンサス規則に従って適切な親集合を参照する。

ここで区別したい概念は次のとおり。

- **Tip**：現在知られているDAGの末端ブロック。選択対象の候補になりうる。
- **Selected parent**：GHOSTDAGの規則に基づいて選択される特別な親。
- **Parents**：新しいブロックが直接参照する親集合。
- **Past / Anticone**：ブロック間の到達可能性から定まる関係。

親集合の決定は、単なるP2P処理ではなく、コンセンサスに関係する操作である。第12章のGHOSTDAG、第14章のVirtual Stateと合わせて読むと理解しやすい。

## 18.6 Mempoolからのトランザクション選択

ブロックテンプレート生成器は、Mempoolに存在するトランザクションを無条件に詰め込むわけではない。少なくとも以下の観点がある。

1. トランザクションの構造・署名・スクリプト・UTXOに関する検証
2. 同一UTXOの二重消費を防ぐこと
3. 親子トランザクションなどの依存関係
4. ブロックのサイズ・質量（mass）等の制限
5. 手数料とブロック内でのトランザクション選択方針
6. 新しいVirtual Stateに対して再評価が必要なトランザクション

Kaspaの**mass**は単純なバイト数と同義ではない。正確な計算式や制限値は、対象ネットワークとノード実装のバージョンを確認する。

### 18.6.1 トランザクション選択の不変条件

```text
選択済み取引集合 S について：

- 同じOutpointを二度消費しない
- 各取引が適用時点で有効である
- 依存関係を満たす順序で評価する
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

> **実装確認ポイント**：Coinbase payload、報酬計算、受理・支払い条件は`rusty-kaspa`の対象リリースで検証する。数値や例外条件を推測で固定しない。

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

難易度調整の詳細は第13章を参照。

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

以下は**コンセンサス非互換**の学習用コード。実際のKaspaでは署名・UTXO・mass・親選択・報酬・PoWを正式なルールで処理する必要がある。ここでは、テンプレート生成時の「トランザクションの重複入力排除」と「容量制約」に絞る。

`cargo new template-lab` で作成したプロジェクトの `src/lib.rs` に配置する。

```rust
use std::collections::HashSet;

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

pub fn build_template(
    parent_ids: Vec<u64>,
    mut candidates: Vec<Tx>,
    max_mass: u64,
) -> Template {
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
        Tx { id, inputs: vec![input], fee, mass }
    }

    #[test]
    fn selects_transactions_within_mass_limit() {
        let result = build_template(
            vec![100, 101],
            vec![tx(1, 10, 10, 4), tx(2, 20, 20, 5)],
            8,
        );
        assert_eq!(result.parent_ids, vec![100, 101]);
        assert_eq!(result.transactions.len(), 1);
        assert!(result.total_mass <= 8);
    }

    #[test]
    fn rejects_conflicting_inputs() {
        let result = build_template(
            vec![100],
            vec![tx(1, 10, 10, 2), tx(2, 10, 20, 2)],
            10,
        );
        assert_eq!(result.transactions.len(), 1);
        assert_eq!(result.transactions[0].id, 2);
    }

    #[test]
    fn rejects_duplicate_inputs_inside_one_transaction() {
        let bad = Tx { id: 1, inputs: vec![7, 7], fee: 100, mass: 1 };
        let result = build_template(vec![100], vec![bad], 10);
        assert!(result.transactions.is_empty());
    }

    #[test]
    fn empty_candidates_produce_empty_template() {
        let result = build_template(vec![100], vec![], 10);
        assert_eq!(result.total_mass, 0);
        assert!(result.transactions.is_empty());
    }
}
```

実行例：

```bash
cargo test
```

> **モデルの限界**：入力の存在確認、署名、Coinbase、親子取引、UTXOセット、DAG状態、手数料の実際の計算、Kaspaのmass規則は実装していない。特に`inputs: Vec<u64>`は本来のOutpointを代用したものである。

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

公式リポジトリ：[kaspanet/rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)

実装の構成はリリースによって変わりうるため、まず対象コミットを固定する。

```bash
git clone https://github.com/kaspanet/rusty-kaspa.git
cd rusty-kaspa
git rev-parse HEAD
rg -n 'get_block_template|submit_block|BlockTemplate|coinbase|pow' .
rg -n 'mining|template|mass|mempool' consensus mining* rpc* 2>/dev/null
```

`rg` の検索結果を起点に、次の呼び出し関係を追う。

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

> `rg` の語句は調査用の手がかりであり、特定の関数名やディレクトリの存在を保証するものではない。

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
| ブロックの構造 | 線形チェーン | BlockDAG |
| 親参照 | 通常は直前ブロック1つ | 複数の親を参照できる |
| 並行生成ブロック | 主チェーン外の競合ブロックが生じる | DAG上に並行ブロックを表現できる |
| 順序決定 | チェーン順序 | GHOSTDAGに基づくコンセンサス順序 |
| PoW | Bitcoin固有のPoW | Kaspa固有のPoW |
| テンプレート | チェーン先端に依存 | DAG状態・親集合に依存 |
| トランザクションの受理 | チェーン上の検証・状態遷移 | DAGの順序とVirtual Stateに依存 |

**「BlockDAGだからPoWが不要」ではない。** データ構造とSybil耐性の仕組みは異なる設計レイヤーに属する。

## 18.19 実践課題

### 課題A：テンプレート更新をモデル化する

第18.13節の`Template`に`generation: u64`を追加し、DAG更新時に世代番号を増やす。古い世代の仕事に対する解を識別するテストを書く。

### 課題B：トランザクション依存関係を追加する

親トランザクションが作る出力を子トランザクションが消費するケースをモデル化する。単純な手数料順ソートでは不十分になる理由を説明する。

### 課題C：障害注入

「テンプレート構築中に新しいブロックを受信した」ケースを想定し、スナップショットの整合性を守る設計案を2つ比較する。

### 課題D：ソースコード調査

対象コミットを固定し、テンプレート生成の入口からブロック提出までを呼び出しグラフにする。確認できた事実と推測を別欄に記録する。

## 18.20 理解度チェック

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

## 18.21 まとめ

本章の核心は、**マイニングをPoW探索だけに還元しないこと**である。

Kaspaでは、BlockDAGの状態に基づく親選択、Mempoolからのトランザクション選択、Coinbase、難易度、PoW探索、提出後の独立検証が一つの処理系としてつながる。しかし、それぞれの責務と信頼境界は異なる。テンプレートは一時点の候補にすぎず、ネットワーク上の最終的な受理はコンセンサスが決める。

次章「Chapter 19 — Mining Economics & Network Security」では、報酬・費用・ハッシュレート・マイニングの集中化がネットワークの安全性にどう影響するかを扱う。

## 18.22 参考資料

- [Kaspa公式サイト](https://kaspa.org/)
- [Kaspa Wiki](https://wiki.kaspa.org/en/home)
- [rusty-kaspa（公式Rustノード実装）](https://github.com/kaspanet/rusty-kaspa)
- [Kaspa Improvement Proposals（KIPs）](https://github.com/kaspanet/kips)
- [Bitcoin Developer Reference — Block Chain](https://developer.bitcoin.org/reference/block_chain.html)
- [Bitcoin Whitepaper](https://bitcoin.org/bitcoin.pdf)

### 技術レビュー時の確認項目

- [ ] 対象`rusty-kaspa`コミットとネットワーク（mainnet/testnet/devnet）を記録したか
- [ ] PoW方式・テンプレートフィールド・Coinbaseの説明を現行実装で確認したか
- [ ] ブロック報酬の適格性と手数料の扱いをソースで確認したか
- [ ] 概念図と実装上の処理順序を混同していないか
- [ ] 教育用Rustコードをコンセンサス互換と誤認させていないか
