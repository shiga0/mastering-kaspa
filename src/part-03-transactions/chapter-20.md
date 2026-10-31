# Chapter 20 — Kaspa Network Security & Finality（Kaspaのネットワークセキュリティとファイナリティ）

> **Part III — Transactions, Mining & Security**  
> 原稿バージョン：v0.2（基準版のFinality主要経路を照合・章全体はレビュー継続中）\
> 対象読者：Rust・バックエンド開発の経験があり、Kaspaの安全性を設計・実装の観点から理解したい読者

## この章で学ぶこと

- **Safety（安全性）**、**Liveness（活性）**、**Finality（確定性）**を区別する。
- GHOSTDAGの順序付けとUTXO状態の確定がどう関係するかを理解する。
- 二重支払い、ネットワーク分断、Eclipse攻撃、ハッシュパワー集中などの脅威を整理する。
- **アプリケーションの入金確認**と**ノードのfinalityルール**を混同しない。
- Rustで簡単なリスク評価・状態遷移モデルを作り、テストできるようにする。

> **重要**：本章はセキュリティモデルを説明する教育用原稿であり、取引所や決済事業者向けの「安全な承認秒数」を保証するものではない。コンセンサスの定数やRPC仕様はネットワーク・バージョン・アップグレードによって変化する。実運用では、利用するノードのリリースとソースコードを確認すること。

---

## 20.1 なぜ確定性が重要なのか

送金を受け取った店舗を考えよう。ウォレットからトランザクションが送信された直後、店舗のノードがそのトランザクションをMempoolで見つけたとする。しかし、この時点で商品を引き渡してよいだろうか。

答えは、トランザクション額・商品特性・相手への信頼・攻撃可能性・ネットワーク状況に依存する。**観測されたこと**と**コンセンサスで受理されたこと**、さらに**後から覆る可能性が十分小さいこと**は異なる。

```text
ウォレットで署名
      │
      ▼
P2Pで伝播 ──────→ Mempoolで観測
      │                 │
      ▼                 └─ まだコンセンサス確定ではない
BlockDAGへ収録
      │
      ▼
GHOSTDAGによる順序付け
      │
      ▼
Virtual UTXO Stateで受理
      │
      ▼
後続のPoWが蓄積・ネットワークが収束
      │
      ▼
アプリケーションがリスクに応じて決済完了と判断
```

ここでいう「決済完了」はアプリケーションの判断であり、プロトコルがすべてのトランザクションに対して絶対的・即時の不可逆性を保証するという意味ではない。

## 20.2 用語を厳密に分ける

| 概念 | 意味 | 混同しやすいもの |
|---|---|---|
| **Validity** | ブロックやトランザクションがプロトコルのルールを満たすこと | トランザクションが最終的に受理されること |
| **Acceptance** | コンセンサス順序とUTXO状態の下でトランザクションが受理されること | Mempoolへの登録 |
| **Safety** | 正直なノード同士が矛盾する確定済み結果を採用しないという性質 | 伝播が速いこと |
| **Liveness** | 適切な条件下で新しいトランザクション・ブロックの処理が進み続ける性質 | 常に即時確定すること |
| **Convergence** | 異なるノードの見解が時間とともに整合すること | 一時的に完全一致すること |
| **Finality** | 一定の条件下で履歴を覆せない／覆しにくいとみなす性質 | 確認数だけで一意に決まること |
| **Confirmation** | トランザクションが受理された後、後続のコンセンサス進行によって信頼度が増すこと | finalityルールそのもの |

PoW型システムの経済的確定性は、一般に攻撃者の能力やネットワーク条件に依存する。したがって「絶対に覆らない」という表現は避ける。

## 20.3 脅威モデル：何を守り、何を仮定するのか

セキュリティの議論は、まず攻撃者の能力を明示するところから始まる。

**守りたい資産**：UTXOの一貫性、トランザクションの受理結果、正しいコンセンサス順序、ノードの可用性、ウォレット秘密鍵、決済サービスの資産。

**攻撃者の能力の例**：

1. 自分のノードを多数起動し、任意のP2Pメッセージを送る。
2. 一部のネットワーク経路を遅延・遮断する。
3. 競合するトランザクションを生成する。
4. 一定量のPoWハッシュパワーを持つ。
5. 特定の被害者ノードの接続先を偏らせる。

**攻撃者に与えない能力の例**：他人の秘密鍵を任意に復元する能力、暗号学的ハッシュを自由に破る能力。これらまで許すなら、コンセンサス以前の安全性が崩れる。

> **設計原則**：P2Pから届くデータは信頼しない。ノードはブロック・取引・親関係・PoW・状態遷移をローカルで検証する。

## 20.4 Kaspaでコンセンサスが担う責務

Kaspaはブロックを一本のチェーンに限定せず、**BlockDAG**として並行ブロックを保持する。GHOSTDAGはDAGに対してselected parentやblue/red分類などを用い、コンセンサス順序の構築を支える。

```text
  BlockDAG（構造）
       │
       ▼
  GHOSTDAG（順序決定の基礎）
       │
       ▼
  Consensus Ordering
       │
       ▼
  Transaction Acceptance
       │
       ▼
  Virtual UTXO State
```

注意点は次のとおり。

- **blue = 正しいトランザクション、red = 不正なトランザクション**ではない。blue/redはDAG上のコンセンサス分類である。
- **ブロック自体が有効**でも、競合するUTXO消費によって**その中のトランザクションが受理されない**場合がある。
- トランザクションの受理状態は、virtual stateの更新・再編成に伴って変わり得る。
- ノードが同じDAGをまだ見ていなければ、一時的に異なるvirtual viewを持つことがある。

詳細な順序付けは第12章、virtual stateは第14章、UTXOの状態遷移は第15章で扱った。

## 20.5 二重支払い攻撃の流れ

二重支払いは、同一のUTXOを消費する互いに両立しないトランザクションを利用する。

```text
            UTXO X
             /  \
            /    \
       Tx-A        Tx-B
    店舗へ支払う   攻撃者へ戻す
            \    /
             \  /
      コンセンサス順序と
      UTXO状態で競合を解決
```

仮にTx-Aが先に受理されれば、同じUTXOを使うTx-Bは同じ状態では受理できない。しかし、後続のDAG進行や再編成によって、最終的な順序・受理結果の見解が変化するリスクを評価しなければならない。

**重要な不変条件**：ある一つの整合したUTXO状態において、同じOutpointを二度消費してはならない。

## 20.6 PoWと攻撃コスト

PoWは「ハッシュ計算を大量に実施した」という検証可能な証拠を与える。攻撃者が履歴の見解を変えようとする場合、単なる偽メッセージ送信だけでは足りず、コンセンサスのルールに従うPoWが問題になる。

ただし「攻撃にはネットワーク全体の51%以上が必ず必要」という単純化は不正確。攻撃成功確率や必要資源は、攻撃戦略、観測時間、ネットワーク遅延、ノード分断、プロトコルの安全性条件に依存する。

ハッシュレートの大小だけでなく、**実効的にどのノードが何を観測できるか**も重要である。

## 20.7 ネットワーク遅延と一時的な不一致

P2Pネットワークではブロック伝播に時間がかかる。同じ時刻に近いタイミングで異なるマイナーがブロックを生成すると、各ノードが見るDAGのtip集合は一時的に異なる。

```text
時刻 t0：Node A が Block P を受信
時刻 t1：Node B が Block Q を受信
時刻 t2：A と B が P/Q を相互に伝播
時刻 t3：両ノードが追加の検証・順序更新を実施
```

並行ブロックが存在すること自体はBlockDAGの想定内である。問題は、遅延・分断・攻撃によって**収束に必要な情報が届かない**、あるいは**異なる履歴の競争が長引く**ことである。

## 20.8 SafetyとLivenessのトレードオフ

分散システムでの安全性と活性は、別の評価軸である。

| 状況 | Safetyへの懸念 | Livenessへの懸念 |
|---|---|---|
| 一時的な通信遅延 | 観測差・順序変更 | トランザクション受理が遅れる |
| 長期ネットワーク分断 | 分断間で異なる履歴が進行 | 一部ノードが十分な情報を得られない |
| 大量の悪意あるピア | Eclipse・誤情報の注入 | 接続枠や処理資源の枯渇 |
| ハッシュパワー集中 | 履歴操作の能力集中 | 検閲・トランザクション遅延 |
| ノードの実装不一致 | 異なるコンセンサス判断 | 同期停止・拒否 |

安全性と活性の評価には、攻撃者の資源上限とネットワークの同期仮定を合わせて示す必要がある。

## 20.9 KaspaのFinalityルールと実用上の確定性

**「finality」という語には少なくとも二つの文脈がある。**

### 20.9.1 コンセンサス実装上のfinality

[本書の基準版](../baseline.md)v1.0.1では、`BlockDepthManager::calc_finality_point` がblue scoreによる深さとselected parent chainを使ってfinality pointを求める。適用する `finality_depth` は、selected parentのDAA scoreに応じたフォーク設定から取得する。DAA scoreをそのまま深さとして引く計算ではない。

| 対象 | 固定ソースで確認した値・扱い |
|---|---|
| 基準版Devnetの既定設定 | Crescendo無効、prior finality depthは86400、prior pruning depthは185798 |
| Crescendo用定数 | `NEW_FINALITY_DURATION = 43200`秒（12時間）、`NEW_PRUNING_DURATION = 108000`秒（30時間） |
| Crescendo用depth導出 | finality depthはBPS×duration。pruning depthはBPS×durationと安全性の下限式の大きい方 |

12時間・30時間は、基準版Devnetにそのまま適用する待ち時間ではない。depthは実装上の進行尺度で、一定の実時間で到達する保証もない。現在のMainnetの有効設定をこの固定ソースの確認だけで断定しない。

VirtualProcessorはsink候補にfinality pointがselected parent chain上の祖先であることを要求し、UTXO状態も評価する。finalityに違反する候補はVirtual chainから除外する経路を持つ。違反候補が存在したことを、すべて同じ `StatusInvalid` へ記録する説明にはしない。IBD開始時にはpruning pointをvirtual finality pointとして使う分岐もある。

根拠：[block_depth.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/block_depth.rs)、[virtual_processor/processor.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/virtual_processor/processor.rs)、[params.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/params.rs)、[constants.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/constants.rs)、[bps.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/bps.rs)。

Pruning point一覧を検査する `are_pruning_points_violating_finality` は、既知のpruning pointとfinality pointをさらに遡った点の関係を確認する。ソースは浅い違反を見逃す可能性も注記しており、この一関数があらゆる履歴改変を拒否するとは説明しない。

### 20.9.2 アプリケーション上の決済確定

取引所・決済サービスが「入金を確定」と扱うまでの待機条件は、独自の**リスクポリシー**である。確認の深さ、トランザクション金額、ネットワーク健全性、ノードの同期状態、攻撃検知などを組み合わせる。

両者は関連するが同一ではない。特に**finality_depthをそのまま入金確認数に置き換えてはいけない**。

## 20.10 Pruning・Finality・ノード同期

Kaspaのような継続的に成長するDAGでは、全履歴を永遠に保持しないノード運用が重要になる。Pruningは過去のデータを削減する仕組みで、finalityと密接に関係する。

```text
過去のDAG履歴
      │
      ├─ Finalityに関する境界
      │
      ├─ Pruning point / proof
      │
      ▼
現在の検証・同期に必要な情報
```

ただし、Pruningは「過去のブロックがすべて不要になった」という意味ではない。新規ノードが信頼できる状態に同期するための証明、現在のUTXO状態、必要なヘッダーなどを適切に扱う必要がある。

このテーマは第21章で詳しく扱う。

## 20.11 Eclipse攻撃・Sybil攻撃・DoS

### Eclipse攻撃

被害者ノードの接続先を攻撃者が支配し、正しいネットワーク情報を届きにくくする。被害者が観測するDAGやトランザクションの見解を偏らせる可能性がある。

### Sybil攻撃

攻撃者が多数の論理的ID・ノードを作り、P2P上で存在感を増す。**多数のピアIDがあることはPoW多数派であることを意味しない**が、接続管理・情報伝播・DoSには影響する。

### DoS攻撃

不正メッセージ、大量のトランザクション、巨大なペイロード、計算量の大きい入力によってノードのCPU・メモリ・帯域を消費させる。

主な防御は、メッセージサイズ上限、入力の早期検証、ピアの多様化、レート制限、タイムアウト、リソース制限、異常監視である。これらは実装のバージョンによって変わる。

## 20.12 マイニング集中化・検閲・プール

PoWネットワークでは、ブロック生成に影響を与えるのはピア数ではなく、実際に投入される計算資源である。

マイニングプールが集中すると、プール運営者がブロックテンプレートの作成やトランザクション選択を事実上支配する可能性がある。一方、プール参加者が別のプールへ移動できることや、個別マイナーがテンプレート作成に関与できる設計かどうかも評価要素になる。

**検閲耐性**は、署名検証や二重支払い防止とは別のセキュリティ目標である。

## 20.13 アプリケーションのリスクベース入金確認

バックエンド開発者が実装するべきなのは、「Mempoolで見えたら即確定」とする処理ではなく、**状態と根拠を明示した入金処理**である。

| 状態 | 意味 | 業務上の扱い（例） |
|---|---|---|
| `observed` | トランザクションを観測した | 画面に「検出」と表示 |
| `accepted` | 現在のコンセンサスviewで受理 | 仮反映、確認待ち |
| `confirming` | 後続のコンセンサス進行を観測 | 金額別ポリシーで待機 |
| `settled` | 自社ポリシーを満たした | 商品提供・残高反映 |
| `reverted` | 再編成等で受理が覆った | 仮反映を取り消す |
| `investigation` | 同期不良・競合・異常検知 | 自動確定を停止 |

この状態機械は**業務アプリケーションの例**であり、KaspaのRPCが同名の状態を返すという意味ではない。

```text
observed → accepted → confirming → settled
   │          │            │
   └──────────┴────────────┴──→ investigation
              │
              └───────────────→ reverted
```

**不変条件の例**：

1. 同じトランザクションIDに対して入金残高を二重計上しない。
2. ノードが同期していないときは自動確定しない。
3. `settled`へ移る判断には、評価時点の根拠を記録する。
4. `accepted`から`reverted`への変更を扱える。
5. 外部APIの再送・順序入れ替わりに対して冪等性を保つ。

## 20.14 Rustによる簡易確認ポリシー

以下は**教育用の業務ロジック**であり、Kaspaの実際のfinality判定や攻撃成功確率を計算しない。コンセンサス状態を外部から正しく取得できることを前提に、アプリケーションが条件を組み合わせる例である。

`Cargo.toml`は追加依存なしでよい。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentState {
    Observed,
    Accepted,
    Confirming,
    Settled,
    Investigation,
}

#[derive(Debug, Clone, Copy)]
pub struct Evidence {
    pub node_synced: bool,
    pub accepted_in_virtual_state: bool,
    pub confirmations: u64,
    pub conflicting_tx_seen: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Policy {
    pub min_confirmations: u64,
}

pub fn evaluate(evidence: Evidence, policy: Policy) -> PaymentState {
    if !evidence.node_synced || evidence.conflicting_tx_seen {
        return PaymentState::Investigation;
    }
    if !evidence.accepted_in_virtual_state {
        return PaymentState::Observed;
    }
    if evidence.confirmations == 0 {
        return PaymentState::Accepted;
    }
    if evidence.confirmations < policy.min_confirmations {
        return PaymentState::Confirming;
    }
    PaymentState::Settled
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Evidence {
        Evidence {
            node_synced: true,
            accepted_in_virtual_state: true,
            confirmations: 5,
            conflicting_tx_seen: false,
        }
    }

    #[test]
    fn enough_confirmations_can_settle() {
        assert_eq!(
            evaluate(base(), Policy { min_confirmations: 3 }),
            PaymentState::Settled
        );
    }

    #[test]
    fn unsynced_node_never_settles() {
        let evidence = Evidence { node_synced: false, ..base() };
        assert_eq!(
            evaluate(evidence, Policy { min_confirmations: 3 }),
            PaymentState::Investigation
        );
    }

    #[test]
    fn conflict_requires_investigation() {
        let evidence = Evidence { conflicting_tx_seen: true, ..base() };
        assert_eq!(
            evaluate(evidence, Policy { min_confirmations: 3 }),
            PaymentState::Investigation
        );
    }

    #[test]
    fn not_accepted_is_only_observed() {
        let evidence = Evidence {
            accepted_in_virtual_state: false,
            ..base()
        };
        assert_eq!(
            evaluate(evidence, Policy { min_confirmations: 3 }),
            PaymentState::Observed
        );
    }
}
```

実行例：

```bash
cargo new payment-policy --lib
cd payment-policy
# 上記コードを src/lib.rs に保存
cargo test
```

**注意**：この例の`confirmations`は外部から与えられる抽象的な指標であり、Kaspaで実際にどのRPC値を採用し、どのように計算するかは別途仕様確認が必要。`min_confirmations: 3`はテストの便宜上の値で、安全な運用推奨値ではない。

## 20.15 セキュリティテストの設計

正常系だけでなく、故障や攻撃を再現する。

| テスト | 入力・状況 | 確認したい性質 |
|---|---|---|
| 重複通知 | 同じトランザクションイベントを2回送る | 二重計上しない |
| 順序逆転 | 古い通知が後から届く | 状態が不正に巻き戻らない |
| 再編成 | 受理済みトランザクションが現在のviewから外れる | 仮反映を訂正できる |
| ノード未同期 | IBD中に入金を観測 | 自動確定しない |
| ネットワーク分断 | 一部ピアが切断 | 健全性を監視できる |
| 競合トランザクション | 同じUTXOを使う2トランザクション | 状態を矛盾なく処理 |
| 高負荷 | 大量のトランザクション通知 | キュー・再試行・監視が機能 |

再現性を高めるには、DevnetやSimnetを使い、使用した`rusty-kaspa`のコミットハッシュ・設定・ログ・観測結果を保存する。

## 20.16 観測可能性：何を監視するか

セキュリティはアルゴリズムだけでは完成しない。運用時には、少なくとも以下を観測したい。

- ノードの同期状態とvirtual stateの進行。
- P2Pピア数、接続の多様性、切断・再接続の頻度。
- BlockDAGのtip数や受信ブロックの遅延。
- Mempoolのサイズ、トランザクション拒否理由、競合検知。
- コンセンサス関連のエラー、異常な再編成。
- アプリケーションの`observed → settled`までの時間分布。

メトリクス名・公開APIはバージョンに依存するため、実際のノードのログとRPC仕様を確認する。

## 20.17 C4モデルと責務の境界

```text
[Wallet / User]
       │  signed transaction
       ▼
[Application Backend] ───────→ [Payment DB]
       │                             │
       │ RPC / event                 └─ 冪等性・監査ログ
       ▼
[Kaspa Node]
  ├─ P2P：伝播・同期
  ├─ Consensus：ブロック・取引の検証
  ├─ GHOSTDAG：順序決定
  ├─ Virtual State：UTXO受理結果
  └─ Storage：DAG・状態・同期データ
```

**責務分離の例**：

- ノードはコンセンサス規則に従って受理結果を計算する。
- アプリケーションはトランザクション額・リスクに応じた入金ポリシーを決める。
- DBはイベントの重複処理を防ぎ、状態変更の履歴を残す。
- 監視基盤はノードの同期異常や接続異常を検知する。

「ノードが返したから絶対安全」という設計ではなく、**ノードの観測情報と業務判断の間に明確な境界**を置く。

## 20.18 `rusty-kaspa`のソースコードを読む

ソースコード調査は、特定の関数名を暗記するより、責務と不変条件を追うとよい。

```bash
git clone https://github.com/kaspanet/rusty-kaspa.git
cd rusty-kaspa

git checkout --detach fcd9c28f9b211b9872a0c90a8c94f79466ffb063
git rev-parse HEAD
rg 'finality_depth|FINALITY_DURATION|pruning_depth' consensus
rg 'virtual|accepted_transaction|reorg' consensus
rg 'pruning_point|pruning proof' consensus
```

調査順序：

1. `consensus/core/src/config/constants.rs`で定数を確認する。
2. `consensus/core/src/config/params.rs`でネットワークごとの設定・派生値を確認する。
3. `finality_depth`の**参照箇所**を追い、何を拒否・制限するかを読む。
4. Virtual stateの更新・再編成処理を確認する。
5. Pruning pointと同期・証明検証の接続を確認する。
6. 該当する単体テスト・統合テストを探す。
7. 調査したコミットを記録し、本文の説明と照合する。

> **バージョン注意**：本章の実装照合は固定した基準版を対象にする。別の提案・開発タスクと、ネットワークで有効なコンセンサス規則は、リリース・有効化条件・対象ネットワークを確認して区別する。

## 20.19 BitcoinとKaspaの比較

| 観点 | Bitcoin | Kaspa |
|---|---|---|
| 基本データ構造 | 一本の主チェーンを選択 | 並行ブロックを含むBlockDAG |
| コンセンサスの見解 | 累積PoWに基づくチェーン選択 | GHOSTDAGによるDAG上の順序付け |
| 並行ブロック | 主チェーンに採用されないブロックが生じる | DAG内で扱い、順序付けに利用する |
| トランザクションの受理 | 主チェーン上のUTXO状態に従う | コンセンサス順序・virtual UTXO stateに従う |
| 確定性 | PoW蓄積に依存する確率的性質 | PoW・DAG進行・コンセンサス規則に依存 |
| アプリ側の判断 | リスク別の承認ポリシーが必要 | 同様にリスク別ポリシーが必要 |

どちらも、秘密鍵の流出、取引所の内部不正、ウォレットのバグをコンセンサスだけで防げるわけではない。

## 20.20 よくある誤解

**Q1. Kaspaはブロックが速いから、送金は絶対に取り消せない？**  
いいえ。ブロック生成の速さと、攻撃下で履歴が覆るリスクは別に評価する。

**Q2. Mempoolに見えたトランザクションは確定済み？**\
いいえ。Mempoolはノードの未確定トランザクションの管理領域であり、コンセンサス上の受理とは異なる。

**Q3. blue blockのトランザクションは必ず有効？**\
いいえ。ブロックのblue/red分類と、その中のトランザクションがUTXO状態で受理されるかは別である。

**Q4. 51%未満ならどんな攻撃も成功しない？**  
いいえ。EclipseやDoS、ネットワーク分断、ウォレット侵害などは別の攻撃面を持つ。

**Q5. finalityの定数が12時間なら、アプリは12時間待つべき？**  
そのような直接の結論にはならない。プロトコルのfinalityルールと業務上の入金確認ポリシーは区別する。

## 20.21 実践課題

1. **脅威モデル**：10万円相当の入金を受けるECサイトを想定し、攻撃者・資産・信頼境界を文章化する。
2. **状態機械**：`reverted`と`investigation`からの復旧条件を追加し、状態遷移表を作る。
3. **冪等性**：同じトランザクション通知が10回届いても入金が一度しか計上されないテストを追加する。
4. **ソース調査**：`finality_depth`が参照される箇所を列挙し、各参照がどの不変条件を守るかを説明する。
5. **分断実験**：Simnet/Devnetの複数ノードで通信を制限し、DAGの見解が収束する過程を記録する。
6. **レビュー**：「確認数だけを見て入金確定する」設計の問題点を列挙する。

## 20.22 理解度チェック

1. SafetyとLivenessの違いを説明できるか。
2. トランザクションのValidityとAcceptanceはなぜ同じではないのか。
3. GHOSTDAGのblue/red分類とトランザクションの有効性の関係は何か。
4. ネットワーク分断が確定性に与える影響は何か。
5. Eclipse攻撃とSybil攻撃はどう異なるか。
6. プロトコルのfinalityルールと、取引所の入金ポリシーはなぜ別なのか。
7. PruningとFinalityが関係する理由は何か。
8. 入金処理で冪等性が必要なのはなぜか。

### 解答の方向性

1. Safetyは矛盾した結果を防ぐ性質、Livenessは処理が進む性質。
2. トランザクション単体のルール適合性だけでは、競合UTXOの最終的な消費順序が決まらないため。
3. blue/redはブロックのコンセンサス分類であり、トランザクションの受理判定とは異なる。
4. 情報の非対称性が増し、ノードのviewの不一致や収束遅延を生むため。
5. Eclipseは特定ノードの観測を孤立させる攻撃、Sybilは多数の偽装参加者を作る攻撃。
6. 前者はノードのコンセンサス規則、後者は事業者が受け入れる損失リスクの判断だから。
7. 削除可能な履歴の境界と、新規ノードが正しく同期・検証できる条件を整合させる必要があるため。
8. ネットワークイベントは再送・重複・順序逆転が起こり得るため。

## 20.23 まとめ

Kaspaのネットワークセキュリティは、PoWだけでも、GHOSTDAGだけでも、P2Pだけでも説明できない。**ブロックの検証、DAGの順序付け、UTXOの受理、ネットワークの収束、finalityルール、アプリケーションの業務判断**が連携して成り立つ。

本章の結論は三つである。

1. **「見えた」「受理された」「十分安全と判断した」は別の状態**である。
2. **攻撃モデルとネットワーク条件を明示せずに、確定性を秒数だけで語れない**。
3. **ノードのコンセンサスとアプリケーションの決済ポリシーは責務を分離する**。

次章では、この安全性を支えるデータ管理として、Pruning・ストレージ・ノード同期を掘り下げる。

---

## 参考資料

1. [Kaspa — 公式サイト](https://kaspa.org/)
2. [rusty-kaspa — Rust製フルノード実装](https://github.com/kaspanet/rusty-kaspa)
3. [rusty-kaspa: Consensus Constants](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/constants.rs)
4. [rusty-kaspa: Consensus Parameters](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/params.rs)
5. [rusty-kaspa: Releases](https://github.com/kaspanet/rusty-kaspa/releases)
6. [Kaspa Research — GHOSTDAG関連論文・資料](https://github.com/kaspanet/docs)
7. [Bitcoin: A Peer-to-Peer Electronic Cash System — Satoshi Nakamoto](https://bitcoin.org/bitcoin.pdf)

> **残る確認**：Finalityの全分岐・攻撃モデルの証明・Pruning後の同期・入金確認指標の実装は、追加のコード照合と実行検証が必要。今回の主要経路の読み取りだけで確定稿とはしない。

基準版のFinality主要経路の照合と教材4テストの記録は `experiments/core-validation-review-2026-10-31.md`。攻撃成功確率の証明・全フォーク境界・公式Finality経路の実行時差分テストは今回未実施。
