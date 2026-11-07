# Chapter 14 — Consensus Ordering & Virtual State（コンセンサス順序と仮想状態）

<p class="chapter-part">Part II — Kaspa Consensus &amp; Architecture（コンセンサスとアーキテクチャ）</p>

> 本章はKaspaの設計概念を解説する。Rustのサンプルは**教育用の状態遷移モデル**であり、`rusty-kaspa` のコンセンサス互換実装ではない。プロトコル上の厳密な順序、受理規則、例外処理は対象バージョンの実装・仕様で確認すること。

## 照合した実装

2026年10月31日、[本書の基準版](../baseline.md) v1.0.1、コミット `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` のソースを読み取った。以下の具体的な処理はSource-checkedであり、公式ノードを実行した差分テストや章全体のVerified判定ではない。照合ファイルと未確認範囲はリポジトリの `experiments/consensus-state-review-2026-10-31.md` に記録する。

## この章で学ぶこと

前章までに、BlockDAG、GHOSTDAG、blue/red、Selected Parent、blue work、難易度調整を学んだ。本章ではそれらをつなぎ、**複数のブロックから一貫したトランザクション処理順序を導き、UTXO集合を更新する**までを考える。

読み終えたら、次の問いに答えられるようになる。

- ブロックの親子関係と、トランザクションの実行順序はなぜ別なのか。
- GHOSTDAGの結果はどのように合意上の順序へ結び付くのか。
- Virtual BlockとVirtual Stateは何を表すのか。
- 並行ブロックに二重支払いが含まれた場合、何が起こるのか。
- DAGの先端が変化すると、どの状態を再評価する必要があるのか。
- `rusty-kaspa` のコードでどの責務を追えばよいのか。

---

## 14.1 「順序」と「状態」は別の問題

ブロックチェーンでは、競合する枝から選ばれた採用チェーンが1本の列を形成するため、ブロックの並びがトランザクションを処理する順序の土台になる。一方、BlockDAGには並行して生成されたブロックが存在する。

```text
        Genesis
           │
           A
         ╱   ╲
        B     C
         ╲   ╱
           D
```

図は上が親、下が子で、参照は子から親へ向かう。`B` と `C` は、いずれも `A` を参照しているが、互いを親子として参照していない。DAGの構造だけでは、`B` と `C` のどちらに含まれるトランザクションを先に処理すべきかは決まらない。

**コンセンサス順序（consensus ordering）**は、このような並行性を含むブロック集合から、参加ノードが同じ規則に従って計算できる順序を定めるために必要となる。

さらに、順序が定まったとしても、すべてのトランザクションを適用できるとは限らない。たとえば同じUTXOを使うトランザクションが2つあれば、先に適用されたトランザクションがそのUTXOを消費し、後のトランザクションは適用できない。

したがって、次の3つを区別する。

| 層 | 主な問い | 出力の例 |
|---|---|---|
| DAG構造 | どのブロックがどのブロックを参照するか | 親・祖先・Anticone |
| コンセンサス順序 | ブロックをどの順に扱うか | 順序付きブロック列 |
| 状態遷移 | トランザクションを適用できるか、結果は何か | UTXO集合・受理結果 |

この区別は、Kaspaの実装を読むときにも重要である。

## 14.2 全体像：BlockDAGからVirtual Stateまで

```text
   P2Pからブロックを受信
             │
             ▼
      ブロックの検証
             │
             ▼
        BlockDAG
     親子関係・DAG tips
             │
             ▼
         GHOSTDAG
  Selected Parent / Blue Work
             │
             ▼
      Consensus Ordering
       合意上の処理順序
             │
             ▼
    Transaction Acceptance
   UTXO・二重支払い等の検査
             │
             ▼
        Virtual State
      現時点の合意状態
             │
             ▼
       RPC / Wallet等
```

GHOSTDAGはヘッダー検証の中で計算される。本体・UTXO検証がすべて終わってから初めてGHOSTDAGを実行する図とは読まない。

これは**概念上の責務図**であり、実装内の関数呼び出し順を示すものではない。検証・状態更新・永続化は複数の段階に分かれ、キャッシュや並列処理も関係する。

### 14.2.1 決定的であること

異なるノードが同じブロック集合と同じ合意規則を持つなら、同じコンセンサス結果を計算できなければならない。ローカル受信時刻やOSのスレッド実行順によって結果が変わってはならない。

### 14.2.2 状態が履歴に依存すること

UTXO集合は、合意された基準状態に受理されたトランザクションの効果を反映した結果である。Genesisから再構築する説明と、Pruning後の基準状態・検証済み差分から更新する処理を区別する。単に「ブロックが存在する」だけでは現在の残高を決定できない。

---

## 14.3 DAGの部分順序と合意上の順序

DAGの辺は依存関係を表す。あるブロックが別のブロックを祖先に持つ場合、その関係は順序の制約になる。一方、並行ブロック間にはそのような祖先関係がない。

数学的には、DAGの到達可能性から**部分順序（partial order）**が得られる。コンセンサス処理では、これと整合する決定的な並びを導く必要がある。

ただし、Kaspaの順序付けは**単なる一般的なトポロジカルソートではない**。GHOSTDAGの選択親、マージ集合、blue/redに関する情報など、プロトコルで定義された規則が関係する。

### 14.3.1 なぜ受信順ではだめなのか

ノードXは `B → C` の順に受信し、ノードYは `C → B` の順に受信することがある。

```text
Node X:  A ── B ───── C
Node Y:  A ── C ───── B
```

受信順でトランザクションを確定すると、ノードごとに二重支払いの勝者が異なり得る。ネットワークの到着順序は合意の根拠にできない。

### 14.3.2 順序は「有効性」と同じではない

有効なブロックが含むトランザクションであっても、合意上のトランザクション受理結果は別途決まる。**ブロックがDAGに残ること**と**その中の全トランザクションがVirtual UTXO集合に反映されること**は同義ではない。

---

## 14.4 Selected Parentとマージ集合

GHOSTDAGでは、ブロックの親のうち規則に従って**Selected Parent**が選ばれる。Selected Parentを繰り返したどる経路は、合意を理解するうえで中心的な構造である。

```text
        A
       / \
      B   C
       \ /
        D
```

`D` が `B` と `C` を親に持つ場合、どちらかがSelected Parentとなり、他方の側で新たに合流したブロックはマージ集合に関係する。

注意点は以下のとおり。

- Selected Parentは「最初に受信した親」ではない。
- Selected Parentは「必ず最も新しいタイムスタンプの親」でもない。
- マージ集合は「すべての祖先」を単純に並べたものではない。
- blue/redは、トランザクションが有効か無効かを直接示す分類ではない。

基準版の親選択は `(blue_work, hash)` の最大値を用いる。マージ集合がselected parentを含むかは関数・データ構造ごとに区別する。[第12章](chapter-12.md)の定義と照らし合わせて読むこと。

## 14.5 blue/red分類と順序付け

GHOSTDAGのblue/redは、DAGの並行性を扱うための**ブロックの分類**である。blueはトランザクション受理の成功、redはトランザクション検証の失敗を意味しない。

| 用語 | 主な意味 | 誤解しやすい点 |
|---|---|---|
| blue | GHOSTDAGの規則によるブロック分類 | 「トランザクションが必ず有効」ではない |
| red | GHOSTDAGの規則によるブロック分類 | 「不正なブロック」ではない |
| blue score | 選択されたDAG履歴に関するスコア | トランザクション件数ではない |
| blue work | 累積的なPoWの比較に用いる指標 | 単純なブロック数ではない |
| Consensus Ordering | プロトコルで導くブロック順序 | ネットワーク受信順ではない |

ブロックの分類と順序を経て、トランザクション処理のための一貫した履歴が構成される。

---

### 14.5.1 基準版の実際の処理順

[GhostdagDataの合意順序](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/model/stores/ghostdag.rs)では、selected parentを先頭に置き、それ以外のmerge setをblue work、同値時はhashの昇順で並べる。blueとredの列を統合するため、「blueだけのトランザクションを処理する」という意味ではない。

[calculate_utxo_state](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/virtual_processor/utxo_validation.rs)はselected parentの過去のUTXO viewを基準とし、次の順に差分を作る。

1. selected parentのCoinbaseを差分とMuHashへ加える。
2. selected parent、続いて残りのmerge setを合意順に評価する。
3. ブロックごとに、基準viewとそれまでのmerge set差分を合成する。そのviewに対して通常トランザクションを検証し、成功した分だけ差分・受理情報・手数料へ反映する。
4. selected parent以外のCoinbase自体はこの通常トランザクションの検証から除く。各ブロックの報酬情報は別に収集する。

selected parentの通常トランザクションでは、同じUTXO文脈で既に検証済みであることを前提にScript再検査を省く。一般の入力が未検証のまま受理される、という規則ではない。

同一ブロック内の通常トランザクションは独立であることを[body検証](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/body_processor/body_validation_in_isolation.rs)で確認し、その後UTXO文脈の検証を並列化する。教材の「1件ずつ入力を消して次のトランザクションへ進む」というモデルを、そのまま実ノードのブロック内処理と見なさない。

受理IDの並びにもフォーク条件がある。Crescendo前はIDをソートし、有効化後はcanonical orderを保持する。判定には処理対象のヘッダーに対応する `pov_daa_score` を使う。GHOSTDAGやDAAの切り替えで使うselected parentのDAA scoreと機械的に置き換えない。これはトランザクションIDのコミットメントに関する処理であり、適用順を後からID順に変えるという意味ではない。

## 14.6 Virtual Blockとは何か

BlockDAGには、複数の**tip（子を持たない先端ブロック）**が存在し得る。新しいブロックがまだすべてのtipを参照していない時点でも、ノードは現在見えているDAG全体に基づいて合意上の状態を計算する必要がある。

Kaspaの**Virtual Block**は、この目的で用いる概念的な仮想ブロックである。実際にマイニングされ、ネットワークへ伝播する通常のブロックではない。

```text
               Genesis
                  │
                  A
                /   \
               B     C
               │     │
               D     E
                \   /
             [Virtual]
            (概念的な先端)
```

Virtual BlockはDAGの先端を評価する抽象化である。ただし、基準版は全tipsを無条件に親へ採用するわけではない。[pick_virtual_parents](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/virtual_processor/processor.rs)が親数やpruning等の制約を適用し、選ばれた親からvirtualのGHOSTDAGデータを計算する。

> **重要：** **Virtual Blockは、実在するブロックのようにPoWを解いて発行されるものではない。** また、図は概念図であり、仮想ブロックの内部表現を厳密に再現したものではない。

### 14.6.1 なぜ必要なのか

もし最後にマイニングされた単一ブロックだけを現在状態の基準にすると、他のtipに存在する並行ブロックを適切に扱えない。Virtual Blockは、観測済みDAGの先端をまとめて評価するための抽象化となる。

### 14.6.2 Virtual Blockは永続的な履歴の1ブロックか

通常の採掘ブロックとしてDAGへ追加されるものではない。ノードが新たなブロックを受け取れば、仮想的な先端の見方も変わり得る。

---

## 14.7 Virtual Stateとは何か

**Virtual State**は、現在のVirtual Blockに対応して計算されるコンセンサス上の状態を指す。基準版の[VirtualState](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/model/stores/virtual_state.rs)は、親群・GHOSTDAGデータ・DAA score・past median time・UTXO差分・受理IDなどを保持し、UTXO集合だけを指す名前ではない。アプリケーションから見えるUTXO情報や、トランザクション受理結果を理解するうえで重要である。

概念的には、次のように表せる。

```text
Initial UTXO Set
      │
      ▼
Consensus-ordered transactions
      │
      ▼
Validation and application
      │
      ▼
Virtual UTXO Set
```

この図は説明用であり、実際のKaspaではブロックごとの受理情報、過去状態、仮想状態の差分、永続化など、さらに多くの要素が関係する。

### 14.7.1 Virtual Stateとウォレット残高

ウォレットが扱う「残高」は、通常、所有するスクリプト条件に対応したUTXOの集合から導かれる。したがって、Virtual Stateの変化はウォレットが観測する使用可能UTXOに影響し得る。

ただし、ウォレットの表示残高には未承認トランザクションやローカルポリシーなどが影響することがあり、**コンセンサス状態とUI上の表示値を無条件に同一視してはいけない**。

## 14.8 UTXO状態遷移を定義する

UTXOモデルでは、未使用の出力を識別するために、通常、トランザクションIDと出力インデックスの組を用いる。

```text
Outpoint = (transaction_id, output_index)
UTXO     = (outpoint, value, locking_condition, ...)
```

トランザクションを適用するときの基本的な考え方は以下である。

1. 入力が参照するUTXOが存在することを確認する。
2. 署名やスクリプトなどの使用条件を検証する。
3. 金額・手数料・その他のコンセンサス条件を検証する。
4. 消費された入力UTXOを集合から除く。
5. 新たな出力UTXOを集合へ追加する。

**状態遷移の不変条件**として、少なくとも「存在しないUTXOを消費しない」「同じUTXOを二重に消費しない」「不正な値を作り出さない」を意識する。

実際のKaspaでは、coinbase、成熟条件、スクリプト検証、ブロック単位の受理、手数料などを含めた厳密なルールがある。ここではそれらを省略する。

---

## 14.9 並行ブロックと二重支払い

Aliceが所有するUTXO `U` を考える。

```text
                    U = 10 KAS
                       │
              ┌────────┴────────┐
              ▼                 ▼
       Tx1: Bobへ送金      Tx2: Carolへ送金
              │                 │
           Block B           Block C
              └────────┬────────┘
                       ▼
                 BlockDAG
                       │
                       ▼
                Consensus Order
                       │
                       ▼
               UTXO State Check
```

`Tx1` と `Tx2` は同じ `U` を消費しようとするため、**両方を同じ履歴上で受理することはできない**。合意上の順序と受理規則に基づき、一方が受理されれば、後の競合トランザクションは同じUTXOを消費できない。

ここで大切なのは、次の2つを混同しないこと。

- ブロック `B` と `C` がDAG上に存在できること。
- `B` と `C` のすべてのトランザクションが両方とも受理されること。

**BlockDAGが並行ブロックを扱えることは、二重支払いを許すことではない。**

## 14.10 ブロックの受理とトランザクションの受理

実装を調べる際には、少なくとも次のレベルを分けて考える。

| 判定対象 | 問い | 例 |
|---|---|---|
| ヘッダー | PoWや親参照などが規則を満たすか | 不正なターゲットを拒否 |
| ブロック | 構造・データ・コンセンサス条件を満たすか | 不正なMerkle情報を拒否 |
| DAGへの組み込み | 親との関係を保持できるか | 未知の親を待つ |
| コンセンサス順序 | どの順に処理するか | 並行ブロックを決定的に並べる |
| トランザクション受理 | 現在の合意履歴で適用可能か | 二重支払いを不受理 |

これらをすべて `valid = true/false` の1変数に押し込めると、障害調査や設計の理解が難しくなる。

## 14.11 DAG tipsの変化とVirtual Stateの再計算

新しいブロックが到着すると、DAG tips、Virtual側のselected parent、仮想的なコンセンサス順序が変化することがある。親集合と共通規則が同じ既存ブロックのselected parentが、新しい子孫の到着だけで変わるという意味ではない。そのためVirtual Stateも更新対象になる。

```text
時刻 t1:
  DAG tips = {B, C}
  Virtual ordering = [..., B, C]  （概念例）
  Virtual UTXO set = S1

時刻 t2:
  新しいブロック D が到着
  DAG tips = {D, ...}
  Virtual ordering = 再評価された順序
  Virtual UTXO set = S2
```

`S1` と `S2` が必ず異なるわけではない。また、上記の順序は**説明用の仮想例**であり、実際のGHOSTDAGの順序を計算した結果ではない。

### 14.11.1 基準版の差分処理

[resolve_virtualとcalculate_utxo_state_relatively](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/virtual_processor/processor.rs)は、旧virtualの差分を逆にしたものから始め、selected-parent chainの分岐に沿って古い差分を戻し、新しい経路の状態を評価する。[UtxoDiff](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/utxo/utxo_diff.rs)は逆差分と差分の合成を提供する。

[commit_virtual_state](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/virtual_processor/processor.rs)では、virtual UTXO集合、virtual state、selected chainの変更を同じRocksDB WriteBatchに入れて書き込む。このソース確認だけで電源断時の耐久性まで検証したとは扱わない。

### 14.11.2 全履歴を毎回再実行するのか

概念上は初期状態から全トランザクションを再適用すれば状態を得られる。しかし実装で毎回それを行うのは非効率である。実際のノードでは過去状態、差分、キャッシュ、ストレージなどを使い、必要な範囲の更新を効率化する。

### 14.11.3 「現在状態」は絶対不変ではない

新しいブロックの到着やネットワーク分断からの回復によって、観測される合意状態が変わる場合がある。これは**確定性（finality）**を考える動機となる。確定性の厳密な扱いは第20章で学ぶ。

---

## 14.12 Reorganizationと状態の巻き戻し

ブロックチェーンの再編成（reorg）は、通常、採用されるチェーンの切り替えとして説明される。KaspaではBlockDAGとVirtual Stateを考慮するため、変化を単純な「最長チェーンの交換」だけで説明できない。

状態を更新する一般的な考え方は次のとおり。

```text
Old accepted history ──┐
                       ├── Difference ──► Revert / Apply ──► New state
New accepted history ──┘
```

ここでの `Revert / Apply` は状態管理を理解するための抽象図であり、`rusty-kaspa` の特定API名ではない。

設計上の重要な問いは次の3つである。

1. どこまでの履歴を安定した基準点として扱えるか。
2. どのトランザクション受理結果が変化したかを検出できるか。
3. 差分適用の途中で障害が発生した場合、整合性を回復できるか。

これらはPruningやストレージ設計（[第21章](../part-04-development/chapter-21.md)）にもつながる。

## 14.13 Safety、Liveness、Finality

分散コンセンサスの評価には、似ているが異なる3つの概念がある。

**Safety（安全性）**：相互に矛盾する状態をノードが確定的な合意結果として採用しないこと。

**Liveness（活性）**：適切なネットワーク条件下で、新しいブロックやトランザクションが処理され続けること。

**Finality（確定性）**：あるトランザクションや状態が後から覆る可能性を、どのような前提で評価するか。

PoW型システムでは、一般に攻撃者の計算力、ネットワーク条件、観測する承認の深さ・スコアなどの前提が重要になる。Kaspaの具体的な確定性を、Bitcoinの「6承認」と機械的に同一視してはいけない。

---

## 14.14 Rustで最小UTXO状態遷移を実装する

ここでは、**すでに決まったトランザクション順序**を入力としてUTXO集合を更新する最小モデルを作る。GHOSTDAGの順序付けは実装しない。

[第5章](../part-01-foundations/chapter-05.md)では、個々の取引を検証し、成功時だけUTXOを更新することを学んだ。本章ではその仕組みを土台に、**同じ取引集合でも処理順序によって結果が変わるか**を確かめる。競合取引では先に受理された側の出力が残り、独立した取引では順序を入れ替えても同じ集合になる。コードを読む際は、特に `reverse_order_changes_accepted_outputs` と `independent_spends_commute` の2テストに注目する。

### 14.14.1 プロジェクト作成

本書とは別の作業用ディレクトリで実行する。

```bash
cargo new --bin --edition 2021 consensus-state-lab
cd consensus-state-lab
```

以下を `src/main.rs` に保存する。外部クレートは不要で、Rust 1.90.0で確認した。

<details class="code-details">
<summary>コンセンサス順序・状態モデルの実装とテストを見る</summary>

```rust
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Outpoint {
    txid: String,
    index: u32,
}

#[derive(Debug, Clone)]
struct Transaction {
    id: String,
    inputs: Vec<Outpoint>,
    outputs: Vec<u64>,
}

#[derive(Debug, Default)]
struct UtxoState {
    utxos: HashMap<Outpoint, u64>,
}

#[derive(Debug, PartialEq, Eq)]
enum ApplyError {
    InvalidStructure,
    OutputIndexOverflow,
    DuplicateInput,
    MissingInput,
    ValueOverflow,
    InsufficientInput,
    OutputAlreadyExists,
}

impl UtxoState {
    fn apply(&mut self, tx: &Transaction) -> Result<(), ApplyError> {
        // 途中失敗で状態を壊さないよう、すべて検査してから更新する。
        if tx.id.is_empty() || tx.inputs.is_empty() || tx.outputs.is_empty() {
            return Err(ApplyError::InvalidStructure);
        }
        // 教材は出力数がu32に収まる範囲に制限する。
        let output_count =
            u32::try_from(tx.outputs.len()).map_err(|_| ApplyError::OutputIndexOverflow)?;
        let mut seen = HashSet::new();
        let mut input_sum = 0u64;

        for input in &tx.inputs {
            if !seen.insert(input) {
                return Err(ApplyError::DuplicateInput);
            }
            let value = *self.utxos.get(input).ok_or(ApplyError::MissingInput)?;
            input_sum = input_sum
                .checked_add(value)
                .ok_or(ApplyError::ValueOverflow)?;
        }

        let mut output_sum = 0u64;
        for value in &tx.outputs {
            output_sum = output_sum
                .checked_add(*value)
                .ok_or(ApplyError::ValueOverflow)?;
        }
        if output_sum > input_sum {
            return Err(ApplyError::InsufficientInput);
        }

        for index in 0..output_count {
            let outpoint = Outpoint {
                txid: tx.id.clone(),
                index,
            };
            if self.utxos.contains_key(&outpoint) {
                return Err(ApplyError::OutputAlreadyExists);
            }
        }

        for input in &tx.inputs {
            self.utxos.remove(input);
        }
        for (index, value) in (0..output_count).zip(&tx.outputs) {
            self.utxos.insert(
                Outpoint {
                    txid: tx.id.clone(),
                    index,
                },
                *value,
            );
        }
        Ok(())
    }
}

fn main() {
    let genesis = Outpoint {
        txid: "genesis".into(),
        index: 0,
    };
    let mut state = UtxoState::default();
    state.utxos.insert(genesis.clone(), 10);

    let tx1 = Transaction {
        id: "tx1".into(),
        inputs: vec![genesis.clone()],
        outputs: vec![7, 2], // 差額1を手数料相当として扱う教育用モデル
    };
    let tx2 = Transaction {
        id: "tx2".into(),
        inputs: vec![genesis],
        outputs: vec![9],
    };

    println!("tx1: {:?}", state.apply(&tx1));
    println!("tx2: {:?}", state.apply(&tx2));
    println!("remaining UTXOs: {}", state.utxos.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (UtxoState, Outpoint) {
        let original = Outpoint {
            txid: "genesis".into(),
            index: 0,
        };
        let mut state = UtxoState::default();
        state.utxos.insert(original.clone(), 10);
        (state, original)
    }

    #[test]
    fn first_spend_wins_in_given_order() {
        let (mut state, original) = setup();
        let first = Transaction {
            id: "first".into(),
            inputs: vec![original.clone()],
            outputs: vec![9],
        };
        let second = Transaction {
            id: "second".into(),
            inputs: vec![original],
            outputs: vec![8],
        };
        assert_eq!(state.apply(&first), Ok(()));
        assert_eq!(state.apply(&second), Err(ApplyError::MissingInput));
    }

    #[test]
    fn failed_transaction_does_not_mutate_state() {
        let (mut state, original) = setup();
        let invalid = Transaction {
            id: "invalid".into(),
            inputs: vec![original.clone()],
            outputs: vec![11],
        };
        assert_eq!(state.apply(&invalid), Err(ApplyError::InsufficientInput));
        assert_eq!(state.utxos.get(&original), Some(&10));
    }

    #[test]
    fn reverse_order_changes_accepted_outputs() {
        for first_id in ["first", "second"] {
            let (mut state, original) = setup();
            let other_id = if first_id == "first" {
                "second"
            } else {
                "first"
            };
            let first = Transaction {
                id: first_id.into(),
                inputs: vec![original.clone()],
                outputs: vec![if first_id == "first" { 9 } else { 8 }],
            };
            let second = Transaction {
                id: other_id.into(),
                inputs: vec![original],
                outputs: vec![if other_id == "first" { 9 } else { 8 }],
            };
            assert_eq!(state.apply(&first), Ok(()));
            assert_eq!(state.apply(&second), Err(ApplyError::MissingInput));
            assert_eq!(
                state.utxos,
                HashMap::from([(
                    Outpoint {
                        txid: first_id.into(),
                        index: 0
                    },
                    if first_id == "first" { 9 } else { 8 }
                )])
            );
        }
    }

    #[test]
    fn validation_errors_preserve_the_entire_state() {
        let (mut state, original) = setup();
        let large = Outpoint {
            txid: "large".into(),
            index: 0,
        };
        state.utxos.insert(large.clone(), u64::MAX);
        let before = state.utxos.clone();
        let candidates = [
            (
                Transaction {
                    id: "".into(),
                    inputs: vec![original.clone()],
                    outputs: vec![9],
                },
                ApplyError::InvalidStructure,
            ),
            (
                Transaction {
                    id: "empty-inputs".into(),
                    inputs: vec![],
                    outputs: vec![0],
                },
                ApplyError::InvalidStructure,
            ),
            (
                Transaction {
                    id: "empty-outputs".into(),
                    inputs: vec![original.clone()],
                    outputs: vec![],
                },
                ApplyError::InvalidStructure,
            ),
            (
                Transaction {
                    id: "missing".into(),
                    inputs: vec![
                        original.clone(),
                        Outpoint {
                            txid: "absent".into(),
                            index: 0,
                        },
                    ],
                    outputs: vec![9],
                },
                ApplyError::MissingInput,
            ),
            (
                Transaction {
                    id: "input-overflow".into(),
                    inputs: vec![original.clone(), large],
                    outputs: vec![9],
                },
                ApplyError::ValueOverflow,
            ),
            (
                Transaction {
                    id: "output-overflow".into(),
                    inputs: vec![original.clone()],
                    outputs: vec![u64::MAX, 1],
                },
                ApplyError::ValueOverflow,
            ),
            (
                Transaction {
                    id: "too-much".into(),
                    inputs: vec![original.clone()],
                    outputs: vec![11],
                },
                ApplyError::InsufficientInput,
            ),
            (
                Transaction {
                    id: "duplicate".into(),
                    inputs: vec![original.clone(), original.clone()],
                    outputs: vec![9],
                },
                ApplyError::DuplicateInput,
            ),
            (
                Transaction {
                    id: original.txid.clone(),
                    inputs: vec![original],
                    outputs: vec![9],
                },
                ApplyError::OutputAlreadyExists,
            ),
        ];
        for (tx, error) in candidates {
            assert_eq!(state.apply(&tx), Err(error));
            assert_eq!(state.utxos, before);
        }
    }

    #[test]
    fn dependent_spend_requires_its_producer() {
        let (mut state, original) = setup();
        let producer = Transaction {
            id: "producer".into(),
            inputs: vec![original],
            outputs: vec![9],
        };
        let consumer = Transaction {
            id: "consumer".into(),
            inputs: vec![Outpoint {
                txid: "producer".into(),
                index: 0,
            }],
            outputs: vec![8],
        };
        let before = state.utxos.clone();
        assert_eq!(state.apply(&consumer), Err(ApplyError::MissingInput));
        assert_eq!(state.utxos, before);
        assert_eq!(state.apply(&producer), Ok(()));
        assert_eq!(state.apply(&consumer), Ok(()));
        assert_eq!(
            state.utxos,
            HashMap::from([(
                Outpoint {
                    txid: "consumer".into(),
                    index: 0
                },
                8
            )])
        );
    }

    #[test]
    fn independent_spends_commute() {
        let (mut left, original) = setup();
        let other = Outpoint {
            txid: "other".into(),
            index: 0,
        };
        left.utxos.insert(other.clone(), 20);
        let mut right = UtxoState {
            utxos: left.utxos.clone(),
        };
        let first = Transaction {
            id: "first".into(),
            inputs: vec![original],
            outputs: vec![9],
        };
        let second = Transaction {
            id: "second".into(),
            inputs: vec![other],
            outputs: vec![19],
        };
        left.apply(&first).unwrap();
        left.apply(&second).unwrap();
        right.apply(&second).unwrap();
        right.apply(&first).unwrap();
        assert_eq!(left.utxos, right.utxos);
        assert_eq!(left.utxos.len(), 2);
    }

    #[test]
    fn repeated_input_is_rejected() {
        let (mut state, original) = setup();
        let invalid = Transaction {
            id: "duplicate".into(),
            inputs: vec![original.clone(), original.clone()],
            outputs: vec![10],
        };
        assert_eq!(state.apply(&invalid), Err(ApplyError::DuplicateInput));
        assert_eq!(state.utxos.get(&original), Some(&10));
    }
}
```

</details>

**実行手順：**

```bash
cargo run
cargo test
```

**期待する挙動**：`tx1` が先に適用されると、同じUTXOを使う `tx2` は `MissingInput` で失敗する。テストは、二重消費の拒否、失敗時の原子性、同じ入力の重複検出を確認する。本文から抽出したコードで7テストと実行出力を確認した（`experiments/chapter-14-review-2026-11-01.md`）。逆順の競合、独立した取引の順序交換、依存取引の再試行も含む。

`genesis` は教材の初期資金を識別する文字列であり、実KaspaのGenesis出力ではない。金額の10・7・2は架空の整数単位で、KAS/sompiの変換や報酬の配分を実装していない。

ここでいう原子性は、通常の検査がErrを返すときにメモリ内の集合を変えないという範囲である。電源断・panic・永続化の耐久性は検証していない。出力数のu32上限を超えるケースは巨大な配列を要するため実行していない。

### 14.14.2 このモデルに含まれないもの

- 署名・スクリプト検証
- Coinbaseおよび成熟条件
- ブロック単位のトランザクション受理規則
- トランザクションIDの暗号学的検証・使用済みIDの履歴管理（出力の衝突検査は現在のUTXO集合だけ）
- UTXOエントリの完全なデータ形式
- GHOSTDAGとVirtual Blockの実装
- 実際のKaspaの手数料・質量（mass）等の規則
- 永続化、並列処理、クラッシュリカバリ

**このコードをKaspaのノードやウォレットの検証処理として流用してはいけない。**

---

## 14.15 実験：順序を変えると結果はどうなるか

前節の `tx1` と `tx2` は同じUTXOを消費する。そこで実験を2回行う。

**実験A**

```text
初期状態: U = 10
処理順序: tx1 → tx2
結果: tx1受理、tx2不受理
```

**実験B**

```text
初期状態: U = 10
処理順序: tx2 → tx1
結果: tx2受理、tx1不受理
```

依存取引の逐次適用はこの教材の動作である。同一Kaspaブロック内の親子トランザクションを許す説明ではない（14.5.1参照）。

この実験は、**UTXO検証が正しくても、順序がノード間で一致しなければ状態が一致しない**ことを示す。

### 14.15.1 さらに試すこと

1. 競合しない2つのUTXOを作り、処理順序を入れ替えてみる。
2. トランザクションAの出力をトランザクションBが消費する依存関係を作る。
3. トランザクションAが失敗した場合、トランザクションBはどうなるか考える。
4. 同じトランザクションIDを持つ別のトランザクションを投入し、モデルの限界を調べる。
5. 状態更新を途中で中断した場合の復旧方法を設計する。

## 14.16 ステートマシンとして捉える

状態更新の流れを、教育用に次のように整理できる。

```text
[Block Received]
        │
        ▼
[Header Checked: GHOSTDAG included]
        │
        ▼
[Header / Body Saved]
        │
        ▼
[UTXO / Virtual Evaluated]
        │
        ▼
[Virtual State Updated]
        │
        ▼
[Changes Exposed]
```

この図は通常経路の責務を整理したもので、公式の永続ステータス名や全処理順ではない。HeaderOnly・本体保存・UTXO/Virtualの評価を区別する（[第11章](chapter-11.md)）。実際には、失敗、保留、再試行、再編成、同期状態などの分岐がある。設計レビューでは、**正常系の矢印だけでなく、各状態からの失敗時の遷移**を確認すること。

### 14.16.1 不変条件を明文化する

| 不変条件 | 破られた場合の問題 |
|---|---|
| 同じ合意入力から同じ順序が得られる | ノード間の状態不一致 |
| 1つのUTXOを二重消費しない | 不正な価値移転 |
| トランザクション適用の失敗で状態が部分更新されない | UTXO集合の破損 |
| 新旧状態の差分を整合的に反映する | 再編成時の不一致 |
| ネットワーク受信順に依存しない | 地理・遅延による合意分岐 |

これらは設計を考えるための**一般的な不変条件の例**であり、Kaspaの仕様書からそのまま抜粋した条文ではない。

---

## 14.17 C4・DDDで責務を分ける

### 14.17.1 コンテナ／コンポーネントの概念図

```text
┌────────────────────────────── Kaspa Node ──────────────────────────────┐
│                                                                       │
│  ┌───────────┐   ┌──────────────────┐   ┌─────────────────────────┐    │
│  │ P2P Layer │──►│ Block Processing │──►│ Consensus / DAG Logic   │    │
│  └───────────┘   └──────────────────┘   └──────────┬──────────────┘    │
│                                                  │                    │
│                                     ┌────────────▼─────────────┐      │
│                                     │ Ordering / Virtual State │      │
│                                     └────────────┬─────────────┘      │
│                                                  │                    │
│                                      ┌───────────▼───────────┐        │
│                                      │ Storage / RPC Views   │        │
│                                      └───────────────────────┘        │
└───────────────────────────────────────────────────────────────────────┘
```

これはC4の考え方を使った**責務の概念図**であり、現行ソースコードのモジュール境界と一対一に対応するものではない。

### 14.17.2 境界づけられた責務

- **DAG管理**：親・祖先・tipsなどの構造を管理する。
- **GHOSTDAG計算**：Selected Parent、blue/red、blue work等を扱う。
- **順序決定**：コンセンサス上の順序を導く。
- **状態評価**：UTXOの消費・生成とトランザクション受理結果を扱う。
- **永続化**：必要な状態・差分・履歴を保存する。
- **外部公開**：RPCや通知を通じて観測結果を提供する。

DDDの観点では、「ブロックが有効」「トランザクションが受理された」「仮想状態に反映された」を異なるドメイン概念として言葉で定義すると、誤解を減らせる。

## 14.18 `rusty-kaspa` のコードを読む

公式リポジトリ： [kaspanet/rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)

本章の概念と実装を結び付けるには、**対象リリース・コミットを固定して調査する**ことが重要である。ファイル名やモジュール構成は変わり得るため、以下は調査の入口として使う。

```bash
# 第1章で取得した基準版の親ディレクトリから実行する
cd rusty-kaspa-v1.0.1
git rev-parse HEAD
grep -n -E 'resolve_virtual|pick_virtual_parents|commit_virtual_state' consensus/src/pipeline/virtual_processor/processor.rs
grep -n -E 'calculate_utxo_state|accepted_tx_ids|pov_daa_score' consensus/src/pipeline/virtual_processor/utxo_validation.rs
grep -n -E 'consensus_ordered_mergeset|ascending_mergeset' consensus/src/model/stores/ghostdag.rs
```

未取得なら[第1章](../part-01-foundations/chapter-01.md)の手順を使い、HEADを本章冒頭の完全SHAと照合する。**文字列検索の一致だけでは、コンセンサスの仕様を証明できない。** 呼び出し元・テスト・データ構造まで追う。

### 14.18.1 読む順序

1. **入力**：新しいブロックの受信・検証がどこから始まるか。
2. **DAG**：親・tipsの情報がどこに保持されるか。
3. **GHOSTDAG**：選択親とblue workをどこで計算するか。
4. **順序**：ブロック順序がどこで導かれるか。
5. **受理**：ブロック中のトランザクション受理情報がどこで計算されるか。
6. **Virtual State**：新しい仮想状態がどう組み立てられるか。
7. **永続化**：状態更新がどのようにコミットされるか。
8. **テスト**：競合トランザクション・並行ブロック・再編成をどう検証しているか。

### 14.18.2 調査メモのテンプレート

```markdown
## 調査対象
- rusty-kaspa commit:
- 対象関数・型:
- 呼び出し元:
- 入力データ:
- 出力データ:
- 変更される状態:
- 不変条件:
- 失敗時の挙動:
- 関連テスト:
- 未確認事項:
```

**読んだ事実、推測、未確認事項を分けて記録すること**。本書を将来アップデートする際にも有用である。

---

## 14.19 Bitcoinとの比較

| 観点 | Bitcoin | Kaspa |
|---|---|---|
| 基本構造 | フォークを含む候補群から採用チェーンを選ぶ | BlockDAG |
| 並行ブロック | 競合する枝として扱う | DAG上の並行ブロックとして扱う |
| 順序の土台 | 採用チェーン上のブロック順序 | GHOSTDAGに基づく合意順序 |
| 状態モデル | UTXO | UTXO |
| 二重支払い | UTXO検証とチェーン上の順序で排除 | UTXO検証とDAG上の合意順序・受理規則で排除 |
| 先端の扱い | 採用チェーンのtip（候補の枝も存在） | 複数tipとVirtual Blockの概念 |
| 状態の変化 | チェーン再編成で変化し得る | Virtual Stateの再評価で変化し得る |

**共通点は「合意した順序に基づいて状態を決めること」**であり、相違点は、その順序を導くために使うブロック構造とコンセンサスの仕組みである。

## 14.20 よくある誤解

<details class="misconception-details">
<summary>よくある誤解を見る</summary>

**1. BlockDAGならトランザクション順序は不要？**

いいえ。並行ブロックを保存できることと、二重支払いを矛盾なく処理できることは別である。

**2. blueブロックのトランザクションはすべて受理される？**

いいえ。blue/redはGHOSTDAGのブロック分類であり、個々のトランザクションの受理判定ではない。

**3. Virtual Blockは誰かがマイニングする？**

いいえ。Virtual Blockはノードが現在のDAG先端に対する合意状態を評価するための概念である。

**4. すべてのノードは同じ時刻に同じVirtual Stateを持つ？**

いいえ。ネットワーク遅延により、各ノードが観測しているブロック集合は一時的に異なり得る。同じ入力と規則から整合した結果を計算できることが重要である。

**5. UTXOの二重支払いを拒否するだけでコンセンサスは完成する？**

いいえ。ノード間でトランザクションの処理順序が一致しなければ、どちらの競合トランザクションが受理されるかも一致しない。

**6. 本章のRustコードはKaspa互換？**

いいえ。順序が与えられた後のUTXO状態遷移だけを説明する教育用モデルである。

</details>

---

## 14.21 理解度チェック

<details class="quiz-answers">
<summary>理解度チェックを見る</summary>

### 基礎

1. DAGの部分順序とコンセンサス順序の違いを説明せよ。
2. Selected Parentと通常の親ブロックの違いを説明せよ。
3. Virtual Blockが必要な理由を説明せよ。
4. Virtual Stateとウォレットの表示残高が常に一致するとは限らない理由を述べよ。
5. blue/redとトランザクション受理結果の違いを説明せよ。

### 応用

6. 2つの並行ブロックに競合トランザクションが含まれている場合、合意順序とUTXO状態遷移がどう関係するか図示せよ。
7. 新しいDAG tipが追加された場合に、再評価が必要となり得る情報を挙げよ。
8. 状態更新中にプロセスが停止した場合、どのような不変条件を守る必要があるか。
9. 「受信順にトランザクションを適用する」設計の問題点を、2ノードの例で説明せよ。
10. `rusty-kaspa` の対象コミットを固定し、Virtual State更新に関係するテストを1つ見つけ、その入力・期待結果を記録せよ。

**解答例**

1. DAGの部分順序は祖先・子孫の制約で、並行ブロック間の順序はそれだけでは決まらない。コンセンサス順序は追加規則によって取引の評価等に使う順序を定める。
2. 通常の親は直接参照する親集合の要素で、Selected Parentはその中からGHOSTDAGの比較規則で選ぶ基準の親である。他の親の情報も捨てない。
3. 複数tipを持つ現在のDAGを評価し、親選択や現在状態を計算するための仮想的な文脈が必要だから。採掘・配信する実ブロックではない。
4. ウォレットは所有するアドレス、成熟・予約、同期・索引の追従、業務上の確認条件等で表示額を絞るため。全UTXO状態そのものとは異なる。
5. blue/redはブロックの分類、取引受理は順序に沿って支出条件・UTXO等を検査した結果であり、色だけで取引の受理可否は決まらない。
6. 一例として「初期UTXO U → 合意順序でTx-Aを検証しUを消費 → 同じUを使うTx-Bは使用可能な入力を失う」と図示できる。どちらが先かは合意規則で決まり、受信順ではない。
7. Virtualの親・Selected Parent、合意順序、取引の受理結果、UTXO差分、Mempool候補等が再評価の対象になり得る。再計算範囲は実装に依存する。
8. UTXO消費・出力追加・関連メタデータの整合性を維持し、再起動後に半適用や二重適用が残らないこと。対象実装に応じた原子的更新・回復手順を設ける。
9. ノード1がTx-A→Tx-B、ノード2がTx-B→Tx-Aの順に競合支出を受信すると、受信順適用では異なる勝者を選び得る。同じDAGから合意規則に従う順序を導く必要がある。
10. 調査課題なので一意のテスト名はない。固定コミットでVirtual更新を対象とする実在テストを探し、パス・関数名・初期状態・入力・期待UTXOや受理結果を記録する。実行した場合だけコマンドと件数を添え、未実行ならその旨を明記する。

### 発展課題

- **課題A**：RustのUTXOモデルにトランザクションの依存関係を追加し、正しい順序でのみ受理されることをテストする。
- **課題B**：状態のスナップショットを保持し、処理順序の変更に応じて状態を再構築する。
- **課題C**：小さなDAGを作り、トポロジカルソートの結果が一意でないことを示す。そのうえで、GHOSTDAGの規則が追加で必要になる理由を説明する。
- **課題D**：ブロック受理・トランザクション受理・Virtual State更新を別のコンポーネントとして設計し、各APIの入力と不変条件を書く。

</details>

## 14.22 まとめ

<details class="summary-details">
<summary>まとめを見る</summary>

本章の中心となる考え方は、次の一文に集約される。

> **BlockDAGは並行するブロックの関係を表し、GHOSTDAGは合意に必要な構造と順序を導き、Virtual Stateはその結果を取引・UTXOの状態として表現する。**

特に重要なのは、以下の4点である。

- **DAG構造とトランザクションの実行順序は別**：並行ブロックの存在だけでは二重支払いの処理結果は決まらない。
- **ブロック分類とトランザクション受理は別**：blue/redを有効・無効と読み替えてはいけない。
- **Virtual Blockは概念上の先端**：複数tipを含む観測済みDAGの合意状態を考えるために使う。
- **Virtual Stateは変化し得る**：新しいブロックによって合意結果が再評価される場合があり、確定性と状態更新の設計が重要になる。

</details>

ここまでの **Part II — Kaspa Consensus & Architecture（第9〜14章）** で、ノードの構造からコンセンサス状態の更新までを扱った。次のPart IIIの[第15章](../part-03-transactions/chapter-15.md)では、UTXOとトランザクションの具体的な処理をさらに深く学ぶ。

---

## 参考資料

<details class="reference-details">
<summary>参考資料を見る</summary>

### 公式実装・ドキュメント

- [Kaspa公式ドキュメント](https://docs.kaspa.org/)
- [Kaspa Wiki](https://wiki.kaspa.org/en/home)
- [rusty-kaspa — 公式Rust実装](https://github.com/kaspanet/rusty-kaspa)
- [Kaspa Improvement Proposals（KIPs）](https://github.com/kaspanet/kips)

### 原論文・背景

- [PHANTOM and GHOSTDAG: A Scalable Generalization of Nakamoto Consensus](https://eprint.iacr.org/2018/104.pdf)
- [Bitcoin: A Peer-to-Peer Electronic Cash System](https://bitcoin.org/bitcoin.pdf)

### 本書の関連章

- [Chapter 07 — Introduction to BlockDAG](../part-01-foundations/chapter-07.md)
- [Chapter 09 — Kaspa Architecture](chapter-09.md)
- [Chapter 11 — Block Processing & Validation](chapter-11.md)
- [Chapter 12 — GHOSTDAG Internals](chapter-12.md)
- [Chapter 13 — Difficulty Adjustment Algorithm](chapter-13.md)
- [Chapter 15 — UTXO Model & Transaction Processing](../part-03-transactions/chapter-15.md)
- [Chapter 20 — Kaspa Network Security & Finality](../part-03-transactions/chapter-20.md)
- [Chapter 21 — Pruning, Storage & Node Synchronization](../part-04-development/chapter-21.md)
- [Chapter 25 — Exploring the rusty-kaspa Codebase](../part-04-development/chapter-25.md)



</details>

<nav class="chapter-next" aria-label="次のページ">

[次章：Chapter 15 — UTXO Model & Transaction Processing（UTXOモデルとトランザクション処理）](../part-03-transactions/chapter-15.md)

</nav>

<details class="verification-details">
<summary>検証状況を見る</summary>

[対象バージョン](../baseline.md)・[このページのレビュー記録](../review-status.md#review-chapter-14)

目次順の本文レビューと改訂教材7テストは `experiments/chapter-14-review-2026-11-01.md`。既存の主要経路と旧教材3テストは `experiments/consensus-state-review-2026-10-31.md` を参照する。公式関数との実行時差分、クラッシュ耐久性、報酬生成・署名・全例外規則、現在のMainnetとの一致は今回検証していない。

</details>
