# Chapter 31 — Covenants & Advanced Scripting（Covenantsと高度なスクリプティング）

> **目次順の本文レビュー：2026-11-01。主要な固定KIP/基準版ソースと独自教材を確認。全仕様・本番有効化の検証ではない。**
> 本章は仕組みと設計の学習を目的とする。Kaspaで実際に利用できるOpcode、スクリプトバージョン、ネットワークでの有効化状況は、対象のノード実装・KIP・ネットワーク設定で必ず確認する。例示するCovenantは概念モデルであり、現在のKaspaメインネットでそのまま動くことを意味しない。

基準版はv1.0.1 / `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`、KIP文書は第30章と同じ `e4ae2332117b5cb68bd6188e065ef885b6d17939`。両者は同時点の仕様集合ではない。確認記録は `experiments/chapter-31-review-2026-11-01.md`。

## 31.1 この章で学ぶこと

たとえば、署名が正しいことに加えて「資金を決められた形式の出力へ引き継ぐこと」まで要求したい場合を考える。Covenantは、このように後続のトランザクションへ制約を課す仕組みである。本章では、支出条件を具体例から整理し、その条件を満たす場合と拒否すべき場合をテストする。

学習目標：

1. 署名条件とCovenantの違いを説明できる。
2. UTXOとトランザクションを用いて、状態遷移の不変条件を定義できる。
3. Script、Covenant、オフチェーン検証、L2の責務を区別できる。
4. Kaspaの実装状況を一次資料で確認できる。
5. Rustで簡略化した制約エンジンを実装・テストできる。

## 31.2 前提：UTXOとScript

UTXO（Unspent Transaction Output）は、まだ消費されていないトランザクション出力である。トランザクションは既存のUTXOを入力として参照する。Script検証の成功に加え、他の妥当性検査を通り、対象の受理順序・状態で適用されて初めてUTXO集合が更新される。Mempoolへの受付を適用済みと扱わない。

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

通常の署名条件は、ある鍵による承認と署名対象への拘束を検証する。「誰が消費できるか」はその要約であり、署名が出力等へコミットしないという意味ではない。Covenantは、これに加えて「**消費した結果、どのような出力を作らなければならないか**」などを制約する考え方である。

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

「出力先にも同じ制約を持たせる」ことで、複数回の支出にわたりポリシーを継続する。出力スクリプトの一致だけでなく、そのバージョン、引き継ぐ状態/金額、制約のない出力へ逃がせる値も検査する。複数入力が同じ出力をそれぞれ継続先として数える設計では、集約後の保存条件も必要になる。制約の解除条件を誤ると、資金が永久に動かせなくなる可能性がある。

### 31.6.4 Vaultと回復経路

通常の支出と緊急回復を分ける。回復鍵、時間条件、キャンセル経路、監視システムを一体として設計する必要がある。

待機中に攻撃者も有効な支出を作れる設計では、回復トランザクションの作成だけで先に受理されるとは限らない。競合、監視・通知の欠落、手数料、時間条件の単位と再編成を脅威モデルへ含める。

## 31.7 Covenantとスマートコントラクトの違い

```text
UTXO + Script / Covenant
  └─ 入力の消費可否と出力の制約を検証

アカウント型スマートコントラクトの例
  └─ コントラクトが保持する状態に対して処理を実行
```

UTXO型でもコントラクトの状態を表現できるため、これはCovenantとスマートコントラクトを排他的な分類にする図ではない。両者ともプログラム可能なルールを表現できるが、状態の持ち方、実行モデル、並行処理、開発者体験が異なる。**Covenantがあれば任意のEVMアプリをそのまま実行できる、という意味ではない。**

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

署名対象はSIGHASHフラグ等で変わる。[第16章](../part-03-transactions/chapter-16.md)の基準版との対応を参照し、承認後に変えられる出力や手数料の範囲も確認する。

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

たとえば、各状態を特定のスクリプトやコミットメントを持つUTXOで表現する。状態名だけを永続的な重複キーとすると、再編成後の正当な再受理を捨て得るため、outpoint・遷移・観測履歴を区別する。遷移が成立するための条件を定義し、許可されていない遷移を拒否する。

## 31.11 不変条件（Invariants）

設計では「何を実現したいか」より先に「**絶対に起きてはいけないこと**」を明確にすると検証しやすい。

| 不変条件 | 検証したいこと |
|---|---|
| 価値の保存 | 入力総額を超える出力を生成しない（発行等の特別規則を除く） |
| 許可先の限定 | 指定外のスクリプトへ移動しない |
| 制約の継承 | 継続すべき制約が次のUTXOにも残る |
| 単一消費 | 同じUTXOを二重に消費できない |
| 回復可能性（進行性の要件） | 必要な鍵・データ・時間/資源条件の下で緊急経路を使える |
| コスト上限 | 検証処理が許容範囲に収まる |

回復可能性は「不正な遷移が起きない」という安全性の不変条件だけでは示せない。必要な鍵・データの保持と、競合・時間条件・ネットワーク環境を含む進行性も別に検証する。

## 31.12 DDDによる責務分離

概念的な設計を、以下のように分離できる。

```text
Application ───────────▶ Domain
  ├─ CreateSpendProposal  ├─ Utxo / Transaction
  └─ ValidateSpendProposal├─ SpendPolicy
          │              └─ PolicyViolation
          ▼
Ports / Interfaces ◀──── Infrastructure
                         ├─ ScriptAdapter
                         ├─ NodeRpcAdapter
                         └─ StorageAdapter
```

アプリケーションが使うインターフェースをアダプターが実装し、Domainの純粋な検査をRPCやDBへ依存させない構成を考える。図は支出時の実行順を示すものではない。

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
    pub expected_fee: u64,
    pub outputs: Vec<Output>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PolicyError {
    UnauthorizedDestination,
    ValueOverflow,
    InsufficientInput,
    EmptyOutputs,
    EmptyDestination,
    UnexpectedFee,
}

pub fn validate_spend(
    proposal: &SpendProposal,
    allowed_destinations: &[&str],
) -> Result<(), PolicyError> {
    if proposal.outputs.is_empty() {
        return Err(PolicyError::EmptyOutputs);
    }

    for output in &proposal.outputs {
        if output.destination.is_empty() {
            return Err(PolicyError::EmptyDestination);
        }
        if !allowed_destinations.contains(&output.destination.as_str()) {
            return Err(PolicyError::UnauthorizedDestination);
        }
    }

    let total_outputs = proposal.outputs.iter().try_fold(0_u64, |acc, output| {
        acc.checked_add(output.amount)
            .ok_or(PolicyError::ValueOverflow)
    })?;

    let required = total_outputs
        .checked_add(proposal.expected_fee)
        .ok_or(PolicyError::ValueOverflow)?;

    if required > proposal.input_amount {
        return Err(PolicyError::InsufficientInput);
    }

    if required != proposal.input_amount {
        return Err(PolicyError::UnexpectedFee);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proposal(destination: &str, amount: u64) -> SpendProposal {
        SpendProposal {
            input_amount: 100,
            expected_fee: 1,
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

    #[test]
    fn rejects_implicit_excess_fee() {
        assert_eq!(
            validate_spend(&proposal("safe", 98), &["safe"]),
            Err(PolicyError::UnexpectedFee)
        );
    }

    #[test]
    fn checks_every_output() {
        let mut p = proposal("safe", 50);
        p.outputs.push(Output {
            destination: "attacker".into(),
            amount: 49,
        });
        assert_eq!(
            validate_spend(&p, &["safe"]),
            Err(PolicyError::UnauthorizedDestination)
        );
    }

    #[test]
    fn accepts_multiple_outputs_and_their_reordering() {
        let mut p = proposal("safe", 40);
        p.outputs.push(Output {
            destination: "recovery".into(),
            amount: 59,
        });
        assert_eq!(validate_spend(&p, &["safe", "recovery"]), Ok(()));
        p.outputs.reverse();
        assert_eq!(validate_spend(&p, &["safe", "recovery"]), Ok(()));
    }

    #[test]
    fn rejects_empty_output_set_and_empty_allowlist() {
        let mut p = proposal("safe", 99);
        assert_eq!(
            validate_spend(&p, &[]),
            Err(PolicyError::UnauthorizedDestination)
        );
        p.outputs.clear();
        assert_eq!(
            validate_spend(&p, &["safe"]),
            Err(PolicyError::EmptyOutputs)
        );
    }

    #[test]
    fn empty_label_is_not_authorized_even_if_allowlisted() {
        assert_eq!(
            validate_spend(&proposal("", 99), &[""]),
            Err(PolicyError::EmptyDestination)
        );
    }

    #[test]
    fn rejects_output_plus_expected_fee_overflow() {
        let mut p = proposal("safe", u64::MAX);
        p.input_amount = u64::MAX;
        assert_eq!(
            validate_spend(&p, &["safe"]),
            Err(PolicyError::ValueOverflow)
        );
    }

    #[test]
    fn accepts_exact_balance_with_zero_expected_fee() {
        let mut p = proposal("safe", 100);
        p.expected_fee = 0;
        assert_eq!(validate_spend(&p, &["safe"]), Ok(()));
    }
}
```

expected_feeは呼出し側の予定手数料で、ノードへ送る独立したfeeフィールドではない。基準版の[UTXO文脈の検査](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/transaction_validator/tx_validation_in_utxo_context.rs)では、通常取引の入力総額−出力総額が手数料になる。残額を「未使用額」として許さず、出力合計+予定手数料=入力額を要求する。釣り銭を戻す場合も、許可先への出力として明示する。

input_amountは正しいUTXOから解決・合計済みという仮定で、destinationは架空の識別ラベルである。実アドレスの検査、ScriptPublicKeyのversion/bytes比較、複数入力の存在・単一消費、署名・時間・再帰的な継続制約をこの関数は検証しない。u64全域の教材上の判定を、ネットワークの供給上限や最小手数料の検証とも扱わない。

### 実行方法

```bash
# 本書の外の調査用ディレクトリで実行
cargo new covenant-lab --lib --edition 2021
cd covenant-lab
# 上記コードを src/lib.rs に配置
cargo test --offline
cargo fmt --all -- --check
cargo clippy --offline --all-targets -- -D warnings
```

**検証状態：** Rust/Cargo 1.90.0、edition 2021、外部依存なしの一時Cargo libで本文を抜き出して検査した。結果は確認記録に示す。実Kaspa Scriptの実行試験ではない。

## 31.14 オフチェーンのポリシーとオンチェーンの強制を区別する

上のRustプログラムをウォレットに組み込めば、通常の操作で誤送金を防ぐ助けになる。署名する実トランザクションから再構築した情報で、署名前に同じポリシーを評価することも必要になる。しかし、攻撃者が別のウォレットを使えば、その検証を迂回できる可能性がある。

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
validate_spend(p, 許可先) == Ok
  ⇒ すべての出力先が許可済み
  ∧ 出力総額 + 予定手数料 = 入力額
```

ここで量化するのは、正しい入力額を与えた独自教材の提案である。全入力を網羅する証明やproptestの実行は今回行っていない。この性質を `proptest` などで検証すると、手作業では思いつかない組み合わせを発見しやすい。なお、実際のScript検証器の性質を証明するには、対応する仕様と実装を対象にする必要がある。

## 31.17 C4モデルで見る構成

```text
[User]
   │
   ▼
[Wallet / Application]
   ├── 支出ポリシーの評価
   └── Transaction Builder / Signer
             │ 署名済み取引の送信
             ▼
       [Kaspa Node RPC]
             │ 受付・検証要求
             ▼
       [Mining Manager / Mempool] ──▶ [P2P peers]
             │ コンセンサス規則による検査
             ▼
       [Transaction Validator]

[Block Processing / Virtual]
             │ 受理順序・状態の評価を経た適用
             ▼
          [UTXO Set]
```

これは責務の概念図であり、全呼び出し順や単一の保存処理ではない。RPC/Mempool受付だけではUTXO集合は更新されない。[第17章](../part-03-transactions/chapter-17.md)と[第26章](../part-04-development/chapter-26.md)の検査・適用の区別を参照する。

Covenantのオンチェーン強制は、コンセンサス検証において、**当該ネットワークで有効なスクリプト規則**として実現されている場合に限る。

## 31.18 Kaspaにおける実装状況の確認

Kaspaの高度なスクリプトを論じるときは、以下を必ず区別する。

1. 論文や議論で提案されている機能
2. KIPに仕様が書かれた機能
3. `rusty-kaspa`にコードが存在する機能
4. 対象ネットワークで有効化されている機能
5. ウォレットやSDKから実際に利用できる機能

**コードが存在するだけで、メインネットで利用可能とは限らない。**

### 固定文書と基準版の対応

| 対象 | 固定KIPの記述 | 基準版v1.0.1で照合した範囲 |
|---|---|---|
| [KIP-10](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0010.md) | 内省命令7種と8-byte数値処理、StatusはActive | OpTxInputCount/OutputCount/InputIndex、InputAmount/InputSpk/OutputAmount/OutputSpkはkip10_enabledで分岐 |
| [KIP-17](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0017.md) | 内省・バイト列操作等を拡張しToccataの有効化条件を記述、StatusはActive | 例えばOpTxVersion/OpTxLockTimeは予約エラーを返す。この固定ソースを後期KIPの実装と同一視しない |
| [KIP-20](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0020.md) | covenant_idで系譜を追う仕組み、StatusはActive | UtxoEntry/TransactionOutputには、この改訂のcovenant_id/covenantフィールドがない |

照合先は[Opcode定義](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/crypto/txscript/src/opcodes/mod.rs)、[UTXO文脈の検証](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/transaction_validator/tx_validation_in_utxo_context.rs)、[UTXO/出力型](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/tx.rs)。基準版のcheck_scriptsはblock_daa_scoreでcrescendo_activationを評価し、KIP-10の有効/無効をエンジンへ渡す。

金額内省は値をsompi単位のScript数値へ変換し、出力/入力のSPK内省はバージョンを含むバイト列を返す。アドレス文字列との単純な文字列比較へ置き換えない。具体的な命令の存在だけで、任意のVault・再帰制約・回復経路が実現できるとは判断しない。

KIP-20の系譜識別と出力のbinding検証があっても、期待する次状態・出力Script・継続条件は正しく設計したCovenant側で強制する必要がある。IDを付けるだけでアプリケーションの遷移が正しくなるわけではない。

ここで確認したのは固定文書の主要説明と固定ソースの該当箇所である。全命令、スクリプト上限・数値エンコード、全ネットワークの有効化、SDK経由の利用は未検証。現在の本番対応は実装コミット・リリース・対象ネットワークを別に照合する。[第30章](chapter-30.md)の文書Statusの読み方も参照する。

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

KIP番号だけではなく、本文URL・コミット・PR・調査日を記録する。文書の改訂とネットワークの有効化を別に調べ、将来の更新では固定版との差分を確認する。

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

1. 複数の許可先を設定から登録・更新し、検証へ渡す。
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

[次章](chapter-32.md)では、こうしたスクリプトの設計をより読みやすく記述するためのアプローチとして、**Silverscriptとスマートコントラクト設計**を扱う。

## 参考資料

- [KIP-10 — 固定版](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0010.md)
- [KIP-17 — 固定版](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0017.md)
- [KIP-20 — 固定版](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0020.md)
- [rusty-kaspa — 基準版のソース](https://github.com/kaspanet/rusty-kaspa/tree/fcd9c28f9b211b9872a0c90a8c94f79466ffb063)
- [Bitcoin Developer Reference — Transactions](https://developer.bitcoin.org/reference/transactions.html)（比較資料。Kaspaの仕様ではない）


**確認範囲：** 本文・主要KIP/基準版の該当箇所と独自教材。実Vault、公式Script試験・暗号方式の監査、現行Mainnet/全SDKの有効化確認は行っていない。記録は `experiments/chapter-31-review-2026-11-01.md`。
