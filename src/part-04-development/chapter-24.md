# Chapter 24 — Wallet Architecture & Key Management（ウォレットアーキテクチャと鍵管理）

> **Mastering Kaspa / Part IV — Kaspa Development & Implementation**  
> 原稿ステータス：v0.1（技術レビュー前）  
> 対象読者：Rust・バックエンド開発の経験があり、Kaspaウォレットの設計と安全性を理解したい人

## 24.1 この章で学ぶこと

ウォレットは「コインを保存する箱」ではない。ブロックチェーン上のUTXOを発見・追跡し、それを支出するための**鍵と署名機能を管理するソフトウェア**である。本章では次を扱う。

1. ウォレット、ノード、UTXO、鍵の責務を分離する。
2. 秘密鍵・公開鍵・アドレス・導出パスの関係を理解する。
3. HDウォレット、バックアップ、復旧の境界条件を整理する。
4. オフライン署名、ハードウェアウォレット、マルチシグなどの設計を比較する。
5. Rustでウォレットの状態管理と不変条件を実験する。

> **重要：** 本章のRustコードは**鍵を生成・署名する実装ではない**。秘密鍵やシードを本番資産で試さず、実際の署名には監査されたKaspa対応ライブラリと、対象ネットワーク・バージョンで検証された手順を用いる。

## 24.2 ウォレットは何を管理するのか

KaspaのKASはウォレットの内部に存在しない。UTXOはネットワークが合意する状態に存在し、ウォレットは自分が支出できるUTXOを識別する。

```text
┌──────────────────────────────────────────────┐
│ Wallet Application                           │
│ ┌─────────────┐ ┌────────────┐ ┌───────────┐ │
│ │ Key Manager │ │ UTXO Index │ │ Tx Builder│ │
│ └──────┬──────┘ └──────┬─────┘ └─────┬─────┘ │
│        │               │             │       │
│        └───────────────┼─────────────┘       │
│                        ▼                     │
│                 Node Gateway                 │
└────────────────────────┬─────────────────────┘
                         │ RPC / wRPC
                         ▼
                   Kaspa Full Node
                         │
                         ▼
                    Kaspa Network
```

| 構成要素 | 責務 | 管理すべきでないもの |
|---|---|---|
| Key Manager | 鍵の生成・導出・署名の権限管理 | ネットワークのコンセンサス |
| Address Manager | 受取・変更アドレスの導出と使用履歴 | 秘密鍵のログ出力 |
| UTXO Tracker | 自分のUTXO、残高、利用可能性の追跡 | 未確認残高の確定扱い |
| Coin Selector | 入力UTXOと変更出力の選択 | 鍵素材の直接保持 |
| Transaction Builder | 支出内容と署名対象の構築 | ネットワークの検証規則の独自変更 |
| Signer | 承認された内容への署名 | 未承認の任意トランザクション署名 |
| Node Gateway | ノード照会・送信・イベント購読 | ウォレットの業務ルール |

**境界を明確にすること**が、安全性と変更容易性の両方につながる。

## 24.3 秘密鍵・公開鍵・アドレス

秘密鍵は支出権限の根拠となる秘密の値である。公開鍵は秘密鍵から導かれ、署名の検証に利用される。アドレスは受取先を人間やアプリケーションが扱いやすい形式で表現する識別子であり、公開鍵そのものと常に同一ではない。

```text
Secret Key ──(楕円曲線演算)──> Public Key
    │                             │
    │                             └──(アドレス形式に応じた符号化)──> Address
    │
    └──(署名)──> Signature ──(公開鍵で検証)──> true / false
```

Kaspaのアドレス形式・署名方式・Scriptバージョンの詳細は、利用するウォレット種別と現行の実装で確認する。Bitcoinの`bc1...`やEthereumの`0x...`をそのままKaspaアドレスとして扱ってはならない。Kaspaではネットワーク識別子を含むアドレス表記が用いられる。

**注意：** 秘密鍵を知っていることと、任意のUTXOを支出できることは同義ではない。対象UTXOのロック条件、署名対象データ、トランザクションの有効性が一致する必要がある。

## 24.4 シードフレーズとHDウォレット

### 24.4.1 なぜ階層的に鍵を導出するのか

単一の秘密鍵しかない場合、受取先を増やすたびに秘密情報を別々に保管しなければならない。HD（Hierarchical Deterministic）ウォレットは、共通のルートから複数の鍵を決定的に導出する。

```text
Recovery Mnemonic
       │
       ▼
Seed / Root Key Material
       │
       ▼
Hierarchical Key Derivation
       ├── Account 0
       │    ├── Receive 0, 1, 2, ...
       │    └── Change  0, 1, 2, ...
       └── Account 1
            ├── Receive 0, 1, 2, ...
            └── Change  0, 1, 2, ...
```

「シードフレーズが同じなら必ず同じアドレスになる」とは限らない。**導出方式、導出パス、パスフレーズ、アドレス形式、ネットワーク**が異なると、復旧結果も変わりうる。

### 24.4.2 BIP-39・BIP-32・BIP-44を混同しない

| 仕様 | 主な役割 | 注意点 |
|---|---|---|
| BIP-39 | ニーモニックとシード生成 | 任意のウォレットが完全互換とは限らない |
| BIP-32 | 階層的な鍵導出 | 導出パスや曲線・鍵形式の互換性を要確認 |
| BIP-44 | アカウント階層の慣習 | coin type とウォレット実装の対応が必要 |

Kaspaで用いられる具体的な導出方式はウォレット実装ごとに確認すること。**Bitcoin用のパスを推測で流用しない**。

### 24.4.3 パスフレーズ

BIP-39の追加パスフレーズは、同じニーモニックから異なるシードを導出する。パスフレーズを失うと、ニーモニックが残っていても目的のウォレットを復旧できない可能性がある。パスフレーズの有無もバックアップ仕様の一部である。

## 24.5 アドレスの再利用と変更出力

UTXOモデルでは、入力の合計が支払額と手数料を超える場合、差額を**変更出力（change output）**として自分の管理するアドレスへ戻す。

```text
Inputs:  12 KAS
          │
          ▼
Transaction
  ├── Recipient: 8 KAS
  ├── Change:    3.9 KAS
  └── Fee:       0.1 KAS
```

数値は説明用。実際の手数料計算はトランザクションのmassやネットワーク規則に依存する。

ウォレットは受取アドレスだけでなく変更アドレスも追跡する必要がある。変更アドレスを復旧対象から外すと、残高の一部を見失う可能性がある。アドレスの使い回しを減らすことは、トランザクションの関連付けを難しくするための基本的なプライバシー対策となる。ただし、UTXOの統合などで関連性が推測されることはある。

## 24.6 残高は一つの数字ではない

ウォレット画面に表示する「残高」には異なる意味がある。

| 表示 | 意味 | 典型的な注意点 |
|---|---|---|
| 観測残高 | ノード・インデクサが観測したUTXOの合計 | 同期遅延や再編成 |
| 利用可能残高 | 現時点で支出候補にできるUTXOの合計 | ロック・予約・ポリシー |
| 送信可能額 | 手数料・変更出力・mass制約を考慮した額 | 残高と一致しないことがある |
| 保留中 | Mempoolや未確定ブロックに関連する額 | コンセンサス受理とは別 |

**不変条件：** `available_balance <= observed_balance` のような関係は、同じスナップショットと定義に基づく場合に限って期待できる。異なる時刻のRPC応答を混ぜると一時的に矛盾して見える。

## 24.7 ウォレットの送金ライフサイクル

```text
User Intent
    │
    ▼
Validate Destination / Amount / Network
    │
    ▼
Query & Select Spendable UTXOs
    │
    ▼
Construct Outputs / Change / Fee
    │
    ▼
Present Exact Transaction for Approval
    │
    ▼
Sign (Local or Hardware Signer)
    │
    ▼
Broadcast via Node
    │
    ▼
Track Mempool / BlockDAG / Virtual State
    │
    ▼
Update Wallet View
```

重要なのは、**送信成功 ≠ コンセンサス受理 ≠ アプリケーション上の確定**であること。送信APIが成功を返しただけで「支払い完了」と断定しない。

## 24.8 Coin Selectionと入力予約

複数のUTXOからどれを選ぶかは、手数料・プライバシー・UTXOの断片化に影響する。

- **少数入力優先：** トランザクションが小さくなりやすいが、特定のUTXOへ依存する。
- **小額UTXO統合：** 将来の管理を簡単にできる一方、手数料やプライバシーへの影響がある。
- **プライバシー配慮：** 無関係なUTXOを不用意に結び付けない。

並行送金で同じUTXOを二度選ばないよう、ウォレット内部で**入力予約**を設ける。ただしローカル予約はネットワーク上の支出確定を保証しない。

```text
Available -> Reserved -> Broadcast -> Accepted
                  │           │
                  └─ Cancelled└─ Rejected / Reorg -> Reconcile
```

状態の戻し方は、再照会したコンセンサス状態と照合して決める。単に通信がタイムアウトしただけなら、**送信が失敗したと断定せず**トランザクションID等で再確認する。

## 24.9 オフライン署名とハードウェアウォレット

署名をネットワーク接続部分から分離すると、秘密鍵をオンラインのバックエンドへ渡さずに済む。

```text
Online Host                          Isolated Signer
┌───────────────────┐                ┌──────────────────┐
│ UTXO Discovery    │                │ Key Storage      │
│ Tx Construction   │── Sign Request▶│ Review & Sign    │
│ Broadcast         │◀─ Signature ──│                  │
└───────────────────┘                └──────────────────┘
```

ただしオフライン署名も万能ではない。署名デバイスが**受取先・金額・手数料・ネットワーク・変更出力**を正しく解釈し、利用者に確認できなければ、悪意あるホストによる誘導を防げない。

ハードウェアウォレット対応は製品・ファームウェア・Kaspa機能の実装状況によって異なる。購入前に公式の対応状況を確認する。

## 24.10 Watch-onlyウォレットと署名権限の分離

Watch-onlyウォレットは、公開情報を用いて残高やトランザクションを監視し、秘密鍵を保持しない構成である。会計システムや入金監視に向いている。

| 構成 | できること | できないこと |
|---|---|---|
| Watch-only | 受取アドレス監視・残高確認 | 秘密鍵を要する署名 |
| Hot wallet | 自動署名・自動送金 | 鍵漏えいリスクをゼロにすること |
| Cold signer | オフライン承認・署名 | 単独でネットワーク照会・送信（通常） |
| Multisig / 条件付き支出 | 複数承認や条件の表現（対応方式による） | 実装非対応のスクリプト条件の実行 |

Kaspaのスクリプト機能で実現できる条件と、特定ウォレットがUIとして提供する機能は分けて確認する。

## 24.11 マルチシグと組織の承認フロー

「複数人の承認が必要」という業務要件と、「オンチェーンで複数署名を検証する」という技術要件は同じではない。

```text
Payment Request
     │
     ▼
Business Approval (Policy)
     │
     ▼
Transaction Construction
     │
     ▼
Cryptographic Signing (Script / Signer Policy)
     │
     ▼
Broadcast
```

業務承認をDB上で2人分記録しても、単一秘密鍵が流出すれば不正送金される可能性がある。逆に、複数署名条件だけでは経理上の請求書照合を保証しない。**業務ポリシーと暗号学的な支出条件を両方設計する**。

## 24.12 バックアップと復旧

ウォレット設計では、作成時より**復旧時**に設計ミスが発覚しやすい。

### 復旧に必要な情報

1. ニーモニック／秘密鍵などの復旧素材
2. 追加パスフレーズの有無
3. 使用したウォレットの導出方式とアカウント構成
4. アドレス形式・ネットワークの情報
5. 必要に応じてアドレス探索範囲やウォレット固有のメタデータ

### 復旧テスト

- 本番資産とは切り離したテスト用ウォレットを用意する。
- 少額・テストネットなど安全な環境で受取アドレスが再現されることを確かめる。
- 受取だけでなく変更アドレスや複数アカウントも確認する。
- 署名と送金が再現できることを確認する。
- 手順を記録する。ただし**秘密情報そのものをログや手順書に書かない**。

> **注意：** ニーモニックをWebフォームやAIチャットに入力して「復旧できるか確認」してはいけない。スクリーンショット・クラウド同期・クリップボード履歴にも注意する。

## 24.13 鍵管理の脅威モデル

| 脅威 | 典型的な失敗 | 対策の例 |
|---|---|---|
| 秘密鍵漏えい | ログ・Git・環境変数の露出 | 非保持設計、秘密情報スキャン、アクセス制御 |
| フィッシング | 偽ウォレットへのシード入力 | 公式配布元の検証、シード非入力原則 |
| マルウェア | 送金先アドレスの置換 | 署名デバイス上で宛先確認 |
| サプライチェーン | 改ざんされた依存ライブラリ | 依存固定、署名・リリース検証、監査 |
| バックアップ紛失 | シードが一箇所だけ | オフライン冗長化、定期的な復旧訓練 |
| ネットワーク誤送信 | mainnet/testnet混同 | ネットワークIDの型とUI確認 |
| 再編成・同期遅延 | 受信済みを確定扱い | 状態追跡、入金確認ポリシー |
| 内部不正 | 単一担当者が全権限を保持 | 職務分離、複数承認、監査ログ |

鍵素材を扱うプロセスの権限は最小化する。バックエンドの一般的なAPIサーバーが常時ホットウォレットの全秘密鍵を保持する設計は、必要性と被害範囲を慎重に検討する。

## 24.14 Rustで設計するウォレットの境界

ウォレットのロジックをSDKの具体型へ直接結び付けると、ネットワーク接続・鍵保管方式の変更が難しくなる。

```text
Application Service
    ├── WalletPolicy
    ├── UtxoRepository (trait)
    ├── TransactionBuilder (trait)
    ├── Signer (trait)
    └── NodeGateway (trait)
```

以下は**秘密鍵を持たない**入力予約の教育用実装である。外部クレート不要で、Rust標準ライブラリのみを利用する。

```rust
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Outpoint {
    txid: String,
    index: u32,
}

#[derive(Debug, Clone)]
struct Utxo {
    outpoint: Outpoint,
    amount: u64, // sompi
}

#[derive(Debug, PartialEq, Eq)]
enum WalletError {
    UnknownUtxo,
    AlreadyReserved,
    DuplicateInput,
    AmountOverflow,
}

#[derive(Default)]
struct WalletState {
    utxos: HashMap<Outpoint, Utxo>,
    reserved: HashSet<Outpoint>,
}

impl WalletState {
    fn insert(&mut self, utxo: Utxo) {
        self.utxos.insert(utxo.outpoint.clone(), utxo);
    }

    // 全入力を検証してから一括予約する。途中失敗時に状態を変えない。
    fn reserve(&mut self, inputs: &[Outpoint]) -> Result<u64, WalletError> {
        let mut seen = HashSet::new();
        let mut total = 0_u64;
        for input in inputs {
            if !seen.insert(input) {
                return Err(WalletError::DuplicateInput);
            }
            let utxo = self.utxos.get(input).ok_or(WalletError::UnknownUtxo)?;
            if self.reserved.contains(input) {
                return Err(WalletError::AlreadyReserved);
            }
            total = total.checked_add(utxo.amount)
                .ok_or(WalletError::AmountOverflow)?;
        }
        self.reserved.extend(inputs.iter().cloned());
        Ok(total)
    }

    fn release(&mut self, inputs: &[Outpoint]) {
        for input in inputs {
            self.reserved.remove(input);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> (WalletState, Outpoint) {
        let op = Outpoint { txid: "tx-a".into(), index: 0 };
        let mut wallet = WalletState::default();
        wallet.insert(Utxo { outpoint: op.clone(), amount: 100 });
        (wallet, op)
    }

    #[test]
    fn reserve_once() {
        let (mut wallet, op) = sample();
        assert_eq!(wallet.reserve(&[op.clone()]), Ok(100));
        assert_eq!(wallet.reserve(&[op]), Err(WalletError::AlreadyReserved));
    }

    #[test]
    fn duplicate_input_does_not_mutate_state() {
        let (mut wallet, op) = sample();
        assert_eq!(wallet.reserve(&[op.clone(), op.clone()]),
                   Err(WalletError::DuplicateInput));
        assert!(wallet.reserved.is_empty());
    }

    #[test]
    fn missing_input_does_not_mutate_state() {
        let (mut wallet, op) = sample();
        let missing = Outpoint { txid: "missing".into(), index: 0 };
        assert_eq!(wallet.reserve(&[op, missing]), Err(WalletError::UnknownUtxo));
        assert!(wallet.reserved.is_empty());
    }

    #[test]
    fn release_allows_retry() {
        let (mut wallet, op) = sample();
        wallet.reserve(&[op.clone()]).unwrap();
        wallet.release(&[op.clone()]);
        assert_eq!(wallet.reserve(&[op]), Ok(100));
    }
}
```

ローカルでの実行例：

```bash
cargo new wallet-state-lab --lib
cd wallet-state-lab
# 上のコードを src/lib.rs に配置
cargo test
```

この実装は**同一プロセス内での予約**しか扱わない。複数プロセスや複数デバイスでの送金には、DBトランザクション、排他制御、再起動後の復元、ネットワークとの再照合が必要である。`release`を通信タイムアウト直後に無条件で呼ぶのも危険だ。

## 24.15 ウォレットのステートマシン

| 状態 | イベント | 次の状態 | 必須チェック |
|---|---|---|---|
| Draft | UTXO選択 | Prepared | 入力の重複なし |
| Prepared | 利用者承認 | Approved | 宛先・金額・手数料一致 |
| Approved | 署名 | Signed | 署名対象と承認内容一致 |
| Signed | 送信 | Broadcast | ネットワーク一致 |
| Broadcast | コンセンサス観測 | Observed | トランザクション識別子一致 |
| Observed | 確認ポリシー充足 | Settled | 同期・再編成条件を考慮 |
| Broadcast / Observed | 競合・再編成 | Reconcile | UTXO状態を再取得 |

**不変条件の例：**

1. 秘密鍵はネットワーク層へ渡さない。
2. 署名対象の受取先・金額・手数料は承認後に黙って変更しない。
3. 同一ウォレット内の未完了送金が同じUTXOを同時予約しない。
4. 送信タイムアウトだけで未送信と断定しない。
5. 同期が不完全なノードの観測を確定判定の唯一の根拠にしない。
6. 失敗した操作は、既存の鍵情報・UTXO状態を壊さない。

## 24.16 `rusty-kaspa` のコードを読む

公式リポジトリ： [kaspanet/rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)

調査の際は**特定のコミットに固定**し、次の観点でコードを追う。

1. Wallet関連のRustクレートとWASM公開APIの境界。
2. アドレス生成、鍵導出、トランザクション構築の責務分担。
3. RPC / wRPCとの接続部分と再接続処理。
4. UTXOイベントの反映、残高更新、再編成時の挙動。
5. 鍵保管・暗号化・シークレット管理の実装。
6. 署名方式・Script・ネットワーク別のテストベクトル。

調査用コマンド例：

```bash
git clone https://github.com/kaspanet/rusty-kaspa.git
cd rusty-kaspa
git rev-parse HEAD
rg -n 'struct .*Wallet|trait .*Signer|derive.*key|signature|address' wallet* crypto* 2>/dev/null | head -80
```

ディレクトリ名や公開APIはバージョンで変わり得る。上の検索は**調査の入口**であり、特定バージョンでの存在を保証するものではない。

## 24.17 Devnetでの実験計画

**目的：** ウォレットの状態とネットワークの状態が一致しないケースを観察する。

| 実験 | 操作 | 確認項目 |
|---|---|---|
| 1 | Devnetウォレットで受取アドレスを生成 | ネットワークとアドレス形式 |
| 2 | テスト資金を受け取る | UTXO検出までの時間 |
| 3 | 送金トランザクションを構築 | 入力・出力・変更・手数料 |
| 4 | 送信直後にノードを切断 | タイムアウト時の再照合 |
| 5 | 再接続して履歴を照会 | 二重送信・重複表示の防止 |
| 6 | 同期の遅いノードを参照 | 残高・確定判定の変化 |

**記録すべき項目：** `rusty-kaspa`コミットID、ネットワーク、ノード起動引数、SDKバージョン、操作時刻、トランザクションID、観測結果。秘密鍵・ニーモニックは記録しない。

## 24.18 よくある誤解

**Q. ウォレットを削除するとKASも消える？**  
A. UTXO自体はネットワーク上に残る。ただし復旧素材を失えば支出権限を再現できなくなる。

**Q. シードフレーズがあればどのウォレットでも必ず復旧できる？**  
A. いいえ。導出方式・パス・パスフレーズ・アドレス形式の互換性が必要。

**Q. 送信APIが成功すれば決済完了？**  
A. いいえ。Mempool、ブロックへの取り込み、コンセンサス受理、サービスの確認ポリシーは別。

**Q. ハードウェアウォレットならすべて安全？**  
A. いいえ。署名内容の確認、復旧素材の保護、ファームウェアやサプライチェーンへの信頼が必要。

**Q. UTXOの合計額がそのまま送金可能額？**  
A. いいえ。手数料、入力選択、mass、予約・未確定状態などの制約がある。

## 24.19 実践課題

1. **C4モデル：** Wallet UI、Wallet Service、Signer、Node Gateway、Kaspa Nodeの境界を描く。
2. **脅威分析：** ホットウォレットの秘密鍵が漏えいした場合の被害範囲と防止策を表にする。
3. **Rust：** 24.14の入力予約モデルに「予約ID」「期限」「再起動後の復旧」を追加する。
4. **テスト：** 送信が成功したのにレスポンスだけ失われたケースをモデル化する。
5. **復旧：** テスト用のHDウォレットで受取アドレスと変更アドレスを再現する。実資産を使わない。

## 24.20 理解度チェック

1. ウォレットがKASそのものを保存していないとはどういう意味か。
2. ニーモニックと導出パスはそれぞれ何を決めるか。
3. 変更出力を追跡しなければ何が起こるか。
4. Watch-onlyウォレットができること・できないことは何か。
5. 送信タイムアウト時に入力予約を即時解除してはいけない理由は何か。
6. 業務上の複数承認と暗号学的マルチシグはどう違うか。
7. 「残高」「送信可能額」「確定済み額」を分ける理由は何か。
8. 署名デバイスが宛先を表示・確認することにはどんな意味があるか。

## 24.21 まとめ

ウォレットの本質は、**鍵による支出権限の管理**と**UTXOの状態追跡**を、安全な境界で結び付けることにある。暗号技術だけでは十分ではない。導出互換性、バックアップ、承認フロー、並行送金、ノード同期、再編成、秘密情報の保護まで含めて設計する必要がある。

Kaspaアプリケーションでは、ウォレットが観測する状態とコンセンサス上の状態を混同せず、RPC・SDK・Signer・業務ロジックの依存方向を明確にすることが重要である。

次章 **Chapter 25 — Exploring the rusty-kaspa Codebase** では、これらの責務が実際のRustコードでどのように分割されているかを調査する。

## 24.22 参考資料

- [Kaspa公式ドキュメント](https://docs.kaspa.org/)
- [Kaspa公式リポジトリ — rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)
- [Kaspa公式Wiki](https://wiki.kaspa.org/en/home)
- [BIP-32: Hierarchical Deterministic Wallets](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki)
- [BIP-39: Mnemonic code for generating deterministic keys](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki)
- [BIP-44: Multi-Account Hierarchy for Deterministic Wallets](https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki)
- [OWASP Cryptographic Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html)

> **技術レビュー時の確認事項：** 現行Kaspaウォレットの導出方式・アドレス形式・署名API・Script対応状況・mass/手数料計算・SDK公開インターフェースを、特定コミットとテストネット実験で検証する。一般的なHDウォレットの概念を、Kaspa固有の実装仕様と混同しない。
