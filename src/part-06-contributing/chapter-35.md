# Chapter 35 — Building a Kaspa Application（Kaspaアプリケーションの構築）

> **原稿ステータス：v0.1／技術レビュー前**  
> 本章は『Mastering Kaspa』Part VI「Contributing」の実践編である。実装例は学習用であり、資金を扱う本番アプリケーションとして監査済みではない。SDKやRPCのAPIはバージョンによって変化するため、実装時は利用するリリースの公式サンプルを確認する。

## 35.1 この章で作るもの

ここまで学んだBlockDAG、UTXO、ウォレット、RPC、ノード、テストの知識を組み合わせて、Kaspaを利用するアプリケーションを設計する。

本章の題材は **Kaspa Payment Tracker（支払い受付・入金監視アプリ）** とする。店舗や個人が請求を発行し、指定したKaspaアドレスへの入金を監視して、請求の状態を更新するサービスである。

この題材を選ぶ理由は、単純な残高照会よりも実務的でありながら、秘密鍵をサーバーで預からずに構築できるからだ。最初のMVPでは**支払いを送信する機能を作らない**。署名・秘密鍵保管のリスクを避け、読み取りと検証に集中する。

### 完成条件

- 請求を作成し、金額・ネットワーク・受取アドレスを記録できる。
- ノードからアドレス関連のUTXO情報を取得できる。
- 入金を検出し、二重計上しない。
- 同期停止、再接続、再編成（reorganization）を考慮できる。
- DevnetまたはTestnetでE2Eテストを実施できる。
- 設計上の前提と未解決リスクを説明できる。

> **重要：** アドレス残高の増加だけで「どの請求への支払いか」を一意に判断できるとは限らない。MVPでは請求ごとに専用アドレスを割り当てる設計を基本とする。ただし、実際のアドレス発行・鍵管理は採用するウォレット方式の仕様に従う。

## 35.2 なぜKaspaを使うのか

技術選定は「ブロックチェーンを使いたい」から始めない。まず業務上の要求を定義する。

| 要求 | 従来のDBでできること | Kaspaが担うこと |
|---|---|---|
| 請求書を作る | できる | 不要 |
| 支払い状態を管理する | できる | チェーン上の根拠を提供 |
| 受取人が自分で資金を管理する | アプリ内台帳だけでは不足 | UTXOと署名による所有権管理 |
| 第三者が支払いを検証する | DBへの信頼が必要 | 公開チェーンのデータを参照可能 |
| 顧客の個人情報を管理する | できる | オンチェーンに置かない |

**原則：ブロックチェーンは支払いの事実を検証するために使い、請求・顧客・UIの状態は通常のデータベースで管理する。**

## 35.3 スコープと非スコープ

### MVPで実装する

1. 請求の作成（管理者操作）。
2. 請求一覧と詳細の表示。
3. ノード接続とネットワーク種別の検証。
4. アドレスの監視と入金イベントの取り込み。
5. 支払い金額の集計と請求ステータスの更新。
6. 重複イベントの排除と再照合。
7. 障害時の復旧とテスト。

### MVPでは実装しない

- サーバー側での秘密鍵保管・自動送金。
- トークン発行や独自コンセンサス。
- ブリッジ、L2、ZK証明、vProgs。
- 法定通貨換算の確定レートや会計・税務の自動処理。
- メインネット上の実資金を使った無監査運用。

## 35.4 ユースケース

**UC-01：請求発行**

1. 管理者が金額と有効期限を入力する。
2. システムが一意の請求IDを生成する。
3. 受取アドレスを請求に関連付ける。
4. `Pending` 状態で保存する。

**UC-02：入金検出**

1. アドレス監視サービスがノードから更新を受け取る。
2. 取り込んだデータを正規化し、重複を検査する。
3. 請求に紐づく入金候補として記録する。
4. コンセンサス上の受理状態・確認ポリシーに基づいて請求を更新する。

**UC-03：再接続・再照合**

1. ノード接続が切れる。
2. サービスは状態を `Degraded` として記録する。
3. 再接続後に監視を再登録する。
4. 取りこぼした可能性がある範囲を再取得・再照合する。
5. DBの派生状態を再計算する。

## 35.5 C4モデルで全体像を整理する

### Context（システムコンテキスト）

```text
[支払者] ───────▶ [Kaspaネットワーク]
   │                     │
   │ 請求閲覧            │ 入金データ
   ▼                     ▼
[Payment Tracker] ◀── [Kaspaノード]
   ▲
   │ 請求作成・照会
[管理者]
```

### Container（コンテナ）

```text
                 ┌─────────────────────────┐
                 │ Web UI / Admin UI       │
                 └────────────┬────────────┘
                              │ HTTPS
                 ┌────────────▼────────────┐
                 │ Backend API             │
                 │ Invoice Application     │
                 └────────┬────────┬───────┘
                          │        │
                          ▼        ▼
                    ┌────────┐ ┌───────────────────┐
                    │ DB     │ │ Chain Observer    │
                    └────────┘ │ RPC/wRPC Adapter  │
                               └─────────┬─────────┘
                                         │
                                  ┌──────▼──────┐
                                  │ Kaspa Node  │
                                  └─────────────┘
```

### Component（コンポーネント）

- `InvoiceService`：請求の作成・取得・期限管理。
- `AddressAllocator`：請求ごとの受取アドレス割当て。
- `ChainObserver`：ノードからイベントを収集。
- `PaymentReconciler`：チェーン情報とDBの整合性を再計算。
- `ConfirmationPolicy`：どの状態で支払いを確定扱いするか判断。
- `InvoiceRepository`：請求を永続化。
- `ObservedOutputRepository`：観測した出力とその状態を永続化。

**依存関係の方向**は、ドメインがKaspa SDKの具象型に依存しない形を目指す。

```text
HTTP / Scheduler / RPC
         │
         ▼
Application Services
         │
         ▼
Domain Model & Policies
         ▲
         │ interface（trait）
Infrastructure Adapters
```

## 35.6 ドメインモデルを定義する

主要な概念を分ける。

| 概念 | 意味 | 主な責務 |
|---|---|---|
| `Invoice` | 支払い要求 | 金額・期限・状態を管理 |
| `PaymentObservation` | チェーン上で観測した入金候補 | 出力識別子・金額・状態を記録 |
| `AddressAssignment` | 請求と受取アドレスの対応 | アドレスの再利用・衝突を防ぐ |
| `ChainCheckpoint` | 最後に照合した地点 | 復旧の手掛かりを保存 |
| `ConfirmationPolicy` | 受理・確定の業務ルール | 状態遷移を判定 |

### 金額の単位

金額は浮動小数点数ではなく、**sompi単位の整数**で扱う。KAS表示への変換はUIの責務に分ける。

### 請求の状態

```text
Pending ──入金候補──▶ Observed
   │                    │
   │ 期限切れ           │ ポリシー達成
   ▼                    ▼
Expired               Paid
   ▲                    │
   └────再照合が必要────┘（状況によって再評価）
```

この図は業務上の簡略化であり、Kaspaプロトコルの状態機械ではない。`Paid`を不可逆に扱うかどうかは、業務上のファイナリティポリシーを定義して決める。

## 35.7 不変条件を先に書く

- **I1**：請求金額は正の整数である。
- **I2**：請求IDは一意である。
- **I3**：同じチェーン上の出力は、同じ請求に二度加算されない。
- **I4**：異なるネットワークの入金は混同しない。
- **I5**：支払い合計は観測済みの出力の単純なイベント回数ではなく、採用対象の出力集合から再計算する。
- **I6**：ノードが未同期のとき、新規の確定判定を無条件に進めない。
- **I7**：期限切れと支払いの競合を明示的な業務ルールで解決する。
- **I8**：監視処理の再実行によって結果が変わらない（冪等性）。
- **I9**：DB更新に失敗しても、再試行によって復旧できる。
- **I10**：秘密鍵・シードをログやDBに記録しない。

## 35.8 RPC・wRPCを選ぶ

Kaspaアプリケーションは、ノードの情報を取得するためにRPCを利用する。ブラウザやNode.jsからはWASM SDKの利用が選択肢になり、RustバックエンドではネイティブクレートやwRPCクライアントが候補になる。

| アクセス方式 | 主な用途 | 留意点 |
|---|---|---|
| gRPC | バックエンド・ノード連携 | 接続設定と認証・ネットワーク境界 |
| wRPC（WebSocket） | イベント購読・Webクライアント | 再接続・購読再登録・通知欠落 |
| コミュニティREST API | 試作・閲覧用途 | 可用性・制限・SLAを保証できない場合がある |

`rusty-kaspa`のREADMEではwRPCはノード側で明示的に有効化する構成が説明されている。フラグ名やデフォルトポートは、実際に使うリリースの `--help` とドキュメントで確認する。

### まず読み取りだけで接続を確認する

1. ノードのサーバー情報を取得する。
2. 接続先ネットワークを確認する。
3. DAG情報を取得する。
4. 同期状態を確認する。
5. イベント購読を開始する。
6. 切断・再接続時の挙動を記録する。

公式の最小Rustクライアント例：

- https://github.com/kaspanet/rusty-kaspa/blob/master/rpc/wrpc/examples/simple_client/src/main.rs

SDKの具体的な関数名・引数はこの例を起点に、利用するコミットに固定して確認する。

## 35.9 データベース設計

学習用のリレーショナルモデルを考える。

```sql
CREATE TABLE invoices (
    id TEXT PRIMARY KEY,
    network TEXT NOT NULL,
    address TEXT NOT NULL,
    amount_sompi BIGINT NOT NULL CHECK (amount_sompi > 0),
    status TEXT NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL
);

CREATE TABLE observed_outputs (
    network TEXT NOT NULL,
    transaction_id TEXT NOT NULL,
    output_index INTEGER NOT NULL,
    invoice_id TEXT NOT NULL REFERENCES invoices(id),
    amount_sompi BIGINT NOT NULL CHECK (amount_sompi >= 0),
    observation_state TEXT NOT NULL,
    PRIMARY KEY (network, transaction_id, output_index)
);
```

> 注意：SQLは概念モデル。`BIGINT`の上限、時刻のタイムゾーン、請求アドレスの一意性、チェーン再編成時の更新、監査ログ、実際のDB方言は本番設計で詰める。

`transaction_id + output_index` はUTXOの出力を識別する基本的な組であり、ネットワークを含めて保存する。**入金候補として見えたこと**と、**コンセンサス上で採用されていること**は別なので、`observation_state` を単なる真偽値にしない。

## 35.10 イベント駆動だけでは足りない

イベント通知は「変化を早く知る」手段であり、永続的な真実の唯一の根拠にしない。

次の障害を考える。

- 通信断の間に通知が発生する。
- 同じ通知が複数回来る。
- アプリケーションが通知処理中に落ちる。
- ノードの同期が巻き戻る・再編成が発生する。
- UTXOが消費され、単純な現在残高照会では過去の支払いを再現できない。

そのため、設計を**通知の取り込み + 定期的な再照合**の二層に分ける。

```text
Chain Notification ──▶ Fast Path ──▶ Candidate Store
                                           │
Periodic Reconciliation ──────────────────┤
                                           ▼
                                  Payment Projection
                                           │
                                           ▼
                                     Invoice Status
```

再照合に必要な履歴情報は、ノードのUTXOインデックスだけでは不足する場合がある。過去のトランザクションや採用状況を追跡する要件があるなら、**履歴インデクサーの導入**や自前のイベント永続化を検討する。

## 35.11 Rustでドメインロジックを実装する

ここからのコードは、Kaspa SDKに依存しない**支払い集計の教育用モデル**。実際のチェーン採用判定・ファイナリティ・署名検証は行わない。

`cargo new payment-domain --lib` で作成した `src/lib.rs` を次のように置き換える。

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OutputId {
    pub tx: [u8; 32],
    pub index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationState {
    Candidate,
    Accepted,
    Retracted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Observation {
    pub id: OutputId,
    pub amount_sompi: u64,
    pub state: ObservationState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvoiceStatus {
    Pending,
    Observed,
    Paid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentError {
    ZeroInvoiceAmount,
    ConflictingObservation,
    AmountOverflow,
}

#[derive(Debug)]
pub struct Invoice {
    amount_sompi: u64,
    outputs: HashMap<OutputId, Observation>,
}

impl Invoice {
    pub fn new(amount_sompi: u64) -> Result<Self, PaymentError> {
        if amount_sompi == 0 {
            return Err(PaymentError::ZeroInvoiceAmount);
        }
        Ok(Self {
            amount_sompi,
            outputs: HashMap::new(),
        })
    }

    // 同じ出力の同じ状態は何度処理しても結果が変わらない。
    // 同じ出力IDに異なる金額が届いた場合はデータ不整合として拒否する。
    pub fn observe(&mut self, observation: Observation) -> Result<(), PaymentError> {
        if let Some(existing) = self.outputs.get(&observation.id) {
            if existing.amount_sompi != observation.amount_sompi {
                return Err(PaymentError::ConflictingObservation);
            }
        }
        self.outputs.insert(observation.id, observation);
        Ok(())
    }

    pub fn accepted_total(&self) -> Result<u64, PaymentError> {
        self.outputs
            .values()
            .filter(|o| o.state == ObservationState::Accepted)
            .try_fold(0_u64, |total, o| {
                total.checked_add(o.amount_sompi)
                    .ok_or(PaymentError::AmountOverflow)
            })
    }

    pub fn status(&self) -> Result<InvoiceStatus, PaymentError> {
        if self.accepted_total()? >= self.amount_sompi {
            Ok(InvoiceStatus::Paid)
        } else if self.outputs.values().any(|o| {
            o.state != ObservationState::Retracted
        }) {
            Ok(InvoiceStatus::Observed)
        } else {
            Ok(InvoiceStatus::Pending)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(state: ObservationState, amount: u64) -> Observation {
        Observation {
            id: OutputId { tx: [1; 32], index: 0 },
            amount_sompi: amount,
            state,
        }
    }

    #[test]
    fn zero_invoice_amount_is_rejected() {
        assert!(matches!(Invoice::new(0), Err(PaymentError::ZeroInvoiceAmount)));
    }

    #[test]
    fn duplicate_notification_is_idempotent() {
        let mut invoice = Invoice::new(100).unwrap();
        let output = sample(ObservationState::Accepted, 100);
        invoice.observe(output).unwrap();
        invoice.observe(output).unwrap();
        assert_eq!(invoice.accepted_total().unwrap(), 100);
        assert_eq!(invoice.status().unwrap(), InvoiceStatus::Paid);
    }

    #[test]
    fn candidate_is_not_paid() {
        let mut invoice = Invoice::new(100).unwrap();
        invoice.observe(sample(ObservationState::Candidate, 100)).unwrap();
        assert_eq!(invoice.status().unwrap(), InvoiceStatus::Observed);
    }

    #[test]
    fn retraction_recalculates_status() {
        let mut invoice = Invoice::new(100).unwrap();
        invoice.observe(sample(ObservationState::Accepted, 100)).unwrap();
        assert_eq!(invoice.status().unwrap(), InvoiceStatus::Paid);
        invoice.observe(sample(ObservationState::Retracted, 100)).unwrap();
        assert_eq!(invoice.status().unwrap(), InvoiceStatus::Pending);
    }

    #[test]
    fn conflicting_amount_is_rejected() {
        let mut invoice = Invoice::new(100).unwrap();
        invoice.observe(sample(ObservationState::Accepted, 100)).unwrap();
        let error = invoice.observe(sample(ObservationState::Accepted, 200));
        assert_eq!(error, Err(PaymentError::ConflictingObservation));
        assert_eq!(invoice.accepted_total().unwrap(), 100);
    }

    #[test]
    fn amount_overflow_is_detected() {
        let mut invoice = Invoice::new(1).unwrap();
        invoice.observe(sample(ObservationState::Accepted, u64::MAX)).unwrap();
        invoice.observe(Observation {
            id: OutputId { tx: [2; 32], index: 0 },
            amount_sompi: 1,
            state: ObservationState::Accepted,
        }).unwrap();
        assert_eq!(invoice.accepted_total(), Err(PaymentError::AmountOverflow));
    }
}
```

実行：

```bash
cargo test
```

### このコードで保証していること・していないこと

**保証を意図していること**：同じ出力の重複加算防止、状態変更後の再集計、金額不整合の検出、整数オーバーフローの検出。

**保証していないこと**：出力が本当にKaspaチェーン上に存在すること、対象アドレスへの支払いであること、受理・確定の正しさ、ネットワーク分離、永続化、複数ワーカーの競合制御。これらはアダプターとアプリケーション層の責務になる。

## 35.12 外部依存をtraitで分離する

ドメインロジックをKaspa SDKやデータベースから切り離す。

```rust
#[derive(Debug, Clone)]
pub struct ChainHealth {
    pub connected: bool,
    pub synced: bool,
    pub network: String,
}

pub trait ChainReader {
    type Error;

    fn health(&self) -> Result<ChainHealth, Self::Error>;
    fn observations_for_address(
        &self,
        address: &str,
    ) -> Result<Vec<Observation>, Self::Error>;
}
```

これは設計を示すインターフェースであり、**Kaspa RPCの実際の型や同期APIを模したものではない**。実装では非同期処理、購読、タイムアウト、キャンセル、接続プール、リトライを考慮する。

### なぜ分けるのか

- Kaspa SDKを更新してもドメインルールは変更しなくてよい。
- Mockを使ってネットワークなしで単体テストできる。
- gRPCからwRPCに切り替える際の変更範囲を限定できる。
- チェーン由来の状態とアプリ独自の状態を混同しにくい。

## 35.13 トランザクションと整合性

RPCで入金を受け取ってからDBに保存するまでの間に、アプリがクラッシュすることがある。

```text
RPC Event Received
       │
       ▼
Validate / Normalize
       │
       ▼
BEGIN DB TRANSACTION
       │
       ├── Upsert Observed Output
       ├── Recompute Invoice Projection
       └── Persist Checkpoint
       │
       ▼
COMMIT
```

`checkpoint` を先に進めてから観測結果の保存に失敗すると、データ欠落につながる。**観測結果と進捗の整合性を同一トランザクションで保つ**設計を基本にする。

複数ワーカーを使う場合、同一請求への並行更新をDB制約・ロック・バージョン番号などで制御する。

## 35.14 確認ポリシーを設計する

「何回確認されたら支払い完了か」を固定の数字だけで決めない。KaspaはBlockDAGであり、Bitcoinの単純なブロック数と同一視しない。

検討事項：

1. 何を「観測」と呼ぶか。
2. コンセンサスで受理されたことをどう確認するか。
3. 業務上のリスクに応じてどのような確定条件を採用するか。
4. 再編成・接続先ノード不一致をどう扱うか。
5. 返金や過払いをどう処理するか。

`ConfirmationPolicy` はネットワークのセキュリティを証明するものではなく、**リスク許容度に応じた業務ルール**である。

## 35.15 エラー設計

| 種類 | 例 | 推奨処理 |
|---|---|---|
| 入力エラー | 金額が0 | 即時拒否 |
| 設定エラー | ネットワーク不一致 | 起動・処理を停止 |
| 一時障害 | RPCタイムアウト | 上限付き再試行 |
| 継続障害 | ノード未同期 | `Degraded` に遷移 |
| 整合性エラー | 同一出力IDで金額不一致 | 隔離・調査 |
| 業務競合 | 期限切れ直後の入金 | 明示的ルールで判定 |
| セキュリティ | 不正な認証要求 | 拒否・監査ログ |

再試行では指数バックオフとジッターを利用し、無制限にRPCへ負荷をかけない。

## 35.16 テスト戦略

### レベル別

| テスト | 対象 | 代表例 |
|---|---|---|
| Unit | Domain | 重複入金、過払い、オーバーフロー |
| Contract | RPC Adapter | ネットワーク情報の変換、通知の解釈 |
| Integration | API + DB | 原子的更新、再起動、同時更新 |
| E2E | Testnet/Devnet | 実際の入金、同期、再接続 |
| Fault Injection | 運用経路 | 通信断、遅延、DB停止 |

### 必須の異常系

- 通知を100回重複配送しても入金合計が変わらない。
- イベント受信直後にプロセスが落ちても復旧できる。
- 同期していないノードの値を確定扱いしない。
- 2つのノードが異なる状態を報告したときに検知できる。
- 再編成で採用状態が変われば派生ステータスを再評価できる。
- Testnetのアドレス・トランザクションをMainnet請求に混ぜない。
- 一部入金・過払い・期限切れ後の入金を扱える。

## 35.17 Devnetでの検証手順

1. `rusty-kaspa`の利用バージョンを固定する。
2. Devnetノードを起動する（フラグは対象リリースで確認）。
3. RPC接続とネットワーク種別を確認する。
4. テスト用ウォレットを作成する。シードは開発環境外へ持ち出さない。
5. テスト用KASを用意する。ネットワークによって入手方法は異なる。
6. 請求を作成し、受取アドレスを表示する。
7. 送金してイベントとDBの変化を記録する。
8. アプリを再起動し、再照合する。
9. ノードを停止・再起動して障害復旧を確認する。
10. 期待結果と実測結果を実験ログに残す。

本章では未確認のRPCコマンドを創作しない。**実行可能なノード起動・送金コマンドは使用バージョンの公式ドキュメントから確定させる**。

## 35.18 観測可能性（Observability）

最低限、以下のメトリクスを用意する。

- RPC接続状態、接続先ノード、ネットワーク種別。
- ノードの同期状態。
- 最後の通知受信時刻。
- 最後の再照合成功時刻。
- 再照合にかかった時間。
- 未処理イベント数、再試行回数。
- 請求ステータスの変更回数。
- 重複通知数と不整合検知数。

ログには`invoice_id`、`network`、`tx_id`、`output_index`、`correlation_id`を必要に応じて記録する。ただし、秘密鍵、シード、個人情報は記録しない。

## 35.19 セキュリティレビュー

**信頼境界**を整理する。

```text
[Internet / User]
       │ Untrusted Input
       ▼
[Web/API] ── AuthN/AuthZ ──▶ [Application]
                                   │
                             Validated Data
                                   ▼
                              [Database]
                                   ▲
                                   │
                          [RPC Adapter]
                                   ▲
                                   │ External Data
                              [Kaspa Node]
```

確認事項：

- API認証と管理者権限を分ける。
- RPCを公開インターネットに無制限公開しない。
- 入力値とネットワーク種別を検証する。
- 公開RPCの可用性・信頼性を過信しない。
- 依存ライブラリの更新・脆弱性確認を行う。
- バックアップからの復元をテストする。
- 監視アドレスの割当て・再利用ルールを明文化する。
- 秘密鍵を扱う機能を追加する際は、別途脅威モデルと監査を実施する。

## 35.20 GitHub ActionsでCIを構築する

Rustドメイン層の最小CI例：

```yaml
name: Rust CI
on:
  push:
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test --all-targets
```

これは**Rustクレートをリポジトリルートに配置した場合の例**。`examples/payment-domain/` に置く場合は`working-directory`などを調整する。GitHub Actionsの利用するアクションのバージョンも継続的に見直す。

## 35.21 READMEと設計書を成果物に含める

アプリケーションの価値は「コードが動く」だけでは測れない。他人が再現できることが重要である。

推奨するディレクトリ：

```text
examples/payment-tracker/
├── README.md
├── Cargo.toml
├── src/
│   ├── domain/
│   ├── application/
│   ├── adapters/
│   └── main.rs
├── migrations/
├── tests/
│   ├── integration/
│   └── fixtures/
└── docs/
    ├── requirements.md
    ├── c4.md
    ├── state-machine.md
    ├── invariants.md
    ├── threat-model.md
    └── experiments.md
```

READMEには「目的」「対象ネットワーク」「実行手順」「テスト手順」「未実装機能」「セキュリティ上の注意」「検証済みバージョン」を必ず記載する。

## 35.22 開発の順序：縦に小さく作る

最初から全機能を並列に実装しない。

| ステップ | 実装 | 完了条件 |
|---|---|---|
| 1 | ドメインモデル | Unit Testが通る |
| 2 | Mock ChainReader | ノードなしで入金を模擬できる |
| 3 | DB永続化 | 再起動後も同じ結果になる |
| 4 | RPC読み取り | 正しいネットワーク・同期状態を取得できる |
| 5 | イベント購読 | 入金候補を取り込める |
| 6 | 再照合 | 重複・欠落から復旧できる |
| 7 | UI/API | 請求の作成・閲覧ができる |
| 8 | E2E | Devnetで一連の流れを再現できる |
| 9 | 障害注入 | 切断・再起動から復旧できる |

この順序なら、各段階で**何が動き、何が未検証なのか**を説明できる。

## 35.23 失敗から学ぶ：よくある設計ミス

### 失敗1：イベント通知を真実の唯一の根拠にする

**問題**：通知欠落で支払いが記録されない。  
**改善**：再照合・チェックポイント・履歴の保持を設計する。

### 失敗2：UTXOの現在残高だけで請求を判定する

**問題**：UTXOが後で消費されると、過去の入金を説明できなくなる。  
**改善**：支払い観測履歴と採用状態を永続化する。

### 失敗3：SDKの型をドメイン全体に広げる

**問題**：SDK更新が業務ルールまで波及する。  
**改善**：Adapterで変換し、ドメインの型を保つ。

### 失敗4：MainnetとTestnetを混同する

**問題**：異なるネットワークのデータを同じ支払いとして扱う。  
**改善**：ネットワークIDを設定・DB・イベント・テストに含める。

### 失敗5：動作デモだけで完成と判断する

**問題**：再起動や障害で状態が壊れる。  
**改善**：再現可能な統合テストと障害注入を実施する。

## 35.24 発展課題

### 課題A：監視ダッシュボード

請求数、未処理数、RPC遅延、ノード同期状態を可視化する。

### 課題B：Webhook通知

`Paid`への遷移時にWebhookを送る。署名付きWebhook、再送、冪等キー、配信履歴を実装する。

### 課題C：マルチノード照合

複数ノードの報告が食い違った場合の検知と保留ポリシーを実装する。

### 課題D：ウォレット連携

ユーザー自身がウォレットで署名・送金する設計を追加する。サーバー側の鍵保管を避け、SDKの署名フローを利用する。

### 課題E：Programmabilityへの拡張

Covenantなどを使う必要があるユースケースを定義し、通常の支払い監視だけでは解決できない課題が何かを説明する。機能のメインネット対応状況と監査状況を必ず確認する。

## 35.25 理解度チェック

1. 請求データをKaspa L1に保存しないのはなぜか。
2. `transaction_id + output_index` を記録する理由は何か。
3. 通知の重複と欠落は、なぜ別々に対策する必要があるのか。
4. 「観測」「受理」「業務上の確定」はどう違うか。
5. ドメイン層をKaspa SDKから切り離すメリットは何か。
6. 現在のUTXO集合だけでは過去の入金履歴を復元できない場合があるのはなぜか。
7. 再編成が起きたとき、請求状態をどう再評価するか。
8. ノードが未同期の場合、どの処理を停止・保留すべきか。
9. 単体テストとDevnet E2Eテストは何をそれぞれ保証するか。
10. 他人がアプリを再現するために、READMEへ何を書くべきか。

## 35.26 まとめ

Kaspaアプリケーションを作るうえで重要なのは、チェーンの仕組みを知ることだけではない。**チェーンが保証することと、アプリケーションが責任を負うことを分ける設計**が必要になる。

本章では、支払い監視アプリを題材に次を学んだ。

- 要件定義から始め、Kaspaを使う理由を説明する。
- C4モデル・DDD・不変条件・状態機械で責務を整理する。
- RPCのイベント購読と再照合を組み合わせる。
- UTXOと業務上の支払い状態を区別する。
- Rustの型とテストでドメインの正しさを確認する。
- Devnet・障害注入・CI・ドキュメントを成果物に含める。

次章では、こうした実装・検証・ドキュメントの経験を、Kaspaへのオープンソース貢献につなげる。

## 35.27 参考資料・一次資料

- Kaspa公式開発者向けページ：https://kaspa.org/build
- Kaspa開発ドキュメント：https://docs.kaspa.org/
- Kaspa Getting Started：https://docs.kaspa.org/integrate/getting-started
- rusty-kaspa公式リポジトリ：https://github.com/kaspanet/rusty-kaspa
- Rust wRPCクライアント例：https://github.com/kaspanet/rusty-kaspa/blob/master/rpc/wrpc/examples/simple_client/src/main.rs
- Rusty Kaspa WASM SDK概要：https://github.com/kaspanet/rusty-kaspa/blob/master/wasm/README.md
- Kaspa改善提案（KIPs）：https://github.com/kaspanet/kips
- mdBook：https://rust-lang.github.io/mdBook/

> **検証メモ（2026-10-30）**：公式ドキュメントと`rusty-kaspa`の公開サンプルを参照して構成を整理した。Rustサンプルは教育用の独立したドメイン実装であり、実際のKaspaネットワークに接続する完成アプリではない。ノード・SDK・RPCの具体的な実装と本番運用条件は、利用時点のリリースに合わせて再検証する。
