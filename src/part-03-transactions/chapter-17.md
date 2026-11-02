# Chapter 17 — Mempool & Transaction Propagation（Mempoolとトランザクション伝播）

> **Mastering Kaspa — Part III: Transactions, Mining & Security**  
> 目次順の本文レビュー：2026-11-01。基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` の主要な受付・保留・伝播経路を照合。全規則・全経路の検証ではない。\
> 本章の実験は教育用モデルであり、Kaspaメインネットのコンセンサス互換実装ではない。

固定ソースの確認と教材の実行記録は `experiments/chapter-17-review-2026-11-01.md`。既存DevnetのMempool照会・競合拒否は `experiments/devnet-send-checks-2026-10-31.md`、応答喪失からの復旧は `experiments/devnet-send-retry-2026-10-31.md` を参照する。今回はノード・送金を再実行していない。

## 学習目標

本章を読み終えると、次を説明・検証できる。

1. Mempoolが担う責務と、コンセンサス状態との違い。
2. トランザクションの受信・検査・伝播・ブロック採用までの経路。
3. UTXOの競合、依存トランザクション、孤立トランザクションを区別する方法。
4. P2Pネットワークの遅延・重複・分断がノード間のMempool差異を生む理由。
5. Rustで最小Mempoolを構築し、受理・拒否・重複・競合の不変条件をテストする方法。

## 17.1 Mempoolとは何か

**Mempool（Memory Pool）** は、ノードが知っているものの、まだ現在のコンセンサス状態に取り込まれていないトランザクションの候補を保持する領域である。

```text
ウォレット
   │ トランザクション送信
   ▼
RPC / P2P受信
   │
   ▼
形式・ポリシー・UTXOなどの検査
   │
   ├── 拒否 ──> 理由を記録
   │
   ▼
ローカルMempool
   ├── 他ピアへの通知・伝播
   └── マイナーのブロック候補生成
                    │
                    ▼
              BlockDAGへ登録
                    │
                    ▼
            コンセンサス状態を更新
```

ここで重要なのは、**Mempoolは合意済みの台帳ではない**という点である。ノードAのMempoolに存在するトランザクションが、ノードBにも存在する保証はない。また、Mempoolに入ったトランザクションが必ずブロックに含まれるわけでもない。

### 17.1.1 Mempoolの責務

| 責務 | 内容 | 責務外のもの |
|---|---|---|
| 候補管理 | 未確定トランザクションを保持する | 永続的なトランザクション確定 |
| 重複排除 | 同一IDの再受信を抑える | ネットワーク全体の重複排除 |
| 競合検出 | 同一UTXOを使う候補を検出する | 最終的な二重支払いの解決 |
| ポリシー適用 | ローカルの資源・手数料・サイズ制約を適用する | プロトコルのコンセンサスルールを変更する |
| 依存関係管理 | 親トランザクションと子トランザクションの関係を追跡する | 無条件のトランザクション採用 |
| マイニング支援 | ブロック候補のトランザクション集合を提供する | PoWやGHOSTDAGの実行 |

> **設計原則:** コンセンサス上有効な取引でも、各ノードのローカルMempoolポリシーにより保留・拒否される場合がある。両者を混同してはいけない。

## 17.2 トランザクションのライフサイクル

トランザクションの状態を、概念的に次のように表せる。

```text
Created
   │ 署名・送信
   ▼
Received
   │ 形式検査
   ▼
Prechecked
   │ UTXO・競合・ポリシー確認
   ├── Rejected
   ├── Deferred（依存関係などの待機）
   ▼
InMempool
   │   ├── ピアへ伝播
   │   └── ブロック候補に選択
   ▼
IncludedInBlock
   │ コンセンサス評価
   ├── NotAccepted（競合・順序など）
   ▼
AcceptedInVirtualState
```

これは説明用の状態機械であり、`rusty-kaspa` 内部の型名・状態列挙をそのまま転記したものではない。実際のMempool実装では、トランザクションの追加・削除・再検査・依存関係処理が複数の経路から発生する。各ノードがすべての段階を観測する必要はなく、ブロック経由で初めて取引を知る場合もある。送信成功の応答が失われたときは、拒否ではなく成否不明の状態も必要となる。

### 17.2.1 「受信」「保持」「採用」「確定」を区別する

- **受信:** ノードがトランザクションを受け取った。
- **Mempool受理:** ローカルノードが候補として保持した。
- **ブロック収録:** あるブロックにトランザクションが含まれた。
- **コンセンサス上の受理:** DAGの順序付けと状態評価を通過した。
- **確定性:** 再編成リスクなどを踏まえた、トランザクション結果への信頼度。ブロック収録と同義ではない。

## 17.3 Kaspaにおけるトランザクションの伝播

P2Pネットワークでは、ウォレットやノードから送信されたトランザクションが、複数のピアを経由して広がる。各ピアは受信内容を検査し、必要な場合に隣接ピアへ通知する。

```text
Wallet ──> Node A ──> Node B ──> Node D
               │           │
               └──> Node C └──> Node E
```

実際のKaspaでは、メッセージ種別、在庫通知、要求・応答、経路制御などを含むプロトコルに従う。上図は**概念図**であり、特定バージョンのワイヤープロトコルを表現したものではない。

基準版の [`v5/txrelay/flow.rs`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/protocol/flows/src/v5/txrelay/flow.rs) は `InvTransactions` で通知されたIDについて必要な取引を要求し、`RequestTransactions` に対して本体または `TransactionNotFound` を返す。取得した本体のIDも要求したIDと照合し、MiningManagerのバッチ検証・登録へ渡す。`v5/` はソースの配置であり、接続で合意したプロトコル番号と同一視しない（[第10章](../part-02-consensus/chapter-10.md)）。

### 17.3.1 Gossipと重複排除

同じトランザクションは複数のピアから届く可能性がある。そのため、受信側では識別子などを用いた重複排除が必要になる。

```text
Node A ──tx X──> Node B
Node A ──tx X──> Node C ──tx X──> Node B
                              （2回目は重複）
```

重複排除がないと、署名検証・DB参照・ネットワーク帯域を無駄に消費し、DoSへの耐性も下がる。

### 17.3.2 ネットワーク全体でMempoolは一致しない

ノードごとに以下が異なる。

- ピア接続と受信順序
- 一時的なネットワーク分断や遅延
- Mempool容量とローカルポリシー
- UTXO状態や同期進捗
- 依存トランザクションの受信有無

したがって「トランザクションがノードAのMempoolにある」ことから「全ノードがトランザクションを認識している」とは推論できない。

## 17.4 トランザクション検証の境界

検証を少なくとも3層に分けると、実装の依存関係を理解しやすい。

| 層 | 主な検査 | 結果 |
|---|---|---|
| 構文・形式 | 入出力の構造、サイズ、値の範囲、識別子 | 明らかな不正入力を排除 |
| Mempool・ポリシー | 重複、競合、容量、依存関係、手数料方針 | ローカル候補として保持するか |
| コンセンサス | スクリプト、UTXO、金額保存、DAG順序・状態に対する妥当性 | 台帳状態に適用できるか |

コンセンサス規則の検査はMempool登録前にも行う。Mempoolの保持を、そのままコンセンサスUTXOへの適用とは扱わない。実際には検証項目の共有やキャッシュがあり、層が完全に独立しているとは限らない。重要なのは**ローカルのリレー方針をコンセンサスルールと同一視しないこと**である。

### 17.4.1 基準版の受付境界

[`MiningManager`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/mining/src/manager.rs) は、Mempoolの読み取りロック下で事前検査と入力情報の補完を行い、ロックを外してコンセンサスAPIによるUTXO文脈の検証を行う。その後、書き込みロック下で重複・受理済み判定や競合等を確認して登録する。事前検査の結果だけで、後の挿入が必ず成功するわけではない。

[`Mempool`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/mining/src/mempool/mod.rs) は `transaction_pool`、`orphan_pool`、最近の処理IDを保持する `accepted_transactions` を分ける。通常プールの件数、孤児を含む照会結果、Virtualの受理結果は別の情報である。

## 17.5 UTXO競合と二重支払い

UTXOを1回だけ使用できるという制約は、トランザクション処理の中心的な不変条件である。

```text
UTXO: (tx0, output0) = 10 KAS
           │
       ┌───┴───┐
       ▼       ▼
      Tx A    Tx B
     送金先X  送金先Y
```

Tx AとTx Bが同じOutpointを入力に含む場合、両方を同じUTXO状態へ適用することはできない。

### 17.5.1 Mempoolでの競合とコンセンサスでの競合

- Mempoolは、ローカルに保持しているトランザクション同士の競合を検出できる。
- ネットワーク分断中、別々のノードが競合トランザクションを保持することはあり得る。
- BlockDAGに並行ブロックとして両方が現れても、コンセンサス順序とUTXO検証によって状態に適用できるトランザクションが決まる。

**BlockDAGが複数ブロックを保持すること**と、**競合する二重支払いを両方受理すること**は異なる。

### 17.5.2 基準版の置換ポリシー

基準版の [`flow_context.rs`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/protocol/flows/src/flow_context.rs) では、通常のRPC送信は `Priority::High` / `RbfPolicy::Forbidden`、置換用RPCは `Priority::High` / `Orphan::Forbidden` / `RbfPolicy::Mandatory` を使う。P2Pのtx relayは `Priority::Low` / `Orphan::Allowed` / `RbfPolicy::Allowed` を渡す。「同じ入力なら常に拒否」でも「高い総手数料なら必ず置換」でもない。High/Lowは保持・期限・再伝播の扱いに関係し、Highという名前だけで採掘の選択優先度を意味するわけではない。

[`replace_by_fee.rs`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/mining/src/mempool/replace_by_fee.rs) は既存の競合取引のfee/massから閾値を求める。Mandatoryでは競合がちょうど1件であることも要求する。具体的な比較と受付条件は固定版の規則を確認し、BitcoinのRBF規則から推測しない。

## 17.6 親子トランザクションと依存関係

あるトランザクションの出力を、別のトランザクションが入力として使う場合がある。

```text
Tx A ── output #0 ──> Tx B ── output #1 ──> Tx C
```

Tx BはTx Aに依存する。Tx Bだけが先に届いた場合、必要なUTXOが見つからない可能性がある。

### 17.6.1 依存関係を表すDAG

トランザクション依存グラフもDAGとして表現できるが、**ブロック同士の親子関係を表すBlockDAGとは別のグラフ**である。

- **Transaction dependency DAG:** どのトランザクション出力を次のトランザクションが消費するか。
- **BlockDAG:** どのブロックがどのブロックを親として参照するか。

同じ「DAG」という言葉でも、頂点・辺・不変条件・目的が異なる。

### 17.6.2 孤立トランザクション・保留トランザクション

参照先のUTXOがまだ見えないトランザクションは、即時拒否されることも、条件付きで保留されることもある。基準版の [`post_validate_and_insert_transaction`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/mining/src/mempool/validate_and_insert_transaction.rs) は、入力不足かつ `Orphan::Allowed` なら孤児プールへの登録を試みる。Forbiddenなら `RejectDisallowedOrphan` とする。通常のRPCでは `allow_orphan` の指定だけでなく、ノードがunsafe RPCを許可している必要もある（[RPC入口](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/rpc/service/src/service.rs)）。

通常プールにある親の出力情報を入力へ補完し、親の到着後は孤児の不足情報を埋めて再検証する。孤児プールは正当な取引であるとの保証ではない。Mempoolで親子を保持できることも、同一ブロック内に親子取引を入れてよいという意味ではない（[第15章](chapter-15.md)）。

## 17.7 手数料とMempoolの資源制約

Mempoolは無制限に拡張できない。保持件数、メモリ、CPU、帯域を消費するため、ノードには資源管理が必要になる。

```text
受信トランザクション
       │
       ▼
検証コストを支払えるか？
       │
       ▼
Mempool容量に余裕があるか？
       │
       ▼
ローカルポリシーに適合するか？
       │
       ▼
候補として保持
```

手数料が高ければ常に受理されるわけではない。形式不正、無効な署名、使用済みUTXO、依存関係不整合などは手数料で解消されない。

### 17.7.1 トランザクション選択の設計

マイナーはブロックサイズ・質量（mass）・手数料・依存関係などの制約を考慮して候補を選ぶ。Kaspaの具体的な質量計算・優先順位・ポリシーはバージョン依存なので、固定の数値や優先順位を一般則として記述しない。

## 17.8 ブロック生成後のMempool更新

ブロックを受信・処理すると、Mempool側にも影響がある。

1. ブロックに収録されたトランザクションを追跡する。
2. 新しいコンセンサス状態と競合する候補を再評価する。
3. 依存関係が解決した候補を処理する。
4. 再編成などによる状態変更を反映する。

BlockDAGでは、単純な「最新ブロックに入ったトランザクションを消す」だけでは十分でない。**コンセンサス上の受理状態**と、**Mempoolに残すべき候補**を分けて考える必要がある。

### 17.8.1 基準版で確認した更新経路

`FlowContext` はブロックのVirtual処理を待ってから、ブロック中の取引をMiningManagerへ渡す。[`handle_new_block_transactions`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/mining/src/mempool/handle_new_block_transactions.rs) はCoinbaseを除く取引について、プールからの削除、競合する候補の削除、孤児の再評価、最近のIDの記録等を行う。この内部の `accepted_transactions` という名前だけを、取引のVirtual受理や不可逆な決済確定の証拠にしない。

再編成で外れたすべての取引が自動的にMempoolへ戻るとの保証は、この経路の照合だけでは得られない。再検証・再伝播と、全再編成シナリオの検証を区別する。

## 17.9 Reorganizationとトランザクションの再評価

Virtual Stateが変化した場合、以前は有効だったトランザクションが無効になることも、その逆もあり得る。

```text
旧Virtual State
      │
      ├── Tx A: 入力UTXOあり
      │
      ▼
DAGの新しい情報・再評価
      │
      ▼
新Virtual State
      └── Tx A: 入力UTXOが別取引で消費済み
```

Mempoolを設計する際は、**状態変更後に古い判定結果を無条件で信頼しない**ことが重要である。どのイベントで再検査するか、どのキャッシュを破棄するかは実装上の要点となる。

## 17.10 P2P伝播のセキュリティ

| 脅威 | 典型的な影響 | 対策の方向性 |
|---|---|---|
| Transaction flooding | CPU・メモリ・帯域の枯渇 | レート制限、サイズ制限、検証順序 |
| 重複メッセージ | 不必要な検証負荷 | IDベースの重複排除 |
| Sybil攻撃 | 偽のピアによる接続資源の占有 | ピア選択の多様性、接続制限 |
| Eclipse攻撃 | ノードの情報源を偏らせる | 接続先分散、異常監視 |
| 不正な依存チェーン | 保留領域や検証負荷の増大 | 依存深度・容量の制御 |
| 不正なトランザクションデータ | パーサや検証器の攻撃面拡大 | 境界での厳密な検証 |

**不変条件:** ネットワークから届く入力を信用せず、Mempoolの受付失敗を理由にコンセンサスUTXO状態を書き換えないこと。

## 17.11 Rustで最小Mempoolを実装する

この節では、ネットワーク・署名・Kaspaの具体的なトランザクション形式を省略し、**重複排除とOutpoint競合**だけを検証する。

### 17.11.1 プロジェクト作成

本書とは別の作業用ディレクトリで実行する。外部依存はなく、Rust 1.90.0で確認した。

```bash
cargo new --lib --edition 2021 minimal-mempool
cd minimal-mempool
```

標準ライブラリのみを使用する。以下を `src/lib.rs` に保存する。

```rust
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Outpoint {
    pub txid: String,
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub id: String,
    pub inputs: Vec<Outpoint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionError {
    EmptyId,
    DuplicateTransaction,
    DuplicateInput,
    ConflictingInput,
    EmptyInputs,
}

#[derive(Default)]
pub struct Mempool {
    transactions: HashMap<String, Transaction>,
    spent_inputs: HashMap<Outpoint, String>,
}

impl Mempool {
    pub fn admit(&mut self, tx: Transaction) -> Result<(), AdmissionError> {
        if tx.id.is_empty() {
            return Err(AdmissionError::EmptyId);
        }
        if self.transactions.contains_key(&tx.id) {
            return Err(AdmissionError::DuplicateTransaction);
        }
        if tx.inputs.is_empty() {
            return Err(AdmissionError::EmptyInputs);
        }

        // すべての入力を検査してから状態を書き換える。
        let mut seen = HashSet::new();
        for input in &tx.inputs {
            if !seen.insert(input) {
                return Err(AdmissionError::DuplicateInput);
            }
            if self.spent_inputs.contains_key(input) {
                return Err(AdmissionError::ConflictingInput);
            }
        }

        for input in &tx.inputs {
            self.spent_inputs.insert(input.clone(), tx.id.clone());
        }
        self.transactions.insert(tx.id.clone(), tx);
        Ok(())
    }

    pub fn remove(&mut self, id: &str) -> Option<Transaction> {
        let tx = self.transactions.remove(id)?;
        for input in &tx.inputs {
            self.spent_inputs.remove(input);
        }
        Some(tx)
    }

    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outpoint() -> Outpoint {
        Outpoint {
            txid: "funding".into(),
            index: 0,
        }
    }

    fn tx(id: &str) -> Transaction {
        Transaction {
            id: id.into(),
            inputs: vec![outpoint()],
        }
    }

    #[test]
    fn accepts_first_transaction() {
        let mut pool = Mempool::default();
        assert_eq!(pool.admit(tx("a")), Ok(()));
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn rejects_duplicate_transaction() {
        let mut pool = Mempool::default();
        pool.admit(tx("a")).unwrap();
        assert_eq!(
            pool.admit(tx("a")),
            Err(AdmissionError::DuplicateTransaction)
        );
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn rejects_conflicting_transaction_without_mutation() {
        let mut pool = Mempool::default();
        pool.admit(tx("a")).unwrap();
        assert_eq!(pool.admit(tx("b")), Err(AdmissionError::ConflictingInput));
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn rejects_duplicate_inputs_within_one_transaction() {
        let mut pool = Mempool::default();
        let bad = Transaction {
            id: "bad".into(),
            inputs: vec![outpoint(), outpoint()],
        };
        assert_eq!(pool.admit(bad), Err(AdmissionError::DuplicateInput));
        assert!(pool.is_empty());
    }

    #[test]
    fn all_admission_errors_preserve_both_maps() {
        let mut pool = Mempool::default();
        pool.admit(tx("a")).unwrap();
        let fresh = Outpoint {
            txid: "fresh".into(),
            index: 0,
        };
        let before = (pool.transactions.clone(), pool.spent_inputs.clone());
        for (candidate, error) in [
            (tx(""), AdmissionError::EmptyId),
            (
                Transaction {
                    id: "empty".into(),
                    inputs: vec![],
                },
                AdmissionError::EmptyInputs,
            ),
            (
                Transaction {
                    id: "a".into(),
                    inputs: vec![fresh.clone()],
                },
                AdmissionError::DuplicateTransaction,
            ),
            (
                Transaction {
                    id: "duplicate".into(),
                    inputs: vec![fresh.clone(), fresh.clone()],
                },
                AdmissionError::DuplicateInput,
            ),
            (
                Transaction {
                    id: "conflict".into(),
                    inputs: vec![fresh, outpoint()],
                },
                AdmissionError::ConflictingInput,
            ),
        ] {
            assert_eq!(pool.admit(candidate), Err(error));
            assert_eq!(
                (&pool.transactions, &pool.spent_inputs),
                (&before.0, &before.1)
            );
        }
    }

    #[test]
    fn removing_unknown_transaction_preserves_reservations() {
        let mut pool = Mempool::default();
        pool.admit(tx("a")).unwrap();
        let before = (pool.transactions.clone(), pool.spent_inputs.clone());
        assert!(pool.remove("absent").is_none());
        assert_eq!(
            (&pool.transactions, &pool.spent_inputs),
            (&before.0, &before.1)
        );
        assert_eq!(pool.admit(tx("b")), Err(AdmissionError::ConflictingInput));
    }

    #[test]
    fn removal_releases_only_its_own_reservations() {
        let mut pool = Mempool::default();
        let other = Outpoint {
            txid: "other".into(),
            index: 0,
        };
        pool.admit(tx("a")).unwrap();
        pool.admit(Transaction {
            id: "c".into(),
            inputs: vec![other.clone()],
        })
        .unwrap();
        assert_eq!(pool.remove("a").unwrap(), tx("a"));
        assert!(pool.remove("a").is_none());
        assert_eq!(pool.admit(tx("b")), Ok(()));
        assert_eq!(
            pool.admit(Transaction {
                id: "d".into(),
                inputs: vec![other.clone()]
            }),
            Err(AdmissionError::ConflictingInput)
        );
        assert_eq!(
            pool.spent_inputs,
            HashMap::from([(outpoint(), "b".into()), (other, "c".into())])
        );
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn removing_transaction_releases_input_reservation() {
        let mut pool = Mempool::default();
        pool.admit(tx("a")).unwrap();
        assert!(pool.remove("a").is_some());
        assert_eq!(pool.admit(tx("b")), Ok(()));
    }
}
```

実行方法:

```bash
cargo test
```

本文から抽出した教材は8テスト成功。空ID、全拒否理由で2つのmapが変わらないこと、未知IDの削除、他の取引の予約を残す削除を確認した（`experiments/chapter-17-review-2026-11-01.md`）。

### 17.11.2 この実装が保証すること

- 同一トランザクションIDは二重登録されない。
- 同一Outpointは、同時に保持する2つのトランザクションから予約されない。
- 不正なトランザクションの追加失敗時、内部状態を部分更新しない。
- トランザクション削除時に、そのトランザクションが予約していた入力を解放する。

### 17.11.3 この実装が保証しないこと

- UTXOの存在や金額の確認
- 署名・Script検証
- トランザクションIDの暗号学的計算
- 手数料やmassの評価
- RBF・容量上限・期限切れ・孤児プール・親子トランザクションの管理
- コンセンサス状態との同期
- 並行アクセス・永続化・ネットワーク伝播

このモデルの `spent_inputs` は**Mempool内の入力予約**であり、コンセンサスUTXO集合そのものではない。削除で予約を解放しても、実際のUTXOが使用可能になったことは示さない。同じ文字列IDの別内容もDuplicateTransactionとして拒否するが、暗号学的なID・内容の対応は検証しない。

失敗時の集合不変はこの教材の方針である。基準版の `post_validate_and_insert_transaction` には、RBFで既存候補を除いた後に容量制限で新規取引が拒否され、先の削除が残る例外がコメントされている。公式Mempoolの受付失敗は常にプール全体を以前と同じにする、と一般化しない。これはコンセンサスUTXOを消費する処理とは別である。

## 17.12 状態機械と不変条件

Mempoolの受理処理は、次のような状態遷移として整理できる。

```text
NotSeen ──receive──> Checking
                        │
              ┌─────────┼──────────┐
              ▼         ▼          ▼
           Rejected   Deferred   Admitted
                                     │
                         ┌───────────┼──────────┐
                         ▼           ▼          ▼
                      Removed     Included    Rechecked
```

実装レビューでは次を確認する。

1. **原子性:** 失敗時に予約状態が半端に残らないか。
2. **一意性:** 同じIDや同じ入力の競合を一貫して扱えるか。
3. **整合性:** コンセンサス状態更新後に古い候補を再評価するか。
4. **資源上限:** 入力サイズ・保留件数・検証コストに上限があるか。
5. **可観測性:** 受理・拒否・削除理由をログやメトリクスで区別できるか。

## 17.13 DDD・C4で責務を整理する

### 17.13.1 コンポーネントの境界

```text
┌──────────────────────────────────────────────────────┐
│                    Kaspa Node                        │
│                                                      │
│  ┌──────────┐    ┌─────────────────────────────┐     │
│  │ P2P / RPC│───>│ Transaction Admission       │     │
│  └──────────┘    └────────────┬────────────────┘     │
│                               │                      │
│                    ┌──────────▼──────────┐           │
│                    │ Mempool             │           │
│                    └──────────┬──────────┘           │
│                               │                      │
│         ┌─────────────────────┼────────────────┐     │
│         ▼                     ▼                ▼     │
│   UTXO / Consensus      Mining Template    P2P Relay │
└──────────────────────────────────────────────────────┘
```

UTXO / Consensusとの線は検証のための参照関係であり、Mempoolの登録がそのままコンセンサスUTXOを書き換える意味ではない。上図は学習用の論理的な責務分解であり、実際のRustクレート境界を示したものではない。

### 17.13.2 ドメイン用語

| 用語 | 意味 |
|---|---|
| Candidate Transaction | Mempoolに保持するトランザクション候補 |
| Outpoint | 消費対象UTXOの識別子 |
| Conflict | 同じOutpointを使うトランザクション同士の競合 |
| Admission Policy | ローカル受理基準 |
| Consensus Rule | ネットワーク全体で一致すべき検証ルール |
| Revalidation | 状態変化に応じた候補の再検査 |

## 17.14 `rusty-kaspa` のソースコードを読む

### 実ノードでMempoolと入力競合を観察する

モデルの検査から実際のノードへ進む場合は、[第28章28.6](../part-04-development/chapter-28.md#286-ローカル環境の準備)でSDK・Devnet環境を準備し、[28.10の送金実習](../part-04-development/chapter-28.md#2810-トランザクションのe2eテスト)で`send.cjs --checks`を実行する。起動・終了とコンテナ名の条件は、その手順に従う。

このモードでは、正常な送金を送る前に採掘を止め、送信後のMempool掲載を確認する。続いて同じ入力を使う競合取引の拒否を調べ、採掘を再開して正常な取引のvirtual受理と入力消費を確認する。

| 観察する出力 | 本章の説明との対応 |
|---|---|
| `checks.mempoolObserved: true` | ノードのMempoolへ候補として入ったこと。決済確定ではない |
| `checks.competingSpend: rejected-as-expected` | 同じ入力を使う競合取引が拒否されたこと |
| `acceptedByVirtualChain: true` | 採掘再開後、正常な取引がvirtual chainで受理されたこと |

これは固定したノードの単一署名・所定の競合ケースの観察である。全RBF条件や複数ノードのMempool一致を検査する実験ではない。実際の処理経路は、次のソース読解で対応付ける。

### 固定ソースの入口

公式リポジトリ: [kaspanet/rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)

調査は、**固定したコミットを起点**に行う。以下のコマンドは手元で実行する例であり、特定バージョンでのディレクトリ名を保証するものではない。

```bash
# 第1章で取得した固定版の親ディレクトリから実行する
cd rusty-kaspa-v1.0.1
git rev-parse HEAD
grep -n -E 'validate_and_insert_transaction|handle_new_block_transactions' mining/src/manager.rs
grep -n -E 'Orphan::|RbfPolicy::|post_validate' mining/src/mempool/validate_and_insert_transaction.rs
grep -n -E 'InvTransactions|RequestTransactions|TransactionNotFound' protocol/flows/src/v5/txrelay/flow.rs
```

未取得なら[第1章](../part-01-foundations/chapter-01.md)の手順を使い、HEADを本章冒頭の完全SHAと照合する。次の問いを順番に追う。

1. RPCでトランザクションを受け取る入口はどこか。
2. P2Pでトランザクションを受信する入口はどこか。
3. 受信後にどの検証APIを呼ぶか。
4. Mempoolのトランザクション・入力競合をどの構造体で管理するか。
5. コンセンサス状態が変化したとき、どの経路で候補を更新するか。
6. ブロックテンプレート生成はどのAPIを利用するか。
7. エラーを「再試行可能」「恒久的拒否」に分類しているか。

調査メモには `commit hash`、ファイルパス、関数名、入出力、呼び出し元、呼び出し先、確認したテスト名を記録する。

## 17.15 運用・観測とトラブルシューティング

### 17.15.1 観測したい指標

- Mempool内のトランザクション件数・推定メモリ使用量
- 受信トランザクション数と重複率
- 受理・拒否・保留の件数と理由
- 検証処理時間
- ピアへの伝播遅延
- ブロック生成・コンセンサス更新に伴う削除数
- ノードの同期状態とピア数

これらは**設計上の推奨指標**であり、標準RPCがすべて提供することを意味しない。

### 17.15.2 典型的な調査手順

**症状: ウォレットから送信したのにトランザクションが見つからない**

1. 送信APIの応答が成功・拒否・成否不明のどれかを確認する。応答喪失を拒否と決めつけない。
2. トランザクションIDを記録する。
3. 送信先ノードの同期状態を確認する。
4. Mempool受理結果・拒否理由を確認する。
5. 別ノードで認識されているか確認する。
6. ブロック収録・コンセンサス受理状態を区別して調べる。

**症状: Mempoolにあったトランザクションが消えた**

ブロック収録、競合、容量制限、状態再評価、ノード再起動など、複数の可能性がある。「消えた＝確定した」とは限らない。

## 17.16 Bitcoinとの比較

| 観点 | Bitcoin | Kaspa |
|---|---|---|
| 台帳構造 | 競合する枝から採用チェーンを選ぶ | BlockDAG |
| 未確定トランザクション候補 | ノードごとのMempool | ノードごとのMempool |
| トランザクション伝播 | P2Pリレー | P2Pリレー |
| 二重支払い | UTXOとチェーン状態で評価 | UTXOとDAG順序・Virtual Stateで評価 |
| ブロック処理 | 主に選択されたチェーンに基づく | 並行ブロックを含むDAGを評価 |
| ローカルポリシー | コンセンサスとは区別 | コンセンサスとは区別 |

共通点は多いが、**Mempoolの更新契機とコンセンサス状態の変化**を調べる際は、Kaspa固有のBlockDAG・Virtual Stateを理解する必要がある。

## 17.17 よくある誤解

**Q1. Mempoolに入れば送金は確定する？**  
いいえ。Mempool受理はローカル候補としての保持を意味する。

**Q2. 全ノードのMempoolは同じ？**  
いいえ。伝播遅延、受信順序、ポリシー、同期状態などで異なる。

**Q3. BlockDAGなら二重支払いを両方処理できる？**  
いいえ。並行ブロックを保持することと、競合するUTXO消費を両方受理することは別である。

**Q4. 署名が正しければ必ずMempoolに入る？**  
いいえ。UTXO競合、金額、サイズ、ローカルポリシーなど別の条件がある。

**Q5. Mempoolのトランザクションが消えたらブロックに入った？**\
必ずしもそうではない。削除理由を調べる必要がある。

## 17.18 実践課題

### 課題A: 依存トランザクション

17.11の教材を拡張し、入力の参照Outpointを、コンセンサスUTXO集合とMempool内の親トランザクションの出力から解決する設計を考える。教材の `Transaction` には出力情報がないため、まず必要な出力データと照会インターフェースを定義する。親がMempoolになくても、参照出力がコンセンサスUTXO集合にあれば、その入力は解決できる。どちらからも解決できない場合は `Deferred` として保留し、不足データの到着後に再検査する流れを示す。入力の解決だけで署名・金額・競合の検査まで成功したとは扱わない。

### 課題B: 容量制限

Mempoolの最大件数を設け、上限到達時の動作をテストする。どのトランザクションを優先するかは明示的なポリシーとして分離する。

### 課題C: 状態再評価

`ConsensusChanged` イベントを追加し、現在のコンセンサスUTXO集合に対してMempool内の入力を再検査する処理を設計する。

### 課題D: 伝播実験

3〜5ノードの簡易シミュレーターを作成し、メッセージ遅延・重複・分断を注入する。ノード間のMempool件数の差を観測する。

## 17.19 理解度チェック

1. Mempool受理とコンセンサス受理は何が違うか。
2. 同じトランザクションが複数ピアから届くのはなぜか。
3. UTXO競合をMempoolで検出しても、コンセンサス検証が必要な理由は何か。
4. 親トランザクションより先に子トランザクションが届くと何が起こるか。
5. BlockDAGの並行ブロックと二重支払いの受理を混同してはいけない理由は何か。
6. Mempoolの容量制限が必要なのはなぜか。
7. Virtual Stateが変化した際に候補の再評価が必要なのはなぜか。
8. ローカルポリシーとコンセンサスルールの違いを説明せよ。
9. Rustの `admit` で全入力を確認してから状態を変更する理由は何か。
10. 送信したトランザクションが見つからない場合、どの順番で調査するか。

## 17.20 まとめ

- Mempoolは**未確定トランザクションのローカルな候補集合**であり、合意済み台帳ではない。
- P2P伝播は非同期であり、各ノードのMempoolは常に一致するとは限らない。
- UTXO競合、親子依存、容量制約、ローカルポリシーが受理判断に影響する。
- BlockDAG上でブロックを保持することと、そのトランザクションをVirtual Stateへ適用することは別である。
- Rustで設計する際は、**原子性・重複排除・競合検出・状態再評価・資源制限**を不変条件として明示する。

[次章](chapter-18.md)では、Mempoolの候補からマイナーがブロックテンプレートを構築する流れを扱う。

## 参考資料

1. [Kaspa公式サイト](https://kaspa.org/)
2. [Kaspa Wiki](https://wiki.kaspa.org/en/home)
3. [rusty-kaspa（公式Rust実装）](https://github.com/kaspanet/rusty-kaspa)
4. [Kaspa Improvement Proposals](https://github.com/kaspanet/kips)
5. [Bitcoin Developer Reference](https://developer.bitcoin.org/reference/)
6. [Bitcoin: A Peer-to-Peer Electronic Cash System](https://bitcoin.org/bitcoin.pdf)

今回の照合は基準版の主要なRPC/P2P受付、競合/孤児管理、ブロック後処理の範囲である。全RBF条件・孤児上限・期限・mass/手数料・全再編成時の再投入や、公式関数との実行時差分は未検証。本文確認・教材8テスト・ソース識別情報は `experiments/chapter-17-review-2026-11-01.md` に記録した。
