# Chapter 11 — Block Processing & Validation（ブロック処理と検証）

> **Part II — Kaspa Consensus & Architecture**  
> 原稿ステータス: v0.1（技術レビュー前）

## この章で学ぶこと

前章ではKaspaノードがP2Pネットワークからデータを受け取るまでを学んだ。本章では、その先の**ブロック検証・BlockDAGへの登録・コンセンサス更新・UTXO状態への反映**を追う。

本章の到達目標は次のとおり。

1. ブロックの「受信」「形式検査」「コンセンサス検証」「状態反映」を区別できる。
2. ブロックヘッダーとトランザクション本体の検証目的を説明できる。
3. 親ブロックの不足、重複受信、無効ブロックを別々の状態として扱える。
4. BlockDAGの登録とVirtual Stateの更新を混同せず説明できる。
5. Rustで簡易検証パイプラインを実装し、不変条件をテストできる。

> **重要:** 本章の処理図とRustコードは学習用モデルであり、`rusty-kaspa` の正確な関数呼び出し順序や現行メインネットの全検証規則を再現したものではない。実装依存の詳細は、対象コミットのソースコードで確認する。

---

## 11.1 ブロックを受け取ったら何が起きるのか

P2Pで受信したデータは、すぐに正しいブロックとして確定するわけではない。ノードは信頼できない外部入力として取り扱い、複数の境界で検証する。

```text
外部ピア
   │  ブロックを送信
   ▼
┌──────────────────────┐
│ P2P / メッセージ解析 │  サイズ・形式・リソース制限
└──────────┬───────────┘
           ▼
┌──────────────────────┐
│ 重複確認・依存関係   │  既知ブロックか？ 親は揃っているか？
└──────────┬───────────┘
           ▼
┌──────────────────────┐
│ ヘッダーの検証       │  PoW・親参照・時刻・難易度など
└──────────┬───────────┘
           ▼
┌──────────────────────┐
│ ブロック本体の検証   │  TX構造・Merkle root・各種制約
└──────────┬───────────┘
           ▼
┌──────────────────────┐
│ DAG / Consensus処理  │  DAGへの登録・GHOSTDAG関連処理
└──────────┬───────────┘
           ▼
┌──────────────────────┐
│ Virtual State更新    │  受理される取引とUTXO状態の評価
└──────────────────────┘
```

これは**概念的な責務の並び**である。実際には検証が段階的・並行的に進み、DAG上の順序や過去の状態に依存してトランザクションの受理が判断される。すべてのブロックが単純に「受信→即時UTXO更新」と進むわけではない。

### 11.1.1 なぜ段階を分けるのか

- 不正な入力を早期に排除してCPU・メモリ・ディスクを守る。
- 不足している親ブロックを後から取得できるようにする。
- 同じブロックを何度受け取っても二重処理しない。
- ブロックの構造的な有効性と、その中のトランザクションが最終的な状態にどう作用するかを分離する。

**設計原則:** 外部入力の妥当性、DAG内の位置、台帳状態への効果は別の問いである。

---

## 11.2 ブロックを構成するデータ

Kaspaのブロックは大きく**ヘッダー**と**トランザクション群**からなる。

| 要素 | 主な役割 | 検証上の観点 |
|---|---|---|
| Version | 形式・規則の識別 | 対応するバージョンか |
| Parents | 参照する親ブロック群 | 参照形式、親の存在、DAGの整合性 |
| Hash Merkle Root | トランザクション群のコミットメント | 本体から再計算した値との一致 |
| Accepted ID Merkle Root | 受理トランザクション情報に関するコミットメント | コンセンサス状態との整合性 |
| UTXO Commitment | UTXO状態に関するコミットメント | 対応する状態との整合性 |
| Timestamp | ブロック時刻 | コンセンサス上の許容条件 |
| Bits | PoWターゲットのコンパクト表現 | 要求難易度との整合性 |
| Nonce | PoW探索用の値 | ヘッダーハッシュとターゲットの関係 |
| DAA Score / blue score / blue work | コンセンサス計算に関連する値 | 親・履歴・規則との整合性 |
| Transactions | コインベース等を含むトランザクション群 | 構造、証明、手数料、状態依存の検証 |

> **注意:** この表は概念整理であり、フィールドの完全なシリアライズ仕様ではない。正確なフィールド、型、ハッシュ計算、各フィールドの検証段階は使用する `rusty-kaspa` のバージョンを確認すること。

### 11.2.1 Bitcoinとの違い

Bitcoinの通常のブロックヘッダーは直前の親を一つ参照する。一方、Kaspaでは親参照が複数になり得る。したがって、ブロックの妥当性を考える際に、**一つの直前ブロックだけを追えばよいわけではない**。

```text
Bitcoin（概念）
A ── B ── C

Kaspa BlockDAG（概念）
      ┌── B ──┐
A ────┤       ├── D
      └── C ──┘
```

DがBとCの両方を参照することは、BとCが同じ親から並行して生成された状況を表現できる。ただし、**複数の親を参照できることと、二重支払いを両方受理することは別問題**である。

---

## 11.3 検証の前に行う入力保護

ネットワーク越しに受信するデータは、誤った形式であるだけでなく、意図的にノード資源を消費させる可能性がある。

### 11.3.1 メッセージ境界の検査

- メッセージ長が許容範囲内か。
- デコード可能な構造か。
- 予期しないフィールドや個数でメモリを過剰消費しないか。
- 同じブロックを繰り返し処理していないか。
- 不正データを受け取った場合に接続やキューをどう扱うか。

ここでの検査は、**コンセンサス規則そのものではなく防御的な入出力処理**も含む。

### 11.3.2 早期拒否と遅延評価

計算コストの小さい検査を先に行えば、明らかな不正データに高価な検証を実行せずに済む。ただし、依存関係が未到着であることを「無効」と判定してはならない。

| 状態 | 意味 | 典型的な対応 |
|---|---|---|
| Already known | 既に処理済み | 再処理を避ける |
| Missing parents | 依存する親が未到着 | 親の取得・保留 |
| Invalid | 検証規則に違反 | 拒否・記録 |
| Eligible for processing | 必要な依存関係が揃った | 次の検証段階へ |

この区別が、分散システムにおける**順不同到着**への対応になる。

---

## 11.4 親ブロックとDAG構造の検証

BlockDAGは有向非巡回グラフ（Directed Acyclic Graph）である。ノードは参照関係を維持しながらブロックを登録する。

### 11.4.1 親の存在

ブロックXを受信した時点で、その親Pがまだ到着していないことがある。

```text
到着順: X → P

X ──references──> P

Xを受信した時点:
  Pが未到着 → Xを直ちに「無効」とは断定しない

Pを取得した後:
  依存関係が揃う → Xの処理を再開できる
```

これはBitcoinでも起こるが、Kaspaでは複数親を扱うため、依存関係の管理がより重要になる。

### 11.4.2 DAGの不変条件

学習用モデルとして、次の条件を考える。

1. ブロック識別子は内容に対応して一意に扱う。
2. 同じブロックを二度登録しても状態が壊れない。
3. 参照する親が存在するか、未解決依存として明示される。
4. 循環参照を認めない。
5. 受信順によって、同一の有効データ集合の意味が変わらないようにする。

実際のKaspaでは、親参照のレベル構造やコンセンサス規則など、ここで示した以上の条件が存在する。

---

## 11.5 Proof of Workの検証

PoWでは、ブロックのヘッダーから求めたハッシュに基づく値が、要求されるターゲットを満たす必要がある。

概念的には次の関係で表せる。

```text
PoW(header) <= target
```

ターゲットが小さいほど、条件を満たす値を見つけることが難しくなる。

### 11.5.1 PoW検証で確認すること

- ヘッダーから正しい方法でPoW値を計算できるか。
- `Bits` などに表現されたターゲットが適切か。
- そのブロックに要求される難易度と整合しているか。
- 対象ネットワークの規則に従っているか。

**注意:** KaspaのPoWアルゴリズムとヘッダーハッシュを、単純なSHA-256一回やBitcoinのダブルSHA-256と同一視してはいけない。アルゴリズムの詳細は対象のネットワーク・バージョンの実装を参照する。

### 11.5.2 PoWが証明しないこと

PoWを満たすブロックであっても、次のことは自動的に保証されない。

- トランザクションがすべて正しい。
- 親参照が正しい。
- UTXO状態が正しい。
- ブロックがコンセンサス上、期待する位置にある。

**PoWは必要な検証要素の一つであり、ブロック全体の正しさそのものではない。**

---

## 11.6 ブロック本体とトランザクションの検証

ブロックに含まれるトランザクションは、形式上の正しさと、台帳状態に依存する正しさを分けて考える。

### 11.6.1 状態に依存しない検査

例として次が挙げられる。

- トランザクションのシリアライズ形式が正しい。
- 入力・出力の数や各フィールドが制約に従う。
- トランザクションIDやMerkle rootを正しく計算できる。
- ブロックに許されるリソース制限を超えていない。
- コインベーストランザクションに関する構造上の制約を満たす。

### 11.6.2 状態に依存する検査

次の判断には、参照するUTXO集合やコンセンサス順序が必要になる。

- 入力が参照するUTXOは存在するか。
- 署名・スクリプトの条件を満たすか。
- 同じUTXOが二重に消費されていないか。
- 入力と出力の価値の整合性は保たれているか。

Kaspaでは並行ブロックを扱うため、**あるトランザクションがブロックに含まれていることと、コンセンサス状態で受理されることは同義ではない**。

### 11.6.3 競合するトランザクションの例

```text
UTXO: U（10 KAS）

Tx-A: U → Alice
Tx-B: U → Bob

Block B に Tx-A
Block C に Tx-B

B と C は並行ブロック
```

BとCが両方ともBlockDAG上に存在しても、Tx-AとTx-Bが同じUTXOを同時に消費することはできない。コンセンサス順序と状態遷移の規則に従い、矛盾を解決する必要がある。

**「ブロックのDAGへの登録」と「その中の全トランザクションの受理」を区別する**ことが、Kaspaの理解に不可欠である。

---

## 11.7 Merkle rootとコミットメント

Merkle rootは、トランザクションの集合や順序に関する暗号学的なコミットメントを提供する。

```text
            Root
           /    \
         H01    H23
        /  \    /  \
       H0  H1  H2  H3
       │   │   │   │
      Tx0 Tx1 Tx2 Tx3
```

ノードはブロック本体から必要な値を再計算し、ヘッダーに含まれるコミットメントと照合する。

### 11.7.1 なぜ照合するのか

- 受信した本体がヘッダーに対応するかを検証するため。
- データの改ざんや欠落を検出するため。
- 他の検証処理が、誤った本体に基づいて進まないようにするため。

ただし、**Merkle rootが一致しても、トランザクションの署名や残高条件が正しいとは限らない**。

---

## 11.8 GHOSTDAGとブロックの位置づけ

BlockDAGの構造検査だけでは、ブロック群のコンセンサス上の扱いは決まらない。KaspaではGHOSTDAGを用いて、並行ブロックを考慮した選択・順序付けを行う。

ここで区別したい概念は次の三つである。

| 概念 | 問い |
|---|---|
| Structural validity | ブロックの形式や参照構造は正しいか |
| Consensus metadata | GHOSTDAGの規則上どのような情報が付与されるか |
| Transaction acceptance | ブロック中のトランザクションが状態遷移に採用されるか |

### 11.8.1 blue / redとValid / Invalidは違う

GHOSTDAGのblue/redという分類を、単純な「有効/無効」の別名として扱ってはいけない。

- blue/redはGHOSTDAGの選択・分類に関わる概念。
- Valid/Invalidはプロトコルの検証規則に適合するかという概念。
- redに分類されたという理由だけで、構造的に不正なブロックだと断定できない。

詳しい定義と選択規則は第12章で扱う。

---

## 11.9 Virtual BlockとVirtual State

KaspaのBlockDAGには、単一の「最新ブロック」の代わりに、複数の先端（tips）が存在し得る。

```text
       A
      / \
     B   C
      \ / \
       D   E

D, E が tips（概念例）
```

**Virtual Block**は、DAGの先端を基にコンセンサス上の状態を評価するための概念的な構成要素である。Virtual Stateは、その評価によって得られる台帳状態を表す。

### 11.9.1 状態更新は単純な追加だけではない

新しいブロックが到着すると、Virtualの選択や順序が変化し得る。そのため、実装は状態の差分適用や再評価を必要とする場合がある。

```text
新ブロック到着
     │
     ▼
DAGとコンセンサスメタデータを更新
     │
     ▼
Virtualの選択・順序を評価
     │
     ▼
必要な状態差分を計算
     │
     ▼
Virtual UTXO Stateを更新
```

この図も処理順を保証するものではなく、**責務の分解**である。

---

## 11.10 失敗をどう分類するか

エラー分類を誤ると、正常なネットワーク遅延を攻撃と誤認したり、無効なデータを無期限に保持したりする。

| 事象 | 分類例 | 考えるべき対応 |
|---|---|---|
| 既知のブロックが再到着 | 重複 | 冪等に扱う |
| 親が未到着 | 依存関係不足 | 保留・取得・タイムアウト |
| デコードできない | 入力形式エラー | 早期拒否 |
| PoW条件を満たさない | コンセンサス違反 | 拒否 |
| Merkle root不一致 | データ不整合 | 拒否 |
| トランザクションがUTXOを二重消費 | 状態上の競合 | コンセンサス順序・受理規則で判断 |
| ディスク書き込み失敗 | ローカル障害 | リトライ可否・整合性保護 |
| 同期中で履歴不足 | ノード状態の制約 | 同期完了待ち・追加取得 |

> **実装設計:** 例外を一つの `InvalidBlock` にまとめると、再試行可能性、ピア評価、運用ログが不明瞭になる。エラーを発生源と意味で分類すると、調査しやすくなる。

---

## 11.11 Rustによる最小ブロック検証器

ここではKaspaそのものを再現するのではなく、**依存関係チェック、重複排除、形式検査、状態遷移の分離**を小さなRustプログラムで学ぶ。

### 11.11.1 モデルの制約

- ハッシュは説明用の文字列で表す。
- PoW、署名、Merkle root、GHOSTDAGは実装しない。
- ブロックの親は任意個とする。
- 親が未到着なら `MissingParents` を返す。
- 不正な入力はDAGへ追加しない。

**このコードを実際のKaspaブロック検証に使用してはいけない。**

### 11.11.2 プロジェクト作成

```bash
cargo new block-validation-lab
cd block-validation-lab
```

`src/lib.rs` を次のようにする。

```rust
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub id: String,
    pub parents: Vec<String>,
    pub payload_size: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ValidationError {
    EmptyId,
    SelfParent,
    DuplicateParent,
    TooLarge,
    MissingParents(Vec<String>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum InsertResult {
    Inserted,
    AlreadyKnown,
}

pub struct BlockStore {
    blocks: HashMap<String, Block>,
    max_payload_size: usize,
}

impl BlockStore {
    pub fn new(max_payload_size: usize) -> Self {
        Self {
            blocks: HashMap::new(),
            max_payload_size,
        }
    }

    pub fn insert(
        &mut self,
        block: Block,
    ) -> Result<InsertResult, ValidationError> {
        if block.id.is_empty() {
            return Err(ValidationError::EmptyId);
        }
        if self.blocks.contains_key(&block.id) {
            return Ok(InsertResult::AlreadyKnown);
        }
        if block.payload_size > self.max_payload_size {
            return Err(ValidationError::TooLarge);
        }
        if block.parents.iter().any(|p| p == &block.id) {
            return Err(ValidationError::SelfParent);
        }

        let unique: HashSet<_> = block.parents.iter().collect();
        if unique.len() != block.parents.len() {
            return Err(ValidationError::DuplicateParent);
        }

        let missing: Vec<String> = block
            .parents
            .iter()
            .filter(|p| !self.blocks.contains_key(*p))
            .cloned()
            .collect();

        if !missing.is_empty() {
            return Err(ValidationError::MissingParents(missing));
        }

        self.blocks.insert(block.id.clone(), block);
        Ok(InsertResult::Inserted)
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(id: &str, parents: &[&str]) -> Block {
        Block {
            id: id.to_string(),
            parents: parents.iter().map(|s| s.to_string()).collect(),
            payload_size: 128,
        }
    }

    #[test]
    fn inserts_genesis_and_parallel_children() {
        let mut store = BlockStore::new(1024);
        assert_eq!(store.insert(block("A", &[])), Ok(InsertResult::Inserted));
        assert_eq!(store.insert(block("B", &["A"])), Ok(InsertResult::Inserted));
        assert_eq!(store.insert(block("C", &["A"])), Ok(InsertResult::Inserted));
        assert_eq!(store.insert(block("D", &["B", "C"])), Ok(InsertResult::Inserted));
        assert_eq!(store.len(), 4);
    }

    #[test]
    fn missing_parent_is_not_inserted() {
        let mut store = BlockStore::new(1024);
        assert_eq!(
            store.insert(block("B", &["A"])),
            Err(ValidationError::MissingParents(vec!["A".to_string()]))
        );
        assert!(store.is_empty());
    }

    #[test]
    fn repeated_block_is_idempotent() {
        let mut store = BlockStore::new(1024);
        store.insert(block("A", &[])).unwrap();
        assert_eq!(store.insert(block("A", &[])), Ok(InsertResult::AlreadyKnown));
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn rejects_self_parent_and_duplicate_parents() {
        let mut store = BlockStore::new(1024);
        store.insert(block("A", &[])).unwrap();
        assert_eq!(store.insert(block("B", &["B"])), Err(ValidationError::SelfParent));
        assert_eq!(
            store.insert(block("C", &["A", "A"])),
            Err(ValidationError::DuplicateParent)
        );
    }

    #[test]
    fn rejects_oversized_payload() {
        let mut store = BlockStore::new(64);
        assert_eq!(store.insert(block("A", &[])), Err(ValidationError::TooLarge));
    }
}
```

実行例:

```bash
cargo test
```

### 11.11.3 この実装の限界

このモデルは、次のような実システムの要件を満たさない。

- IDと内容の暗号学的な結び付きがない。
- 親参照の実際のレベル構造や合意規則を実装していない。
- PoWと難易度調整を検証しない。
- トランザクションやUTXOを検証しない。
- 永続化、Pruning、同期、再編成を実装しない。
- 未到着の親を取得する機構がない。
- 同一IDで内容が異なるデータの扱いが簡略化されている。

このように**何を保証していないか**を明示することも、技術書や検証用コードに重要である。

---

## 11.12 ステートマシンとしてのブロック処理

処理フローを状態遷移として捉えると、どこで再試行し、どこで拒否するかを設計しやすい。

```text
Received
   │
   ▼
Parsed ────────────────> Rejected
   │                      ▲
   ▼                      │
DependenciesChecked ──────┤
   │                      │
   ├──> WaitingForParents │
   │         │            │
   │         └──(到着)────┘ ※再検証へ戻る経路を実装
   ▼
Validated ────────────────┘
   │
   ▼
RegisteredInDAG
   │
   ▼
ConsensusEvaluated
   │
   ▼
StateEffectsEvaluated
```

図中の `WaitingForParents` は拒否状態ではない。親が揃ったとき、適切な検証段階から処理を再開する。

### 11.12.1 不変条件

- **I1:** 無効と判定されたブロックは、有効ブロックとして登録されない。
- **I2:** 既知ブロックの再受信で、ブロック数や状態が不当に増えない。
- **I3:** 親不足を「規則違反」と混同しない。
- **I4:** BlockDAGへの登録だけで、すべてのトランザクションが受理されたと見なさない。
- **I5:** 外部入力の失敗が、既存の有効な台帳状態を破壊しない。

これらはテスト名やログ項目にも転用できる。

---

## 11.13 `rusty-kaspa` を読むための調査手順

Kaspaの実装を理解するには、特定の関数名を暗記するより、**データが責務の境界を越える場所**を追うほうが有効である。

### 11.13.1 調査の入口

公式リポジトリ:

- [kaspanet/rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)

以下の順序で探索する。

1. 使用するリリースまたはコミットを固定する。
2. P2Pでブロック受信を処理する箇所を探す。
3. コンセンサスへブロックを投入する公開インターフェースを探す。
4. ヘッダー検証・本体検証・親不足の扱いを分けて読む。
5. DAG登録後のGHOSTDAG関連処理を追う。
6. Virtual State更新とトランザクション受理の処理を追う。
7. 同じ経路を通るユニットテスト・統合テストを探す。

### 11.13.2 検索コマンドの例

```bash
git clone https://github.com/kaspanet/rusty-kaspa.git
cd rusty-kaspa

git rev-parse HEAD
rg -n 'validate.*header|validate.*body|validate.*block' consensus
rg -n 'missing.*parent|orphan|block.*status' consensus
rg -n 'virtual.*state|utxo.*diff|ghostdag' consensus
```

検索語やディレクトリはリポジトリの変更に合わせて調整する。`rg` はコード探索のための例であり、**該当する関数やファイルが必ず存在することを保証しない**。

### 11.13.3 調査メモのテンプレート

| 項目 | 記録すること |
|---|---|
| Commit | 調査対象のコミットハッシュ |
| Entry point | 受信データが最初に入る関数 |
| Validation | どの条件をどこで検証するか |
| Dependency | 親や過去状態への依存 |
| Error | どのエラーが返るか |
| State change | 何が永続化・更新されるか |
| Test | 該当するテストと再現方法 |

**設計書は、コードを読む前に完成している必要はない。** 実装調査の結果として更新していくことが重要である。

---

## 11.14 C4とDDDで責務を整理する

ブロック処理を一つの巨大な関数として理解しようとすると、依存関係が見えにくい。概念上のコンポーネントを分けて考える。

```text
┌─────────────────────┐
│ P2P Adapter         │ 外部入力の受信
└──────────┬──────────┘
           ▼
┌─────────────────────┐
│ Block Ingestion     │ 重複・依存関係・キュー管理
└──────────┬──────────┘
           ▼
┌─────────────────────┐
│ Consensus Validation│ プロトコル規則の評価
└──────────┬──────────┘
           ▼
┌─────────────────────┐
│ DAG / Consensus Data│ ブロックと合意メタデータ
└──────────┬──────────┘
           ▼
┌─────────────────────┐
│ Virtual State       │ 台帳状態の評価・更新
└─────────────────────┘
```

この図はC4の厳密な実装図ではなく、責務を説明するためのコンポーネント図である。

### 11.14.1 責務の境界を考える質問

- P2Pから届いた不正データの処理責任はどこにあるか。
- 親ブロックの取得はコンセンサスの責務か、同期処理の責務か。
- コンセンサス規則の変更が、ネットワーク通信コードにどれだけ影響するか。
- UTXO状態の更新に失敗した場合、DAG登録はどう扱われるか。
- 障害発生時に、どのコンポーネントのログを見ればよいか。

**依存関係を明示する目的は、変更の影響範囲と障害の発生箇所を小さくすること**にある。

---

## 11.15 障害調査と観測可能性

実際のノードでブロック処理が進まないときは、次の切り分けが役立つ。

| 症状 | 最初に確認する観点 |
|---|---|
| ピアは接続しているがブロックが増えない | 同期状態、受信メッセージ、親不足 |
| 同じブロックのログが多い | 重複排除、再送、ピア接続状態 |
| 検証エラーが増えた | ネットワーク設定、バージョン差、無効データ |
| CPU使用率が高い | PoW/トランザクション検証、同期負荷、異常入力 |
| ディスクI/Oが高い | DAG・UTXO・インデックスの永続化 |
| Virtual Stateの更新が遅い | コンセンサス更新、状態差分、同期進捗 |

### 11.15.1 観測指標の例

- 受信ブロック数 / 秒
- 検証成功数 / 失敗数（理由別）
- 親不足で保留中のブロック数
- DAG登録までの遅延
- Virtual State更新までの遅延
- 同期進捗とピア数
- メモリ使用量とキュー長

これらは**設計上の推奨観測項目**であり、すべてが標準メトリクスとして公開されていることを意味しない。

---

## 11.16 BitcoinとKaspaのブロック検証比較

| 観点 | Bitcoin | Kaspa |
|---|---|---|
| 基本構造 | 線形チェーン | BlockDAG |
| 親参照 | 通常一つ | 複数になり得る |
| 並行ブロック | チェーン選択で扱う | DAG内で並行性を表現する |
| PoW検証 | 必要 | 必要 |
| トランザクション検証 | UTXOとスクリプトの規則 | UTXOとスクリプト等の規則 |
| 順序の決定 | チェーン上のブロック順序 | GHOSTDAGに基づく順序付け |
| 状態更新 | 選択チェーンに沿って評価 | Virtual Stateを通じて評価 |
| 競合トランザクション | 同じUTXOを二重に使えない | 同じUTXOを二重に使えない |

両者はPoWやUTXOという基礎を共有するが、**並行ブロックをどう扱い、どの順序でトランザクションを評価するか**が大きく異なる。

---

## 11.17 よくある誤解

### Q1. PoWが正しければ、そのブロックは有効か？

いいえ。PoWは検証条件の一つにすぎない。親参照、コンセンサス規則、ブロック本体なども確認する。

### Q2. 親が未到着のブロックは不正か？

必ずしも不正ではない。単にネットワーク上の到着順が前後した可能性がある。

### Q3. DAGに登録されたブロックのトランザクションはすべて受理されるか？

いいえ。トランザクションの受理にはコンセンサス順序とUTXO状態の評価が必要である。

### Q4. redブロックは無効ブロックか？

いいえ。GHOSTDAGの分類とプロトコルの妥当性判定は異なる概念である。

### Q5. ブロック検証は一回だけ行われるか？

単純な一回の判定として理解すべきではない。形式検査、依存関係、コンセンサス状態に依存する評価など、異なる責務が存在する。

---

## 11.18 実践課題

### 課題A: 親不足キューを実装する

第11.11節の `BlockStore` に保留キューを追加する。

1. 親が未到着のブロックを保留する。
2. 親を登録した後に、保留ブロックを再試行する。
3. 複数の親がすべて揃うまで登録しない。
4. キューの最大件数を設ける。

**合格条件:** 到着順が `D, C, B, A` でも、必要な親が揃った後に `A, B, C, D` の依存関係を満たす登録ができる。

### 課題B: エラー分類を設計する

次のエラーを、`Retryable` / `Invalid` / `LocalFailure` などに分類する。

- 親ブロックが未到着
- Merkle rootが一致しない
- ディスク容量が不足
- 同じブロックを再受信
- PoWターゲットを満たさない

分類の理由と、ピアへのペナルティを与えるべきかどうかも考える。

### 課題C: `rusty-kaspa` の実装と対応付ける

実際のソースコードから、以下を確認する。

- ブロック投入の入口
- ヘッダー検証
- 本体検証
- 親不足の処理
- DAG関連データの更新
- Virtual State更新

調査結果を**コミットハッシュ・ファイルパス・関数名・テスト名**付きで記録する。

---

## 11.19 理解度チェック

1. ブロックの「受信」と「受理」は何が違うか。
2. なぜ親ブロックの未到着を即座に不正と判定してはいけないのか。
3. PoWが成立していても検証が必要な項目を三つ挙げよ。
4. Merkle rootが一致することは、何を保証し、何を保証しないか。
5. GHOSTDAGのblue/redとValid/Invalidの違いを説明せよ。
6. 並行ブロックに競合するトランザクションが含まれた場合、何を基準に状態を決めるのか。
7. Virtual Stateを独立した概念として扱う理由は何か。
8. ブロック処理の冪等性を確認するテストを一つ設計せよ。
9. コンセンサス検証とP2P入力検査の責務を分ける利点は何か。
10. 本章のRustモデルに不足している実プロトコルの要素を五つ挙げよ。

---

## 11.20 まとめ

Kaspaのブロック処理は、**受信したデータを一回チェックして終わる処理ではない**。

- P2Pはブロックを運ぶが、ブロックの正しさを保証しない。
- 検証は、入力保護、依存関係、ヘッダー、本体、コンセンサス状態など複数の責務に分かれる。
- 親不足・重複・不正は、それぞれ異なる状態である。
- BlockDAGへの登録と、トランザクションの状態上の受理は別の概念である。
- GHOSTDAGは並行ブロックを扱うコンセンサスの重要な構成要素である。
- Virtual Stateは、コンセンサスに基づいてUTXO状態を評価するために重要である。

**次章「Chapter 12 — GHOSTDAG Internals」では、BlockDAGからコンセンサス順序を導く仕組みを詳しく学ぶ。**

---

## 参考資料

1. [Kaspa — Official Website](https://kaspa.org/)
2. [Kaspa Wiki](https://wiki.kaspa.org/en/home)
3. [rusty-kaspa — Official Rust Implementation](https://github.com/kaspanet/rusty-kaspa)
4. [Kaspa Improvement Proposals (KIPs)](https://github.com/kaspanet/kips)
5. [Satoshi Nakamoto, *Bitcoin: A Peer-to-Peer Electronic Cash System*](https://bitcoin.org/bitcoin.pdf)
6. [Kaspa Research Papers](https://kaspa.org/research/)

### 技術レビュー時の確認事項

- [ ] 対象 `rusty-kaspa` コミットの固定
- [ ] ヘッダー項目と検証段階の実装照合
- [ ] 親不足・保留・再試行の実際の処理経路
- [ ] ブロック本体の検証条件と状態依存の検証条件の照合
- [ ] GHOSTDAG・Virtual State更新の実際の処理経路
- [ ] Rustサンプルの `cargo test` 実行記録
- [ ] 参考資料のリンク・仕様変更の確認
