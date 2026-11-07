# Chapter 16 — Kaspa Script & Transaction Signing（Kaspa Scriptとトランザクション署名）

<p class="chapter-part">Part III — Transactions, Mining &amp; Security（トランザクション・マイニング・セキュリティ）</p>

> 本章では、KaspaのUTXOを支出するための**権限の証明**を、鍵・署名・スクリプト・トランザクション検証という責務に分解する。プロトコル固有のバイト列、署名ハッシュ、opcode、ネットワーク別ルールは変更され得るため、対象バージョンの`rusty-kaspa`で必ず確認する。Rustの簡略モデルは教育用であり、実際のKaspaトランザクションの署名・検証には使用しない。16.12.3では、これと区別して公式SDKを使う既存のDevnet実習を読む。

## 照合した実装

2026年10月31日、[本書の基準版](../baseline.md) v1.0.1、コミット `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` の主要ソースを照合した。以下の固定ソースへの参照はSource-checkedの範囲を示す。別途、ランダム鍵による単一署名のDevnet送金・virtual受理は実測した（`experiments/devnet-send-review-2026-10-31.md`）。所定の拒否は `experiments/devnet-send-checks-2026-10-31.md`、送信成功応答の喪失からの復旧は `experiments/devnet-send-retry-2026-10-31.md` に記録済み。これらは以前の実行結果で、今回は再実行していない。HD復旧は本章の単一署名実験の対象外であり、復旧した鍵によるDevnet送金と変更枝のお釣りの支出は、別実習として[第24章24.14.1](../part-04-development/chapter-24.md#24141-教材の入力予約と公式sdkをつなぐ)で扱う。暗号化バックアップや履歴探索を含むHDウォレット全体の復旧、全Script・全異常系・現在のMainnetとの一致は未検証。記録はリポジトリの `experiments/signing-wallet-review-2026-10-31.md` を参照する。

## この章で学ぶこと

- UTXOの所有権と、支出条件を満たすことの違い
- 秘密鍵・公開鍵・アドレス・ロッキングスクリプトの関係
- トランザクション署名が**何を承認するのか**
- Schnorr署名とECDSA署名を混同しないための基礎
- `scriptPubKey`、`signatureScript`、`SIGHASH`の役割
- スクリプト評価とUTXO状態検証を分離する設計
- 署名・Scriptの代表的な失敗とセキュリティ上の注意点
- Rustで教育用の署名検証フローをテストする方法
- 公式SDKの署名とノードによる拒否を観察し、モックの検査との違いを説明する方法

---

## 16.1 「コインを持つ」とは何か

[第15章](chapter-15.md)では、Kaspaの残高はUTXOの集合から導かれると説明した。では、UTXOを使える人はどう決まるのだろうか。

UTXOは通常、単なるアドレス宛の残高ではなく、**支出条件を持つ出力**である。トランザクションの入力は、過去の出力を参照し、その支出条件を満たす証拠を提示する。

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

両者は同じ楕円曲線の数学を利用し得るが、署名形式、検証式、メッセージ処理、運用上の注意点が異なる。両方式とも署名用ノンスの扱いが重大であり、暗号ライブラリの規定に従わず独自生成することは避ける。

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
2. 新しいトランザクションの入力と出力を組み立てる。
3. プロトコルに定義された署名用メッセージを計算する。
4. 対応する秘密鍵で署名を生成する。
5. 必要な署名データを入力に格納する。
6. ノードが参照UTXOの支出条件を評価する。
7. その他のトランザクション・UTXO・コンセンサス規則を検査する。
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

最も重要な設計ポイントの一つが**署名対象メッセージ**である。ハッシュへ入れるデータ（signing preimage）と、その計算結果である署名用ダイジェスト（sighash）は区別する。

署名対象が曖昧だと、ユーザーが承認していない出力や金額に変更されても、署名が有効なままになり得る。そのためプロトコルは、どのフィールドをどの順序・エンコーディングで署名に結び付けるかを厳密に定義する。

署名対象を考える際の代表的な要素は以下である。

- トランザクションのバージョン
- 入力が参照するOutpoint
- 入力に付随するSequenceなどの制御情報
- 支払い先と金額を含む出力
- Lock TimeやSubnetwork関連フィールド
- 署名対象入力の位置や署名ハッシュ種別
- 対象UTXOの情報やScript情報（規則で必要とされる場合）

### 基準版で確認した署名ハッシュ

[calc_schnorr_signature_hash](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/hashing/sighash.rs)は、version、入力群のOutpoint/sequence/sig_op_countのハッシュ、対象入力のOutpoint、参照UTXOのScriptとamount、対象入力のsequence/sig_op_count、出力ハッシュ、lock_time、subnetwork_id、gas、payloadハッシュ、SIGHASH種別を規定の順でまとめる。

ここで使うUTXO情報は、署名しようとする入力の `signatureScript` 自身ではない。入力だけのJSONやTransaction IDを署名する代用はできない。アドレスのネットワーク接頭辞もこの列挙の署名データではないため、送信先のネットワークはアドレス解釈・接続先の検査でも確認する。

SIGHASH種別により、集約ハッシュの一部はゼロになる。native subnetworkかつ空payloadも専用のゼロハッシュ分岐を持つ。ECDSA用の[calc_ecdsa_signature_hash](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/hashing/sighash.rs)はSchnorr用ダイジェストをさらにECDSA用ハッシュへ通す。同じ入力でも、両方式の署名対象を無条件に同一視しない。

基準版の `SigHashReusedValues` は入力群・出力・payloadのハッシュをキャッシュする。同じ取引の複数入力での再利用と、取引を書き換えた後の再利用を区別し、入力や出力を変更したらキャッシュも作り直す。

この説明はソースの主要構成の確認であり、ハッシュ用ドメインやエンコードも含めた互換実装・既知値テストの完了を意味しない。

### 16.7.1 Transaction IDと署名ハッシュは別物

| 値 | 主な目的 |
|---|---|
| Transaction ID | トランザクションを識別する |
| Sighash | 署名により承認する内容を規定する |
| Block Hash | ブロックを識別する。KaspaのPoW値とは別の計算 |

これらは同じ「ハッシュ」と呼ばれても、**対象データ・ドメイン・計算規則が異なる**。Transaction IDをそのまま署名してよいと決めつけてはいけない。

### 16.7.2 SIGHASH種別

UTXO系プロトコルでは、署名が承認する入力・出力の範囲を変えるSIGHASH種別が設けられることがある。たとえば「すべての出力を承認する」「特定の出力との対応を承認する」といった設計がある。

基準版の[SigHashType](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/hashing/sighash_type.rs)はALL=`0x01`、NONE=`0x02`、SINGLE=`0x04`、ANYONECANPAY=`0x80`を定義し、前3種類のそれぞれとANYONECANPAYの組み合わせを含む6値を許可する。SINGLEをBitcoinの数値から推測してはならない。

[outputs_hash](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/hashing/sighash.rs)ではNONEは出力の集約をゼロ、SINGLEは入力と同じindexの出力だけを対象とし、対応する出力がなければゼロを返す。ANYONECANPAYでは入力群のOutpoint・sig_op_count等の集約が変わる。これらは検証済みの特定版の規則であり、署名モードを変更する際は承認範囲も見直す。

Transaction v1では、compute budget・storage massがALL署名の対象外で、出力のCovenant bindingは対象になるなど、基準版との違いがある。[15.5.1の比較表と14ケース](chapter-15.md#1551-transaction-v0とv1を区別する)を参照し、ここでのv0のフィールド列を後続版へそのまま適用しない。ALLは「取引の全バイトを署名する」という意味ではない。

### 16.7.3 標準アドレスからScriptへの変換

[pay_to_address_script](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/crypto/txscript/src/standard.rs)はアドレスのversionで分岐する。PubKeyは32バイトの公開鍵データと `OpCheckSig`、PubKeyECDSAは33バイトと `OpCheckSigECDSA` を使う。ScriptHashはBLAKE2bによる32バイトのスクリプトハッシュを比較する。任意のアドレスpayloadを同じ署名方式で処理しない。

## 16.8 トランザクション署名のライフサイクル

```text
Wallet / Application
       │
       ▼
UTXO Selection
       │
       ▼
Transaction Construction / UTXO Resolution
       │
       ▼
Signing Digest Calculation
       │
       ▼
Private-key Signing / Input Signature Assembly
       │
       ▼
Local Verification → Broadcast
       │
       ▼
Node Validation / Mempool Policy
       │
       ▼
Block / Consensus Evaluation
       │
       ▼
Virtual UTXO State
```

図は概念モデルであり、Mempoolを含む全段階をすべてのノードが観測するわけではない。ノード内部の金額・ポリシー検査がScript検証より先に行われる場合もある。厳密な順序や並列化は固定版の検証経路を参照する。

### 16.8.1 署名の前に確定させるもの

たとえばSIGHASH_ALLで承認した出力金額や送金先を署名後に変更すると、その署名は変更後の内容に対応しなくなる。NONE/SINGLE等では承認する範囲が異なるため、すべての変更で必ず無効になると一般化しない。ウォレットは**入力・出力・手数料・変更出力・ネットワーク**を確認してから署名する必要がある。

複数入力を持つトランザクションでは、入力ごとに必要な署名が異なる場合がある。すべての入力が同じ鍵で署名できるとは限らない。

## 16.9 署名検証とトランザクション検証の境界

Scriptが成功しても、次のような問題が残り得る。

| 状況 | 主に検査する責務 |
|---|---|
| 署名が不正 | 暗号署名・Script検証 |
| 署名対象メッセージが異なる | Sighash計算・署名検証 |
| 参照UTXOが存在しない | UTXO参照・状態検証 |
| 参照UTXOが消費済み | UTXO状態・二重支払い判定 |
| 出力合計が入力合計を超える | 金額・手数料検証 |
| Scriptの実行制限を超える | Script VM・コンセンサス規則 |
| トランザクションがMempoolの受理ポリシーに反する | Mempoolポリシー |
| 競合トランザクションが先に順序付けされた | コンセンサス順序・UTXO状態遷移 |

この境界を曖昧にすると、「署名は通るのに送金できない」という障害を調査できなくなる。

## 16.10 KaspaとBitcoinのScriptを比較する

| 観点 | Bitcoin | Kaspa |
|---|---|---|
| 基本モデル | UTXO | UTXO |
| 支出条件 | Script系の検証 | Script系の検証 |
| 署名 | ECDSA、TaprootではSchnorr | 対応署名方式はKaspaの規則に従う |
| 署名ハッシュ | Bitcoin固有の仕様 | Kaspa固有の仕様 |
| ブロック構造 | 競合する枝から採用チェーンを選ぶ | BlockDAG |
| トランザクションの最終的な受理 | チェーン上の順序とUTXO状態 | コンセンサス順序とVirtual UTXO状態 |

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

本書とは別の作業用ディレクトリで実行する。

```bash
cargo new --lib --edition 2021 script-boundary-lab
cd script-boundary-lab
```

`src/lib.rs`に以下を記述する。外部依存はなく、Rust 1.90.0で確認した。検証器には取引・入力位置・参照UTXOも渡すが、モックは文脈を無視する。その限界もテストで示す。

<details class="code-details">
<summary>署名・支出条件モデルの実装とテストを見る</summary>

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
    EmptyInputs,
    EmptyOutputs,
    MissingUtxo,
    InvalidSignature,
    DuplicateInput,
    Overspend,
    ArithmeticOverflow,
}

/// 実際の暗号署名検証の代わりとなる教育用インターフェース。
/// `signature` を本物の署名として扱ってはならない。
pub trait SignatureVerifier {
    fn verify(
        &self,
        signer: &str,
        tx: &Transaction,
        input_index: usize,
        previous_utxo: &Utxo,
        signature: &[u8],
    ) -> bool;
}

pub fn validate_transaction<V: SignatureVerifier>(
    tx: &Transaction,
    utxos: &HashMap<Outpoint, Utxo>,
    verifier: &V,
) -> Result<u64, ValidationError> {
    use std::collections::HashSet;
    if tx.inputs.is_empty() {
        return Err(ValidationError::EmptyInputs);
    }
    if tx.outputs.is_empty() {
        return Err(ValidationError::EmptyOutputs);
    }
    let mut seen = HashSet::new();
    let mut total_in = 0u64;

    for (input_index, input) in tx.inputs.iter().enumerate() {
        if !seen.insert(&input.previous_outpoint) {
            return Err(ValidationError::DuplicateInput);
        }
        let prev = utxos
            .get(&input.previous_outpoint)
            .ok_or(ValidationError::MissingUtxo)?;

        if input.claimed_signer != prev.required_signer
            || !verifier.verify(
                &prev.required_signer,
                tx,
                input_index,
                prev,
                &input.signature,
            )
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
        fn verify(
            &self,
            signer: &str,
            _tx: &Transaction,
            _input_index: usize,
            _previous_utxo: &Utxo,
            signature: &[u8],
        ) -> bool {
            signature == format!("approved:{signer}").as_bytes()
        }
    }

    fn fixture() -> (HashMap<Outpoint, Utxo>, Outpoint) {
        let outpoint = Outpoint {
            txid: "tx-a".into(),
            index: 0,
        };
        let mut utxos = HashMap::new();
        utxos.insert(
            outpoint.clone(),
            Utxo {
                value: 100,
                required_signer: "alice".into(),
            },
        );
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
        assert_eq!(
            validate_transaction(&transaction, &utxos, &FakeVerifier),
            Ok(10)
        );
    }

    #[test]
    fn wrong_signer_is_rejected() {
        let (utxos, outpoint) = fixture();
        let transaction = tx(outpoint, "mallory", b"approved:mallory", 90);
        assert_eq!(
            validate_transaction(&transaction, &utxos, &FakeVerifier),
            Err(ValidationError::InvalidSignature)
        );
    }

    #[test]
    fn overspend_is_rejected() {
        let (utxos, outpoint) = fixture();
        let transaction = tx(outpoint, "alice", b"approved:alice", 101);
        assert_eq!(
            validate_transaction(&transaction, &utxos, &FakeVerifier),
            Err(ValidationError::Overspend)
        );
    }

    #[test]
    fn duplicate_input_is_rejected() {
        let (utxos, outpoint) = fixture();
        let mut transaction = tx(outpoint, "alice", b"approved:alice", 90);
        transaction.inputs.push(transaction.inputs[0].clone());
        assert_eq!(
            validate_transaction(&transaction, &utxos, &FakeVerifier),
            Err(ValidationError::DuplicateInput)
        );
    }

    #[test]
    fn empty_transactions_are_rejected() {
        let (utxos, outpoint) = fixture();
        let mut transaction = tx(outpoint, "alice", b"approved:alice", 90);
        transaction.inputs.clear();
        assert_eq!(
            validate_transaction(&transaction, &utxos, &FakeVerifier),
            Err(ValidationError::EmptyInputs)
        );
        let mut transaction = tx(
            Outpoint {
                txid: "tx-a".into(),
                index: 0,
            },
            "alice",
            b"approved:alice",
            90,
        );
        transaction.outputs.clear();
        assert_eq!(
            validate_transaction(&transaction, &utxos, &FakeVerifier),
            Err(ValidationError::EmptyOutputs)
        );
    }

    #[test]
    fn missing_utxos_and_overflow_are_rejected() {
        let (mut utxos, outpoint) = fixture();
        let missing = tx(
            Outpoint {
                txid: "absent".into(),
                index: 0,
            },
            "alice",
            b"approved:alice",
            90,
        );
        assert_eq!(
            validate_transaction(&missing, &utxos, &FakeVerifier),
            Err(ValidationError::MissingUtxo)
        );
        let large = Outpoint {
            txid: "large".into(),
            index: 0,
        };
        utxos.insert(
            large.clone(),
            Utxo {
                value: u64::MAX,
                required_signer: "alice".into(),
            },
        );
        let mut transaction = tx(outpoint.clone(), "alice", b"approved:alice", 90);
        transaction.inputs.push(Input {
            previous_outpoint: large,
            claimed_signer: "alice".into(),
            signature: b"approved:alice".to_vec(),
        });
        assert_eq!(
            validate_transaction(&transaction, &utxos, &FakeVerifier),
            Err(ValidationError::ArithmeticOverflow)
        );
        let mut transaction = tx(outpoint, "alice", b"approved:alice", u64::MAX);
        transaction.outputs.push(Output {
            value: 1,
            required_signer: "bob".into(),
        });
        assert_eq!(
            validate_transaction(&transaction, &utxos, &FakeVerifier),
            Err(ValidationError::ArithmeticOverflow)
        );
    }

    #[test]
    fn invalid_later_input_rejects_the_transaction() {
        let (mut utxos, outpoint) = fixture();
        let second = Outpoint {
            txid: "second".into(),
            index: 0,
        };
        utxos.insert(
            second.clone(),
            Utxo {
                value: 10,
                required_signer: "carol".into(),
            },
        );
        let before = utxos.clone();
        let mut transaction = tx(outpoint, "alice", b"approved:alice", 90);
        transaction.inputs.push(Input {
            previous_outpoint: second,
            claimed_signer: "carol".into(),
            signature: b"invalid".to_vec(),
        });
        assert_eq!(
            validate_transaction(&transaction, &utxos, &FakeVerifier),
            Err(ValidationError::InvalidSignature)
        );
        assert_eq!(utxos, before);
    }

    #[test]
    fn mock_accepts_changed_outputs_to_show_its_limit() {
        let (utxos, outpoint) = fixture();
        let mut transaction = tx(outpoint, "alice", b"approved:alice", 90);
        assert_eq!(
            validate_transaction(&transaction, &utxos, &FakeVerifier),
            Ok(10)
        );
        transaction.outputs[0].value = 80;
        transaction.outputs[0].required_signer = "mallory".into();
        // このモックは取引内容を署名に結び付けていない。
        assert_eq!(
            validate_transaction(&transaction, &utxos, &FakeVerifier),
            Ok(20)
        );
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

</details>

実行コマンド：

```bash
cargo test
```

本文から抽出した教材は9テスト成功（`experiments/chapter-16-review-2026-11-01.md`）。検証器へ取引・入力位置・参照UTXOを渡すが、`FakeVerifier` はそれらを無視して文字列だけを比較する。出力を変更しても通るテストは、その限界を示す反例であり、暗号署名の改変検出に成功した結果ではない。

このモデルには秘密鍵・公開鍵・Sighash・本物の署名検証がない。UTXO集合は読み取り専用の参照で渡し、検証が成功しても入力を消費しない。したがって同じ取引を再検証できるが、コンセンサス上の再支出を受理したことにはならない。

### 16.12.2 このモデルに欠けているもの

- 本物の署名アルゴリズムと公開鍵検証
- Kaspaのトランザクションシリアライズ
- KaspaのSighash計算
- Scriptパーサとopcode実行
- Scriptバージョン、実行制限、コンセンサスフラグ
- Coinbase、Lock Time、Subnetworkなどの規則
- Mempoolのポリシーと競合トランザクションの扱い
- ブロック順序とVirtual Stateの再計算

このように**何を実装していないかを明示すること**が、教育用コードとコンセンサス互換実装を区別する第一歩である。

### 16.12.3 `FakeVerifier`から公式SDKの署名へ

`FakeVerifier`は`approved:alice`のような文字列を比較するだけなので、秘密鍵を持っている証拠にも、取引内容への承認にもならない。実際の送金では、**SDKが入力の支出条件を満たす署名を作り、ノードが受け取った取引を検証する**。

[第15章15.17.1](chapter-15.md#15171-教材から公式sdkの送金処理へ)では、公式SDKによる取引構築と結果照会を追った。ここでは同じ`experiments/devnet/send.cjs --checks`の署名・不正署名の拒否に注目する。SDK導入・ノード起動・実行と終了は[第28章28.6](../part-04-development/chapter-28.md#286-ローカル環境の準備)と[28.10](../part-04-development/chapter-28.md#2810-トランザクションのe2eテスト)に従う。抜粋だけでは実行せず、既存スクリプト全体を使う。

#### 鍵・アドレス・入力UTXOを結び付ける

```javascript
{{#include ../../experiments/devnet/send.cjs:temporary-keys}}
```

この実習はNode.jsの乱数から一時鍵を作り、公式SDKでDevnetアドレスを導出する。その`source`へ採掘報酬を受け取り、成熟したUTXOを`createTransactions`へ渡す。`sourceKey`はその入力を支出するための鍵であり、アドレス文字列を指定するだけでは署名できない。

構築結果の`pending`へ`pending.sign([sourceKey])`を呼ぶとSDKが署名を行う。通常の送信では、その後に`pending.submit(rpc)`を呼ぶ。アプリ側で`approved:alice`を署名へ置き換えたり、教材の文字列`owner`を公開鍵の代わりに使ったりはしない。鍵・参照UTXO・取引形式はSDKの型に従う。

この一時鍵は実行ごとに作り、保存も表示もしない。既存ウォレットの鍵を入力する例ではなく、終了後の復旧機能もない。HD導出・入力予約・鍵保管の設計は[第24章](../part-04-development/chapter-24.md)で扱う。

#### 不正な署名をノードへ送り、拒否と入力の維持を確認する

`--checks`は正常な取引を署名した後、そのコピーの`signatureScript`を1ビット変更して送信する。元の`pending`は変更せず、不正コピーの拒否を確認してから正常な送金へ進む。

<details class="code-details">
<summary>署名検証失敗を確認するコードを見る</summary>

```javascript
{{#include ../../experiments/devnet/send.cjs:reject-invalid-signature}}
```

</details>

`rejected`はスクリプト内の検査関数である。処理が成功した場合や、タイムアウト・切断などの通信失敗では検査を失敗にする。期待するScript・署名・検証系の拒否を得た後、入力Outpointが現在UTXOに残っていることも確認する。

`signature[2]`はこの固定SDKで作った単一署名取引に対する実験用の変更位置であり、任意のScriptで署名の位置が同じという意味ではない。`finalize()`は変更後の取引フィールドから取引IDを再計算・確定する処理で、不正な署名を正しく署名し直す処理ではない。

2026-10-31の実測結果では、次の値を確認した。記録は`experiments/devnet/send-checks-observed-2026-10-31.json`にある。

| 出力項目 | 結果 | 確認したこと |
|---|---|---|
| `checks.invalidSignature` | `rejected-as-expected` | 不正コピーが検証系のエラーで拒否された |
| `checks.failedSignaturePreservedInput` | `true` | 拒否後も元の入力が現在UTXOに残った |
| `acceptedByVirtualChain` | `true` | その後に送った正常な取引がvirtual chainで受理された |

#### 二つのテストで分かることを区別する

| 検査 | 分かること |
|---|---|
| `FakeVerifier`の単体テスト | 検証結果の受け渡し、入力・金額の検査、失敗時の状態維持 |
| 公式SDKとノードの不正署名実験 | この固定版・取引・変更方法で実際の検証系が拒否し、入力が維持されたこと |

この実験は全署名方式・全Scriptの検証や、出力改変に対する拒否の試験ではない。後者を確認するなら、署名ハッシュの条件を固定し、署名後の出力を変える別の試験が必要になる。単に本物の暗号関数を`FakeVerifier`へ差し込むだけでは、取引形式・参照UTXO・Sighash・Scriptの意味まで公式実装と一致したことにはならない。

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

[第1章](../part-01-foundations/chapter-01.md)で取得した固定版を再利用する。

```bash
# 基準版ソースの親ディレクトリから実行する
cd rusty-kaspa-v1.0.1
git rev-parse HEAD
grep -n -E 'calc_schnorr_signature_hash|calc_ecdsa_signature_hash|outputs_hash' consensus/core/src/hashing/sighash.rs
grep -n -E 'SIG_HASH_|ALLOWED_SIG_HASH_TYPES' consensus/core/src/hashing/sighash_type.rs
grep -n -E 'pay_to_address_script|PubKeyECDSA|ScriptHash' crypto/txscript/src/standard.rs
```

HEADが本章冒頭の完全SHAと一致することを確認する。固定版の検索結果から、呼び出し元・データ構造・テストまで追う。

### 16.16.1 調査の順序

1. **トランザクションの型定義**：Input、Output、Outpoint、Scriptの保持形式を確認する。
2. **署名ハッシュの計算**：どのフィールドをどの順序でハッシュ化するか調べる。
3. **署名検証**：公開鍵・署名形式・エラー条件を確認する。
4. **Scriptエンジン**：opcodeと実行制限、失敗条件を調べる。
5. **トランザクション検証**：Script結果がどこでUTXO検証と統合されるか確認する。
6. **テスト**：既知ベクトル、境界値、回帰テストを読む。

### 16.16.2 ソース調査記録テンプレート

<details class="code-details">
<summary>調査記録のテンプレートを見る</summary>

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

</details>

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
Node Validation
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

送信成功の応答が失われた場合は、拒否ではなく受理の成否が不明な状態も必要となる。所定の応答喪失・照会・同一取引の再送は、冒頭の既存Devnet記録を参照する。

これは**説明用の状態遷移**であり、Kaspaノードの正式な状態列挙ではない。また、BlockDAGでは後続ブロックによりコンセンサス上の評価が変わる場合があるため、最後の状態を無条件に永久確定とみなさない。

## 16.19 よくある誤解

<details class="misconception-details">
<summary>よくある誤解を見る</summary>

**1. アドレスが分かればUTXOを使える？**

いいえ。支出条件を満たす署名などが必要になる。

**2. 署名が有効なら送金は確定？**

いいえ。UTXOの存在、未使用状態、手数料、コンセンサス上の順序などの検証が必要になる。

**3. Transaction IDと署名ハッシュは同じ？**

別の値である。目的と計算規則が異なる。

**4. BitcoinのScriptをKaspaでそのまま動かせる？**

互換性を前提にしてはいけない。opcode、署名規則、トランザクション形式、コンセンサスルールを確認する。

**5. Scriptは一般的なEVMスマートコントラクトと同じ？**

いいえ。UTXOの支出条件を評価するScriptと、共有状態を操作するEVMの実行モデルは設計目的が異なる。

**6. `FakeVerifier`を本物の署名検証に置き換えれば完成？**

いいえ。Sighash、シリアライズ、Script VM、各種コンセンサス規則、テストベクトルなどが欠けている。

</details>

## 16.20 実践課題

<details class="exercise-details">
<summary>実践課題を見る</summary>

### 課題A：境界を図示する

自分の言葉で次の4つの責務を分離して描く。

1. トランザクション構築
2. 署名生成
3. Script評価
4. UTXO状態遷移

それぞれの**入力・出力・失敗条件**を1行ずつ書く。

### 課題B：失敗ケースを増やす

16.12節の追加済みテストを読み、次の入力がどの検証で拒否されるか説明する。さらに、検証器へ渡された取引・入力位置・参照UTXOを記録するモックを作り、呼び出し内容を確認する。

- 存在しないOutpointを指定したら`MissingUtxo`
- `u64::MAX`を超える入力合計で`ArithmeticOverflow`
- 正しい署名でも、同じ入力を2回参照したら`DuplicateInput`
- 複数入力で一方だけ署名が不正なら全体を拒否

### 課題C：公式実装と照合する

対象の`rusty-kaspa`コミットを固定し、実際の署名ハッシュ処理から、どのトランザクションフィールドが署名対象に含まれるかを調べる。**推測ではなく、コードとテストを根拠に表を作る。**

### 課題D：Devnet実験計画

実際の秘密鍵を共有せず、Devnet用ウォレットで次の実験を計画する。

1. 署名前に送金先と金額を記録する。
2. SDKで署名したトランザクションを検証する。
3. SIGHASH_ALLを使い、署名後に送金先を変更した場合の検証失敗を観察する。FakeVerifierの反例と混同しない。
4. トランザクションID・署名対象・入力UTXOの違いを記録する。

</details>

## 16.21 理解度チェック

<details class="quiz-answers">
<summary>理解度チェックを見る</summary>

1. `scriptPubKey`と`signatureScript`は、それぞれどこに格納されるか。
2. 公開鍵とアドレスは同じものか。
3. Scriptが成功してもトランザクションが受理されない例を3つ挙げよ。
4. Transaction IDとSighashを区別する理由を説明せよ。
5. 署名検証をUTXO状態更新から分離する利点は何か。
6. なぜ署名対象のバイト列を独自に推測して実装してはいけないのか。
7. BitcoinとKaspaのScriptで共通する概念と、互換性を仮定してはいけない部分を挙げよ。
8. 鍵が漏えいしていなくても、不正送金につながり得るUI上の問題は何か。

<a id="解答の要点"></a>

**解答例**

1. 出力側の支出条件と、入力側の支出証拠。
2. いいえ。アドレスは公開鍵や出力種別等を扱うための表現。
3. UTXO消費済み、入力額不足、コンセンサス順序上の競合など。
4. 識別用ハッシュと署名対象メッセージは目的が異なるから。
5. 決定的な検証、単体テスト、失敗時の状態不変を実現しやすい。
6. バイト単位の違いで検証結果が変わり、互換性と安全性を損なうから。
7. UTXOの支出条件という概念は共通だが、具体的な署名ハッシュ・opcode・形式は個別仕様。
8. 実際と異なる送金先や金額を表示して署名させる問題。

</details>

## 16.22 本章のまとめ

<details class="summary-details">
<summary>まとめを見る</summary>

本章では、Kaspaのトランザクション署名とScriptを、単なる暗号関数ではなく、**UTXOの支出条件を満たすための検証システム**として整理した。

- 秘密鍵は署名を生成し、公開鍵は署名を検証する。
- アドレス、公開鍵、Scriptは役割が異なる。
- 署名は規定されたSighashに対する承認である。
- Script検証とUTXO検証、コンセンサス順序は別の責務である。
- 正しい署名だけではトランザクションの受理は保証されない。
- 教育用コードと、実際のKaspaコンセンサス互換実装は明確に区別する。

</details>

次章の**[Chapter 17 — Mempool & Transaction Propagation（Mempoolとトランザクション伝播）](chapter-17.md)**では、署名済みトランザクションがP2Pネットワークへ流れ、各ノードのMempoolでどのように扱われるかを学ぶ。

## 16.23 参考資料

<details class="reference-details">
<summary>参考資料を見る</summary>

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

</details>

<nav class="chapter-next" aria-label="次のページ">

[次章：Chapter 17 — Mempool & Transaction Propagation（Mempoolとトランザクション伝播）](chapter-17.md)

</nav>

<details class="verification-details">
<summary>検証状況を見る</summary>

[対象バージョン](../baseline.md)・[このページのレビュー記録](../review-status.md#review-chapter-16)

### 原稿を技術レビューする際の確認項目

- [x] 対象ネットワークと`rusty-kaspa`コミットを明記したか。
- [ ] 実際に許可される署名方式・Script opcodeをコードとテストで照合したか。
- [ ] Sighashの対象フィールド、シリアライズ、ドメイン分離を一次資料で確認したか。
- [ ] アドレス種別とScript生成の対応をSDKで検証したか。
- [x] 教育用コードの範囲を示し、出力の改変が通るモックの反例も確認したか。
- [x] Devnetの所定の実測結果と未確認事項を記録したか（生ログ・秘密情報は公開しない）。

目次順の本文レビューと教材9テストは `experiments/chapter-16-review-2026-11-01.md`。上の未チェック項目は全方式・全opcodeやバイト単位の照合を含み、主要経路のソース確認・単一署名送金だけで完了とはしない。今回は鍵生成・署名・送金・ノードの起動を新規に実行していない。

</details>
