# Chapter 17 — Mempool & Transaction Propagation（Mempoolとトランザクション伝播）

> **Mastering Kaspa — Part III: Transactions, Mining & Security**  
> 原稿バージョン: v0.1（技術レビュー前）  
> 本章の実験は教育用モデルであり、Kaspaメインネットのコンセンサス互換実装ではない。

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

ここで重要なのは、**Mempoolは合意済みの台帳ではない**という点である。ノードAのMempoolに存在するトランザクションが、ノードBにも存在する保証はない。また、Mempoolに入った取引が必ずブロックに含まれるわけでもない。

### 17.1.1 Mempoolの責務

| 責務 | 内容 | 責務外のもの |
|---|---|---|
| 候補管理 | 未確定取引を保持する | 永続的な取引確定 |
| 重複排除 | 同一IDの再受信を抑える | ネットワーク全体の重複排除 |
| 競合検出 | 同一UTXOを使う候補を検出する | 最終的な二重支払いの解決 |
| ポリシー適用 | ローカルの資源・手数料・サイズ制約を適用する | プロトコルのコンセンサスルールを変更する |
| 依存関係管理 | 親取引と子取引の関係を追跡する | 無条件の取引採用 |
| マイニング支援 | ブロック候補の取引集合を提供する | PoWやGHOSTDAGの実行 |

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

これは説明用の状態機械であり、`rusty-kaspa` 内部の型名・状態列挙をそのまま転記したものではない。実際のMempool実装では、取引の追加・削除・再検査・依存関係処理が複数の経路から発生する。

### 17.2.1 「受信」「保持」「採用」「確定」を区別する

- **受信:** ノードがトランザクションを受け取った。
- **Mempool受理:** ローカルノードが候補として保持した。
- **ブロック収録:** あるブロックにトランザクションが含まれた。
- **コンセンサス上の受理:** DAGの順序付けと状態評価を通過した。
- **確定性:** 再編成リスクなどを踏まえた、取引結果への信頼度。ブロック収録と同義ではない。

## 17.3 Kaspaにおけるトランザクションの伝播

P2Pネットワークでは、ウォレットやノードから送信された取引が、複数のピアを経由して広がる。各ピアは受信内容を検査し、必要な場合に隣接ピアへ通知する。

```text
Wallet ──> Node A ──> Node B ──> Node D
               │           │
               └──> Node C └──> Node E
```

実際のKaspaでは、メッセージ種別、在庫通知、要求・応答、経路制御などを含むプロトコルに従う。上図は**概念図**であり、特定バージョンのワイヤープロトコルを表現したものではない。

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
- 依存取引の受信有無

したがって「取引がノードAのMempoolにある」ことから「全ノードが取引を認識している」とは推論できない。

## 17.4 トランザクション検証の境界

検証を少なくとも3層に分けると、実装の依存関係を理解しやすい。

| 層 | 主な検査 | 結果 |
|---|---|---|
| 構文・形式 | 入出力の構造、サイズ、値の範囲、識別子 | 明らかな不正入力を排除 |
| Mempool・ポリシー | 重複、競合、容量、依存関係、手数料方針 | ローカル候補として保持するか |
| コンセンサス | スクリプト、UTXO、金額保存、DAG順序・状態に対する妥当性 | 台帳状態に適用できるか |

実際には検証項目の共有やキャッシュがあり、層が完全に独立しているとは限らない。重要なのは**ローカルのリレー方針をコンセンサスルールと同一視しないこと**である。

## 17.5 UTXO競合と二重支払い

UTXOを1回だけ使用できるという制約は、取引処理の中心的な不変条件である。

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

- Mempoolは、ローカルに保持している取引同士の競合を検出できる。
- ネットワーク分断中、別々のノードが競合取引を保持することはあり得る。
- BlockDAGに並行ブロックとして両方が現れても、コンセンサス順序とUTXO検証によって状態に適用できる取引が決まる。

**BlockDAGが複数ブロックを保持すること**と、**競合する二重支払いを両方受理すること**は異なる。

## 17.6 親子トランザクションと依存関係

ある取引の出力を、別の取引が入力として使う場合がある。

```text
Tx A ── output #0 ──> Tx B ── output #1 ──> Tx C
```

Tx BはTx Aに依存する。Tx Bだけが先に届いた場合、必要なUTXOが見つからない可能性がある。

### 17.6.1 依存関係を表すDAG

取引依存グラフもDAGとして表現できるが、**ブロック同士の親子関係を表すBlockDAGとは別のグラフ**である。

- **Transaction dependency DAG:** どの取引出力を次の取引が消費するか。
- **BlockDAG:** どのブロックがどのブロックを親として参照するか。

同じ「DAG」という言葉でも、頂点・辺・不変条件・目的が異なる。

### 17.6.2 孤立取引・保留取引

参照先のUTXOがまだ見えない取引は、即時拒否されることも、条件付きで保留されることもある。実際の取り扱いは実装バージョンとポリシーに依存するため、ソースコードで確認する。

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

### 17.7.1 取引選択の設計

マイナーはブロックサイズ・質量（mass）・手数料・依存関係などの制約を考慮して候補を選ぶ。Kaspaの具体的な質量計算・優先順位・ポリシーはバージョン依存なので、固定の数値や優先順位を一般則として記述しない。

## 17.8 ブロック生成後のMempool更新

ブロックを受信・処理すると、Mempool側にも影響がある。

1. ブロックに収録された取引を追跡する。
2. 新しいコンセンサス状態と競合する候補を再評価する。
3. 依存関係が解決した候補を処理する。
4. 再編成などによる状態変更を反映する。

BlockDAGでは、単純な「最新ブロックに入った取引を消す」だけでは十分でない。**コンセンサス上の受理状態**と、**Mempoolに残すべき候補**を分けて考える必要がある。

## 17.9 Reorganizationと取引の再評価

Virtual Stateが変化した場合、以前は有効だった取引が無効になることも、その逆もあり得る。

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
| 不正な取引データ | パーサや検証器の攻撃面拡大 | 境界での厳密な検証 |

**不変条件:** ネットワークから届く入力を信用せず、失敗した取引が確定済みUTXO状態を変更しないこと。

## 17.11 Rustで最小Mempoolを実装する

この節では、ネットワーク・署名・Kaspaの具体的な取引形式を省略し、**重複排除とOutpoint競合**だけを検証する。

### 17.11.1 プロジェクト作成

```bash
cargo new minimal-mempool
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

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub inputs: Vec<Outpoint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionError {
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
        Outpoint { txid: "funding".into(), index: 0 }
    }

    fn tx(id: &str) -> Transaction {
        Transaction { id: id.into(), inputs: vec![outpoint()] }
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
        assert_eq!(pool.admit(tx("a")), Err(AdmissionError::DuplicateTransaction));
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
        let bad = Transaction { id: "bad".into(), inputs: vec![outpoint(), outpoint()] };
        assert_eq!(pool.admit(bad), Err(AdmissionError::DuplicateInput));
        assert!(pool.is_empty());
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

### 17.11.2 この実装が保証すること

- 同一トランザクションIDは二重登録されない。
- 同一Outpointは、同時に保持する2つの取引から予約されない。
- 不正な取引の追加失敗時、内部状態を部分更新しない。
- 取引削除時に、その取引が予約していた入力を解放する。

### 17.11.3 この実装が保証しないこと

- UTXOの存在や金額の確認
- 署名・Script検証
- 取引IDの暗号学的計算
- 手数料やmassの評価
- 親子取引の管理
- コンセンサス状態との同期
- 並行アクセス・永続化・ネットワーク伝播

このモデルの `spent_inputs` は**Mempool内の入力予約**であり、確定済みUTXO集合そのものではない。

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

上図は学習用の論理的な責務分解であり、実際のRustクレート境界を示したものではない。

### 17.13.2 ドメイン用語

| 用語 | 意味 |
|---|---|
| Candidate Transaction | Mempoolに保持する取引候補 |
| Outpoint | 消費対象UTXOの識別子 |
| Conflict | 同じOutpointを使う取引同士の競合 |
| Admission Policy | ローカル受理基準 |
| Consensus Rule | ネットワーク全体で一致すべき検証ルール |
| Revalidation | 状態変化に応じた候補の再検査 |

## 17.14 `rusty-kaspa` のソースコードを読む

公式リポジトリ: [kaspanet/rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)

調査は、**固定したコミットを起点**に行う。以下のコマンドは手元で実行する例であり、特定バージョンでのディレクトリ名を保証するものではない。

```bash
git clone https://github.com/kaspanet/rusty-kaspa.git
cd rusty-kaspa
git rev-parse HEAD
rg -n 'mempool|Mempool|orphan|transaction relay|submit_transaction' --glob '*.rs' | head -100
```

次の問いを順番に追う。

1. RPCで取引を受け取る入口はどこか。
2. P2Pで取引を受信する入口はどこか。
3. 受信後にどの検証APIを呼ぶか。
4. Mempoolの取引・入力競合をどの構造体で管理するか。
5. コンセンサス状態が変化したとき、どの経路で候補を更新するか。
6. ブロックテンプレート生成はどのAPIを利用するか。
7. エラーを「再試行可能」「恒久的拒否」に分類しているか。

調査メモには `commit hash`、ファイルパス、関数名、入出力、呼び出し元、呼び出し先、確認したテスト名を記録する。

## 17.15 運用・観測とトラブルシューティング

### 17.15.1 観測したい指標

- Mempool内の取引件数・推定メモリ使用量
- 受信取引数と重複率
- 受理・拒否・保留の件数と理由
- 検証処理時間
- ピアへの伝播遅延
- ブロック生成・コンセンサス更新に伴う削除数
- ノードの同期状態とピア数

これらは**設計上の推奨指標**であり、標準RPCがすべて提供することを意味しない。

### 17.15.2 典型的な調査手順

**症状: ウォレットから送信したのに取引が見つからない**

1. 送信APIが成功したか確認する。
2. トランザクションIDを記録する。
3. 送信先ノードの同期状態を確認する。
4. Mempool受理結果・拒否理由を確認する。
5. 別ノードで認識されているか確認する。
6. ブロック収録・コンセンサス受理状態を区別して調べる。

**症状: Mempoolにあった取引が消えた**

ブロック収録、競合、容量制限、状態再評価、ノード再起動など、複数の可能性がある。「消えた＝確定した」とは限らない。

## 17.16 Bitcoinとの比較

| 観点 | Bitcoin | Kaspa |
|---|---|---|
| 台帳構造 | 線形ブロックチェーン | BlockDAG |
| 未確定取引候補 | ノードごとのMempool | ノードごとのMempool |
| 取引伝播 | P2Pリレー | P2Pリレー |
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

**Q5. Mempoolの取引が消えたらブロックに入った？**  
必ずしもそうではない。削除理由を調べる必要がある。

## 17.18 実践課題

### 課題A: 依存取引

17.11のMempoolに、親取引が存在するかを判定する機能を追加する。親取引が見つからない場合に `Deferred` として管理する設計を考える。

### 課題B: 容量制限

Mempoolの最大件数を設け、上限到達時の動作をテストする。どの取引を優先するかは明示的なポリシーとして分離する。

### 課題C: 状態再評価

`ConsensusChanged` イベントを追加し、確定済みUTXO集合に対してMempool内の入力を再検査する処理を設計する。

### 課題D: 伝播実験

3〜5ノードの簡易シミュレーターを作成し、メッセージ遅延・重複・分断を注入する。ノード間のMempool件数の差を観測する。

## 17.19 理解度チェック

1. Mempool受理とコンセンサス受理は何が違うか。
2. 同じ取引が複数ピアから届くのはなぜか。
3. UTXO競合をMempoolで検出しても、コンセンサス検証が必要な理由は何か。
4. 親取引より先に子取引が届くと何が起こるか。
5. BlockDAGの並行ブロックと二重支払いの受理を混同してはいけない理由は何か。
6. Mempoolの容量制限が必要なのはなぜか。
7. Virtual Stateが変化した際に候補の再評価が必要なのはなぜか。
8. ローカルポリシーとコンセンサスルールの違いを説明せよ。
9. Rustの `admit` で全入力を確認してから状態を変更する理由は何か。
10. 送信した取引が見つからない場合、どの順番で調査するか。

## 17.20 まとめ

- Mempoolは**未確定トランザクションのローカルな候補集合**であり、合意済み台帳ではない。
- P2P伝播は非同期であり、各ノードのMempoolは常に一致するとは限らない。
- UTXO競合、親子依存、容量制約、ローカルポリシーが受理判断に影響する。
- BlockDAG上でブロックを保持することと、その取引をVirtual Stateへ適用することは別である。
- Rustで設計する際は、**原子性・重複排除・競合検出・状態再評価・資源制限**を不変条件として明示する。

次章では、Mempoolの候補からマイナーがブロックテンプレートを構築する流れを扱う。

## 参考資料

1. [Kaspa公式サイト](https://kaspa.org/)
2. [Kaspa Wiki](https://wiki.kaspa.org/en/home)
3. [rusty-kaspa（公式Rust実装）](https://github.com/kaspanet/rusty-kaspa)
4. [Kaspa Improvement Proposals](https://github.com/kaspanet/kips)
5. [Bitcoin Developer Reference](https://developer.bitcoin.org/reference/)
6. [Bitcoin: A Peer-to-Peer Electronic Cash System](https://bitcoin.org/bitcoin.pdf)

> **技術レビュー時の確認事項:** Kaspaの具体的なP2Pメッセージ名、Mempoolの内部型、依存取引の保留規則、手数料・massに基づく取引選択、再編成時の再投入条件は、採用する `rusty-kaspa` のコミットを固定して検証する。本章はそれらの未検証事項を断定していない。
