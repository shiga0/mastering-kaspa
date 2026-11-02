# Chapter 33 — Layer 2, Bridges & Zero-Knowledge Proofs

> **レイヤー2・ブリッジ・ゼロ知識証明**
> 目次順の本文レビュー：2026-11-01。主要一次資料と独自状態遷移教材を確認。実ブリッジ・証明系・本番有効化の検証ではない。
> 本章は概念・設計・検証手法を説明する。個別プロジェクトの稼働状況やKaspaメインネットの機能対応は、利用時点の一次資料で確認する。

KaspaのKIP参照版は第30〜32章と同じ `e4ae2332117b5cb68bd6188e065ef885b6d17939`、ノードの基準版はv1.0.1 / `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。両者は同時点の機能集合ではない。確認記録は `experiments/chapter-33-review-2026-11-01.md`。

## 33.1 この章で学ぶこと

KaspaのL1を基盤としてアプリケーションを拡張する際、L2、ブリッジ、ゼロ知識証明（ZKP）は異なる課題を解決する。これらを混同せず、**何を誰が検証し、どの前提を信頼するか**を説明できることを目標とする。

- L1とL2の責務を分ける
- ロールアップとサイドチェーンを区別する
- ブリッジの資産移動とメッセージ検証を整理する
- ZK証明の「何を証明するか」を明確にする
- データ可用性・決済・ファイナリティの依存関係を調べる
- Rustで検証可能な最小状態遷移モデルを作る

## 33.2 まず全体像

```text
ユーザー / ウォレット
       |
       v
  L2アプリケーション ---------> L2実行環境
                                    |
                         状態遷移・取引履歴
                                    |
                         証明 / コミットメント
                                    |
                                    v
                           L1の検証・決済層
                                    |
                                    v
                             Kaspa BlockDAG
```

この図は**設計上の一般形**であり、Kaspaの現行プロトコルがすべての矢印をネイティブに実装しているという意味ではない。とくにL1がZK証明を検証できるか、L2状態の確定をどう扱うかは個別仕様に依存する。

## 33.3 L1が提供するもの、L2が追加するもの

| 観点 | L1（Kaspa） | L2（設計に依存） |
|---|---|---|
| コンセンサス | BlockDAGと合意規則 | 独自シーケンサーや分散合意など |
| 資産 | L1のUTXO | L2内の残高・トークン表現 |
| 実行 | L1で許可されたScriptと検証規則 | EVMなど独自実行環境も可能 |
| データ保存 | L1が保持・伝播するデータ | L1、外部DA、運営者など |
| 退出 | L1上の支出条件に依存 | 証明・ブリッジ・運営者に依存 |

**重要：** 「Kaspa上に構築された」という宣伝文句だけでは、L1のセキュリティをそのまま継承していると判断できない。

## 33.4 L2とサイドチェーンの違い

一般に「L2」はL1に何らかの形で検証・決済を依存させる仕組みを指すが、業界全体で呼称が完全に統一されているわけではない。

- **ロールアップ**：L1で検証・決済し、状態再構築に必要なデータもL1のDAへ公開する構成。コミットメントや証明だけをL1へ置く方式と区別する。公開データは圧縮や状態差分でもよく、必ず元の取引の全バイトとは限らない。
- **サイドチェーン**：独自のコンセンサスを持ち、L1との資産移動にブリッジを利用することが多い。
- **Validium型**：有効性証明を使っても、トランザクションデータをL1外に置く構成。データが得られないと退出が困難になり得る。
- **状態チャネル**：当事者間で更新を重ね、必要時に基盤チェーンへ決済する構成。

ここでの区別は[EthereumのZK-rollup](https://ethereum.org/en/developers/docs/scaling/zk-rollups/)と[Validium](https://ethereum.org/en/developers/docs/scaling/validium/)の説明を比較基準にしたもの。Kaspaで同じ仕組みが稼働しているという根拠ではない。

名称よりも、**不正な状態を拒否できる主体**と**データを失った際に退出できるか**を調べる。

## 33.5 L2の状態遷移を式で捉える

L2の状態を `S`、トランザクションバッチを `B`、遷移関数を `F` とする。

```text
S_next = F(S_previous, B)
```

ここで必要な問いは次のとおり。

1. `F` の仕様は公開されているか。
2. `B` の内容を利用者が入手できるか。
3. `S_previous` と `S_next` のコミットメントはどこに記録されるか。
4. `F` が正しく適用されたことを誰が検証するか。
5. 不正や停止が起きた際、資産を取り戻せるか。

## 33.6 Optimistic方式とValidity Proof方式

| 項目 | Optimistic方式 | Validity Proof方式 |
|---|---|---|
| 基本姿勢 | 提出された状態を暫定的に受け入れ、異議申立てを許す | 有効性証明を検証して状態更新を受け入れる |
| 必要な仕組み | 不正証明、挑戦期間、監視者 | 証明生成、検証器、公開入力 |
| 主な負荷 | 紛争処理と監視 | 証明生成計算 |
| 注意点 | 挑戦できる環境・データが必要 | 証明が正しくてもデータ可用性は別問題 |

Optimistic方式では、少なくとも1人が正しいデータを入手して異議を提出でき、L1が期限内に取り込める等の条件も必要になる。Validity方式でも検証器の更新権限、停止時の退出・強制取り込みを別に調べる。

どちらもL1の実装可能な機能に制約される。Ethereumのロールアップの説明を、そのままKaspaで実現済みの機能として扱わない。

## 33.7 ゼロ知識証明とは何か

ZKPは、公開された命題の真偽の検証に必要な情報を超えてWitnessの情報を漏らさない、という性質を持つ証明技術である。これは公開入力自体を隠す性質ではない。完全性・健全性（方式により知識健全性等）とゼロ知識性は別の要件として読む。定義の入口は[NISTのZKP説明](https://csrc.nist.gov/projects/pec/zkproof)を参照する。ただしロールアップの**有効性証明**では、秘密を隠すこと自体が目的でない場合もある。

```text
主張: 「この状態更新は定めたルールに従う」

Prover（証明者）
  ├─ 公開入力: 旧状態root、新状態root、バッチcommitment
  ├─ Witness: 取引・中間状態など
  └─ Proof: ルールに従った計算の証明

Verifier（検証者）
  ├─ 公開入力
  ├─ 検証鍵
  └─ Proof
       |
       v
     accept / reject
```

証明が示すのは**回路・プログラムに符号化された命題**である。検証鍵やプログラムID、公開入力のエンコードも特定して初めて「何を証明したか」が定まる。回路にバグがあれば、証明が正しくても意図した業務ルールを保証しない。

## 33.8 ZK-SNARKとZK-STARK

| 観点 | SNARK系 | STARK系 |
|---|---|---|
| 証明サイズ | 比較的小さい構成が多い | 比較的大きい構成が多い |
| セットアップ | 方式により信頼されたセットアップが必要 | 透明なセットアップの方式が代表的 |
| 計算・検証コスト | 方式と回路による | 方式とパラメータによる |
| 暗号学的前提 | 曲線・ペアリング等を使う方式がある | ハッシュ関数ベースの方式が代表的 |

STARKの透明性等の研究上の性質は[元論文の公開要旨](https://eprint.iacr.org/2018/046)も参照する。個別の設定・実装・ラッパーの性質へ自動的には一般化しない。例えばSTARKを別のSNARKで包むと、最終検証の暗号学的前提やセットアップ条件も変わる。

どちらが常に優れているという結論にはならない。比較には具体的な証明系、回路規模、検証環境が必要である。

## 33.9 コミットメントとMerkle Tree

コミットメントはデータを短い値に結び付ける手段。Merkle rootは、集合に含まれるデータを効率よく検証するために利用できる。

```text
      root
     /    \
   h01    h23
  /  \    /  \
 h0  h1  h2  h3
```

葉/内部ノードのエンコード、順序・index、ドメイン分離、信頼するrootを定義して使う。攻撃者が自由に選んだrootに対する包含だけでは、正当なチェーンや資産ロックの証拠にならない。

Merkle証明は「ある葉がコミットされた集合に含まれる」ことを示せるが、それだけで**状態遷移が正しい**ことは証明しない。

## 33.10 データ可用性（DA）を分離する

状態の正しさとデータの入手可能性は別である。

```text
Validity: その状態遷移は正しいか？
Availability: その状態を再構築するデータは得られるか？
Finality: いつ・どの条件で覆りにくくなるか？
```

証明だけが存在しても、利用者が必要なトランザクションデータを入手できなければ、独立検証や資産退出が困難になることがある。

DAは永久アーカイブと同じではない。KaspaのPruningでは履歴の削除と現行UTXOの保持を区別する（[第21章](../part-04-development/chapter-21.md)）。Pruningや他のDA層の保持期間をまたいで、誰が復旧・退出に必要なデータを保存するかを決める。rootや検証鍵が残っていても、状態・経路・Witnessが自動復元されるわけではない。

## 33.11 ブリッジの基本モデル

ブリッジは「コインが物理的にチェーン間を移動する」仕組みではない。典型的には、一方で資産をロックまたはバーンし、他方で対応する資産を発行またはアンロックする。

```text
Kaspa L1                         外部チェーン / L2
   |                                    |
   |  ロック / バーン                    |
   v                                    |
Bridge Contract / Custody               |
   |                                    |
   +---- 証明・署名・メッセージ -------->|
                                        v
                               発行 / アンロック
```

この模式図の `Bridge Contract` は一般概念であり、Kaspa L1上に同等の汎用コントラクトが実装済みであることを意味しない。

## 33.12 ブリッジの信頼モデル

| モデル | 何を信頼するか | 主なリスク |
|---|---|---|
| カストディ型 | 資産管理者 | 鍵流出、凍結、運営停止 |
| マルチシグ型 | 署名者の閾値と運用 | 共謀、鍵紛失、閾値設定 |
| ライトクライアント型 | 相手チェーンのヘッダー検証など | 検証実装、更新・再編成 |
| 証明検証型 | 証明系、検証器、コミットメント | 回路バグ、検証器バグ、DA |

これらは排他的な分類ではなく、証明検証と署名者、ヘッダー検証と管理者更新等が併用され得る。ライトクライアントでは、初期の信頼点・ヘッダーの合意/Finality・イベント包含・宛先拘束まで読む。設計ごとの[ブリッジのリスク](https://ethereum.org/en/bridges/)も区別する。

「分散型ブリッジ」という名称だけでは安全性は判断できない。

## 33.13 ブリッジの状態機械

```text
Created
   |
   v
SourceObserved
   |
   v
SourceFinalized
   |
   v
ProofVerified
   |
   v
DestinationExecuted
   |
   v
Completed

完了前の各段階 -> Failed / Expired / ManualReview / OutcomeUnknown
```

状態の名前は設計例。SourceFinalizedは設定したFinalityPolicyを満たした観測で、無条件に覆らないという意味ではない。送信結果が不明な場合はDestinationPending/OutcomeUnknown等も設け、相手側を照合してから再送・補償する。タイムアウトだけを根拠に返金すると、遅れて実行された支払いと二重に成立し得る。

実際のチェーンでは、ファイナリティの定義、再編成、失敗時の補償手順を個別に決める必要がある。

## 33.14 ブリッジの不変条件

1. **二重実行防止**：同じ論理メッセージを二度実行しない。送信元の完全なネットワーク識別、bridge/資産の識別、outpointまたはイベントindex、宛先ネットワーク・受取人・金額・方式の版等を結び付ける。再編成で同じ取引が再観測されても、新しいブロックhashだけで別メッセージにしない。
2. **金額保存**：手数料や換算ルールを除き、発行量が裏付けを超えない。
3. **宛先一致**：チェーンID、受取人、資産IDを署名・証明の対象に含める。
4. **順序・再編成耐性**：未確定イベントを確定済みと扱わない。
5. **失敗時の回復（進行性・運用要件）**：片側だけ成功した状態と成否不明を区別し、再送・照合・補償の条件を定める。
6. **鍵管理**：秘密鍵をアプリケーションの通常ログや設定ファイルに置かない。

## 33.15 代表的な攻撃と故障

- **Replay攻撃**：過去の有効メッセージを再利用する。
- **署名者侵害**：マルチシグの閾値を超える鍵が奪われる。
- **偽イベント**：存在しないロックを観測したと誤認する。
- **Reorganization**：送金元の確定判断が覆る。
- **証明器と検証器の不一致**：実装の意味が異なる。
- **DA停止**：正しいコミットメントはあるが状態を再構築できない。
- **運営者の検閲**：トランザクションや出金要求が取り込まれない。

安全性レビューでは「悪意ある攻撃者」と「正常な運営者の故障」を両方扱う。

## 33.16 Kaspaで考える際の制約

KaspaはBitcoin型のUTXO設計を基礎とし、BlockDAGとGHOSTDAG系の合意機構を採用する。L2を考える際には、少なくとも以下を確認する。

- 現在のL1 Scriptで表現できる検証条件は何か。
- 証明検証をL1で実行できるのか、それとも外部の検証者が担うのか。
- L1へのコミットメントの記録方法は何か。
- L1の再編成・ファイナリティをどう扱うか。
- L2が停止した場合、ユーザーは単独で退出できるか。
- プロトコル変更が必要なら、どのKIPに依存するか。

**KIPの提案、開発ネットワーク上のデモ、メインネットで有効な機能は必ず区別する。**

固定[KIP-16](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0016.md)はOpZkPrecompileによる証明系の呼び分けを記述し、固定[KIP-21](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0021.md)はレーンごとのsequencing commitmentを記述する。いずれも文書のStatusはActiveだが、任意のL2状態や資産裏付けが自動的に検証されるという意味ではない。対象命題・プログラム/検証鍵・公開入力・支出条件との結合を読む。第30〜32章で使った基準版ソースやSilverscriptの依存版とも別に照合する。

## 33.17 EVM互換L2の構成を読む

EVM互換といっても、EVMトランザクションを実行できるという意味にとどまる場合がある。

```text
EVM Wallet
   |
   v
RPC Gateway
   |
   v
Sequencer -> EVM Execution -> State Root
   |                            |
   v                            v
Data Availability           Settlement Adapter
                                |
                                v
                            Kaspa L1 (?)
```

最後の接続がどのように成立するかが最重要である。単なる定期的なハッシュ記録なら、L1がL2の不正状態を拒否する保証にはならない。

## 33.18 C4モデルで整理する

### Context

- ユーザー：入金・トランザクション・出金を行う。
- L2運営者：実行・順序付け・データ公開を行う。
- Kaspa L1：L1の合意・トランザクション検証を行う。
- 監視者：不整合、停止、遅延を検出する。

### Container

```text
[Wallet] -> [API/RPC] -> [Sequencer] -> [Executor]
   |                         |             |
   |                         v             v
   |                     [DA Layer]    [State DB]
   |                                       |
   +------------> [Bridge Service] <-------+
                         |
                         v
                 [L1 Integration]
```

### Component

`Bridge Service` は、`EventIndexer`、`FinalityPolicy`、`ProofVerifier`、`ReplayGuard`、`WithdrawalCoordinator` に分割できる。各コンポーネントの境界を先に定義すると、脅威モデルとテストを対応付けやすい。

## 33.19 DDDで考える境界

| 境界づけられたコンテキスト | 主な概念 | 重要な不変条件 |
|---|---|---|
| L2 Execution | Transaction, StateTransition | 同じ入力なら同じ結果 |
| Settlement | Commitment, Batch | 無効な更新を確定しない |
| Bridge | Deposit, Withdrawal, Message | 二重発行・二重払い防止 |
| Indexing | Block, Event, Cursor | 再編成時に再処理可能 |
| Operations | Alert, Incident | 障害を検知・追跡可能 |

上の図・表は責務の概念整理で、全呼び出し順や一つの原子的更新を表すものではない。同じ入力には初期状態・ネットワーク/プロトコル設定・順序も含める。

`Bridge` と `Indexing` を同じ責務にしない。インデクサの「観測した」という事実と、ブリッジの「支払いを許可した」という判断は異なる。

## 33.20 Rustで最小の状態遷移を作る

以下は**教育用の純粋関数**であり、ZK回路、Kaspa Script、実際のブリッジ実装ではない。外部クレート不要で `cargo test` できる。

`src/lib.rs`:

```rust
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transfer {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyError {
    EmptyAccount,
    ZeroAmount,
    InsufficientFunds,
    Overflow,
}

pub type Balances = BTreeMap<String, u64>;

pub fn apply_transfer(current: &Balances, tx: &Transfer) -> Result<Balances, ApplyError> {
    if tx.from.is_empty() || tx.to.is_empty() {
        return Err(ApplyError::EmptyAccount);
    }
    if tx.amount == 0 {
        return Err(ApplyError::ZeroAmount);
    }
    let from_balance = current.get(&tx.from).copied().unwrap_or(0);
    if from_balance < tx.amount {
        return Err(ApplyError::InsufficientFunds);
    }
    // 同一アカウントへの送金は残高を変えない。
    if tx.from == tx.to {
        return Ok(current.clone());
    }
    let to_balance = current.get(&tx.to).copied().unwrap_or(0);
    let new_to = to_balance
        .checked_add(tx.amount)
        .ok_or(ApplyError::Overflow)?;

    let mut next = current.clone();
    next.insert(tx.from.clone(), from_balance - tx.amount);
    next.insert(tx.to.clone(), new_to);
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn initial() -> Balances {
        BTreeMap::from([("alice".to_string(), 100)])
    }

    #[test]
    fn transfer_preserves_total() {
        let before = initial();
        let after = apply_transfer(
            &before,
            &Transfer {
                from: "alice".into(),
                to: "bob".into(),
                amount: 30,
            },
        )
        .unwrap();
        assert_eq!(after["alice"], 70);
        assert_eq!(after["bob"], 30);
        assert_eq!(before["alice"], 100); // 元の状態は不変
        assert_eq!(
            after.values().map(|v| u128::from(*v)).sum::<u128>(),
            before.values().map(|v| u128::from(*v)).sum::<u128>()
        );
    }

    #[test]
    fn insufficient_funds_does_not_mutate_state() {
        let before = initial();
        let result = apply_transfer(
            &before,
            &Transfer {
                from: "alice".into(),
                to: "bob".into(),
                amount: 101,
            },
        );
        assert_eq!(result, Err(ApplyError::InsufficientFunds));
        assert_eq!(before["alice"], 100);
    }

    #[test]
    fn self_transfer_keeps_balance() {
        let before = initial();
        let after = apply_transfer(
            &before,
            &Transfer {
                from: "alice".into(),
                to: "alice".into(),
                amount: 10,
            },
        )
        .unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn rejects_zero_amount() {
        assert_eq!(
            apply_transfer(
                &initial(),
                &Transfer {
                    from: "alice".into(),
                    to: "bob".into(),
                    amount: 0,
                }
            ),
            Err(ApplyError::ZeroAmount)
        );
    }

    #[test]
    fn recipient_overflow_preserves_the_complete_state() {
        let before = BTreeMap::from([("alice".into(), 10), ("bob".into(), u64::MAX)]);
        let saved = before.clone();
        let result = apply_transfer(
            &before,
            &Transfer {
                from: "alice".into(),
                to: "bob".into(),
                amount: 1,
            },
        );
        assert_eq!(result, Err(ApplyError::Overflow));
        assert_eq!(before, saved);
    }

    #[test]
    fn rejects_empty_account_labels() {
        for (from, to) in [("", "bob"), ("alice", "")] {
            assert_eq!(
                apply_transfer(
                    &initial(),
                    &Transfer {
                        from: from.into(),
                        to: to.into(),
                        amount: 1
                    }
                ),
                Err(ApplyError::EmptyAccount)
            );
        }
    }

    #[test]
    fn self_transfer_still_requires_sufficient_funds() {
        for (account, amount) in [("alice", 101), ("missing", 1)] {
            assert_eq!(
                apply_transfer(
                    &initial(),
                    &Transfer {
                        from: account.into(),
                        to: account.into(),
                        amount
                    }
                ),
                Err(ApplyError::InsufficientFunds)
            );
        }
    }

    #[test]
    fn finite_grid_preserves_total_and_input_state() {
        let mut successes = 0;
        let mut failures = 0;
        for a in [0, 1, 10, u64::MAX] {
            for b in [0, 1, 10, u64::MAX] {
                for amount in [0, 1, 10, u64::MAX] {
                    let before = BTreeMap::from([("alice".into(), a), ("bob".into(), b)]);
                    let saved = before.clone();
                    match apply_transfer(
                        &before,
                        &Transfer {
                            from: "alice".into(),
                            to: "bob".into(),
                            amount,
                        },
                    ) {
                        Ok(after) => {
                            assert_eq!(
                                after.values().map(|v| u128::from(*v)).sum::<u128>(),
                                u128::from(a) + u128::from(b)
                            );
                            successes += 1;
                        }
                        Err(_) => failures += 1,
                    }
                    assert_eq!(before, saved);
                }
            }
        }
        assert!(successes > 0 && failures > 0);
    }

    #[test]
    fn identical_transfers_are_not_replay_protected() {
        let tx = Transfer {
            from: "alice".into(),
            to: "bob".into(),
            amount: 30,
        };
        let once = apply_transfer(&initial(), &tx).unwrap();
        let twice = apply_transfer(&once, &tx).unwrap();
        assert_eq!(twice["alice"], 40);
        assert_eq!(twice["bob"], 60);
    }
}
```

### 実行

```bash
# 本書の外の調査用ディレクトリで実行
cargo new l2-state-lab --lib --edition 2021
cd l2-state-lab
# src/lib.rs を上記コードで置き換える
cargo test --offline
cargo fmt --all -- --check
cargo clippy --offline --all-targets -- -D warnings
```

このモデルはアカウント残高の教材であり、KaspaのUTXO検証器ではない。from/toは架空の非空ラベルで、署名・権限・nonce・Replay防止・手数料・bridgeの裏付け・状態rootは実装しない。同じ取引を2回与えると2回移転できることもテストしている。残高0のentryも残るため、経済的に同じ残高集合でも辞書が同一とは限らない。

Rust/Cargo 1.90.0、edition 2021、外部依存なしの一時libで本文を抜き出して検査した。総量を数えるテストはu128を使い、個別u64残高の合計でテスト自身がoverflowすることを避ける。

この実験で学べるのは、**状態遷移を決定的・副作用なしに設計すること**である。実際のZKシステムでは、この遷移を証明可能な回路またはVMへ落とし込み、コミットメントと公開入力を整合させる必要がある。

## 33.21 証明対象を仕様化する

例えば「バッチ内の送金が有効である」を証明するなら、先に次の仕様を決める。

```text
Public inputs:
  old_state_root
  new_state_root
  batch_commitment
  protocol_domain / version
  relevant_L1_anchor / deposit_cursor (方式に応じる)

Witness:
  transactions
  relevant_account_paths
  intermediate_states

Constraints:
  valid signatures (if required by the protocol)
  nonnegative balances
  no arithmetic overflow
  deterministic transaction order
  old_root -> new_root consistency
```

検証側もold_state_rootが現在の受理状態と一致すること、batch_commitmentが証明に使う取引列と結び付くこと、正しいプログラム/検証鍵の証明であることを確認する。公開入力を用意するだけで回路がそれを拘束しなければ不十分である。

署名、nonce、手数料、二重支払い防止、チェーンIDなどを回路から漏らすと、証明が通ってもシステムが安全とは限らない。

## 33.22 ブリッジ実装の最小API設計

```rust,ignore
// 設計例。実際のSDK型・APIではない。
pub trait SourceFinality {
    type Event;
    type Error;
    fn finalized(&self, event: &Self::Event) -> Result<bool, Self::Error>;
}

pub trait MessageVerifier {
    type Message;
    type Error;
    fn verify(&self, message: &Self::Message) -> Result<(), Self::Error>;
}

pub trait ReplayProtection {
    type Id;
    type Error;
    fn claim_once(&mut self, id: &Self::Id) -> Result<bool, Self::Error>;
}
```

この型断片はSDKの再現ではなく、別の一時libで型検査した設計例である。finalizedのboolも指定したポリシーを満たす判定にすぎず、永続的な確定やネットワーク観測をこのtrait自体は保証しない。

claim_onceが予約を表すのか完了記録かを仕様化する。ローカルDBの一意制約・トランザクションで同時のclaimを防いでも、外部チェーン上の実行と一つのDBトランザクションにはできない。claim後のクラッシュ、送信後のタイムアウト、実行成功後の記録失敗を扱い、宛先の重複防止・同じメッセージID・成否不明の照合で再送を設計する。このtraitだけではexactly-onceの外部作用を保証しない。

## 33.23 テスト戦略

| レベル | 検証対象 | 例 |
|---|---|---|
| Unit | 状態遷移と不変条件 | 残高保存、オーバーフロー拒否 |
| Property | 広い入力集合 | 固定順序での決定性、総量、定義した入力変換 |
| Differential | 独立実装との一致 | Rust実装と回路実装 |
| Integration | RPC・インデクサ・DB | 再起動、イベント再取得 |
| Adversarial | 敵対的入力 | Replay、再編成、署名偽造 |
| End-to-end | 入金から出金 | タイムアウト、部分失敗、復旧 |

「テストが通った」ことと「暗号学的に安全である」ことは異なる。証明系やブリッジでは監査と形式的な仕様検証も検討する。

## 33.24 観測可能性と障害対応

最低限、次の指標を計測する。

- L1入金イベント観測から確定までの時間
- L2での実行遅延
- 証明生成時間・失敗率
- L1コミットメントの提出・受理状況
- 出金待機時間
- 再編成で取り消されたイベント数
- 未処理・重複処理メッセージ数
- L2データ取得失敗率

ログには秘密鍵・秘密のWitness・個人情報を出力しない。

## 33.25 Kaspa関連プロジェクトを調べる手順

個別のL2やブリッジを評価するときは、紹介記事だけでなく、一次資料で次の項目を確認する。

1. **コード**：公開リポジトリ、ライセンス、監査対象コミット。
2. **仕様**：L1に何を記録し、誰が検証するか。
3. **資産保全**：入金資産の管理主体、出金手順。
4. **データ可用性**：誰がデータを保持し、停止時にどう復元するか。
5. **アップグレード権限**：管理者鍵、タイムロック、緊急停止。
6. **実稼働状態**：研究、テストネット、メインネットを区別。
7. **障害履歴**：インシデントと復旧手順。

個別プロジェクトを見かけた場合も、EVM互換という特徴だけでKaspa L1による検証保証があると推定しない。

## 33.26 実践演習A：ブリッジの脅威モデル

次の要件で設計書を作る。

> Kaspa L1上の入金を確認した後、別の実行環境で対応残高を付与する。

成果物：

- システムコンテキスト図
- 状態遷移図
- 不変条件（最低5個）
- 失敗シナリオ（最低5個）
- 信頼する主体と鍵の一覧
- 監視指標と復旧手順

**合格基準：** 「L1が止まる」「インデクサが誤検知する」「運営者が停止する」それぞれで資産がどうなるか説明できる。

## 33.27 実践演習B：ZK化の前に決定性を確認する

1. 33.20の状態に送信者ごとの次の期待nonceを、取引に使用するnonceを追加する。初期値と未知アカウントの扱いも定義する。
2. 取引のnonceが状態の期待値と一致することを検査し、成功時だけ残高とnonceをまとめて更新する。nonceの加算にもオーバーフロー検査を行い、同一アカウントへの送金を成功とする場合もnonceを進める。
3. 掲載済みの `identical_transfers_are_not_replay_protected` を拡張後の仕様に合わせ、同じ取引の再実行がnonce不一致で拒否されることを確認する。nonce不一致・残高不足・受取残高のオーバーフロー・nonceの上限では、残高とnonceの両方が維持されることもテストする。
4. 同じ初期状態・取引列・規則から同じ結果になることを確認し、乱数シードを記録したランダム入力でも残高保存を検査する。
5. 状態rootを導入する場合はnonceもコミット対象に含め、直列化順序を固定する。回路化した場合に必要な公開入力とWitnessを表にする。

nonceの検査は送信者の権限を証明しない。拡張後も署名・認証を省略する教材であることを明示し、実装する場合はnonceを署名・証明の対象にも結び付ける。

**注意：** `BTreeMap` を使うだけでは、正式な状態コミットメントの仕様を定義したことにはならない。キー・値のエンコード、ハッシュ関数、ドメイン分離まで決める必要がある。

## 33.28 理解度チェック

1. L2とサイドチェーンの境界は何で判断するべきか。
2. 有効性証明があってもデータ可用性が必要なのはなぜか。
3. Merkle証明と状態遷移の有効性証明は何が違うか。
4. ブリッジでReplay攻撃を防ぐには何を一意に識別するか。
5. EVM互換L2がKaspa L1の安全性を継承するために、何を確認する必要があるか。
6. L1へのハッシュ記録だけでL2の不正状態を拒否できるか。
7. ZK回路の仕様が誤っていた場合、証明は何を保証するか。

## 33.29 まとめ

L2、ブリッジ、ZKPは同義語ではない。L2は実行・スケーリングの設計、ブリッジはチェーン間の資産・メッセージ連携、ZKPは命題の正しさを検証する暗号技術である。

最も重要なのは、**信頼境界を図示し、各境界で何が検証されるかを説明できること**だ。

```text
設計上の主張
    |
    v
検証可能な仕様
    |
    v
実装・証明・テスト
    |
    v
運用時の監視と復旧
```

Kaspaへの適用では、論文・KIP・実装・実際に有効なネットワーク機能を分けて調査する。

## 33.30 参考資料と調査先

### Kaspa一次資料

- [rusty-kaspa（基準版）](https://github.com/kaspanet/rusty-kaspa/tree/fcd9c28f9b211b9872a0c90a8c94f79466ffb063)
- [KIP-16（固定版）](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0016.md)
- [KIP-21（固定版）](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0021.md)
- [Kaspa公式サイト](https://kaspa.org/)

### 基礎技術

- [Ethereum.org — Scaling](https://ethereum.org/en/layer-2/)
- [Ethereum.org — Zero-knowledge rollups](https://ethereum.org/en/developers/docs/scaling/zk-rollups/)
- [Ethereum.org — Bridges](https://ethereum.org/en/bridges/)
- [NIST — PEC / ZKProof](https://csrc.nist.gov/projects/pec/zkproof)
- [STARKの元論文 — ePrint 2018/046](https://eprint.iacr.org/2018/046)（公開要旨・書誌の参照）
- [Ethereum.org — Validium](https://ethereum.org/en/developers/docs/scaling/validium/)

### 技術レビュー時の確認事項

- Kaspa L1のScript・検証機能と最新KIPの対応
- L2プロジェクトの最新実装と稼働状況
- ブリッジの鍵管理・監査報告・退出条件
- 証明検証の実行場所とデータ可用性の提供主体

---

次は[第34章 — vProgs & Programmability（vProgsとKaspaのプログラマビリティ）](chapter-34.md)。

**確認範囲：** 主要一次資料・固定KIPの説明、独自状態遷移教材と型断片。実ブリッジ・ZK回路/証明生成・暗号方式の全面照合、現行ネットワーク有効化やDA/退出試験は行っていない。記録は `experiments/chapter-33-review-2026-11-01.md`。
