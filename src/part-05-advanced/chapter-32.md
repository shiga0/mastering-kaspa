# Chapter 32 — Silverscript & Smart Contract Design（Silverscriptとスマートコントラクト設計）

> **目次順の本文レビュー：2026-11-01。固定Silverscript文書/CLIの主要箇所と独自Rust教材を確認。CLIコンパイルも確認。追加検証2026-11-02：2-of-2のローカルVM評価5件・ABI拒否1件を確認。本番利用は未検証。**
> 本章は設計の考え方と検証方法を中心に扱う。Silverscriptの文法、対応オペコード、コンパイラの仕様、Kaspaメインネットでの利用可能性は、使用する実装のバージョンと有効化状況を必ず確認する。第32.10節は固定CLIでのコンパイル確認例であり、他の概念図や独自Rustモデルと検証範囲を分ける。

Silverscriptの参照版はv1.0.0 / `3ed973335b59269293564805cc2c58a14595ec03`。本書のノード基準版v1.0.1とは別の資料である。確認記録は `experiments/chapter-32-review-2026-11-01.md`。追加のVM検査は `experiments/silverscript-vm-review-2026-11-02.md`。

## 32.1 この章で学ぶこと

前章ではCovenantを「UTXOの次の支出方法まで制約する仕組み」として学んだ。本章では、そのような制約を人間が理解し、レビューし、テストできる形で記述するための言語と設計方法を考える。

学習目標は次のとおり。

1. スクリプト言語、コントラクト言語、実行環境を区別する。
2. Silverscriptを使う場合の設計・コンパイル・検証の流れを説明する。
3. UTXOベースのコントラクトを状態遷移と不変条件で設計する。
4. オンチェーンの強制条件とオフチェーンのアプリケーション処理を分離する。
5. Rustによる参照モデルとテストから、仕様の曖昧さを発見する。

---

## 32.2 なぜ高水準言語が必要なのか

低水準のスタック型スクリプトは、実行時の意味が明確でも、人間にとって読みやすいとは限らない。

```text
業務上のルール
  「受取人の署名がある場合だけ送金できる」
              │
              ▼
高水準のコントラクト記述
              │ コンパイル
              ▼
低水準のスクリプト
              │ 実行
              ▼
検証結果：許可／拒否
```

高水準言語は**検証ルールの記述を助ける道具**であって、基盤プロトコルに存在しない能力を自動的に追加するものではない。たとえば「次のトランザクションの出力を検査する」という命令が基盤側で使えなければ、言語の構文だけで同等の強制力を実現できるとは限らない。

## 32.3 Silverscriptとは何か

Silverscriptは、CashScriptに着想を得た、Kaspa Scriptを生成する言語とコンパイラーである。[公式README](https://github.com/kaspanet/silverscript/blob/3ed973335b59269293564805cc2c58a14595ec03/README.md)と[チュートリアル](https://github.com/kaspanet/silverscript/blob/3ed973335b59269293564805cc2c58a14595ec03/docs/TUTORIAL.md)を起点にする。[v1.0.0のリリース](https://github.com/kaspanet/silverscript/releases/tag/v1.0.0)と、そのタグの完全SHAを確認した。

ただし、技術書では次の事柄を混同しない。

| 対象 | 確認すべきこと |
|---|---|
| 言語仕様 | 文法、型、組み込み関数、コンパイル規則 |
| コンパイラ | 入力ソースから何を生成するか、再現可能か |
| 実行環境 | 生成物がどのスクリプトVMで実行されるか |
| プロトコル | 必要なオペコード・トランザクション内省機能が有効か |
| デプロイ | 実際にどのネットワークで利用可能か |
| ツール群 | テスト、デバッグ、監査、アドレス生成への対応 |

**重要：** Silverscriptを利用したサンプルが存在することは、Kaspaメインネットで任意のCovenantが利用可能であることの証明にはならない。

### 32.3.1 実装状況を調べる手順

1. 対象のSilverscriptリポジトリとコミットを記録する。
2. README、言語仕様、テスト、リリース情報を読む。
3. サンプルをコンパイルし、生成物を保存する。
4. 生成物が要求する命令と、対象ネットワークの有効なルールを照合する。
5. Devnetなどの隔離環境で実行し、成功例と失敗例を確認する。

固定版の[Cargo.toml](https://github.com/kaspanet/silverscript/blob/3ed973335b59269293564805cc2c58a14595ec03/Cargo.toml)はedition 2024 / rust-version 1.94.0を指定し、Kaspa関連依存を `a41a333b08848f41bf737b72592e463a6011b8ac` へ固定する。本書の基準ノードやRust教材の1.90.0へそのまま置き換えられる構成ではない。

### 32.3.2 CLIの読み方と未実行の範囲

以下は固定READMEと[silvercの実装](https://github.com/kaspanet/silverscript/blob/3ed973335b59269293564805cc2c58a14595ec03/silverscript-lang/src/bin/silverc.rs)で確認した手順である。実際には第32.10節の教材ソースと公開点のconstructor引数を与えてコンパイルした。本書とは別の調査用ディレクトリで、要求版のRustと依存のビルド環境を整えて使う。

```bash
git clone --branch v1.0.0 --depth 1 https://github.com/kaspanet/silverscript.git silverscript-v1.0.0
git -C silverscript-v1.0.0 rev-parse HEAD
cd silverscript-v1.0.0
# HEADを本章冒頭の完全SHAと照合する
cargo +1.94.0 run --locked -p silverscript-lang --bin silverc -- contract.sil -o contract.json
```

contract.silは読者が用意するソースの仮名である。コンストラクター引数があれば、portable ABI値の配列を[引数JSONの仕様](https://github.com/kaspanet/silverscript/blob/3ed973335b59269293564805cc2c58a14595ec03/docs/CONSTRUCTOR_ARGS.md)に従って用意し、--constructor-argsで渡す。CLIは既定でソースと同じ場所へJSONを書き、-oで保存先、-cで標準出力を選ぶ。--ast-onlyはASTの出力で、コンパイル/生成Scriptの妥当性確認とは別である。

既定のstableはRust 1.90.0のまま、要求版の1.94.0を追加してcargo +1.94.0で呼び分ける。Rust教材の検査と、公式コンパイラー/VMの実行結果は確認記録で分けて示す。

---

## 32.4 Bitcoin Script・Kaspa Script・高水準言語の関係

```text
[アプリケーションの要件]
            │
            ▼
[高水準のコントラクト言語]  ← Silverscriptのような層
            │
            ▼
[スクリプト／トランザクション形式]
            │
            ▼
[ノードによるコンセンサス検証]
```

コンパイラは**記述の変換**を行う。一方、ノードは**プロトコルの合意規則に従った検証**を行う。この責任境界を明確にする。

| レイヤー | 主な責務 | 失敗例 |
|---|---|---|
| UI・API | ユーザーの操作を受け付ける | 金額の入力ミス |
| ウォレット | 鍵管理・署名 | 誤った鍵で署名 |
| コンパイラ | ソースをスクリプトへ変換 | 意図しない条件式へ変換 |
| Script VM | スクリプトを評価 | 条件を満たさず失敗 |
| コンセンサス | トランザクションを受理するか判断 | 不正なトランザクションを拒否 |

## 32.5 EVM型スマートコントラクトとの違い

EVM型のアカウントモデルでは、コントラクトの永続ストレージを更新する設計が一般的である。一方、UTXO型の設計では、コインの所有条件と支出によって状態遷移を表現することが多い。

```text
EVM型の概念図
  Contract Storage S0
          │ call
          ▼
  Contract Storage S1

UTXO型の概念図
  UTXO（条件・値）
          │ 支出トランザクション
          ▼
  新しいUTXO（次の条件・値）
```

ただし、**すべてのUTXOコントラクトが再帰的な状態遷移を強制できるわけではない**。それには、必要な内省命令やCovenant機能などの基盤側の支援が要る。

## 32.6 コントラクト設計の出発点は要件

例として、2人の承認者が共同で管理する資金を考える。

### 機能要件

- 2人の承認者がそろったときだけ資金を移動できる。
- 不正な署名では移動できない。
- 資金の移動履歴を追跡できる。

### 非機能要件

- 秘密鍵をサーバーに保存しない。
- 署名対象と金額をユーザーが確認できる。
- 不正な入力を再現可能なテストで拒否できる。

### 非目標

- 管理者が任意に他人の資金を回収する機能。
- ネットワークのコンセンサスを変更すること。

## 32.7 ドメインモデルを作る

```text
Vault（保管条件）
 ├─ Participants（承認者）
 ├─ Threshold（必要署名数）
 ├─ LockedValue（拘束される価値）
 └─ SpendingPolicy（支出ポリシー）

SpendAttempt（支出試行）
 ├─ Transaction
 ├─ Witnesses（署名など）
 └─ EvaluationResult
```

**ドメインモデルは、実装言語の構造体と同一ではない。** まず業務上の意味を定義し、それをRustやコントラクト言語へ写像する。

## 32.8 不変条件を先に定義する

コントラクトの安全性は「正常系で動くこと」だけでは判断できない。**絶対に破ってはいけない条件**を先に書く。

| ID | 不変条件 | 検証例 |
|---|---|---|
| INV-01 | 必要署名数を満たさない支出を拒否する | 署名1件では拒否 |
| INV-02 | 同じ鍵を複数の承認者として数えない | 設定・証拠の両方で重複を検査 |
| INV-03 | 許可されていない承認者を数えない | 第三者の署名を無視／拒否 |
| INV-04 | 署名が対象トランザクションに結び付く | 別トランザクションの署名を拒否 |
| INV-05 | UTXOは二重に消費できない | 同一入力の競合を検証 |

承認者の文字列IDが異なっても同じ公開鍵へ対応するなら、独立した2人の承認にはならない。署名前の鍵の同一性・正規化も定義する。

INV-05は通常、個別のスクリプトだけでなく**UTXOモデルとコンセンサス検証**が担う。責務の所在を取り違えない。

## 32.9 ステートマシンで支出を考える

```text
        ┌─────────────┐
        │ Unspent     │
        └──────┬──────┘
               │ 支出を試行
               ▼
        ┌─────────────┐
        │ Evaluating  │
        └──────┬──────┘
           ┌───┴────┐
       不正│        │ 正当
           ▼        ▼
       Rejected   Accepted
                    │
                    ▼
                 Spent
```

これは理解のための**概念的な状態図**であり、Kaspaノードの内部状態名を示すものではない。また、Mempoolでの受付、Virtual等の文脈での受理・UTXO適用、アプリの確定ポリシーは別である。UTXOのRejectedは支出試行の結果であり、そのUTXO自体が常に消費済みになるわけではない。再編成で受理/未受理が変わる可能性もあり、状態名だけで履歴を永久に重複排除しない。

## 32.10 支出条件をSilverscriptで書く

以下は固定v1.0.0のCLIでコンパイルを確認した学習用ソースである。32.15.1では、同じソースから生成したScriptへ実際の署名を渡し、ローカルVMでの許可・拒否まで確認する。出力先や継続状態のCovenantを実装した例ではなく、資金投入やノードへの送信は行わない。

```text
{{#include ../../experiments/silverscript-vm/two-of-two.sil}}
```

[COMPILER_VERSION](https://github.com/kaspanet/silverscript/blob/3ed973335b59269293564805cc2c58a14595ec03/silverscript-lang/src/compiler/mod.rs)はこの版で0.1.0。リリース/パッケージのv1.0.0とは別で、生成JSONのcompiler_versionも0.1.0だった。^1.0.0のpragmaは拒否されたため、タグ名から言語pragmaを作らない。

constructor引数のpubkeyは、[portable ABI](https://github.com/kaspanet/silverscript/blob/3ed973335b59269293564805cc2c58a14595ec03/docs/CONSTRUCTOR_ARGS.md)ではkind: bytesの配列で表す。kind: pubkeyは拒否される。初回のCLI試験では既知の公開点G/2Gをx-onlyの32-byte引数に使用した。そのCLI試験では鍵生成・署名・ネットワークへの送信は行っていない。32.15.1のVM試験では、別途一時鍵を生成して署名する。

2回のコンパイルでJSONは同一、デコードしたScriptは168 bytesだった。これは同一の実行環境での再実行一致であり、異なるマシン間の再現ビルドや意味の正しさを証明しない。JSON/Scriptのhashと引数は確認記録へ残した。--ast-onlyの出力と、不正文法・不正なABIのkind・未対応pragmaの拒否も確認した。

公式の別例は[2_of_3_multisig.sil](https://github.com/kaspanet/silverscript/blob/3ed973335b59269293564805cc2c58a14595ec03/silverscript-lang/tests/examples/2_of_3_multisig.sil)。そのcheckMultiSigと、以下の文字列IDを数えるRust教材は同一の受理条件を再現したものではない。

この例から、実装前に少なくとも次の点を決める必要がある。

- `transaction` のどの部分に署名するのか。
- 署名の再利用やリプレイを防ぐ方法は何か。
- 鍵の順序・形式・重複をどう扱うか。
- スクリプトのサイズや実行コストは許容範囲か。
- 対象ネットワークで必要な検証機能が利用できるか。

## 32.11 オンチェーンとオフチェーンの境界

```text
オフチェーン                         オンチェーン
───────────                         ──────────
UI入力                               UTXOの存在
送金内容の表示                       スクリプト条件の評価
署名の収集                           署名等の検証
トランザクションの組み立て           二重支払い防止
監視・通知                           コンセンサスによる順序決定
```

UIやAPIで「承認済み」と表示しても、オンチェーンの条件がそれを強制しなければ安全性の根拠にならない。

## 32.12 コンパイル工程の検証

コントラクト言語を使う場合、**ソースコードだけでなく生成された成果物も検証対象**にする。

```text
Source
  │
  ▼
Parser → AST → Type Check → Compiler
                              │
                              ▼
                         Script Artifact
                              │
                              ▼
                     VM / Integration Test
```

最低限記録する情報：

1. コンパイラのバージョン・コミット。
2. ソースのハッシュ。
3. 生成されたスクリプトのハッシュ。
4. コンパイルオプション。
5. 実行したテストベクトルと結果。
6. constructor引数・依存/lockfile・ビルド環境、対象VM/ネットワークと有効化条件。
7. hash対象のバイト列とアルゴリズム（JSON全体か、デコードしたScriptか等）。

ABIやconstructor引数、エントリー選択・引数のエンコードも支出条件に関わる。JSONファイルのhashと生成Scriptのhashを混同しない。記録は再現性の照合に必要だが、同じ出力が出ること自体は仕様の正しさを証明しない。

## 32.13 コンパイラを信頼しすぎない

高水準言語は読みやすさを改善する一方、コンパイラの不具合という新しいリスクを生む。

| リスク | 対策 |
|---|---|
| 意図と異なるコード生成 | 生成スクリプトの検査・差分テスト |
| コンパイラ更新による挙動変化 | バージョン固定・再現ビルド |
| 不明瞭な暗黙の変換 | 型と境界値のテスト |
| 想定外の実行コスト | サイズ・コストの計測 |
| ライブラリ依存の脆弱性 | 依存関係と変更履歴の確認 |

## 32.14 Rustで支出ポリシーの参照モデルを作る

ここからは、**実際の署名検証を行わない教育用モデル**を作る。目的は、仕様をコードとテストに落とし込むことである。

```rust
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultPolicy {
    owners: HashSet<String>,
    threshold: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PolicyError {
    InvalidThreshold,
    InvalidOwner,
    DuplicateOwner,
    NotEnoughApprovals,
    UnknownApprover,
    DuplicateApprover,
}

impl VaultPolicy {
    pub fn new(owners: Vec<String>, threshold: usize) -> Result<Self, PolicyError> {
        let mut unique = HashSet::new();
        for owner in owners {
            if owner.is_empty() {
                return Err(PolicyError::InvalidOwner);
            }
            if !unique.insert(owner) {
                return Err(PolicyError::DuplicateOwner);
            }
        }
        if threshold == 0 || threshold > unique.len() {
            return Err(PolicyError::InvalidThreshold);
        }
        Ok(Self {
            owners: unique,
            threshold,
        })
    }

    // approvals は同じ対象取引・署名条件に結び付く検証済みIDという仮定。
    // 本番では署名・署名対象・鍵を別途検証する必要がある。
    pub fn authorize(&self, approvals: &[String]) -> Result<(), PolicyError> {
        let mut seen = HashSet::new();
        for approver in approvals {
            if !self.owners.contains(approver) {
                return Err(PolicyError::UnknownApprover);
            }
            if !seen.insert(approver) {
                return Err(PolicyError::DuplicateApprover);
            }
        }
        if seen.len() < self.threshold {
            return Err(PolicyError::NotEnoughApprovals);
        }
        Ok(())
    }
}
```

owners/thresholdを非公開にし、new以外から不正な設定を作れないようにする。入力はVecで受け、HashSetへ変換する前に重複を拒否する。最初から集合へ変換すると、重複した設定を黙って捨ててしまう。

**このモデルはコンセンサス互換でも、Kaspa Scriptの実装でもない。** `String`で表した承認者IDは公開鍵ではなく、暗号署名の検証は省略している。同じ取引・SIGHASH/規則・鍵への結合と、IDから鍵への一意な対応を呼出し側へ仮定する。この関数だけではその前提の真偽を検査できない。閾値成立後も未知/重複IDを拒否する独自の厳格な方針であり、VMのcheckMultiSigと同じ受理条件だとは主張しない。

### 32.14.1 テスト

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> VaultPolicy {
        VaultPolicy::new(
            ["alice".to_string(), "bob".to_string()]
                .into_iter()
                .collect(),
            2,
        )
        .unwrap()
    }

    #[test]
    fn accepts_two_distinct_approvers() {
        assert_eq!(policy().authorize(&["alice".into(), "bob".into()]), Ok(()));
    }

    #[test]
    fn rejects_one_approver() {
        assert_eq!(
            policy().authorize(&["alice".into()]),
            Err(PolicyError::NotEnoughApprovals)
        );
    }

    #[test]
    fn rejects_duplicate_approver() {
        assert_eq!(
            policy().authorize(&["alice".into(), "alice".into()]),
            Err(PolicyError::DuplicateApprover)
        );
    }

    #[test]
    fn rejects_unknown_approver() {
        assert_eq!(
            policy().authorize(&["alice".into(), "mallory".into()]),
            Err(PolicyError::UnknownApprover)
        );
    }

    #[test]
    fn rejects_invalid_threshold() {
        assert_eq!(
            VaultPolicy::new(["alice".to_string()].into_iter().collect(), 2),
            Err(PolicyError::InvalidThreshold)
        );
    }

    #[test]
    fn rejects_zero_threshold_and_empty_owner_set() {
        assert_eq!(
            VaultPolicy::new(vec!["alice".into()], 0),
            Err(PolicyError::InvalidThreshold)
        );
        assert_eq!(
            VaultPolicy::new(vec![], 0),
            Err(PolicyError::InvalidThreshold)
        );
        assert_eq!(
            VaultPolicy::new(vec![], 1),
            Err(PolicyError::InvalidThreshold)
        );
    }

    #[test]
    fn rejects_duplicate_and_empty_configured_owners() {
        assert_eq!(
            VaultPolicy::new(vec!["alice".into(), "alice".into()], 2),
            Err(PolicyError::DuplicateOwner)
        );
        assert_eq!(
            VaultPolicy::new(vec!["".into()], 1),
            Err(PolicyError::InvalidOwner)
        );
    }

    #[test]
    fn rejects_empty_approvals() {
        assert_eq!(
            policy().authorize(&[]),
            Err(PolicyError::NotEnoughApprovals)
        );
    }

    #[test]
    fn checks_extra_evidence_after_threshold_is_met() {
        let p = VaultPolicy::new(vec!["alice".into(), "bob".into()], 1).unwrap();
        assert_eq!(p.authorize(&["alice".into()]), Ok(()));
        assert_eq!(
            p.authorize(&["alice".into(), "mallory".into()]),
            Err(PolicyError::UnknownApprover)
        );
        assert_eq!(
            p.authorize(&["alice".into(), "alice".into()]),
            Err(PolicyError::DuplicateApprover)
        );
    }

    #[test]
    fn any_two_of_three_are_authorized_in_either_order() {
        let p = VaultPolicy::new(vec!["alice".into(), "bob".into(), "carol".into()], 2).unwrap();
        for pair in [["alice", "bob"], ["alice", "carol"], ["bob", "carol"]] {
            assert_eq!(p.authorize(&pair.map(String::from)), Ok(()));
            assert_eq!(p.authorize(&[pair[1].into(), pair[0].into()]), Ok(()));
        }
    }
}
```

本書外の調査用ディレクトリで `cargo new vault-policy --lib --edition 2021` を作り、src/lib.rsへ上記2断片を続けて配置する。Rust/Cargo 1.90.0、外部依存なしで検査した独自教材であり、Silverscriptの要求版とは別である。

```bash
cargo test --offline
cargo fmt --all -- --check
cargo clippy --offline --all-targets -- -D warnings
```

## 32.15 参照モデルから実コントラクトへ

Rustの参照モデルと実際のコントラクトの間にはギャップがある。

```text
仕様（自然言語）
   ├── Rust参照モデル → 単体テスト
   │
   └── Silverscript等 → コンパイル → Script VM
                                   │
                                   ▼
                            Devnetで統合テスト
```

検証済みIDだけを受ける参照モデルと、署名/取引を評価するVMでは入力領域が異なる。署名検証結果からIDへの写像を定義したうえで、両者に対応するテストベクトルを与え、判定が一致するか比較する。これは**差分テスト**の基本形である。ただし、実コントラクトが表現できない条件を参照モデルに含めた場合、比較以前に設計を見直す必要がある。

### 32.15.1 同じソースを公式コンパイラーとScript VMで検査する

Rustの参照モデルだけで終わらず、32.10の`TwoOfTwoVault`を実際にコンパイルし、署名付きの取引入力をVMへ渡す実習を`experiments/silverscript-vm/`へ収録した。本文のソースは同ディレクトリの`two-of-two.sil`を直接読み込んでいる。

この実習は公式コンパイラー・ABI・Kaspaの`TxScriptEngine`を使う。固定版はSilverscript v1.0.0 / `3ed973335b59269293564805cc2c58a14595ec03`と、その依存先Kaspa `a41a333b08848f41bf737b72592e463a6011b8ac`である。本書の基準ノードv1.0.1とは別の版なので、生成物を既存Devnetへ送る手順とは分ける。

#### 実行する

Rust 1.94.0 / edition 2024、Python 3.11、C/C++ビルド環境を使う。Rustの既定は変更せず、`+1.94.0`で指定する。まず本書の外へ公式ソースを取得する。

```bash
rustup toolchain install 1.94.0 --profile minimal
git clone --branch v1.0.0 --depth 1 https://github.com/kaspanet/silverscript.git /tmp/mastering-kaspa-silverscript-v1.0.0
git -C /tmp/mastering-kaspa-silverscript-v1.0.0 rev-parse HEAD
```

既に取得済みなら再cloneせず、固定コミットと作業ツリーを確認する。本書リポジトリのルートで次を実行する。

```bash
python3 experiments/silverscript-vm/run.py \
  --source /tmp/mastering-kaspa-silverscript-v1.0.0 \
  --work-dir /tmp/mastering-kaspa-silverscript-vm-build
```

初回は依存取得のためのネットワーク接続とビルド時間・空き容量が必要になる。依存取得済みなら`--offline`を追加できる。runnerはコミットと作業ツリーを検査し、専用work-dirへCargoプロジェクトを生成して、収録したハーネス用Cargo.lockで`cargo +1.94.0 run --locked --jobs 2`を実行する。公式ソースとそのCargo.lockは変更しない。詳しい条件は[実習README](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/silverscript-vm/README.md)を参照する。

#### 参照モデルから置き換わる処理

| 段階 | 実習で行うこと |
|---|---|
| 所有者の用意 | 一時鍵を2本生成し、x-only公開鍵をconstructorへ渡す |
| コンパイル | `compile_to_sil_abi_artifact_with_options`でScriptを生成する |
| 署名対象 | 取引入力と参照UTXOを用意し、`calc_schnorr_signature_hash`で`SIG_HASH_ALL`のハッシュを計算する |
| 署名 | 各鍵でSchnorr署名を作り、署名種別のバイトを付ける |
| 引数の組立て | `encode_contract_entry_sig_script`で`spend`へ渡す引数をエンコードする |
| 条件の評価 | `TxScriptEngine::execute`で生成Scriptを評価する |

文字列の`alice`・`bob`を数える代わりに、実際の公開鍵・署名・取引・参照UTXOを評価する。[ハーネスの実装](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/silverscript-vm/main.rs)は、固定公式テストのAPIの使い方を基にしている。秘密鍵・署名は実行ごとにメモリ内で生成し、出力へ含めない。

#### 結果と失敗する層を読む

2026-11-02、macOS/ARM64・Rust 1.94.0で次の6ケースが期待どおりになった。生成Scriptは168 bytesだった。

| ケース | 結果 | 判定した層 |
|---|---|---|
| A・B両方の正常署名 | 許可 | Script VM |
| Bの署名引数を空にする | 拒否 | ABI。65-byte長を要求するためVM未到達 |
| Aの署名を両方へ渡す | 拒否 | Script VM |
| A・Bの署名を逆順に渡す | 拒否 | Script VM |
| 署名後に出力額を変更する | 拒否 | Script VM |
| 署名の1ビットを変える | 拒否 | Script VM |

結果JSONの`allChecksPassed: true`は、これらの期待結果との一致を表す。ABIでの拒否1件とVMでの評価5件を`stage`で区別している。特に、参照モデルの「同じ承認者IDを二度数えない」という検査と、「Bの公開鍵にAの署名を渡しても通らない」というVMの検査は、同じ処理ではない。

入力取引・UTXOは合成データで、VMの`covenants_enabled`を明示的に有効にしている。**ローカルVMで通ることと、実ノードが現在のネットワーク規則で受理することは別である。** 成熟・Mass・全コンセンサス規則・ネットワークの有効化・Devnetでの送信はこの実習では確認しない。記録は`experiments/silverscript-vm-review-2026-11-02.md`に残す。

## 32.16 テストベクトルを設計する

| ケース | 承認者 | 期待結果 | 目的 |
|---|---|---|---|
| 正常 | A, B | 許可 | 基本動作 |
| 不足 | A | 拒否 | 閾値 |
| 重複 | A, A | 拒否 | 重複カウント防止 |
| 不明 | A, X | 拒否 | 権限外の承認者 |
| 署名不正 | Aの無効署名, B | 拒否 | 暗号学的検証 |
| 署名再利用 | 別トランザクション向け署名 | 拒否 | 署名対象の結合 |
| 競合支出 | 同じUTXOを二度支出 | 同じ受理状態で両方を適用しない | UTXOの整合性 |

最後の3ケースは、先ほどの簡略Rustモデルだけでは検証できない。**何をテストできて、何をまだテストしていないか**を明示する。

## 32.17 Property-based Testingの考え方

具体例だけでは境界値を見落とす。性質として検証したいことを定義する。

- 有効な承認者が閾値未満なら必ず拒否する。
- 重複した承認者を追加しても、承認数は増えない。
- 未登録の承認者を追加しても、権限は得られない。
- 承認者の順序が変わっても、妥当性は変わらない（仕様が順序非依存の場合）。

有効/無効の結果と最初に返るエラー理由も区別する。未知と重複を同時に含む場合、入力順で最初の理由が変わり得る。この回の有限テストを全入力の証明やproperty-based試験の実行とは扱わない。

Rustでは`proptest`などを利用できるが、生成データの意味と前提条件を明確にする必要がある。

## 32.18 C4モデルで責務を可視化する

```text
[Contract Source + Constructor Args]
                  │
                  ▼
        [Silverscript Compiler]
                  │
                  ▼
        [Artifact / ABI / Script]
                  │ 生成物を参照
                  ▼
[Wallet / Builder / Signer] ◀── [User]
                  │ 署名済みトランザクション
                  ▼
             [Kaspa Node]
                  │
                  ▼
           [Validation / VM]
```

これは責務の概念図であり、全処理の実行順や単一の更新を示すものではない。JSON成果物全体をノードへ送るのではなく、対象形式・ABIに従って支出条件/証拠を取引へ組み込む。

コンパイラは通常、トランザクションがネットワークで検証されるたびに呼ばれるわけではない。**コンパイル時の処理と検証時の処理**を区別する。

## 32.19 DDD：ルールの所有者は誰か

支出ルールがUI・API・ウォレット・コントラクトに重複していると、変更時に不整合が生じる。

- **オンチェーンで強制すべきルール**：資金の移動を許可する条件。
- **オフチェーンで検証するルール**：入力補助、通知、業務フロー。
- **コンセンサスが保証するルール**：UTXOの存在、二重支払い防止など。

たとえばUIに「2人の承認が必要」と表示しても、スクリプトが1人の署名で支出を許可するなら、安全な2-of-2にはならない。

## 32.20 セキュリティレビューの観点

### 32.20.1 署名と鍵

- 公開鍵の検証形式は正しいか。
- 署名が別トランザクションに再利用されないか。
- 鍵のローテーションと紛失時の扱いは明確か。

### 32.20.2 トランザクション

- 金額、出力先、手数料の扱いは正しいか。
- 予期しない追加出力を許可していないか。
- 署名ハッシュの適用範囲を理解しているか。

### 32.20.3 言語とコンパイラ

- コンパイル結果を検証したか。
- 型変換・整数境界・データ長に問題はないか。
- 依存ライブラリのバージョンは固定したか。

### 32.20.4 運用

- 誤送金を防ぐ表示があるか。
- アップグレード時に既存UTXOがどう扱われるか。
- 監査可能なログと成果物が残るか。

## 32.21 制約の表現力とトレードオフ

より複雑なCovenantを実現できるほど、設計できる機能は増える。一方、検証コストや監査の難しさも増える。

| 設計選択 | 利点 | 注意点 |
|---|---|---|
| 単純な署名条件 | 理解・監査しやすい | 表現力が限定的 |
| 複数署名条件 | 共同管理に向く | 鍵管理・署名収集が複雑 |
| 時間条件 | 支出時期を制御できる | 時刻・ブロック条件の意味を確認 |
| 出力制約（Covenant） | 支出先や次状態を制御可能 | 対応するプロトコル機能が必要 |
| オフチェーン実行＋証明 | 計算を外に移せる場合がある | 検証機能・信頼モデルを別途検討 |

## 32.22 典型的な設計ミス

**ミス1：UIのチェックをオンチェーン保証と誤認する。**  
対策：ルールを実際に強制する層を図示する。

**ミス2：高水準言語の機能とVMの機能を混同する。**  
対策：コンパイル先の命令とネットワークの有効化状況を照合する。

**ミス3：成功テストしか書かない。**  
対策：権限外・重複・再利用・競合を必ず試す。

**ミス4：コンパイラの更新を無条件で取り込む。**  
対策：バージョンを固定し、生成スクリプトの差分を確認する。

**ミス5：Mempoolでの受理を確定とみなす。**  
対策：アプリケーション上の表示とコンセンサス状態を区別する。

## 32.23 Kaspaの実装状況を調査する

この章を実際のKaspaに結び付けるには、以下の一次資料を照合する。

1. `rusty-kaspa`のスクリプト検証実装。
2. トランザクション形式と署名ハッシュの仕様。
3. 関連するKIPの本文とステータス。
4. Silverscriptの対象リポジトリ・ドキュメント・テスト。
5. 実験するネットワークのプロトコル有効化状況。

調査結果は、次の表で管理する。

| 確認項目 | 参照コミット／資料 | 判定 | 証拠 |
|---|---|---|---|
| Silverscriptコンパイラの利用方法 | v1.0.0 / 3ed9733 | ビルド・CLIコンパイルを確認 | 第32.10節、確認記録 |
| 2-of-2のローカルVM・ABI検査 | Silverscript v1.0.0 / 3ed9733、依存Kaspa a41a333 | VM評価5件・ABI拒否1件が期待結果と一致 | [32.15.1の結果](#32151-同じソースを公式コンパイラーとscript-vmで検査する)・[検証記録](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/silverscript-vm-review-2026-11-02.md)。実ノードへの送信ではない |
| 対応するScript命令 | 依存はa41a333、本書の基準版はfcd9c28 | 生成物との対応は未確認 | 依存版が違うため基準版へ自動適用しない |
| Covenant関連機能の有効化 | 第30・31章の固定KIP/ソース比較 | 本番は未確認 | 文書Statusとネットワークの状態を分ける |
| Devnetでの実行 | 未記入 | 未実施 | ログ・TX ID |
| メインネットでの実行 | 未記入 | 未確認 | 公式仕様・実測 |

**「未確認」は「使えない」と同義ではない。** ただし、確認が終わるまでは利用可能と断言しない。

## 32.24 実践演習：小さなコントラクトを設計する

### 演習A：2-of-2 Vault

1. 32.10のソースから、承認者の条件・必要な署名・署名対象を整理する。署名対象は32.15.1のハーネスが使う `SIG_HASH_ALL` と対応付ける。
2. 32.14のRust参照モデルをテストし、文字列IDの検査と、実際の公開鍵・署名の検証で異なる点を書く。
3. 32.15.1の手順で既存のコンパイラー・VM実習を実行する。結果JSONの `stage` を読み、VM評価5件と、VMへ到達する前のABI拒否1件を区別する。
4. 各ケースが32.8のどの不変条件に関係するか整理し、この実習で検査していない条件も記録する。
5. 未収録のケースを一つ選び、入力・期待結果・判定する層を先に定義する。例えばconstructorへ同じ公開鍵を2回渡す場合を考える。実装する場合は既存の正常系も再検査する。

Devnetへ進む場合は別の統合課題とし、対象ノード版・必要な機能の有効化・取引構築方法を確認してから計画する。ローカルVMの成功だけで、既存DevnetやMainnetで利用可能と判断しない。

### 演習B：時間制約付き支出

「一定の条件が満たされた後に支出できる」という要件を考える。ここでは実装に進む前に、時間の基準（ブロック高さ、時刻など）と利用可能な命令を確認する。

### 演習C：支出先を限定するVault

次の支出先を固定する要件について、基盤のScript VMに必要なトランザクション内省機能があるか調査する。なければ**実現できない制約を無理に実装したことにしない**。

## 32.25 理解度チェック

1. Silverscriptのような高水準言語とScript VMの役割はどう違うか。
2. なぜコンパイラの存在だけではCovenantの実行可能性を証明できないのか。
3. UTXOベースのコントラクトにおける「状態遷移」とは何か。
4. オンチェーンで強制すべきルールとUI上のルールをどう区別するか。
5. 2-of-2 Vaultで、同一承認者の重複を拒否すべき理由は何か。
6. 署名が有効でも、別のトランザクションに再利用される危険はないか。
7. Rust参照モデルと実際のScript VMの差分をどう検証するか。
8. コンパイラの更新時に何を記録・比較すべきか。
9. Mempoolへの受理とコンセンサス上の確定はどう違うか。
10. メインネットで利用可能と主張するにはどの証拠が必要か。

## 32.26 まとめ

本章の中心は、特定の言語構文を覚えることではない。**資金の移動を制約するルールを、仕様・実装・検証可能な証拠へと分解すること**である。

```text
要件
 ↓
ドメインモデル
 ↓
不変条件・状態遷移
 ↓
高水準コントラクト記述
 ↓
コンパイルと生成物の検査
 ↓
Script VM・コンセンサスでの検証
 ↓
テスト・監査・運用
```

Silverscriptは、その過程の「高水準の記述とコンパイル」を考えるうえで重要な題材となる。しかし、最終的な安全性は**ネットワークで実際に強制される条件**によって決まる。

次の補足ページ[Argent — Actorベースのアプリケーション言語](argent.md)では、SilverScriptを基盤に複数Actorの状態遷移を記述する仕組みを学ぶ。続く[第33章](chapter-33.md)では、L2・ブリッジ・ゼロ知識証明を扱う。

## 参考資料

- [Silverscript v1.0.0 — README](https://github.com/kaspanet/silverscript/blob/3ed973335b59269293564805cc2c58a14595ec03/README.md)
- [Silverscript v1.0.0 — Tutorial](https://github.com/kaspanet/silverscript/blob/3ed973335b59269293564805cc2c58a14595ec03/docs/TUTORIAL.md)
- [固定版の依存とMSRV](https://github.com/kaspanet/silverscript/blob/3ed973335b59269293564805cc2c58a14595ec03/Cargo.toml)
- [言語バージョン定数](https://github.com/kaspanet/silverscript/blob/3ed973335b59269293564805cc2c58a14595ec03/silverscript-lang/src/compiler/mod.rs)
- [CLI実装 silverc](https://github.com/kaspanet/silverscript/blob/3ed973335b59269293564805cc2c58a14595ec03/silverscript-lang/src/bin/silverc.rs)
- [rusty-kaspa — Kaspa Rustノード実装](https://github.com/kaspanet/rusty-kaspa)
- [Kaspa Improvement Proposals（KIPs）](https://github.com/kaspanet/kips)
- [Bitcoin Developer Guide — Transactions](https://developer.bitcoin.org/devguide/transactions.html)
- [Rust公式ドキュメント](https://doc.rust-lang.org/book/)

**確認範囲：** 固定公式文書・CLIの主要箇所と独自Rust教材。公式CLIのビルド・2-of-2教材コンパイル・生成JSONの再実行一致と拒否ケースまで確認。2026-11-02には同じ2-of-2ソースで、ローカルVM評価5件とABI拒否1件も確認した。Devnet/本番利用・暗号方式の全面照合は未実施。記録は `experiments/chapter-32-review-2026-11-01.md` と `experiments/silverscript-vm-review-2026-11-02.md`。
