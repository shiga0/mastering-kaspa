# Chapter 15 — UTXO Model & Transaction Processing（UTXOモデルとトランザクション処理）

> **Mastering Kaspa / Part III — Transactions, Mining & Security**  
> 原稿ステータス：**v0.1（学習用初稿・技術レビュー前）**  
> 本章はKaspaのUTXOモデルとトランザクション処理を理解するための原稿である。Rustの実装例は**教育用の簡略モデル**であり、Kaspaの署名、スクリプト、トランザクション形式、コンセンサス規則を完全再現しない。プロトコル依存の詳細は対象バージョンの`rusty-kaspa`で照合すること。

## この章で学ぶこと

この章では、ブロックの順序を決めた後に、**どの取引がどのUTXOを消費し、新しいUTXOを生み出すのか**を理解する。

- UTXOが「残高」ではなく「未使用の出力」である理由
- Outpoint、Input、Output、Transaction IDの責務
- トランザクションの構造検査、署名・スクリプト検査、UTXO検査の違い
- 手数料、変更出力、二重支払いの判定
- Mempool上の未確定取引とコンセンサス状態の区別
- BlockDAGで順序が変わったときのUTXO状態の再評価
- RustでUTXO状態遷移の最小モデルを作り、不変条件をテストする方法

---

## 15.1 なぜUTXOモデルを学ぶのか

ウォレットに「10 KAS」と表示されていても、ノード内部に`balance = 10`という単一の変数があるとは限らない。UTXO方式では、利用可能な**未使用の取引出力（Unspent Transaction Output）**を集計して残高を求める。

たとえば、あるアドレスに対応する未使用出力が次の3件なら、合計は10 KASになる。

| UTXO | 金額 | 状態 |
|---|---:|---|
| A:0 | 4 KAS | 未使用 |
| B:1 | 3 KAS | 未使用 |
| C:0 | 3 KAS | 未使用 |

重要なのは、**UTXOは分割して一部だけ消費できない**という点だ。4 KASのUTXOから1 KASを送る場合、元の4 KASを入力として丸ごと消費し、送金先への出力と自分へのお釣り（変更出力）を新しく作る。

```text
取引前
  UTXO A:0 ── 4 KAS

取引
  入力:  A:0 (4 KAS)
  出力:  送金先 1 KAS
         変更先 2.99 KAS
  差額:  手数料 0.01 KAS

取引後
  A:0 は使用済み
  新しい出力 TxX:0 ── 1 KAS
  新しい出力 TxX:1 ── 2.99 KAS
```

この仕組みがトランザクション検証、並列処理、二重支払い防止、ウォレット設計の基礎となる。

## 15.2 アカウントモデルとの比較

| 観点 | UTXOモデル | アカウントモデル |
|---|---|---|
| 状態の基本単位 | 未使用の出力 | アカウントの残高・状態 |
| 支払いの表現 | 既存出力を消費して新しい出力を作る | アカウント状態を更新する |
| 二重支払いの競合 | 同じOutpointを使う入力 | 残高、Nonce、実行順序など |
| 並列性 | 独立UTXO間の競合が明示的 | 状態アクセスの競合解析が必要 |
| ウォレットの仕事 | UTXO選択と変更出力の管理 | Nonceや残高の管理など |

どちらが絶対的に優れているわけでもない。必要なアプリケーションの表現力、状態管理、並列処理、検証可能性によって設計上の選択が変わる。

## 15.3 Kaspaの金額単位

Kaspaのネイティブ資産はKASであり、プロトコルでは整数の最小単位を扱う。一般に **1 KAS = 100,000,000 sompi** である。

```text
1 KAS        = 100,000,000 sompi
0.1 KAS      =  10,000,000 sompi
0.00000001 KAS =         1 sompi
```

金額計算には浮動小数点数ではなく整数を用いる。たとえば`0.1 + 0.2`のような浮動小数点演算を手数料や残高の判定に直接使うと、丸め誤差の原因になる。

**設計上の不変条件**：資産の増減を検査するときは、常に整数単位で計算し、オーバーフローを検出する。

## 15.4 UTXOの識別子：Outpoint

UTXOは通常、**どのトランザクションの何番目の出力か**で識別する。

```text
Outpoint = (Transaction ID, Output Index)
```

例：

```text
TxA:0 ── 5 KAS
TxA:1 ── 2 KAS
TxB:0 ── 1 KAS
```

`TxA:0`と`TxA:1`は同じトランザクションから生まれても、別々のUTXOである。

Outpointは**位置の識別子**であり、アドレスや金額そのものではない。同じアドレスが複数のUTXOを所有できる。

## 15.5 トランザクションの概念的な構造

Kaspaの取引を理解するため、まず次の概念モデルを使う。

```text
Transaction
├── Version
├── Inputs[]
│   ├── Previous Outpoint
│   ├── Signature Script
│   ├── Sequence
│   └── その他の入力フィールド
├── Outputs[]
│   ├── Value (sompi)
│   └── Script Public Key
├── Lock Time
├── Subnetwork ID
├── Gas
└── Payload
```

これは学習用の見取り図である。**実際のフィールドの型、シリアライズ、ハッシュ対象、バージョン別の扱いは実装で確認する**。

### Inputの責務

入力は、既存UTXOを参照し、そのUTXOを使う権限があることを示すためのデータを含む。

### Outputの責務

出力は、新たに作る価値の量と、将来それを使用できる条件を定義する。ウォレット画面のアドレスは、通常、出力スクリプトなどを扱いやすく表現したものとして理解するとよい。

### Transaction IDの責務

Transaction IDはトランザクションを識別するハッシュ値である。**表示用のJSONや任意の文字列をハッシュすれば互換のIDになるわけではない**。コンセンサス仕様が定めるシリアライズとハッシュ規則が必要になる。

## 15.6 UTXO集合はどこにあるのか

ノードが保持するUTXO集合は、概念的には次の写像で表せる。

```text
UTXO Set:
  Outpoint -> UTXO Entry
```

`UTXO Entry`には、金額やロック条件のほか、コンセンサス検証に必要なメタデータが含まれる場合がある。

```text
TxA:0 -> { value: 500000000, script: ... }
TxB:1 -> { value: 200000000, script: ... }
```

ウォレットは自分に関係するUTXOを把握したいが、**ウォレットのローカル記録と、ノードが認識するコンセンサスUTXO集合は同一ではない**。同期遅延、未確定取引、再編成などにより見え方が異なることがある。

## 15.7 トランザクションのライフサイクル

```text
Wallet
  │  UTXO選択・出力構築・署名
  ▼
Transaction Broadcast
  │
  ▼
P2P Network
  │
  ▼
Node / Mempool Validation
  │  未確定取引として保持される場合がある
  ▼
Mining / Block Inclusion
  │
  ▼
BlockDAG / Consensus Ordering
  │
  ▼
Transaction Acceptance Evaluation
  │
  ▼
Virtual UTXO State
```

この図の各段階は異なる責務を持つ。**P2Pで受信した、Mempoolに入った、ブロックに含まれた、コンセンサス状態に受理された**という表現を混同してはいけない。

## 15.8 検証を段階に分ける

検証は単一の`valid = true`では説明できない。

| 検証層 | 主な問い | 失敗例 |
|---|---|---|
| 構造・形式 | データは正しい形か | 空の入力、サイズ制限違反など |
| 文脈非依存の規則 | 状態を参照せず確認できるか | 重複入力、数値範囲違反など |
| 署名・スクリプト | 使用条件を満たすか | 署名不正、スクリプト失敗 |
| UTXO参照 | 入力先が使用可能か | 未知のOutpoint、既使用出力 |
| 金額保存 | 入力と出力の関係が妥当か | 出力合計が入力合計を超える |
| コンセンサス文脈 | 現在の順序と状態で受理できるか | 先に競合取引が受理された |
| Mempoolポリシー | 未確定取引として中継・保管するか | ノードのポリシーに合わない |

**Mempoolポリシーとコンセンサス規則は別物**である。あるノードが中継しない取引でも、コンセンサス規則上の有効性は別途判断する必要がある。

## 15.9 金額保存と手数料

通常の資産移転を簡略化すると、次の関係が成り立つ。

```text
入力合計 = 出力合計 + 手数料
```

例：

```text
Inputs:
  3.0 KAS
  2.0 KAS
  --------
  5.0 KAS

Outputs:
  受取人 3.5 KAS
  お釣り 1.4 KAS
  --------
  4.9 KAS

Fee:
  0.1 KAS
```

**注意**：この式は通常のUTXO資産移転を説明するための単純化である。Coinbaseやサブネットワークに関連する特殊な規則を同一視しないこと。

### 手数料は明示的な出力ではない

一般的なUTXO型の取引では、手数料は入力と出力の差額として表れる。マイナーへの支払いを「通常の受取人向け出力」として必ず指定するわけではない。

## 15.10 UTXO選択と変更出力

ウォレットが2.2 KASを送りたいとき、所有するUTXOが次のようになっているとする。

```text
U1 = 1.0 KAS
U2 = 1.5 KAS
U3 = 4.0 KAS
```

選択肢は複数ある。

- `U3`だけを使う：入力が少なくて済むが、変更出力が大きい。
- `U1 + U2`を使う：入力が増えるが、変更出力を小さくできる。

どちらが適切かは、手数料、入力数、UTXO断片化、プライバシー、将来の支出予定による。UTXO選択は単なる金額合わせではなく、**ウォレットのポリシーと最適化問題**である。

### プライバシー上の注意

複数のUTXOを同じ取引で入力として使うと、それらが共通の管理下にあると推測されることがある。ただし、これは常に正しい推論ではない。

## 15.11 二重支払いは何を競合させるのか

```text
           UTXO X:0
           /      \
          /        \
      Tx-A          Tx-B
    Aliceへ       Bobへ
```

`Tx-A`と`Tx-B`が同じ`X:0`を入力として消費しようとすれば、**両方を同じUTXO状態に適用することはできない**。

線形チェーンではチェーン上の順序が、KaspaではBlockDAGから導かれるコンセンサス上の順序と受理規則が、どちらの取引を適用できるかに関係する。

ただし、**ブロックがDAGに存在することは、そのブロックに含まれる全取引がUTXO状態に受理されたことを意味しない**。

## 15.12 並行ブロックと取引の受理

Kaspaでは、ネットワーク遅延によって複数のブロックが並行して生成される。

```text
          Block A: Tx-A
         /             \
Genesis                 Merge
         \             /
          Block B: Tx-B
```

この図は概念図であり、具体的なGHOSTDAGの順序や受理結果を表すものではない。

同じUTXOを消費する取引が別々の並行ブロックに含まれた場合でも、コンセンサスが導く処理順序に沿って状態を評価する必要がある。**DAGに複数ブロックを保持できることと、二重支払いを認めることは全く違う**。

## 15.13 Virtual StateとUTXO状態遷移

前章のVirtual Stateを、UTXOの視点から捉え直す。

```text
旧Virtual UTXO Set
        │
        │ コンセンサスで受理される取引を順序に沿って適用
        ▼
新Virtual UTXO Set
```

通常の取引を適用する概念的な手順は次のとおり。

1. 入力OutpointがUTXO集合に存在するか調べる。
2. 入力の重複やスクリプト条件などを検証する。
3. 入力合計と出力合計を安全に計算する。
4. 受理可能なら入力UTXOを削除する。
5. 新しい出力をUTXO集合に追加する。
6. 必要なインデックスや状態情報を更新する。

**原子性**が重要である。途中で失敗したときに「入力だけ消えた」状態を残してはいけない。

## 15.14 Reorganizationと状態の巻き戻し

新しいブロックの到着によって、コンセンサス上の選択や順序が変化することがある。その場合、以前のVirtual Stateから新しいVirtual Stateへの更新が必要になる。

概念的には次のいずれかの設計が考えられる。

- 変更前の状態に戻し、新しい順序で取引を再適用する。
- 状態差分（差分ログ）を用いて巻き戻しと適用を行う。
- 適切なチェックポイントから状態を再構築する。

これは**一般的な状態管理の考え方**であり、Kaspaの現行実装が必ずこの3方式のいずれかをそのまま採用しているという主張ではない。実際の差分処理は`rusty-kaspa`で確認する。

### ウォレット側で起きること

以前「受理済み」と見えていた取引が、再編成や同期状態の変化によって別の扱いになる可能性がある。そのため、アプリケーションは単一の受信イベントだけで不可逆な決済完了を決めない。

## 15.15 MempoolとUTXO集合は同じではない

| 対象 | 役割 |
|---|---|
| コンセンサスUTXO集合 | コンセンサスが認識する利用可能な出力を表す |
| Mempool | まだブロック内で確定的に受理されていない取引候補を扱う |
| ウォレットのUTXO表示 | アドレスに関連する資金をユーザーに示す |

未確定取引が使うUTXOを別の未確定取引が使おうとする場合、Mempoolでは競合管理が必要になる。また、親取引が未確定のまま、その出力を子取引が参照することも概念的には可能だが、**具体的な受け入れ条件は実装・ポリシーに依存する**。

## 15.16 Coinbaseと通常取引の違い

通常の取引は既存UTXOを入力として消費する。一方、Coinbaseはマイニング報酬等を扱う特別な取引であり、通常の「入力合計−出力合計＝手数料」という説明をそのまま適用できない。

また、Kaspaではブロック報酬、手数料、Coinbaseの扱いがDAG上のコンセンサスと関係する。BitcoinのCoinbase処理を単純にコピーして説明しないこと。詳細は第18章・第19章で扱う。

## 15.17 Rustで最小UTXO状態遷移を作る

ここからは**Kaspa互換ではない教育用モデル**を作る。署名検証、Script、Coinbase、シリアライズ、DAG順序は省略し、UTXOの消費と生成に集中する。

### プロジェクト作成

```bash
cargo new utxo-lab
cd utxo-lab
```

`src/lib.rs`を次の内容にする。

```rust
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Outpoint {
    pub txid: String,
    pub index: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub value: u64, // 教育用: 最小通貨単位の整数
    pub owner: String,
}

#[derive(Clone, Debug)]
pub struct Transaction {
    pub txid: String,
    pub inputs: Vec<Outpoint>,
    pub outputs: Vec<Output>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplyError {
    EmptyInputs,
    EmptyOutputs,
    DuplicateInput,
    MissingInput,
    DuplicateOutput,
    ValueOverflow,
    Overspend,
}

#[derive(Default, Clone, Debug)]
pub struct UtxoSet {
    entries: HashMap<Outpoint, Output>,
}

impl UtxoSet {
    pub fn insert_genesis(&mut self, outpoint: Outpoint, output: Output) {
        self.entries.insert(outpoint, output);
    }

    pub fn get(&self, outpoint: &Outpoint) -> Option<&Output> {
        self.entries.get(outpoint)
    }

    pub fn apply(&mut self, tx: &Transaction) -> Result<u64, ApplyError> {
        if tx.inputs.is_empty() {
            return Err(ApplyError::EmptyInputs);
        }
        if tx.outputs.is_empty() {
            return Err(ApplyError::EmptyOutputs);
        }

        let mut seen = HashSet::new();
        let mut input_total = 0_u64;
        for input in &tx.inputs {
            if !seen.insert(input) {
                return Err(ApplyError::DuplicateInput);
            }
            let previous = self.entries.get(input).ok_or(ApplyError::MissingInput)?;
            input_total = input_total
                .checked_add(previous.value)
                .ok_or(ApplyError::ValueOverflow)?;
        }

        let mut output_total = 0_u64;
        for (index, output) in tx.outputs.iter().enumerate() {
            output_total = output_total
                .checked_add(output.value)
                .ok_or(ApplyError::ValueOverflow)?;

            let outpoint = Outpoint {
                txid: tx.txid.clone(),
                index: u32::try_from(index).map_err(|_| ApplyError::ValueOverflow)?,
            };
            if self.entries.contains_key(&outpoint) {
                return Err(ApplyError::DuplicateOutput);
            }
        }

        let fee = input_total
            .checked_sub(output_total)
            .ok_or(ApplyError::Overspend)?;

        // すべての検査が成功してから状態を書き換える。
        for input in &tx.inputs {
            self.entries.remove(input);
        }
        for (index, output) in tx.outputs.iter().enumerate() {
            let outpoint = Outpoint {
                txid: tx.txid.clone(),
                index: index as u32,
            };
            self.entries.insert(outpoint, output.clone());
        }
        Ok(fee)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn original() -> Outpoint {
        Outpoint { txid: "genesis".into(), index: 0 }
    }

    fn funded() -> UtxoSet {
        let mut set = UtxoSet::default();
        set.insert_genesis(
            original(),
            Output { value: 100, owner: "alice".into() },
        );
        set
    }

    #[test]
    fn transfer_consumes_input_and_creates_outputs() {
        let mut set = funded();
        let tx = Transaction {
            txid: "tx1".into(),
            inputs: vec![original()],
            outputs: vec![
                Output { value: 60, owner: "bob".into() },
                Output { value: 35, owner: "alice".into() },
            ],
        };
        assert_eq!(set.apply(&tx), Ok(5));
        assert_eq!(set.get(&original()), None);
        assert_eq!(set.get(&Outpoint { txid: "tx1".into(), index: 0 }).unwrap().value, 60);
    }

    #[test]
    fn double_spend_is_rejected() {
        let mut set = funded();
        let first = Transaction {
            txid: "first".into(),
            inputs: vec![original()],
            outputs: vec![Output { value: 90, owner: "bob".into() }],
        };
        let second = Transaction {
            txid: "second".into(),
            inputs: vec![original()],
            outputs: vec![Output { value: 90, owner: "carol".into() }],
        };
        assert_eq!(set.apply(&first), Ok(10));
        assert_eq!(set.apply(&second), Err(ApplyError::MissingInput));
    }

    #[test]
    fn failed_transaction_does_not_mutate_state() {
        let mut set = funded();
        let before = set.clone();
        let tx = Transaction {
            txid: "invalid".into(),
            inputs: vec![original()],
            outputs: vec![Output { value: 101, owner: "bob".into() }],
        };
        assert_eq!(set.apply(&tx), Err(ApplyError::Overspend));
        assert_eq!(set.entries, before.entries);
    }

    #[test]
    fn duplicate_inputs_are_rejected() {
        let mut set = funded();
        let tx = Transaction {
            txid: "duplicate".into(),
            inputs: vec![original(), original()],
            outputs: vec![Output { value: 100, owner: "bob".into() }],
        };
        assert_eq!(set.apply(&tx), Err(ApplyError::DuplicateInput));
    }
}
```

テストを実行する。

```bash
cargo test
```

このサンプルは`txid`を文字列で直接受け取るため、**本物の取引IDの一意性や署名の正当性を保証しない**。また、所有者`owner`も単なる文字列であり、支出権限を検証していない。実サービスの決済ロジックに転用しないこと。

## 15.18 Rust実装の責務を分離する

上記の`apply`は学習のため多くの処理をまとめている。実際の設計では責務を分けたい。

```text
Transaction Decoder
        │
        ▼
Structural Validator
        │
        ▼
Signature / Script Validator
        │
        ▼
UTXO Resolver
        │
        ▼
Contextual Validator
        │
        ▼
State Transition
        │
        ▼
Persistent Storage / Indexes
```

### 依存関係の設計

- **Decoder**はバイト列をデータ構造に変換する。残高判定をしない。
- **Structural Validator**は形式的な制約を検査する。ネットワークから状態を取得しない。
- **UTXO Resolver**はOutpointに対応する既存出力を解決する。
- **Script Validator**は支出条件を検査する。
- **State Transition**は検証済みの入力を消費し、出力を追加する。
- **Storage**は状態の永続化・整合性を担う。

これらの境界を分けると、署名アルゴリズム、ストレージ実装、ネットワーク層の変更による影響を局所化しやすい。

## 15.19 状態遷移の不変条件

UTXO処理で守るべき条件を明文化する。

| ID | 不変条件 | 検査の考え方 |
|---|---|---|
| INV-01 | 1つの取引で同じ入力Outpointを重複使用しない | `HashSet`で重複検出 |
| INV-02 | 使用する入力が存在する | 全入力を状態から解決 |
| INV-03 | 通常取引の出力合計が入力合計を超えない | checked arithmetic |
| INV-04 | 失敗した取引でUTXO集合を変更しない | 検査後にコミット |
| INV-05 | 既存UTXOを誤って上書きしない | 出力Outpointの衝突検査 |
| INV-06 | 同一の状態と同一の取引順序から同じ結果を得る | 決定的な状態遷移 |
| INV-07 | 使用権限を持たない入力を消費させない | 署名・スクリプト検証 |

サンプルでは`INV-07`を実装していない。これを「実装していない不変条件」として明記することが、技術的な説明の信頼性を高める。

## 15.20 ステートマシンとして見る

トランザクションの状態は、観測する主体によって異なる。次は**アプリケーション向けの概念モデル**であり、Kaspaの公式状態列挙ではない。

```text
Created
   │
   ▼
Signed
   │
   ▼
Broadcast
   │
   ▼
Mempool Observed
   │
   ▼
Included in Block
   │
   ▼
Accepted in Virtual State
   │
   ▼
Increasing Confirmation / Confidence
```

再編成や同期遅延があるため、実際には単純な一方向遷移とは限らない。アプリケーションは、状態が後戻りしうる場面を明示的に扱う。

## 15.21 `rusty-kaspa`で確認するポイント

実装調査では、先に**何を確かめたいのか**を決めてから検索する。

| 調査テーマ | 探す概念・名前の例 | 確認すること |
|---|---|---|
| 取引データ構造 | `Transaction`, `TransactionInput`, `TransactionOutput` | フィールド、型、バージョン |
| UTXO識別 | `TransactionOutpoint`, `UtxoEntry` | Outpointの表現とメタデータ |
| 取引検証 | `transaction_validator`, `validate_transaction` | 文脈非依存／依存の区別 |
| UTXO差分 | `utxo_diff`, `utxo_set` | 状態更新とロールバック |
| 署名検証 | `script`, `signature` | どの層で支出条件を判定するか |
| Mempool | `mempool`, `orphan` | 未確定取引の扱い |
| RPC | `get_utxos_by_addresses`等 | ウォレット向け照会の意味 |

リポジトリのディレクトリ名や関数名は更新されるため、上記は**探索の手掛かり**であって固定のAPI仕様ではない。

```bash
git clone https://github.com/kaspanet/rusty-kaspa.git
cd rusty-kaspa
rg 'struct (Transaction|TransactionOutpoint|UtxoEntry)' .
rg 'utxo_diff|validate_transaction|mempool' .
```

可能なら調査時のコミットハッシュを記録する。

```bash
git rev-parse HEAD
```

### ソースコード読解の順序

1. 取引とOutpointの型定義を確認する。
2. UTXO参照を解決する関数を追う。
3. 構造検査・署名検査・文脈依存検査の境界を調べる。
4. UTXO差分の作成・適用・巻き戻しを確認する。
5. Mempoolの検査とコンセンサス側の検査を比較する。
6. テストで二重支払い、欠落入力、オーバーフローのケースを探す。

## 15.22 よくある誤解

**誤解1：「アドレスには残高が1つ保存されている」**  
UTXOモデルでは、残高は通常、関連する未使用出力の合計として導かれる。

**誤解2：「入力UTXOの一部分だけを消費できる」**  
入力として参照したUTXOは全体を消費する。差額は変更出力で表現する。

**誤解3：「取引がMempoolにあれば確定している」**  
Mempoolは未確定取引を扱う。コンセンサス状態への受理とは異なる。

**誤解4：「ブロックに含まれれば、取引は必ず受理される」**  
KaspaではブロックのDAG上の存在と、取引の受理を分けて考える必要がある。

**誤解5：「Blueブロック内の取引はすべて受理される」**  
Blue/RedはGHOSTDAGのブロック分類であり、個々の取引のUTXO有効性を直接保証するものではない。

**誤解6：「Rustの`u64`なら金額計算は必ず安全」**  
加算時のオーバーフローや、減算時のアンダーフローを検出する必要がある。

## 15.23 実践課題

### 課題A：UTXO選択

手持ちのUTXOが`[2, 5, 7, 11]`（単位は任意）で、送金額が9、固定手数料が1のとき、入力の組み合わせを3通り以上考え、変更出力の額を求める。

### 課題B：競合取引

同じOutpointを消費する`Tx-A`と`Tx-B`を作り、適用順序を入れ替えた場合のUTXO集合を比較する。**両方を受理できない理由**を説明する。

### 課題C：原子性

`apply`の途中でエラーが起きた場合に状態が変わらないことを確認するテストを追加する。

### 課題D：依存関係

サンプルの`apply`を、`validate`と`commit`に分離する設計を考える。検証結果を別スレッドに渡す場合、**検証後にUTXO状態が変わる競合**をどう防ぐか説明する。

### 課題E：実装との照合

`rusty-kaspa`の対象コミットを固定し、次の3点を調べる。

1. UTXOエントリに含まれるフィールド。
2. 取引検証の文脈非依存部分と文脈依存部分。
3. Virtual Stateの更新に関係するUTXO差分の扱い。

調査結果は、ソースファイル・関数名・コミットハッシュ付きで記録する。

## 15.24 理解度チェック

1. UTXOとウォレット残高は何が違うか。
2. Outpointは何によって一意に識別されるか。
3. 4 KASのUTXOから1 KASを支払うとき、なぜ変更出力が必要か。
4. 通常取引の手数料はどこから求められるか。
5. 同じOutpointを消費する2つの取引が同時に存在した場合、何が問題になるか。
6. Mempoolに入ったことと、Virtual UTXO Stateに反映されたことはどう違うか。
7. BlockDAGが並行ブロックを保持しても、二重支払いが許されないのはなぜか。
8. `checked_add`や`checked_sub`を使う理由は何か。
9. 失敗した取引でUTXO集合を変更してはいけないのはなぜか。
10. `rusty-kaspa`を調べるとき、どの情報を記録すれば説明の再現性が高まるか。

## 15.25 まとめ

本章の中心は、**トランザクションを「残高の増減」ではなく「UTXO集合の決定的な状態遷移」として捉えること**である。

```text
UTXO集合
   │
   ▼
入力Outpointの解決
   │
   ▼
署名・スクリプト・金額などの検証
   │
   ▼
コンセンサス上の順序で受理判定
   │
   ▼
入力UTXOを消費し、出力UTXOを生成
   │
   ▼
新しいUTXO集合
```

Kaspaでは、これにBlockDAG、GHOSTDAG、Virtual Stateという要素が重なる。**ブロックの順序、取引の受理、UTXO状態の更新を分離して理解する**ことが、ノード実装を読むための土台になる。

次章では、UTXOを使うための条件を記述する**Kaspa Scriptとトランザクション署名**を扱う。

## 15.26 参考資料

一次資料を優先し、バージョン差分に注意して読む。

- [Kaspa公式サイト](https://kaspa.org/)
- [Kaspa Wiki](https://wiki.kaspa.org/en/home)
- [rusty-kaspa（公式Rust実装）](https://github.com/kaspanet/rusty-kaspa)
- [Kaspa Improvement Proposals（KIPs）](https://github.com/kaspanet/kips)
- [Kaspa Research Papers](https://kaspa.org/research/)
- [Bitcoin: A Peer-to-Peer Electronic Cash System](https://bitcoin.org/bitcoin.pdf) — UTXOの背景を理解するための比較資料
- [Mastering Bitcoin（オープンソース版）](https://github.com/bitcoinbook/bitcoinbook) — UTXO、取引、Scriptの一般的な概念を学ぶための参考。Kaspa固有の仕様とは区別すること。

---

> **編集メモ（v0.1）**：出版前に、対象とする`rusty-kaspa`のコミットを固定し、取引フィールド、UTXOエントリ、サブネットワーク、Coinbase、署名検証、手数料計算、Virtual State更新の説明を一次資料と照合する。Rustコードは教育用のため、コンセンサス互換性・実運用適性を主張しない。
