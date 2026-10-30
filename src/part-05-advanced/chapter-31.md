# Chapter 31 — Covenants & Advanced Scripting（Covenantsと高度なスクリプティング）

> **原稿ステータス：v0.1（技術レビュー前）**  
> 本章は仕組みと設計の学習を目的とする。Kaspaで実際に利用できるOpcode、スクリプトバージョン、ネットワークでの有効化状況は、対象のノード実装・KIP・ネットワーク設定で必ず確認する。例示するCovenantは概念モデルであり、現在のKaspaメインネットでそのまま動くことを意味しない。

## 31.1 この章で学ぶこと

本章の目的は、**UTXOの使用条件を記述するスクリプト**から、**UTXOを次にどう使えるかまで制約するCovenant**へと発想を広げることである。さらに、その制約を安全なシステムとして設計・検証する方法を学ぶ。

学習目標：

1. 署名条件とCovenantの違いを説明できる。
2. UTXOとトランザクションを用いて、状態遷移の不変条件を定義できる。
3. Script、Covenant、オフチェーン検証、L2の責務を区別できる。
4. Kaspaの実装状況を一次資料で確認できる。
5. Rustで簡略化した制約エンジンを実装・テストできる。

## 31.2 前提：UTXOとScript

UTXO（Unspent Transaction Output）は、まだ消費されていないトランザクション出力である。トランザクションは既存のUTXOを入力として参照し、検証に成功すると新しいUTXOを生成する。

```text
UTXO A ─┐
        ├── Transaction ──> UTXO C
UTXO B ─┘                └─> UTXO D
```

スクリプトの基本的な役割は、**ある入力が参照するUTXOを消費する条件を検証すること**である。たとえば、公開鍵に対応する署名が正しいかを検証する。

```text
Input + Witness/Signature
          │
          ▼
  Spending Condition
          │
    ┌─────┴─────┐
   True        False
    │            │
  消費可能      拒否
```

スクリプトの評価結果だけでなく、トランザクション全体の整合性、金額、手数料、二重消費などのコンセンサス規則も満たす必要がある。

## 31.3 Covenantとは何か

通常の署名条件は「**誰が消費できるか**」を中心に扱う。Covenantは、これに加えて「**消費した結果、どのような出力を作らなければならないか**」などを制約する考え方である。

| 観点 | 一般的な署名条件 | Covenantの考え方 |
|---|---|---|
| 主な問い | 誰がこのUTXOを消費できるか | どういう形で次のUTXOを作れるか |
| 検証対象 | 署名、公開鍵、時間条件など | 出力先、出力額、構造、遷移条件など |
| 典型用途 | 通常の送金 | Vault、支出制限、段階的な資金移動 |
| 注意点 | 鍵の保護 | 表現力と検証可能性、柔軟性のトレードオフ |

**重要：** Covenantは特定のOpcode名ではない。トランザクションに課す制約の設計概念であり、実現方法はチェーンごとのスクリプト仕様に依存する。

## 31.4 なぜCovenantが必要なのか

たとえば、秘密鍵が漏えいしても、資金をすぐ任意のアドレスへ移動できないVaultを考える。

```text
Vault UTXO
    │
    ├── 通常経路：待機期間 → 指定先へ出金
    │
    └── 緊急経路：回復鍵 → 安全な保管先へ移動
```

単に「鍵を持っているか」だけでは表現しにくい資金移動のルールを、コンセンサスが検証可能な条件にできる点が利点である。ただし、上の図は設計例であり、Kaspaでこの経路が現時点で実装可能であると主張するものではない。

## 31.5 スクリプトの表現力とトレードオフ

高度なスクリプトでは次の問いが重要になる。

- 入力・出力のどのフィールドを参照できるか。
- 出力のスクリプトや金額を比較できるか。
- ハッシュや署名によって遷移先をコミットできるか。
- 時間・ブロック高・その他のチェーン状態に依存できるか。
- 検証コストを上限内に抑えられるか。
- 将来の拡張や既存のウォレットとの互換性をどう保つか。

表現力を増やすほど、便利なルールを記述しやすくなる。一方で、検証負荷、監査の難しさ、想定外の組み合わせも増える。

## 31.6 Covenantの代表的な設計パターン

### 31.6.1 出力先の制限

「次の出力は許可リストのアドレスに限定する」というパターン。支出ポリシーの表現に向くが、アドレスの変更・鍵の更新をどう扱うかが問題になる。

### 31.6.2 金額の制限

「一定額以上は一度に移動できない」などの制約。UTXOの分割・結合や手数料を考慮しないと、意図しない抜け道が生まれる。

### 31.6.3 継続する制約（再帰的Covenant）

「出力先にも同じ制約を持たせる」ことで、複数回の支出にわたりポリシーを継続する。制約の解除条件を誤ると、資金が永久に動かせなくなる可能性がある。

### 31.6.4 Vaultと回復経路

通常の支出と緊急回復を分ける。回復鍵、時間条件、キャンセル経路、監視システムを一体として設計する必要がある。

## 31.7 Covenantとスマートコントラクトの違い

```text
UTXO + Script / Covenant
  └─ 入力の消費可否と出力の制約を検証

アカウント型の汎用スマートコントラクト
  └─ コントラクトが保持する状態に対して処理を実行
```

両者ともプログラム可能なルールを表現できるが、状態の持ち方、実行モデル、並行処理、開発者体験が異なる。**Covenantがあれば任意のEVMアプリをそのまま実行できる、という意味ではない。**

## 31.8 トランザクションの内省（Introspection）

Covenantを考える際に重要なのが、スクリプトからトランザクション自身の構造を参照する能力である。

例：

```text
入力：Vault UTXO
出力0：許可されたスクリプトへの支払い
出力1：残額を同じ制約付きスクリプトへ返す
```

この条件を検証するには、出力の金額やロックスクリプトを評価できる仕組みが必要になる。**どの情報にアクセスできるかは実際のOpcode仕様次第**である。

## 31.9 署名ハッシュとコミットメント

署名は通常、トランザクションの特定の部分に対するコミットメントとして機能する。何が署名対象に含まれるかによって、署名後に変更できるフィールドが変わる。

Covenantを設計するときは、次の点を区別する。

1. 署名が保証する内容
2. スクリプトが参照できる内容
3. コンセンサスが別途検証する内容

これらを混同すると、「署名しているから出力先も固定される」といった誤った前提が生まれる。

## 31.10 状態遷移として捉える

Covenantは、UTXOを状態の担い手として使う**制約付き状態遷移**として考えられる。

```text
State S0 (Locked)
     │ valid transition
     ▼
State S1 (Pending)
     │ valid transition
     ▼
State S2 (Released)
```

たとえば、各状態を特定のスクリプトやコミットメントを持つUTXOで表現する。遷移が成立するための条件を定義し、許可されていない遷移を拒否する。

## 31.11 不変条件（Invariants）

設計では「何を実現したいか」より先に「**絶対に起きてはいけないこと**」を明確にすると検証しやすい。

| 不変条件 | 検証したいこと |
|---|---|
| 価値の保存 | 入力総額を超える出力を生成しない（発行等の特別規則を除く） |
| 許可先の限定 | 指定外のスクリプトへ移動しない |
| 制約の継承 | 継続すべき制約が次のUTXOにも残る |
| 単一消費 | 同じUTXOを二重に消費できない |
| 回復可能性 | 緊急経路が意図どおり存在する |
| コスト上限 | 検証処理が許容範囲に収まる |

## 31.12 DDDによる責務分離

概念的な設計を、以下のように分離できる。

```text
Application
  ├── CreateSpendProposal
  └── ValidateSpendProposal
          │
          ▼
Domain
  ├── Utxo
  ├── Transaction
  ├── SpendPolicy
  └── PolicyViolation
          │
          ▼
Infrastructure
  ├── ScriptAdapter
  ├── NodeRpcAdapter
  └── StorageAdapter
```

`SpendPolicy` はアプリケーションのドメイン概念であり、**そのままオンチェーンで強制されるわけではない**。オンチェーンで保証するには、対応するスクリプトとコンセンサス規則が必要である。

## 31.13 Rustで学ぶ簡略Covenantモデル

以下は「許可された受取先以外への送金を拒否する」という**オフチェーンの教育用モデル**である。Kaspa ScriptのOpcodeを実装したものではない。

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub destination: String,
    pub amount: u64,
}

#[derive(Debug, Clone)]
pub struct SpendProposal {
    pub input_amount: u64,
    pub fee: u64,
    pub outputs: Vec<Output>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PolicyError {
    UnauthorizedDestination,
    ValueOverflow,
    InsufficientInput,
    EmptyOutputs,
}

pub fn validate_spend(
    proposal: &SpendProposal,
    allowed_destinations: &[&str],
) -> Result<(), PolicyError> {
    if proposal.outputs.is_empty() {
        return Err(PolicyError::EmptyOutputs);
    }

    for output in &proposal.outputs {
        if !allowed_destinations.contains(&output.destination.as_str()) {
            return Err(PolicyError::UnauthorizedDestination);
        }
    }

    let total_outputs = proposal.outputs.iter().try_fold(0_u64, |acc, output| {
        acc.checked_add(output.amount)
            .ok_or(PolicyError::ValueOverflow)
    })?;

    let required = total_outputs
        .checked_add(proposal.fee)
        .ok_or(PolicyError::ValueOverflow)?;

    if required > proposal.input_amount {
        return Err(PolicyError::InsufficientInput);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proposal(destination: &str, amount: u64) -> SpendProposal {
        SpendProposal {
            input_amount: 100,
            fee: 1,
            outputs: vec![Output {
                destination: destination.to_owned(),
                amount,
            }],
        }
    }

    #[test]
    fn allows_authorized_destination() {
        assert_eq!(validate_spend(&proposal("safe", 99), &["safe"]), Ok(()));
    }

    #[test]
    fn rejects_unauthorized_destination() {
        assert_eq!(
            validate_spend(&proposal("attacker", 99), &["safe"]),
            Err(PolicyError::UnauthorizedDestination)
        );
    }

    #[test]
    fn rejects_overspending() {
        assert_eq!(
            validate_spend(&proposal("safe", 100), &["safe"]),
            Err(PolicyError::InsufficientInput)
        );
    }

    #[test]
    fn rejects_overflow() {
        let mut p = proposal("safe", u64::MAX);
        p.outputs.push(Output {
            destination: "safe".to_owned(),
            amount: 1,
        });
        assert_eq!(
            validate_spend(&p, &["safe"]),
            Err(PolicyError::ValueOverflow)
        );
    }
}
```

この例では、入力額と出力額の差額を「未使用額」として許容する。実際のトランザクション検証では、その差額の意味、手数料、複数入力、ネットワーク固有の規則を別途扱う必要がある。

### 実行方法

```bash
cargo new covenant-lab --lib
cd covenant-lab
# 上記コードを src/lib.rs に配置
cargo test
```

**検証状態：** 掲載コードの実行結果は、この原稿の作成時点では未検証。読者が `cargo test` で再現することを前提とする。

## 31.14 オフチェーンのポリシーとオンチェーンの強制を区別する

上のRustプログラムをウォレットに組み込めば、通常の操作で誤送金を防ぐ助けになる。しかし、攻撃者が別のウォレットを使えば、その検証を迂回できる可能性がある。

```text
Wallet Policy ──> ウォレットを通した操作のみ制約
Consensus Rule ─> ネットワークで受理される支出を制約
```

**「アプリで拒否する」ことと「ネットワーク全体で拒否する」ことは、セキュリティ上まったく異なる。**

## 31.15 テスト戦略

Covenantの検証では正常系だけでなく、攻撃者が意図的に境界条件を突くケースを重視する。

- 許可されていない受取先への支出
- 複数出力のうち1つだけ不正な支出
- `u64` の加算オーバーフロー
- 手数料を考慮しない過剰支出
- 残額を制約のないUTXOに移す抜け道
- 異なるトランザクション形式による制約回避
- 同一UTXOの二重消費
- 回復鍵を紛失した場合の資金凍結
- Reorganization後の状態の再評価

テストでは、**実装したルールが本当にすべての支出経路を制約しているか**を確認する。

## 31.16 Property-based Testingの発想

具体例だけではなく、任意の入力に対して成立すべき性質を定義する。

```text
任意の提案 p について：
validate_spend(p) == Ok
  ⇒ すべての出力先が許可済み
  ∧ 出力総額 + 手数料 ≤ 入力額
```

この性質を `proptest` などで検証すると、手作業では思いつかない組み合わせを発見しやすい。なお、実際のScript検証器の性質を証明するには、対応する仕様と実装を対象にする必要がある。

## 31.17 C4モデルで見る構成

```text
[User]
   │
   ▼
[Wallet / Application]
   │  支出提案・UI上のポリシー確認
   ▼
[Transaction Builder]
   │  署名対象の構築
   ▼
[Kaspa Node RPC]
   │  トランザクションの伝播
   ▼
[Consensus Validation]
   │  ネットワーク規則に従う検証
   ▼
[UTXO Set]
```

Covenantのオンチェーン強制は、最後のコンセンサス検証において、**当該ネットワークで有効なスクリプト規則**として実現されている場合に限る。

## 31.18 Kaspaにおける実装状況の確認

Kaspaの高度なスクリプトを論じるときは、以下を必ず区別する。

1. 論文や議論で提案されている機能
2. KIPに仕様が書かれた機能
3. `rusty-kaspa`にコードが存在する機能
4. 対象ネットワークで有効化されている機能
5. ウォレットやSDKから実際に利用できる機能

**コードが存在するだけで、メインネットで利用可能とは限らない。**

調査の出発点：

- [Kaspa KIPs](https://github.com/kaspanet/kips)
- [rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)
- [Kaspa Wiki](https://wiki.kaspa.org/)

リポジトリ内では、対象バージョンを固定して、スクリプト検証・Opcode定義・トランザクション検証・ネットワーク有効化条件に関連する実装を探す。ファイル名やAPIは変更される可能性があるため、本章では未確認の具体的なパスを断定しない。

## 31.19 KIPを読むときのチェックリスト

KIPを読む際には、次を記録する。

| 項目 | 確認する内容 |
|---|---|
| Motivation | 既存の仕組みで何ができないか |
| Specification | 何をどの条件で検証するか |
| Compatibility | 既存トランザクションへの影響 |
| Activation | どのネットワークで、いつ有効になるか |
| Security | 攻撃面・検証コスト・資金凍結リスク |
| Tests | 仕様を確認するテストベクトル |
| Implementation | 仕様に対応するコードとコミット |

KIP番号と内容は変更・追加される可能性がある。**最新版の一次資料を参照し、ステータスを原稿に記録すること。**

## 31.20 CovenantとL2・ZKの関係

Covenantは、L2やブリッジを構成するための**一つの部品**になり得る。しかし、それだけでL2や安全なブリッジが完成するわけではない。

```text
L1 Script / Covenant
      │
      ├── 資金の拘束条件
      ├── 支出・解放条件
      └── コミットメントの検証

L2 / Off-chain
      │
      ├── 状態更新
      ├── 証明生成
      └── データ可用性・運用
```

ゼロ知識証明をL1で検証するには、証明の検証方式や必要なプリミティブが利用可能である必要がある。**CovenantとZK証明は別の技術**であり、組み合わせられるかどうかは具体的な仕様に依存する。

## 31.21 セキュリティ設計：失敗モードから考える

| 失敗モード | 影響 | 対策の考え方 |
|---|---|---|
| 制約の抜け道 | 想定外の資金移動 | すべての出力・支出経路を検査 |
| 厳しすぎる制約 | 資金が動かせない | 回復経路・形式検証・テスト |
| 鍵の漏えい | 不正な署名 | 複数鍵・遅延・監視 |
| 手数料変動 | 出金不能 | 手数料設計の余地を確保 |
| 仕様の誤読 | コンセンサス不一致 | KIP・コード・テストの照合 |
| アップグレード | 互換性問題 | バージョン・有効化条件の記録 |

## 31.22 実践課題A：ウォレットの支出ポリシー

Rustのサンプルを拡張し、以下を実装する。

1. 複数の許可先を登録する。
2. 1トランザクションあたりの最大支出額を設定する。
3. 制約違反の理由をユーザーに表示する。
4. `cargo test` で正常系・異常系を検証する。
5. この制約が**オフチェーンでしか強制されない**ことをREADMEに明記する。

## 31.23 実践課題B：Covenantの仕様書を書く

次の見出しで設計文書を作成する。

```markdown
# Vault Covenant Design (Concept)

## Problem
## Goals / Non-goals
## State Model
## Allowed Transitions
## Invariants
## Threat Model
## Recovery Path
## Required Script Capabilities
## Test Cases
## Open Questions
```

**Required Script Capabilities** には、現時点でKaspaに存在する機能と、仮定している未実装機能を分けて記述する。

## 31.24 実践課題C：一次資料との対応表

| 要件 | KIP・仕様 | Rust実装 | 有効化状態 | テスト |
|---|---|---|---|---|
| 出力先制約 | 要調査 | 要調査 | 要調査 | 要作成 |
| 時間条件 | 要調査 | 要調査 | 要調査 | 要作成 |
| 継続制約 | 要調査 | 要調査 | 要調査 | 要作成 |

空欄を推測で埋めず、確認できない項目は「未確認」と明記する。これは『Mastering Kaspa』全体の技術的信頼性を高めるための作業でもある。

## 31.25 理解度チェック

1. 署名条件とCovenantは何が違うか。
2. Covenantの「制約」がどこで強制されるかは、なぜ重要か。
3. 再帰的Covenantで資金凍結が起こり得るのはなぜか。
4. トランザクションの内省が必要になる例を説明できるか。
5. オフチェーンのRustバリデーターだけでは防げない攻撃は何か。
6. KIPの存在とメインネット有効化を区別するには何を調べるべきか。
7. CovenantとZK証明はどう異なるか。
8. 自分の設計したVaultの不変条件を3つ挙げられるか。

## 31.26 まとめ

- ScriptはUTXOを消費するための条件を表現する。
- Covenantは、消費によって生成される出力や次の状態に制約を課す考え方である。
- 高度な表現力には、検証コスト・互換性・監査難易度というトレードオフがある。
- **ウォレット側の制約と、コンセンサスで強制される制約は別物**である。
- Kaspaで利用可能な機能は、KIP・実装・有効化状況を突き合わせて確認する。
- 安全な設計では、状態遷移、不変条件、攻撃モデル、回復経路を先に定義する。

次章では、こうしたスクリプトの設計をより読みやすく記述するためのアプローチとして、**Silverscriptとスマートコントラクト設計**を扱う。

## 参考資料

- [Kaspa KIPs — GitHub](https://github.com/kaspanet/kips)
- [rusty-kaspa — GitHub](https://github.com/kaspanet/rusty-kaspa)
- [Kaspa Wiki](https://wiki.kaspa.org/)
- [Bitcoin Developer Reference — Transactions](https://developer.bitcoin.org/reference/transactions.html)（比較資料。Kaspaの仕様ではない）
- [Bitcoin Optech — Covenants topic](https://bitcoinops.org/en/topics/covenants/)（比較資料。Kaspaへの適用を意味しない）
