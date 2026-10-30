# Chapter 16 — Kaspa Script & Transaction Signing（Kaspa Scriptとトランザクション署名）

> **Mastering Kaspa / Part III — Transactions, Mining & Security**  
> 原稿ステータス：**v0.1（学習用初稿・技術レビュー前）**  
> 本章では、KaspaのUTXOを支出するための**権限の証明**を、鍵・署名・スクリプト・トランザクション検証という責務に分解する。プロトコル固有のバイト列、署名ハッシュ、opcode、ネットワーク別ルールは変更され得るため、対象バージョンの`rusty-kaspa`で必ず確認する。コードは教育用であり、実際のKaspaトランザクションの署名・検証には使用しない。

## この章で学ぶこと

- UTXOの所有権と、支出条件を満たすことの違い
- 秘密鍵・公開鍵・アドレス・ロッキングスクリプトの関係
- トランザクション署名が**何を承認するのか**
- Schnorr署名とECDSA署名を混同しないための基礎
- `scriptPubKey`、`signatureScript`、`SIGHASH`の役割
- スクリプト評価とUTXO状態検証を分離する設計
- 署名・Scriptの代表的な失敗とセキュリティ上の注意点
- Rustで教育用の署名検証フローをテストする方法

---

## 16.1 「コインを持つ」とは何か

第15章では、Kaspaの残高はUTXOの集合から導かれると説明した。では、UTXOを使える人はどう決まるのだろうか。

UTXOは通常、単なるアドレス宛の残高ではなく、**支出条件を持つ出力**である。取引の入力は、過去の出力を参照し、その支出条件を満たす証拠を提示する。

```text
過去のトランザクション
  └─ Output #0
      ├─ value: 5 KAS
      └─ scriptPubKey: 支出条件
                 │
                 ▼
新しいトランザクション
  └─ Input #0
      ├─ previousOutpoint: 過去のOutput #0
      └─ signatureScript: 条件を満たす証拠
                 │
                 ▼
           Script Evaluation
                 │
          成功 / 失敗
```

ここで重要なのは、**署名が正しいだけでは支出が成立しない**という点である。参照するUTXOが未使用であること、金額や手数料の条件を満たすこと、トランザクション全体がコンセンサス規則に適合することも必要になる。

## 16.2 鍵・署名・アドレス・スクリプトの責務

| 要素 | 役割 | 公開してよいか |
|---|---|---|
| 秘密鍵（Private Key） | 署名を作成する | **不可** |
| 公開鍵（Public Key） | 署名を検証する | 原則可 |
| アドレス（Address） | 支払い先の指定を扱いやすく表現する | 原則可 |
| ロッキングスクリプト（`scriptPubKey`） | 出力の支出条件を定義する | ブロックチェーン上で公開される |
| アンロッキングスクリプト（`signatureScript`） | 入力側から証拠を渡す | トランザクション上で公開される |
| 署名（Signature） | 指定されたメッセージへの承認を証明する | 原則可 |

**アドレスは秘密鍵ではない。** また、アドレスと公開鍵は常に同じバイト列とは限らない。アドレスはネットワーク識別やエンコーディング、出力種別などを含む表現であり、最終的には対応する支出条件に変換される。

Kaspaではアドレス形式とScriptの関係を理解する際に、ウォレットSDKの変換処理を追うとよい。文字列を独自実装で解析して送金するより、対象ネットワークに対応した公式SDKのAPIを使うべきである。

## 16.3 デジタル署名の基本

デジタル署名は、秘密鍵を持つ者がメッセージを承認したことを、公開鍵によって検証可能にする暗号技術である。

```text
署名者                                検証者

秘密鍵 ─┐
        ├─ Sign(message) ── signature ──┐
message ┘                              │
                                       ├─ Verify(public key, message, signature)
公開鍵 ────────────────────────────────┘
                                                │
                                             true/false
```

署名方式が安全であり、鍵が適切に保護されているという前提の下で、第三者が秘密鍵なしに有効な署名を偽造することは計算上困難になる。ただし、**秘密鍵の所有者が誰か**という実社会の身元は、署名だけでは保証されない。

### 16.3.1 ハッシュと署名の違い

- ハッシュはデータから固定長のダイジェストを計算する。
- 署名は秘密鍵を用いてメッセージに対する検証可能な証拠を作る。
- ハッシュだけでは「誰が承認したか」は証明できない。
- 署名は通常、規定されたメッセージのハッシュまたは署名用ダイジェストに対して行う。

### 16.3.2 SchnorrとECDSA

Bitcoinの歴史的なトランザクション署名ではECDSAが広く使われ、TaprootではSchnorr署名が導入された。Kaspaの署名・スクリプト設計を理解する際も、**SchnorrとECDSAは異なるアルゴリズム**であることを押さえる必要がある。

両者は同じ楕円曲線の数学を利用し得るが、署名形式、検証式、メッセージ処理、運用上の注意点が異なる。特にSchnorr署名ではノンスの扱いが重大であり、暗号ライブラリの規定に従わず独自生成することは避ける。

> **実装上の注意**：Kaspaのどのトランザクション種別・Script opcodeでどの署名方式が許可されるかは、対象のネットワークとノードバージョンのコンセンサス実装を確認する。BitcoinのTaprootルールをKaspaにそのまま当てはめてはいけない。

## 16.4 UTXOのロックと解除

UTXOの支出条件を、鍵付きの金庫にたとえると理解しやすい。

```text
UTXO = 金庫の中身（価値）
scriptPubKey = 鍵穴と開錠ルール
signatureScript = 開錠に必要な証拠
Script VM = ルールを評価する装置
UTXO Set = まだ使われていない金庫の一覧
```

ただし、金庫の比喩には限界がある。実際のUTXOは物理的なものではなく、**ノードが合意した状態集合の要素**である。Scriptが成功しても、UTXOがすでに消費済みなら支出できない。

### 16.4.1 典型的な単一署名の流れ

1. ウォレットが支出対象UTXOを選ぶ。
2. 新しい取引の入力と出力を組み立てる。
3. プロトコルに定義された署名用メッセージを計算する。
4. 対応する秘密鍵で署名を生成する。
5. 必要な署名データを入力に格納する。
6. ノードが参照UTXOの支出条件を評価する。
7. その他の取引・UTXO・コンセンサス規則を検査する。
8. 受理されれば、状態遷移時に古いUTXOを消費し、新しいUTXOを生成する。

**6と7は別の責務**である。署名・Scriptの検証結果をもって、Mempool受理やブロック内での受理を自動的に保証することはできない。

## 16.5 Scriptとは何か

Scriptは、トランザクションの支出条件を表す小さなプログラムである。一般的なスマートコントラクト実行環境とは異なり、UTXOの支出条件を評価する用途に特化している。

Scriptの理解には、以下の3つを分離する。

| 層 | 問い | 例 |
|---|---|---|
| データ表現 | どのバイト列がScriptか | opcodeとpush data |
| 実行意味論 | opcodeは何をするか | スタック操作・署名検証 |
| コンセンサス規則 | どのScriptを有効と認めるか | バージョン・入力制約・検証フラグ |

**Scriptが表現できること**と、**現行ネットワークで使用できること**は同義ではない。新しいScript機能の提案や、将来のアップグレードを扱う際にはこの区別が不可欠である。

### 16.5.1 スタックマシンの考え方

多くのUTXO型Scriptはスタックベースの評価モデルで説明できる。

```text
初期スタック: []

PUSH 2       -> [2]
PUSH 3       -> [2, 3]
ADD          -> [5]
PUSH 5       -> [5, 5]
EQUAL        -> [true]
```

これは**スタック評価の概念図**であり、Kaspaの実際のopcodeセットや使用可能なScriptを示すものではない。Scriptの実装では、スタック要素数、データサイズ、演算コスト、署名検証コストなどの制約も重要になる。

## 16.6 `scriptPubKey` と `signatureScript`

`scriptPubKey`は出力側に格納される支出条件、`signatureScript`は入力側が提示するデータを表す名称である。

```text
Transaction A
  Output #0
    value: 5 KAS
    scriptPubKey: <支出条件>
          ▲
          │ Outpointで参照
          │
Transaction B
  Input #0
    previousOutpoint: A:0
    signatureScript: <署名などの証拠>
```

検証者は、**Transaction Bの入力だけでなく、Transaction Aの参照出力**を確認する必要がある。ここがアカウントモデルの「送信者アドレスから差し引く」という直感と異なる部分である。

## 16.7 署名は「何」に対して行うのか

最も重要な設計ポイントの一つが**署名対象メッセージ（Signing Preimage / Sighash）**である。

署名対象が曖昧だと、ユーザーが承認していない出力や金額に変更されても、署名が有効なままになり得る。そのためプロトコルは、どのフィールドをどの順序・エンコーディングで署名に結び付けるかを厳密に定義する。

署名対象を考える際の代表的な要素は以下である。

- トランザクションのバージョン
- 入力が参照するOutpoint
- 入力に付随するSequenceなどの制御情報
- 支払い先と金額を含む出力
- Lock TimeやSubnetwork関連フィールド
- 署名対象入力の位置や署名ハッシュ種別
- 対象UTXOの情報やScript情報（規則で必要とされる場合）

**この一覧は概念的な観点であり、Kaspaの実際のSighash仕様の完全な列挙ではない。** 実装時には必ず公式のSighash計算処理を参照する。

### 16.7.1 Transaction IDと署名ハッシュは別物

| 値 | 主な目的 |
|---|---|
| Transaction ID | トランザクションを識別する |
| Sighash | 署名により承認する内容を規定する |
| Block Hash | ブロックを識別しPoWなどに用いる |

これらは同じ「ハッシュ」と呼ばれても、**対象データ・ドメイン・計算規則が異なる**。Transaction IDをそのまま署名してよいと決めつけてはいけない。

### 16.7.2 SIGHASH種別

UTXO系プロトコルでは、署名が承認する入力・出力の範囲を変えるSIGHASH種別が設けられることがある。たとえば「すべての出力を承認する」「特定の出力との対応を承認する」といった設計がある。

ただし、**各フラグの数値、組み合わせ、例外条件はプロトコル固有**である。Bitcoinのフラグ名・動作をそのままKaspaの仕様とみなさない。実際の対応は`rusty-kaspa`の署名ハッシュ実装とテストベクトルで確認する。

## 16.8 トランザクション署名のライフサイクル

```text
Wallet / Application
       │
       ▼
UTXO Selection ────────────────┐
       │                       │
       ▼                       │
Transaction Construction      │
       │                       │
       ▼                       │
Signing Payload Calculation   │
       │                       │
       ▼                       │
Private-key Signing           │
       │                       │
       ▼                       │
Input Signature Assembly      │
       │                       │
       ▼                       │
Local Verification            │
       │                       │
       ▼                       │
Broadcast ────────────────────┤
       │                       │
       ▼                       │
Node Script Validation        │
       │                       │
       ▼                       │
Mempool Policy Checks         │
       │                       │
       ▼                       │
Block / Consensus Evaluation  │
       │                       │
       ▼                       │
Virtual UTXO State            │
```

図は概念モデルである。ノード内部の検証順序や並列化、Mempoolポリシーは実装により異なる。

### 16.8.1 署名の前に確定させるもの

署名後に出力金額や送金先を変更すると、通常は署名が無効になる。ウォレットは**入力・出力・手数料・変更出力・ネットワーク**を確認してから署名する必要がある。

複数入力を持つ取引では、入力ごとに必要な署名が異なる場合がある。すべての入力が同じ鍵で署名できるとは限らない。

## 16.9 署名検証と取引検証の境界

Scriptが成功しても、次のような問題が残り得る。

| 状況 | 主に検査する責務 |
|---|---|
| 署名が不正 | 暗号署名・Script検証 |
| 署名対象メッセージが異なる | Sighash計算・署名検証 |
| 参照UTXOが存在しない | UTXO参照・状態検証 |
| 参照UTXOが消費済み | UTXO状態・二重支払い判定 |
| 出力合計が入力合計を超える | 金額・手数料検証 |
| Scriptの実行制限を超える | Script VM・コンセンサス規則 |
| 取引がMempoolの受理ポリシーに反する | Mempoolポリシー |
| 競合取引が先に順序付けされた | コンセンサス順序・UTXO状態遷移 |

この境界を曖昧にすると、「署名は通るのに送金できない」という障害を調査できなくなる。

## 16.10 KaspaとBitcoinのScriptを比較する

| 観点 | Bitcoin | Kaspa |
|---|---|---|
| 基本モデル | UTXO | UTXO |
| 支出条件 | Script系の検証 | Script系の検証 |
| 署名 | ECDSA、TaprootではSchnorr | 対応署名方式はKaspaの規則に従う |
| 署名ハッシュ | Bitcoin固有の仕様 | Kaspa固有の仕様 |
| ブロック構造 | 線形チェーン | BlockDAG |
| 取引の最終的な受理 | チェーン上の順序とUTXO状態 | コンセンサス順序とVirtual UTXO状態 |

両者の概念は似ていても、**トランザクションのバイナリ形式、署名メッセージ、Scriptルールに互換性があるとは限らない**。

## 16.11 Script VMを設計するなら

Script VMの責務は「与えられたScriptと検証コンテキストに対して、決定的な結果を返すこと」である。

```text
                  Script Engine
                        │
          ┌─────────────┼──────────────┐
          ▼             ▼              ▼
     Script Parser   VM Evaluator   Sig Checker
          │             │              │
          └─────────────┼──────────────┘
                        ▼
                 Success / Error
```

設計上の不変条件として、少なくとも以下を考える。

1. 同じ入力と同じルールなら、すべてのノードで同じ結果になる。
2. Script実行は外部API、ローカル時刻、乱数に依存しない。
3. 失敗したScriptはUTXO集合を変更しない。
4. 実行時間・メモリ・署名検証の負荷が制限される。
5. コンセンサス上の失敗と、アプリケーション上のエラーを区別できる。

Script VMをデータベース更新と直接結び付けず、**評価結果を返す純粋な処理**として設計すると、テストしやすくなる。

## 16.12 Rustによる教育用の署名検証モデル

ここでは、暗号署名そのものは実装せず、**署名検証とUTXO状態更新の責務を分ける**モデルを作る。実際の暗号処理は、監査されたライブラリと公式SDKを利用する。

### 16.12.1 プロジェクト作成

```bash
cargo new script-boundary-lab
cd script-boundary-lab
```

`src/lib.rs`に以下を記述する。

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Outpoint {
    pub txid: String,
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utxo {
    pub value: u64,
    pub required_signer: String,
}

#[derive(Debug, Clone)]
pub struct Input {
    pub previous_outpoint: Outpoint,
    pub claimed_signer: String,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Output {
    pub value: u64,
    pub required_signer: String,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ValidationError {
    MissingUtxo,
    InvalidSignature,
    DuplicateInput,
    Overspend,
    ArithmeticOverflow,
}

/// 実際の暗号署名検証の代わりとなる教育用インターフェース。
/// `signature` を本物の署名として扱ってはならない。
pub trait SignatureVerifier {
    fn verify(&self, signer: &str, signature: &[u8]) -> bool;
}

pub fn validate_transaction<V: SignatureVerifier>(
    tx: &Transaction,
    utxos: &HashMap<Outpoint, Utxo>,
    verifier: &V,
) -> Result<u64, ValidationError> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut total_in = 0u64;

    for input in &tx.inputs {
        if !seen.insert(&input.previous_outpoint) {
            return Err(ValidationError::DuplicateInput);
        }
        let prev = utxos
            .get(&input.previous_outpoint)
            .ok_or(ValidationError::MissingUtxo)?;

        if input.claimed_signer != prev.required_signer
            || !verifier.verify(&prev.required_signer, &input.signature)
        {
            return Err(ValidationError::InvalidSignature);
        }

        total_in = total_in
            .checked_add(prev.value)
            .ok_or(ValidationError::ArithmeticOverflow)?;
    }

    let total_out = tx.outputs.iter().try_fold(0u64, |sum, output| {
        sum.checked_add(output.value)
            .ok_or(ValidationError::ArithmeticOverflow)
    })?;

    if total_out > total_in {
        return Err(ValidationError::Overspend);
    }
    Ok(total_in - total_out)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeVerifier;
    impl SignatureVerifier for FakeVerifier {
        fn verify(&self, signer: &str, signature: &[u8]) -> bool {
            signature == format!("approved:{signer}").as_bytes()
        }
    }

    fn fixture() -> (HashMap<Outpoint, Utxo>, Outpoint) {
        let outpoint = Outpoint { txid: "tx-a".into(), index: 0 };
        let mut utxos = HashMap::new();
        utxos.insert(outpoint.clone(), Utxo {
            value: 100,
            required_signer: "alice".into(),
        });
        (utxos, outpoint)
    }

    fn tx(outpoint: Outpoint, signer: &str, sig: &[u8], value: u64) -> Transaction {
        Transaction {
            inputs: vec![Input {
                previous_outpoint: outpoint,
                claimed_signer: signer.into(),
                signature: sig.into(),
            }],
            outputs: vec![Output {
                value,
                required_signer: "bob".into(),
            }],
        }
    }

    #[test]
    fn valid_authorization_returns_fee() {
        let (utxos, outpoint) = fixture();
        let transaction = tx(outpoint, "alice", b"approved:alice", 90);
        assert_eq!(validate_transaction(&transaction, &utxos, &FakeVerifier), Ok(10));
    }

    #[test]
    fn wrong_signer_is_rejected() {
        let (utxos, outpoint) = fixture();
        let transaction = tx(outpoint, "mallory", b"approved:mallory", 90);
        assert_eq!(validate_transaction(&transaction, &utxos, &FakeVerifier),
            Err(ValidationError::InvalidSignature));
    }

    #[test]
    fn overspend_is_rejected() {
        let (utxos, outpoint) = fixture();
        let transaction = tx(outpoint, "alice", b"approved:alice", 101);
        assert_eq!(validate_transaction(&transaction, &utxos, &FakeVerifier),
            Err(ValidationError::Overspend));
    }

    #[test]
    fn duplicate_input_is_rejected() {
        let (utxos, outpoint) = fixture();
        let mut transaction = tx(outpoint, "alice", b"approved:alice", 90);
        transaction.inputs.push(transaction.inputs[0].clone());
        assert_eq!(validate_transaction(&transaction, &utxos, &FakeVerifier),
            Err(ValidationError::DuplicateInput));
    }

    #[test]
    fn failed_validation_does_not_modify_utxo_set() {
        let (utxos, outpoint) = fixture();
        let before = utxos.clone();
        let transaction = tx(outpoint, "alice", b"invalid", 90);
        assert!(validate_transaction(&transaction, &utxos, &FakeVerifier).is_err());
        assert_eq!(utxos, before);
    }
}
```

実行コマンド：

```bash
cargo test
```

> **安全上の警告**：`FakeVerifier`は文字列を比較しているだけであり、**暗号署名ではない**。秘密鍵、公開鍵、ハッシュ、Kaspaの署名方式を実装していない。テスト対象は「権限検証をUTXO検証から分離する設計」である。送金、ウォレット、実ネットワークには絶対に流用しない。

### 16.12.2 このモデルに欠けているもの

- 本物の署名アルゴリズムと公開鍵検証
- Kaspaのトランザクションシリアライズ
- KaspaのSighash計算
- Scriptパーサとopcode実行
- Scriptバージョン、実行制限、コンセンサスフラグ
- Coinbase、Lock Time、Subnetworkなどの規則
- Mempoolのポリシーと競合取引の扱い
- ブロック順序とVirtual Stateの再計算

このように**何を実装していないかを明示すること**が、教育用コードとコンセンサス互換実装を区別する第一歩である。

## 16.13 テスト戦略：何を保証するか

| テスト層 | 目的 | 例 |
|---|---|---|
| 単体テスト | 境界ごとの正否 | 不正署名の拒否 |
| プロパティテスト | 不変条件の検査 | 失敗時にUTXO状態不変 |
| 既知ベクトルテスト | バイト列互換性 | 署名ハッシュと期待値の一致 |
| 統合テスト | SDKとノードの連携 | Devnetで送金・受理確認 |
| 回帰テスト | 変更による破壊の防止 | Scriptルール変更前後の差分 |

コンセンサス実装の検証では、**独立した期待値（公式テストベクトルなど）**が重要である。自分の実装で計算した値をそのまま正解とみなすテストは、同じバグを見逃す。

## 16.14 鍵管理とウォレットのセキュリティ

署名アルゴリズムが安全でも、鍵の管理が不適切なら資産は守れない。

### 16.14.1 秘密鍵の取り扱い

- 秘密鍵やシードフレーズをログに出さない。
- テスト用の鍵を本番資産に使わない。
- 秘密鍵をリポジトリや環境変数のダンプに残さない。
- ハードウェアウォレット等を使う場合は、**署名する内容を信頼できる画面で確認する**。
- バックアップの復元テストと、紛失・漏えい時の運用を設計する。

### 16.14.2 署名の対象をユーザーに説明する

ウォレットの画面では、少なくとも送金先、金額、手数料、ネットワークを明示する。単に「署名してください」と表示するだけでは、何を承認するのか判断できない。

### 16.14.3 ドメイン分離

異なる用途の署名が別の用途に再利用されないよう、署名対象の構築に**ドメイン分離**を用いる設計がある。たとえば「ログイン用の署名」と「送金用の署名」は、意味の異なるメッセージとして扱う必要がある。Kaspaの実際の署名用ハッシュのドメインと手順は、実装で確認する。

## 16.15 脅威モデル

| 脅威 | 何が起こるか | 対策の方向性 |
|---|---|---|
| 秘密鍵漏えい | 攻撃者が正規の署名を作れる | 鍵分離、秘密情報保護 |
| 署名対象の誤構築 | 意図しない内容が承認される | 公式Sighash実装、ベクトル検証 |
| リプレイ | 別の文脈で署名が再利用される | ドメイン分離、文脈の固定 |
| 不正なScript入力 | 過剰な計算やパーサ障害 | 入力制限、決定的なVM |
| 不正なウォレットUI | 実際と異なる送金先を表示 | 署名前の内容照合 |
| 二重支払い | 同じUTXOを複数回使おうとする | コンセンサス順序・UTXO検証 |

**Scriptの正しさだけでは資産の安全性は保証されない。** ウォレット、署名端末、P2P伝播、コンセンサス状態を含む全体の信頼境界を確認する必要がある。

## 16.16 `rusty-kaspa` のコードを読む

公式リポジトリ：[kaspanet/rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)

まずリポジトリを取得する。

```bash
git clone https://github.com/kaspanet/rusty-kaspa.git
cd rusty-kaspa
git rev-parse HEAD
```

次に、次のキーワードを検索する。

```bash
rg -n 'signature_script|signatureScript|script_public_key|scriptPubKey' .
rg -n 'sighash|SigHash|SignatureHash' .
rg -n 'Schnorr|ECDSA|checksig|CheckSig' .
rg -n 'script.*engine|ScriptEngine|ScriptPublicKey' .
```

> リポジトリのディレクトリ構成やシンボル名はバージョンによって変わる。検索結果を起点に、実際の呼び出し関係を追うこと。

### 16.16.1 調査の順序

1. **トランザクションの型定義**：Input、Output、Outpoint、Scriptの保持形式を確認する。
2. **署名ハッシュの計算**：どのフィールドをどの順序でハッシュ化するか調べる。
3. **署名検証**：公開鍵・署名形式・エラー条件を確認する。
4. **Scriptエンジン**：opcodeと実行制限、失敗条件を調べる。
5. **トランザクション検証**：Script結果がどこでUTXO検証と統合されるか確認する。
6. **テスト**：既知ベクトル、境界値、回帰テストを読む。

### 16.16.2 ソース調査記録テンプレート

```markdown
## 調査対象
- Repository: kaspanet/rusty-kaspa
- Commit: <git rev-parse HEAD>
- Network: <mainnet/testnet/devnet>

## 署名処理
- Entry point:
- Sighash implementation:
- Signature verifier:
- Script engine:

## 不変条件
- 署名が無効なら受理しない
- Script失敗時にUTXOを更新しない
- 同じルールと入力なら同じ検証結果

## 実験
- Input:
- Expected:
- Actual:
- Evidence (test/log):
```

## 16.17 C4モデルと責務の分割

```text
┌─────────────────────┐
│ Wallet / SDK        │
│ - UTXO選択          │
│ - 取引構築          │
│ - 署名              │
└──────────┬──────────┘
           │ Signed Transaction
           ▼
┌─────────────────────┐
│ Node Transaction API│
└──────────┬──────────┘
           ▼
┌─────────────────────┐
│ Transaction Validator
│ - 構造・金額検査    │
│ - UTXO参照          │
└──────────┬──────────┘
           ▼
┌─────────────────────┐
│ Script Engine       │
│ - Script評価        │
│ - 署名検証          │
└──────────┬──────────┘
           ▼
┌─────────────────────┐
│ Consensus / State   │
│ - 取引順序          │
│ - UTXO状態遷移      │
└─────────────────────┘
```

この図は責務を説明する概念モデルであり、`rusty-kaspa`の実際のコンポーネント境界をそのまま示したものではない。

### 16.17.1 DDDの観点

- **Wallet Context**：ユーザーが何を承認するか、鍵をどう保護するか。
- **Transaction Context**：入力・出力・手数料などの構造的な妥当性。
- **Script Context**：与えられた支出条件を満たすか。
- **Consensus Context**：DAG上の順序と現在のUTXO状態で受理できるか。

境界を分けることで、Script opcodeの追加がウォレットUIやUTXOデータベースへ不必要に波及するのを防ぎやすくなる。

## 16.18 ステートマシンで考える

トランザクションの処理を次のように抽象化できる。

```text
Constructed
    │
    ▼
Signing
    │
    ├─ failure ──> Rejected
    ▼
Signed
    │
    ▼
Broadcast
    │
    ▼
Locally Validated
    │
    ├─ invalid ──> Rejected
    ▼
Mempool Candidate
    │
    ▼
Included in Block
    │
    ▼
Evaluated in Consensus Order
    │
    ├─ conflict / invalid ──> Not Accepted
    ▼
Accepted in Virtual State
```

これは**説明用の状態遷移**であり、Kaspaノードの正式な状態列挙ではない。また、BlockDAGでは後続ブロックによりコンセンサス上の評価が変わる場合があるため、最後の状態を無条件に永久確定とみなさない。

## 16.19 よくある誤解

**Q1. アドレスが分かればUTXOを使える？**  
いいえ。支出条件を満たす署名などが必要になる。

**Q2. 署名が有効なら送金は確定？**  
いいえ。UTXOの存在、未使用状態、手数料、コンセンサス上の順序などの検証が必要になる。

**Q3. Transaction IDと署名ハッシュは同じ？**  
同じとは限らない。目的と計算規則が異なる。

**Q4. BitcoinのScriptをKaspaでそのまま動かせる？**  
互換性を前提にしてはいけない。opcode、署名規則、トランザクション形式、コンセンサスルールを確認する。

**Q5. Scriptは一般的なEVMスマートコントラクトと同じ？**  
いいえ。UTXOの支出条件を評価するScriptと、共有状態を操作するEVMの実行モデルは設計目的が異なる。

**Q6. `FakeVerifier`を本物の署名検証に置き換えれば完成？**  
いいえ。Sighash、シリアライズ、Script VM、各種コンセンサス規則、テストベクトルなどが欠けている。

## 16.20 実践課題

### 課題A：境界を図示する

自分の言葉で次の4つの責務を分離して描く。

1. トランザクション構築
2. 署名生成
3. Script評価
4. UTXO状態遷移

それぞれの**入力・出力・失敗条件**を1行ずつ書く。

### 課題B：失敗ケースを増やす

第16.12節の教育用コードに次のテストを追加する。

- 存在しないOutpointを指定したら`MissingUtxo`
- `u64::MAX`を超える入力合計で`ArithmeticOverflow`
- 正しい署名でも、同じ入力を2回参照したら`DuplicateInput`
- 複数入力で一方だけ署名が不正なら全体を拒否

### 課題C：公式実装と照合する

対象の`rusty-kaspa`コミットを固定し、実際の署名ハッシュ処理から、どのトランザクションフィールドが署名対象に含まれるかを調べる。**推測ではなく、コードとテストを根拠に表を作る。**

### 課題D：Devnet実験計画

実際の秘密鍵を共有せず、Devnet用ウォレットで次の実験を計画する。

1. 署名前に送金先と金額を記録する。
2. SDKで署名した取引を検証する。
3. 署名後に送金先を変更した場合の検証失敗を観察する。
4. 取引ID・署名対象・入力UTXOの違いを記録する。

## 16.21 理解度チェック

1. `scriptPubKey`と`signatureScript`は、それぞれどこに格納されるか。
2. 公開鍵とアドレスは同じものか。
3. Scriptが成功しても取引が受理されない例を3つ挙げよ。
4. Transaction IDとSighashを区別する理由を説明せよ。
5. 署名検証をUTXO状態更新から分離する利点は何か。
6. なぜ署名対象のバイト列を独自に推測して実装してはいけないのか。
7. BitcoinとKaspaのScriptで共通する概念と、互換性を仮定してはいけない部分を挙げよ。
8. 鍵が漏えいしていなくても、不正送金につながり得るUI上の問題は何か。

### 解答の要点

1. 出力側の支出条件と、入力側の支出証拠。
2. いいえ。アドレスは公開鍵や出力種別等を扱うための表現。
3. UTXO消費済み、入力額不足、コンセンサス順序上の競合など。
4. 識別用ハッシュと署名対象メッセージは目的が異なるから。
5. 決定的な検証、単体テスト、失敗時の状態不変を実現しやすい。
6. バイト単位の違いで検証結果が変わり、互換性と安全性を損なうから。
7. UTXOの支出条件という概念は共通だが、具体的な署名ハッシュ・opcode・形式は個別仕様。
8. 実際と異なる送金先や金額を表示して署名させる問題。

## 16.22 本章のまとめ

本章では、Kaspaのトランザクション署名とScriptを、単なる暗号関数ではなく、**UTXOの支出条件を満たすための検証システム**として整理した。

- 秘密鍵は署名を生成し、公開鍵は署名を検証する。
- アドレス、公開鍵、Scriptは役割が異なる。
- 署名は規定されたSighashに対する承認である。
- Script検証とUTXO検証、コンセンサス順序は別の責務である。
- 正しい署名だけではトランザクションの受理は保証されない。
- 教育用コードと、実際のKaspaコンセンサス互換実装は明確に区別する。

次章の**Chapter 17 — Mempool & Transaction Propagation（Mempoolとトランザクション伝播）**では、署名済みトランザクションがP2Pネットワークへ流れ、各ノードのMempoolでどのように扱われるかを学ぶ。

## 16.23 参考資料

### Kaspa公式・一次資料

- [Kaspa公式サイト](https://kaspa.org/)
- [Kaspa Wiki](https://wiki.kaspa.org/)
- [kaspanet/rusty-kaspa — 公式Rustノード実装](https://github.com/kaspanet/rusty-kaspa)
- [kaspanet/kips — Kaspa Improvement Proposals](https://github.com/kaspanet/kips)
- [Kaspa Research Papers](https://kaspa.org/research/)

### 暗号・Scriptの基礎

- [Bitcoin Developer Guide — Transactions](https://developer.bitcoin.org/devguide/transactions.html)
- [BIP-340 — Schnorr Signatures for secp256k1](https://github.com/bitcoin/bips/blob/master/bip-0340.mediawiki)（**Bitcoinの仕様**。Kaspaへの直接適用を意味しない）
- [BIP-341 — Taproot](https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki)（比較学習用）
- [SEC 1: Elliptic Curve Cryptography](https://www.secg.org/sec1-v2.pdf)

### 原稿を技術レビューする際の確認項目

- [ ] 対象ネットワークと`rusty-kaspa`コミットを明記したか。
- [ ] 実際に許可される署名方式・Script opcodeをコードとテストで照合したか。
- [ ] Sighashの対象フィールド、シリアライズ、ドメイン分離を一次資料で確認したか。
- [ ] アドレス種別とScript生成の対応をSDKで検証したか。
- [ ] 教育用コードをコンセンサス互換と誤認させる記述がないか。
- [ ] Devnetの実験ログとテストベクトルを追加したか。
