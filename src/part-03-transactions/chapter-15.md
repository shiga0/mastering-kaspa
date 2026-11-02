# Chapter 15 — UTXO Model & Transaction Processing（UTXOモデルとトランザクション処理）

> **Mastering Kaspa / Part III — Transactions, Mining & Security**  
> 目次順の本文レビュー：2026-11-01。主要経路は基準版と照合。ページの確認と全規則・全経路の検証は区別する。\
> 本章はKaspaのUTXOモデルとトランザクション処理を理解するための原稿である。Rustの実装例は**教育用の簡略モデル**であり、Kaspaの署名、スクリプト、トランザクション形式、コンセンサス規則を完全再現しない。プロトコル依存の詳細は対象バージョンの`rusty-kaspa`で照合すること。

## 照合した実装

2026年10月31日、[本書の基準版](../baseline.md) v1.0.1、コミット `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` のソースを読み取った。以下の具体的な処理はSource-checkedであり、公式ノードを実行した差分テストや章全体のVerified判定ではない。照合ファイルと未確認範囲はリポジトリの `experiments/consensus-state-review-2026-10-31.md` に記録する。

## この章で学ぶこと

この章では、ブロックの順序を決めた後に、**どのトランザクションがどのUTXOを消費し、新しいUTXOを生み出すのか**を理解する。

- UTXOが「残高」ではなく「未使用の出力」である理由
- Outpoint、Input、Output、Transaction IDの責務
- トランザクションの構造検査、署名・スクリプト検査、UTXO検査の違い
- 手数料、変更出力、二重支払いの判定
- Mempool上の未確定トランザクションとコンセンサス状態の区別
- BlockDAGで順序が変わったときのUTXO状態の再評価
- RustでUTXO状態遷移の最小モデルを作り、不変条件をテストする方法
- 教材の入力・出力・手数料を、公式SDKの取引構築とRPCによる結果確認へ対応付ける方法

---

## 15.1 なぜUTXOモデルを学ぶのか

ウォレットに「10 KAS」と表示されていても、ノード内部に`balance = 10`という単一の変数があるとは限らない。UTXO方式では、利用可能な**未使用のトランザクション出力（Unspent Transaction Output）**を集計して残高を求める。

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

変更出力は必須のフィールドではない。差額を自分の出力として残さなければ、その分も手数料になる。例の0.01 KASは説明用の値であり、実際の必要手数料を示さない。

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

Kaspaのネイティブ資産はKASであり、プロトコルでは整数の最小単位を扱う。基準版の [SOMPI_PER_KASPA](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/constants.rs) は **1 KAS = 100,000,000 sompi** と定義する。

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

Kaspaのトランザクションを理解するため、まず次の概念モデルを使う。

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

この図は主要項目の見取り図である。基準版の[Transaction](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/tx.rs)は `version: u16`、inputs、outputs、`lock_time: u64`、subnetwork_id、`gas: u64`、payloadを持ち、さらにmassのコミットメントとIDキャッシュを保持する。Inputはprevious_outpoint、signature_script、sequence、sig_op_count、Outputはvalueとscript_public_keyを持つ。メモリ上のフィールド列を、そのままシリアライズやIDのハッシュ対象と見なさない。ハッシュ・署名対象については[第16章](chapter-16.md)で扱う範囲と検証記録を参照する。

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

基準版の[UtxoEntry](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/tx.rs)は `amount`、`script_public_key`、`block_daa_score`、`is_coinbase` を保持する。Outpointは32バイトのtransaction_idとu32のindexの組である。特にCoinbase成熟の検査には、金額だけでなく受理時のDAA scoreとCoinbaseであるかの情報が必要になる。

```text
TxA:0 -> { value: 500000000, script: ... }
TxB:1 -> { value: 200000000, script: ... }
```

ウォレットは自分に関係するUTXOを把握したいが、**ウォレットのローカル記録と、ノードが認識するコンセンサスUTXO集合は同一ではない**。同期遅延、未確定トランザクション、再編成などにより見え方が異なることがある。未使用でもCoinbaseの成熟などを満たすまで使えない場合があり、未使用であることと即座に使用可能であることも区別する。

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

これは代表的な流れであり、各ノードが必ずすべての段階を観測するわけではない。Mempoolで見ていない取引をブロック経由で知る場合もある。

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
| コンセンサス文脈 | 現在の順序と状態で受理できるか | 先に競合トランザクションが受理された |
| Mempoolポリシー | 未確定トランザクションとして中継・保管するか | ノードのポリシーに合わない |

**Mempoolポリシーとコンセンサス規則は別物**である。あるノードが中継しないトランザクションでも、コンセンサス規則上の有効性は別途判断する必要がある。

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

一般的なUTXO型のトランザクションでは、手数料は入力と出力の差額として表れる。マイナーへの支払いを「通常の受取人向け出力」として必ず指定するわけではない。

### 15.9.1 トークンの数量とKASを分ける

ここまでの入出力額はネイティブKASである。トークンを扱う場合も、トークン数量と、そのUTXOに載せるKAS、取引手数料を別々に計算する。

| 対象 | 残高・状態を解釈する仕組み | 取り違えてはいけないこと |
|---|---|---|
| ネイティブKAS | コンセンサスが検証するUTXOの`value` | トークンの数量ではない |
| KRC-20 | オンチェーンの操作を専用インデクサーが解釈して残高を再構築する | KASのUTXO応答からKRC-20残高を直接得られるわけではない |
| KCC-20 | Covenantの状態・承認・後継出力の規約に従うトークン | 規約への適合、正しいプログラムと系列の識別が必要。名前だけでは資産を識別できない |

KRC-20の一次資料は[Kasplexの専用ノード](https://github.com/kasplex/go-krc20d/blob/6d21c8668bd77db6d89f28560cc37aaa205f8517/README.md)と[残高API](https://github.com/kasplex/go-krc20d/blob/6d21c8668bd77db6d89f28560cc37aaa205f8517/api/v1address.go)を参照する。Kaspaノードから受理順を取り込み、専用の状態とAPIを持つ構成である。KRC-20の残高と、[KCC-20の状態・移転規則](https://github.com/kaspanet/kccs/blob/1ed87aef59faff8354061aee30f734a3b8b2c3d2/kcc-0020.md)を同じ検証方式として扱わない。KCC-1／2／20／23の役割と文書の状態は[30.14.1](../part-05-advanced/chapter-30.md#30141-kccはアプリケーション間の規約)、系列と偽造の区別は[31.10.1](../part-05-advanced/chapter-31.md#31101-idプログラム状態の出所を分ける)にまとめる。

**トークンだけ送る場合もKASは必要か。** 新しいトークンUTXOにKASを載せ、取引のMass・手数料も満たす必要がある。誰がそのKASを負担するかはアプリの設計事項である。KCC-20のBorrowed Receiveは、対応する受取人の既存UTXOを使い、所有権・拡張状態を保ち、トークン量を増やし、KAS額を減らさない仕組みを定める。すべての受取人に既存の適合UTXOがあるとは限らず、手数料も不要にはならない。

**Storage Massは預金ではない。** [第19章](chapter-19.md)と[付録E](../appendices/appendix-e.md)で扱う資源評価・手数料の規則であり、支払った手数料が後日自動返金される制度ではない。UTXOの`value`として残るKASを回収できるかは、そのScriptの支出・終了条件に依存する。トークンを焼却すれば必ず全額回収できると仮定しない。

**KAS20は別の提案である。** 2026-11-03に確認した[KCC-20参照実装へのPR #4](https://github.com/argent-lang/kcc20-reference/pull/4)はOpen・未マージで、[固定例](https://github.com/Manyfestation/kcc20-reference/blob/2f7cc78f13744edeb24823f6e931b5ab0774de77/examples/kas20/README.md)は、各トークン出力のKAS額とトークン基底単位を一致させ、所有者によるburnで裏付けKASを取り出す設計を示す。これは合成資金で動かすオフライン例の説明で、本書では実行していない。正規デプロイの指定やMainnetでの利用確認ではなく、別Covenant IDのデプロイは別資産となる。

### 15.9.2 UTXO検索の版とページネーション

本書v1.0.1の`getUtxosByAddresses`実習と、後続版のAPIを混ぜない。2026-11-03に読んだ公式masterのコミット`b85380d16514e65136001530e7c717eef304e654`には、[GetUtxosByAddressesV2](https://github.com/kaspanet/rusty-kaspa/blob/b85380d16514e65136001530e7c717eef304e654/rpc/core/src/model/message.rs)の`cursor`・`limit`と応答の`next_cursor`がある。SDK・ノードの対応版を合わせて使い、次カーソルがなくなるまで取得する。ただし、このソース確認だけで実機接続・リリース・全ネットワークでの提供を検証したことにはならない。

ページ間にはUTXOの消費・追加・再編成があり得る。単一スナップショットだと仮定せず、outpointで重複を排除し、変更通知と再照会を照合する。途中失敗時の再開・最初からの再取得、同一カーソルの繰り返しも扱う。

Covenant ID別の検索用インデックス（Covenant Index）とページ付きRPCは、同日確認した[rusty-kaspa PR #1152](https://github.com/kaspanet/rusty-kaspa/pull/1152)ではOpen・未マージだった。Covenant IDを持つUTXOがあることと、対象ノードに検索APIが提供されていることは別である。本書のSDK実習を、未確認のAPI名へ置換しない。

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

いずれも送金額に加えて必要な手数料を賄えることが前提となる。どちらが適切かは、手数料、入力数、UTXO断片化、プライバシー、将来の支出予定による。UTXO選択は単なる金額合わせではなく、**ウォレットのポリシーと最適化問題**である。

### プライバシー上の注意

複数のUTXOを同じトランザクションで入力として使うと、それらが共通の管理下にあると推測されることがある。ただし、これは常に正しい推論ではない。

## 15.11 二重支払いは何を競合させるのか

```text
           UTXO X:0
           /      \
          /        \
      Tx-A          Tx-B
    Aliceへ       Bobへ
```

`Tx-A`と`Tx-B`が同じ`X:0`を入力として消費しようとすれば、**両方を同じUTXO状態に適用することはできない**。

線形チェーンではチェーン上の順序が、KaspaではBlockDAGから導かれるコンセンサス上の順序と受理規則が、どちらのトランザクションを適用できるかに関係する。

ただし、**ブロックがDAGに存在することは、そのブロックに含まれる全トランザクションがUTXO状態に受理されたことを意味しない**。

## 15.12 並行ブロックとトランザクションの受理

Kaspaでは、ネットワーク遅延によって複数のブロックが並行して生成される。

```text
          Block A: Tx-A
         /             \
Genesis                 Merge
         \             /
          Block B: Tx-B
```

図は左が親、右が子で、実際の参照は右から左へ向かう。具体的なGHOSTDAGの順序や受理結果を表すものではない。

同じUTXOを消費するトランザクションが別々の並行ブロックに含まれた場合でも、コンセンサスが導く処理順序に沿って状態を評価する必要がある。**DAGに複数ブロックを保持できることと、二重支払いを認めることは全く違う**。

## 15.13 Virtual StateとUTXO状態遷移

前章のVirtual Stateを、UTXOの視点から捉え直す。

```text
旧Virtual UTXO Set
        │
        │ 旧状態との差分を戻し、新しい文脈で再評価・適用
        ▼
新Virtual UTXO Set
```

通常のトランザクションを適用する概念的な手順は次のとおり。

1. 入力OutpointがUTXO集合に存在するか調べる。
2. 入力の重複やスクリプト条件などを検証する。
3. 入力合計と出力合計を安全に計算する。
4. 受理可能なら入力UTXOを削除する。
5. 新しい出力をUTXO集合に追加する。
6. 必要なインデックスや状態情報を更新する。

**原子性**が重要である。途中で失敗したときに「入力だけ消えた」状態を残してはいけない。

### 15.13.1 基準版の検証と教材の差

[UTXO文脈の検証](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/transaction_validator/tx_validation_in_utxo_context.rs)はCoinbase成熟、入力合計、出力合計、手数料、フォーク条件に応じたmass、sequence lock、Script等を検査する。手数料は検査済みの入力合計から出力合計を引いて得る。共通関数内にはMempoolの手数料率ポリシー用の検査もあるため、すべての分岐をコンセンサス規則と同一視しない。

[ブロックbodyの検証](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/body_processor/body_validation_in_isolation.rs)は、同一ブロック内の二重消費と、同じブロック内の出力を入力にする親子トランザクションを拒否する。UTXO文脈では、独立な通常トランザクションを同じviewに対して並列検証できる。後半の教材はこのbody検証を省略しており、先に適用したトランザクションの出力を同じ教材内で続けて消費できることをKaspaの仕様と解釈しない。

入力解決後のCoinbase成熟では、基準版は `entry.block_daa_score + maturity(block_daa_score) <= pov_daa_score` を必要とする。エントリの受理スコア、成熟パラメータを選ぶスコア、評価時点のスコアを同じ変数とみなさない。

入力合計はu64に収まるだけでなく `MAX_SOMPI` 以下かも確認する。後半の教材はchecked arithmeticだけで、公式の金額上限や成熟条件は再現しない。

merge set全体の処理は[第14章14.5.1](../part-02-consensus/chapter-14.md#1451-基準版の実際の処理順)を参照する。selected parentのCoinbaseを特別に適用した後、通常トランザクションの受理を評価する。ブロック自身のUTXO妥当性の検証と、後続のmerge setでの各トランザクションの受理を区別する。

## 15.14 Reorganizationと状態の巻き戻し

新しいブロックの到着によって、コンセンサス上の選択や順序が変化することがある。その場合、以前のVirtual Stateから新しいVirtual Stateへの更新が必要になる。

概念的には次のいずれかの設計が考えられる。

- 変更前の状態に戻し、新しい順序でトランザクションを再適用する。
- 状態差分（差分ログ）を用いて巻き戻しと適用を行う。
- 適切なチェックポイントから状態を再構築する。

この一覧は一般的な選択肢である。基準版は[UtxoDiffの逆差分・合成](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/utxo/utxo_diff.rs)と[virtual processorの相対状態計算](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/virtual_processor/processor.rs)を使う。実際の更新・保存経路は[第14章14.11.1](../part-02-consensus/chapter-14.md#14111-基準版の差分処理)で扱う。

### ウォレット側で起きること

以前「受理済み」と見えていたトランザクションが、再編成や同期状態の変化によって別の扱いになる可能性がある。そのため、アプリケーションは単一の受信イベントだけで不可逆な決済完了を決めない。

## 15.15 MempoolとUTXO集合は同じではない

| 対象 | 役割 |
|---|---|
| コンセンサスUTXO集合 | コンセンサスが認識する利用可能な出力を表す |
| Mempool | ノードが中継・採掘等のために保持する取引候補。保持はコンセンサス上の受理・決済確定ではない |
| ウォレットのUTXO表示 | アドレスに関連する資金をユーザーに示す |

未確定トランザクションが使うUTXOを別の未確定トランザクションが使おうとする場合、Mempoolでは競合管理が必要になる。また、親トランザクションが未確定のまま、その出力を子トランザクションが参照することも概念的には可能だが、**具体的な受け入れ条件は実装・ポリシーに依存する**。

## 15.16 Coinbaseと通常トランザクションの違い

通常のトランザクションは既存UTXOを入力として消費する。一方、Coinbaseはマイニング報酬等を扱う特別なトランザクションであり、通常の「入力合計−出力合計＝手数料」という説明をそのまま適用できない。

また、Kaspaではブロック報酬、手数料、Coinbaseの扱いがDAG上のコンセンサスと関係する。BitcoinのCoinbase処理を単純にコピーして説明しないこと。詳細は[第18章](chapter-18.md)・[第19章](chapter-19.md)で扱う。

## 15.17 Rustで最小UTXO状態遷移を作る

ここからは**Kaspa互換ではない教育用モデル**を作る。署名検証、Script、Coinbase、シリアライズ、DAG順序は省略し、UTXOの消費と生成に集中する。

[第5章](../part-01-foundations/chapter-05.md)では個々の取引の検証と更新、[第14章](../part-02-consensus/chapter-14.md)では処理順序と結果の関係を学んだ。本章では、**所有者情報を持つ出力・手数料・お釣り**に注目する。`transfer_consumes_input_and_creates_outputs` で送金先と変更先の出力を確認し、`fees_and_dependent_outputs_preserve_value` で残った出力と手数料の合計が初期金額と一致することを追う。`owner` は説明用のラベルであり、所有者の支出権限は検証しない。続く15.17.1で、これらを公式SDKの送金処理へ対応付ける。

### プロジェクト作成

本書とは別の作業用ディレクトリで実行する。Rust 1.90.0で確認した。

```bash
cargo new --lib --edition 2021 utxo-lab
cd utxo-lab
```

`src/lib.rs`を次の内容にする。

```rust
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;

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
    EmptyTransactionId,
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
    // 教材の初期状態を用意する補助。通常取引の検証ではない。
    pub fn insert_initial_output(
        &mut self,
        outpoint: Outpoint,
        output: Output,
    ) -> Result<(), ApplyError> {
        if outpoint.txid.is_empty() {
            return Err(ApplyError::EmptyTransactionId);
        }
        if self.entries.contains_key(&outpoint) {
            return Err(ApplyError::DuplicateOutput);
        }
        self.entries.insert(outpoint, output);
        Ok(())
    }

    pub fn get(&self, outpoint: &Outpoint) -> Option<&Output> {
        self.entries.get(outpoint)
    }

    pub fn apply(&mut self, tx: &Transaction) -> Result<u64, ApplyError> {
        if tx.txid.is_empty() {
            return Err(ApplyError::EmptyTransactionId);
        }
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
                index: u32::try_from(index).expect("output index checked before mutation"),
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
        Outpoint {
            txid: "genesis".into(),
            index: 0,
        }
    }

    fn funded() -> UtxoSet {
        let mut set = UtxoSet::default();
        set.insert_initial_output(
            original(),
            Output {
                value: 100,
                owner: "alice".into(),
            },
        )
        .unwrap();
        set
    }

    #[test]
    fn transfer_consumes_input_and_creates_outputs() {
        let mut set = funded();
        let tx = Transaction {
            txid: "tx1".into(),
            inputs: vec![original()],
            outputs: vec![
                Output {
                    value: 60,
                    owner: "bob".into(),
                },
                Output {
                    value: 35,
                    owner: "alice".into(),
                },
            ],
        };
        assert_eq!(set.apply(&tx), Ok(5));
        assert_eq!(set.get(&original()), None);
        assert_eq!(
            set.entries,
            HashMap::from([
                (
                    Outpoint {
                        txid: "tx1".into(),
                        index: 0
                    },
                    Output {
                        value: 60,
                        owner: "bob".into()
                    }
                ),
                (
                    Outpoint {
                        txid: "tx1".into(),
                        index: 1
                    },
                    Output {
                        value: 35,
                        owner: "alice".into()
                    }
                ),
            ])
        );
    }

    #[test]
    fn double_spend_is_rejected() {
        let mut set = funded();
        let first = Transaction {
            txid: "first".into(),
            inputs: vec![original()],
            outputs: vec![Output {
                value: 90,
                owner: "bob".into(),
            }],
        };
        let second = Transaction {
            txid: "second".into(),
            inputs: vec![original()],
            outputs: vec![Output {
                value: 90,
                owner: "carol".into(),
            }],
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
            outputs: vec![Output {
                value: 101,
                owner: "bob".into(),
            }],
        };
        assert_eq!(set.apply(&tx), Err(ApplyError::Overspend));
        assert_eq!(set.entries, before.entries);
    }

    #[test]
    fn initial_outputs_cannot_overwrite_existing_entries() {
        let mut set = funded();
        let before = set.entries.clone();
        assert_eq!(
            set.insert_initial_output(
                original(),
                Output {
                    value: 999,
                    owner: "other".into()
                }
            ),
            Err(ApplyError::DuplicateOutput)
        );
        assert_eq!(
            set.insert_initial_output(
                Outpoint {
                    txid: "".into(),
                    index: 0
                },
                Output {
                    value: 1,
                    owner: "alice".into()
                }
            ),
            Err(ApplyError::EmptyTransactionId)
        );
        assert_eq!(set.entries, before);
    }

    #[test]
    fn validation_errors_preserve_all_entries() {
        let mut set = funded();
        let large = Outpoint {
            txid: "large".into(),
            index: 0,
        };
        set.insert_initial_output(
            large.clone(),
            Output {
                value: u64::MAX,
                owner: "alice".into(),
            },
        )
        .unwrap();
        let before = set.entries.clone();
        let output = Output {
            value: 90,
            owner: "bob".into(),
        };
        for (tx, expected) in [
            (
                Transaction {
                    txid: "".into(),
                    inputs: vec![original()],
                    outputs: vec![output.clone()],
                },
                ApplyError::EmptyTransactionId,
            ),
            (
                Transaction {
                    txid: "empty-inputs".into(),
                    inputs: vec![],
                    outputs: vec![output.clone()],
                },
                ApplyError::EmptyInputs,
            ),
            (
                Transaction {
                    txid: "empty-outputs".into(),
                    inputs: vec![original()],
                    outputs: vec![],
                },
                ApplyError::EmptyOutputs,
            ),
            (
                Transaction {
                    txid: "duplicate-input".into(),
                    inputs: vec![original(), original()],
                    outputs: vec![output.clone()],
                },
                ApplyError::DuplicateInput,
            ),
            (
                Transaction {
                    txid: "missing".into(),
                    inputs: vec![
                        original(),
                        Outpoint {
                            txid: "absent".into(),
                            index: 0,
                        },
                    ],
                    outputs: vec![output.clone()],
                },
                ApplyError::MissingInput,
            ),
            (
                Transaction {
                    txid: "input-overflow".into(),
                    inputs: vec![original(), large],
                    outputs: vec![output.clone()],
                },
                ApplyError::ValueOverflow,
            ),
            (
                Transaction {
                    txid: "output-overflow".into(),
                    inputs: vec![original()],
                    outputs: vec![
                        Output {
                            value: u64::MAX,
                            owner: "bob".into(),
                        },
                        Output {
                            value: 1,
                            owner: "alice".into(),
                        },
                    ],
                },
                ApplyError::ValueOverflow,
            ),
            (
                Transaction {
                    txid: "genesis".into(),
                    inputs: vec![original()],
                    outputs: vec![output.clone()],
                },
                ApplyError::DuplicateOutput,
            ),
            (
                Transaction {
                    txid: "overspend".into(),
                    inputs: vec![original()],
                    outputs: vec![Output {
                        value: 101,
                        owner: "bob".into(),
                    }],
                },
                ApplyError::Overspend,
            ),
        ] {
            assert_eq!(set.apply(&tx), Err(expected));
            assert_eq!(set.entries, before);
        }
    }

    #[test]
    fn fees_and_dependent_outputs_preserve_value() {
        let mut set = funded();
        let first = Transaction {
            txid: "first".into(),
            inputs: vec![original()],
            outputs: vec![
                Output {
                    value: 60,
                    owner: "bob".into(),
                },
                Output {
                    value: 35,
                    owner: "alice".into(),
                },
            ],
        };
        let second = Transaction {
            txid: "second".into(),
            inputs: vec![Outpoint {
                txid: "first".into(),
                index: 0,
            }],
            outputs: vec![Output {
                value: 50,
                owner: "carol".into(),
            }],
        };
        let before = set.entries.clone();
        assert_eq!(set.apply(&second), Err(ApplyError::MissingInput));
        assert_eq!(set.entries, before);
        let first_fee = set.apply(&first).unwrap();
        let second_fee = set.apply(&second).unwrap();
        assert_eq!((first_fee, second_fee), (5, 10));
        assert_eq!(
            set.entries.values().map(|o| o.value).sum::<u64>() + first_fee + second_fee,
            100
        );
        assert_eq!(
            set.entries,
            HashMap::from([
                (
                    Outpoint {
                        txid: "first".into(),
                        index: 1
                    },
                    Output {
                        value: 35,
                        owner: "alice".into()
                    }
                ),
                (
                    Outpoint {
                        txid: "second".into(),
                        index: 0
                    },
                    Output {
                        value: 50,
                        owner: "carol".into()
                    }
                ),
            ])
        );
    }

    #[test]
    fn duplicate_inputs_are_rejected() {
        let mut set = funded();
        let tx = Transaction {
            txid: "duplicate".into(),
            inputs: vec![original(), original()],
            outputs: vec![Output {
                value: 100,
                owner: "bob".into(),
            }],
        };
        assert_eq!(set.apply(&tx), Err(ApplyError::DuplicateInput));
    }
}
```

テストを実行する。

```bash
cargo test
```

本文から抽出した教材は7テスト成功。初期出力の上書き拒否、検査エラー時の集合全体の維持、手数料と変更出力、依存先到着前後の再試行を確認した（`experiments/chapter-15-review-2026-11-01.md`）。

`insert_initial_output` は教材の初期状態を作る補助であり、取引検証や実KaspaのGenesis報酬を再現しない。`genesis`、100、ownerは架空の入力である。

このサンプルは`txid`を文字列で直接受け取るため、**本物のトランザクションIDの一意性や署名の正当性を保証しない**。また、所有者`owner`も単なる文字列であり、支出権限を検証していない。現在のUTXOとの衝突を検査するだけで、消費済みIDの履歴管理や再送の冪等な応答は持たない。

出力番号の範囲を超える巨大入力、MAX_SOMPI、成熟条件、mass、永続化・クラッシュは今回の教材テストでは扱わない。Errでのメモリ内の状態維持と、障害時の耐久性は区別する。

### 15.17.1 教材から公式SDKの送金処理へ

前節の`UtxoSet::apply`はノード側の状態更新を考える教材である。アプリ開発では、この関数を自分の残高へ適用して送金を成立させるのではなく、**公式SDKで取引を構築・署名してノードへ送り、ノードが受理した結果を照会する**。

ここでは、実行確認済みの`experiments/devnet/send.cjs`を読み、教材との対応を追う。実装全体は[収録ソース](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/devnet/send.cjs)を参照する。公式v1.0.1 Node.js/WASM SDKを使った実例であり、前節のRust型をSDKへ渡すコードではない。

#### 実行する入口

[第28章28.6](../part-04-development/chapter-28.md#286-ローカル環境の準備)で公式SDKとDevnet環境を用意し、[28.10の送金実習](../part-04-development/chapter-28.md#2810-トランザクションのe2eテスト)で`send.cjs --checks`を実行する。スクリプトは一時鍵の作成・採掘・成熟待ち・送金・検査まで行い、鍵を保存・表示しない。採掘にはlinux/arm64の既存構成を使う。ノードの起動・終了と固定コンテナ名の条件は28.10に従う。

以下はその実装からの抜粋である。単独では実行せず、接続や一時鍵、待機処理を含むスクリプト全体で動かす。

#### 1. 参照するUTXOを取得する

```javascript
{{#include ../../experiments/devnet/send.cjs:mature-input}}
```

教材の`insert_initial_output`に相当する初期データを架空の値で作る代わりに、`getUtxosByAddresses`でノードから取得する。この実習は自分で採掘した報酬を使うため、SDKのネットワーク設定から成熟期間を取得して入力候補を絞る。

ここで行うのは報酬の成熟条件による候補選びであり、全Script・Lock Time等を含む支出可能性の検証ではない。通常のウォレットに必要な継続監視・入力予約・複数UTXOの選択方針も、この一件だけの実験とは分けて設計する。

#### 2. 取引を構築し、手数料とお釣りを検算する

```javascript
{{#include ../../experiments/devnet/send.cjs:build-and-fee}}
```

`entries`はSDKが受け取るUTXO参照、`outputs`は送金先と金額、`changeAddress`はお釣りの宛先である。金額はsompiの`bigint`で指定する。`priorityFee: 0n`でも必要な手数料全体が0になるわけではない。

`createTransactions`は複数取引を返すことがある。この実験は入力を一つに限定し、返された取引も一つであることを検査してから、その入力・出力合計を比較している。一般のウォレットで、全候補UTXOの合計をそのまま一取引の入力合計として扱ってはいけない。

2026-10-31の`--checks`実測では、次の関係になった。手数料は当時の入力条件での観測値であり、固定の推奨値ではない。

| 項目 | 実測値（sompi） |
|---|---:|
| 入力合計 | 50,000,000,000 |
| 送金先出力 | 100,000,000 |
| 変更出力 | 49,899,990,000 |
| 出力合計 | 49,999,990,000 |
| 手数料＝入力合計−出力合計 | 10,000 |

教材の金額保存・変更出力の考え方は、この検算でそのまま使える。実測データはリポジトリの`experiments/devnet/send-checks-observed-2026-10-31.json`にある。

#### 3. 署名・送信後の状態を照会する

スクリプトは`pending.sign([sourceKey])`で署名し、通常の`--checks`経路では`pending.submit(rpc)`で送信する。署名の仕組みは[第16章](chapter-16.md)で扱う。送信成功の返答だけで、教材の`apply`に相当する状態更新が完了したとは判断しない。

```javascript
{{#include ../../experiments/devnet/send.cjs:observe-transfer}}
```

宛先UTXOが存在すること、virtual chainの受理結果にTX IDがあること、元の入力が現在UTXOから消えたこと、お釣りが検算値と一致することを確認する。これらの照会は同一瞬間の原子的なスナップショットではなく、再編成を含む履歴台帳や不可逆な決済確定を保証するものでもない。

#### 教材と実APIの対応

| 教材で扱ったもの | 実装で対応するもの |
|---|---|
| `Outpoint`で既存出力を探す | SDKの`entry.outpoint.transactionId`と`index` |
| `owner`で所有者を表す | 実UTXOの支出条件と、それを満たす鍵・署名 |
| 入力・出力の金額保存 | 取引構築後の合計値と`pending.feeAmount`の検算 |
| `apply`で入力を消費し出力を生成 | ノードの検証・コンセンサス処理。アプリはRPCで結果を観測 |
| エラー時に集合を変えない | `--checks`で不正署名が拒否され、入力が残ることを確認 |

まずUTXOを読むだけなら[公式SDK監視サンプル](../part-04-development/chapter-28.md#2864-公式sdkを使った最小監視サンプル)、送金の失敗理由や応答喪失も追うなら28.10の各モードへ進む。教材の`UtxoSet`を本番の台帳として流用するのではなく、学んだ不変条件を実際のAPIの入力・出力・検査へ対応付ける。

## 15.18 Rust実装の責務を分離する

上記の`apply`は学習のため多くの処理をまとめている。実際の設計では責務を分けたい。

```text
Transaction Decoder
        │
        ▼
Structural Validator
        │
        ▼
UTXO Resolver
        │
        ▼
Signature / Script Validator
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

この図は責務の整理であり、公式関数の実行順ではない。Script検証には入力が参照するUTXOの金額・スクリプト等が必要なため、入力の解決を先に示した。

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
| INV-01 | 1つのトランザクションで同じ入力Outpointを重複使用しない | `HashSet`で重複検出 |
| INV-02 | 使用する入力が存在する | 全入力を状態から解決 |
| INV-03 | 通常トランザクションの出力合計が入力合計を超えない | checked arithmetic |
| INV-04 | 失敗したトランザクションでUTXO集合を変更しない | 検査後にコミット |
| INV-05 | 既存UTXOを誤って上書きしない | 出力Outpointの衝突検査 |
| INV-06 | 同一の状態と同一のトランザクション順序から同じ結果を得る | 決定的な状態遷移 |
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
| トランザクションデータ構造 | `Transaction`, `TransactionInput`, `TransactionOutput` | フィールド、型、バージョン |
| UTXO識別 | `TransactionOutpoint`, `UtxoEntry` | Outpointの表現とメタデータ |
| トランザクション検証 | `transaction_validator`, `validate_transaction` | 文脈非依存／依存の区別 |
| UTXO差分 | `utxo_diff`, `utxo_set` | 状態更新とロールバック |
| 署名検証 | `script`, `signature` | どの層で支出条件を判定するか |
| Mempool | `mempool`, `orphan` | 未確定トランザクションの扱い |
| RPC | `get_utxos_by_addresses`等 | ウォレット向け照会の意味 |

リポジトリのディレクトリ名や関数名は更新されるため、上記は**探索の手掛かり**であって固定のAPI仕様ではない。

```bash
# 第1章で取得した基準版の親ディレクトリから実行する
cd rusty-kaspa-v1.0.1
git rev-parse HEAD
grep -n -E 'struct (Transaction|TransactionOutpoint|UtxoEntry)' consensus/core/src/tx.rs
grep -n -E 'validate_tx_in_isolation|MAX_SOMPI|check_duplicate' consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs
grep -n -E 'coinbase_maturity|input_amounts|check_scripts|feerate' consensus/src/processes/transaction_validator/tx_validation_in_utxo_context.rs
```

未取得なら[第1章](../part-01-foundations/chapter-01.md)の手順を使う。HEADが本章冒頭の完全SHAと一致することを確認する。

### ソースコード読解の順序

1. トランザクションとOutpointの型定義を確認する。
2. UTXO参照を解決する関数を追う。
3. 構造検査・署名検査・文脈依存検査の境界を調べる。
4. UTXO差分の作成・適用・巻き戻しを確認する。
5. Mempoolの検査とコンセンサス側の検査を比較する。
6. テストで二重支払い、欠落入力、オーバーフローのケースを探す。

## 15.22 よくある誤解

**誤解1：「アドレスには残高が1つ保存されている」**  
UTXOモデルでは、残高は通常、関連する未使用出力の合計として導かれる。

**誤解2：「入力UTXOの一部分だけを消費できる」**  
入力として参照したUTXOは全体を消費する。手元に残す差額は変更出力で表現し、入力と全出力の差額は手数料となる。

**誤解3：「トランザクションがMempoolにあれば確定している」**\
Mempoolは未確定トランザクションを扱う。コンセンサス状態への受理とは異なる。

**誤解4：「ブロックに含まれれば、トランザクションは必ず受理される」**\
KaspaではブロックのDAG上の存在と、トランザクションの受理を分けて考える必要がある。

**誤解5：「blueブロック内のトランザクションはすべて受理される」**\
blue/redはGHOSTDAGのブロック分類であり、個々のトランザクションのUTXO有効性を直接保証するものではない。

**誤解6：「Rustの`u64`なら金額計算は必ず安全」**  
加算時のオーバーフローや、減算時のアンダーフローを検出する必要がある。

## 15.23 実践課題

### 課題A：UTXO選択

手持ちのUTXOが`[2, 5, 7, 11]`（単位は任意）で、送金額が9、固定手数料が1のとき、入力の組み合わせを3通り以上考え、変更出力の額を求める。

### 課題B：競合トランザクション

同じOutpointを消費する`Tx-A`と`Tx-B`を作り、適用順序を入れ替えた場合のUTXO集合を比較する。**両方を受理できない理由**を説明する。

### 課題C：原子性

掲載済みの `validation_errors_preserve_all_entries` を読み、各入力がどの検査で失敗し、UTXO集合全体が変わらないことをどう確認しているか説明する。そのうえで、未検査の入力の組み合わせを一つ考え、期待するエラーと、失敗後に維持すべき状態を先に書いてからテストを追加する。たとえば、複数の入力の合計を超える出力を指定したとき、すべての入力が未使用のまま残ることを確かめる。

### 課題D：依存関係

サンプルの`apply`を、`validate`と`commit`に分離する設計を考える。検証結果を別スレッドに渡す場合、**検証後にUTXO状態が変わる競合**をどう防ぐか説明する。

### 課題E：実装との照合

`rusty-kaspa`の対象コミットを固定し、次の3点を調べる。

1. UTXOエントリに含まれるフィールド。
2. トランザクション検証の文脈非依存部分と文脈依存部分。
3. Virtual Stateの更新に関係するUTXO差分の扱い。

調査結果は、ソースファイル・関数名・コミットハッシュ付きで記録する。

## 15.24 理解度チェック

1. UTXOとウォレット残高は何が違うか。
2. Outpointは何によって一意に識別されるか。
3. 4 KASのUTXOから1 KASを支払い、手数料以外の差額を手元に残すには、なぜ変更出力が必要か。
4. 通常トランザクションの手数料はどこから求められるか。
5. 同じOutpointを消費する2つのトランザクションが同時に存在した場合、何が問題になるか。
6. Mempoolに入ったことと、Virtual UTXO Stateに反映されたことはどう違うか。
7. BlockDAGが並行ブロックを保持しても、二重支払いが許されないのはなぜか。
8. `checked_add`や`checked_sub`を使う理由は何か。
9. 失敗したトランザクションでUTXO集合を変更してはいけないのはなぜか。
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

Kaspaでは、これにBlockDAG、GHOSTDAG、Virtual Stateという要素が重なる。**ブロックの順序、トランザクションの受理、UTXO状態の更新を分離して理解する**ことが、ノード実装を読むための土台になる。

次章では、UTXOを使うための条件を記述する**[Kaspa Scriptとトランザクション署名](chapter-16.md)**を扱う。

## 15.26 参考資料

一次資料を優先し、バージョン差分に注意して読む。

- [Kaspa公式サイト](https://kaspa.org/)
- [Kaspa Wiki](https://wiki.kaspa.org/en/home)
- [rusty-kaspa（公式Rust実装）](https://github.com/kaspanet/rusty-kaspa)
- [Kaspa Improvement Proposals（KIPs）](https://github.com/kaspanet/kips)
- [Kaspa Research Papers](https://kaspa.org/research/)
- [Bitcoin: A Peer-to-Peer Electronic Cash System](https://bitcoin.org/bitcoin.pdf) — UTXOの背景を理解するための比較資料
- [Mastering Bitcoin（オープンソース版）](https://github.com/bitcoinbook/bitcoinbook) — UTXO、トランザクション、Scriptの一般的な概念を学ぶための参考。Kaspa固有の仕様とは区別すること。

---

> **残る確認:** ハッシュ・シリアライズ・署名の完全照合、subnetwork固有の規則、報酬生成の詳細、公式実装との実行時の差分テストは未完了。主要な型と状態計算をソースで確認した範囲を章全体へ拡張しない。

目次順の本文レビューと改訂教材7テストは `experiments/chapter-15-review-2026-11-01.md`。主要経路と旧教材4テストの記録は `experiments/consensus-state-review-2026-10-31.md` を参照する。今回、公式テストやノードを新規に実行したものではない。
