# Chapter 05 — Bitcoin Architecture（Bitcoinのアーキテクチャ）

> **Mastering Kaspa** / Part I — Foundations（基礎編）  
> 基礎概説・UTXO教材を確認：2026-10-31。Kaspa側は[本書の基準版](../baseline.md)を参照。
>
> 本章の位置づけ：KaspaのBlockDAGを学ぶ前に、Bitcoinの構成要素と、それぞれがどのように連携して分散台帳を維持するかを理解する。

## 1. はじめに

Bitcoinは、中央の台帳管理者を置かずに価値の移転を記録する分散型システムである。その仕組みは「ブロックチェーン」という一つのデータ構造だけでは説明できない。秘密鍵と署名、トランザクション、UTXO、スクリプト、Mempool、P2Pネットワーク、マイニング、コンセンサス、ストレージ、ウォレットなど、異なる責務を持つ要素が組み合わさっている。

KaspaはBitcoinと同様にProof of Work（PoW）とUTXOモデルを採用する一方、ブロックの構造と順序付けにBlockDAGとGHOSTDAGを使用する。**何がBitcoinから引き継がれ、何がKaspaで変わったのか**を見分けるためには、まずBitcoinのアーキテクチャを理解する必要がある。

本章ではBitcoinを「部品の一覧」としてではなく、**データの流れ・責務・不変条件・依存関係**として捉える。

## 2. この章で学ぶこと

本章を読み終えると、次のことを説明できるようになる。

1. Bitcoinのノード、ウォレット、マイナーの役割の違い
2. トランザクションが生成され、伝播し、ブロックに含まれるまでの流れ
3. UTXOモデルと署名・スクリプトによる支出条件
4. Mempoolとブロックチェーンの役割の違い
5. ブロックヘッダー、Merkle root、PoWの関係
6. フォークと累積チェーンワークによるチェーン選択
7. Bitcoinの主要なセキュリティ上の不変条件
8. BitcoinとKaspaで共通する構造と異なる構造
9. 小さなUTXO検証器をRustで実装する考え方

---

## 3. Bitcoinの全体像

### 3.1 役割と接続の概念図

```text
┌────────────────────────────────────────────────────────────┐
│                    利用者・外部サービス                    │
│      Wallet UI / Exchange / Explorer / Application        │
└───────────────────────────┬────────────────────────────────┘
                            │ RPC / P2P / 独自API
┌───────────────────────────▼────────────────────────────────┐
│                     Bitcoin Full Node                     │
│                                                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐ │
│  │ P2P Network  │  │ Transaction  │  │ Consensus Rules  │ │
│  │ & Relay      │──│ Validation   │──│ & Chainstate     │ │
│  └──────┬───────┘  └──────┬───────┘  └─────────┬────────┘ │
│         │                 │                    │          │
│  ┌──────▼───────┐  ┌──────▼───────┐  ┌─────────▼────────┐ │
│  │ Block        │  │ Mempool      │  │ UTXO Set         │ │
│  │ Validation   │  │              │  │                  │ │
│  └──────┬───────┘  └──────────────┘  └─────────┬────────┘ │
│         │                                      │          │
│  ┌──────▼──────────────────────────────────────▼────────┐ │
│  │             Blocks / Index / Storage                 │ │
│  └──────────────────────────────────────────────────────┘ │
└───────────────────────────┬────────────────────────────────┘
                            │
                      Other Peers
```

この図は学習用の論理構成であり、線は呼び出しや処理の厳密な順序を示すものではない。Bitcoin Coreのソースコード上のモジュール境界をそのまま示すものでもない。また、ウォレットはフルノードと同じプロセスで動作する場合も、外部に分離される場合もある。

### 3.2 主な構成要素

| 要素      | 主な責務                             | 保持・処理するもの                   |
| --------- | ------------------------------------ | ------------------------------------ |
| Wallet    | 鍵の管理、支払いの作成、署名         | 秘密鍵、公開鍵、アドレス、支払い候補 |
| Full Node | ルールの独立検証                     | ブロック、トランザクション、UTXO集合 |
| P2P       | ピア発見、データの伝播               | トランザクション、ブロックなど       |
| Mempool   | 未承認トランザクションの一時管理     | 検証済みの未承認トランザクション候補             |
| Miner     | ブロック候補の構築、PoW計算          | ブロックテンプレート、Nonceなど      |
| Consensus | ブロック・トランザクションの有効性とチェーン選択 | 検証規則、チェーンワーク、状態       |
| Storage   | データの永続化と検索                 | ブロック、インデックス、チェーン状態 |

**ウォレット、フルノード、マイナーは同義ではない。** ウォレットは署名を行い、フルノードはルールを検証し、マイナーはPoWによりブロックを提案する。実際のソフトウェアでは複数の役割が同居することもある。

## 4. トランザクションのライフサイクル

Bitcoinの支払いを、送信者から受信者へ向かう一連の状態遷移として見る。

```text
送信者のウォレット
    │ ① 支払い先と金額を指定
    ▼
UTXOの選択・手数料の見積もり
    │ ② Inputs / Outputs を組み立てる
    ▼
署名・トランザクション生成
    │ ③ ローカルで形式を確認
    ▼
ノードへ送信
    │ ④ ノードが検証
    ▼
Mempoolへ登録（受理された場合）
    │ ⑤ P2Pで伝播
    ▼
マイナーがブロック候補に選択
    │ ⑥ 有効なPoWを発見
    ▼
ブロック伝播・フルノードで検証
    │ ⑦ 最良の有効チェーンに含まれる
    ▼
UTXO集合が更新される
```

これは一般的な経路の例であり、全ノードのMempoolを順に通る必要はない。自分のMempoolになかったトランザクションを含むブロックも、規則を満たせば受け入れられる。

注意点：**Mempoolへの受理は、コンセンサス上の確定を意味しない。** Mempoolはノードごとのローカルなポリシーにも左右され、ネットワーク全体で完全に同じ内容になるとは限らない。

### 4.1 ウォレットによる作成

ウォレットは、支出可能なUTXOを選び、送金先と必要に応じてお釣りの出力を作成する。署名に使う秘密鍵そのものを支出の証明として送信することはない。

### 4.2 ノードによる検証

ノードは、トランザクションの構文、入力参照、金額の整合性、スクリプトの実行結果、二重支払いの有無などを確認する。**署名が正しいだけではトランザクション全体が有効とは限らない。**

### 4.3 ブロックへの取り込み

マイナーはトランザクションを選択してブロック候補を作る。ただし、どのトランザクションを優先するかには手数料や依存関係、ポリシーなどが関わる。ブロックが生成されても、他ノードによる検証とチェーン選択を経なければ、そのノードの有効な履歴として採用されない。

---

## 5. UTXOモデル

### 5.1 UTXOとは

UTXO（Unspent Transaction Output）は、まだ支出されていないトランザクション出力を指す。Bitcoinは「アカウント残高を直接更新する」方式ではなく、既存のUTXOを消費し、新しいUTXOを作る方式を採用する。

```text
既存のUTXO
  A: 0.8 BTC ──┐
               ├── Transaction ──► 受信者への出力: 1.0 BTC
  B: 0.5 BTC ──┘                 └► お釣りの出力: 0.299 BTC
```

この例の手数料0.001 BTCは独立した出力ではなく、入力合計1.3 BTCと出力合計1.299 BTCの差額である。金額は計算例であり、実際の必要手数料を示すものではない（[Bitcoin開発者ガイド](https://developer.bitcoin.org/devguide/transactions.html#transaction-fees-and-change)）。実際には出力ごとに支出条件が付く。通常のトランザクションでは、**入力の合計額 = 出力の合計額 + 手数料**となる。Coinbaseトランザクションなどは別の規則で扱う。

### 5.2 Outpoint

UTXOは一般に、トランザクションIDと出力インデックスの組で識別できる。

```text
Outpoint = (transaction_id, output_index)
```

同じOutpointを二度使うトランザクションが競合すれば、両方を同じ有効な台帳状態に適用することはできない。

### 5.3 UTXO集合

フルノードは現在の未使用出力の集合（UTXO set）を使い、新しいトランザクションが既存の出力を正しく消費しているかを検証する。

| 操作                 | UTXO集合への影響     |
| -------------------- | -------------------- |
| 既存UTXOを入力に使用 | 対象UTXOを削除       |
| 支出可能な新しい出力を作成 | 新規UTXOを追加 |
| 不正な入力を参照     | トランザクションを拒否           |
| 同じUTXOを再度使用   | 二重支払いとして拒否 |

**不変条件：有効なチェーン状態において、一つのUTXOが二重に消費されてはならない。**

## 6. スクリプト・署名・支出権限

Bitcoinの出力には、その出力を使うための条件が記録される。多くの場合、署名などの証拠を提示することで支出条件を満たす。

```text
秘密鍵 ──署名──► トランザクションの支出証明
                           │
                           ▼
                 スクリプト条件を検証
                           │
                    成功 / 失敗
```

Bitcoin Scriptは、一般的なアプリケーション言語とは異なる制約を持つ。P2PKH、P2WPKH、Taprootなど、出力形式によって支出条件や署名データの扱いが異なる。

### 6.1 アドレスと秘密鍵を混同しない

- **秘密鍵**：支出を承認するための秘密情報
- **公開鍵**：署名検証などに使う情報
- **アドレス**：特定の支出条件を表現・伝達するためのエンコード
- **UTXO**：特定の支出条件を伴う未使用出力

アドレスそのものが「コインを保管している箱」なのではない。台帳上の出力を、適切な条件を満たして消費する仕組みである。

## 7. ブロックの構造

Bitcoinのブロックは、ブロックヘッダーとトランザクション群から構成される。

```text
Block
├── Header (80 bytes)
│   ├── version
│   ├── previous_block_hash
│   ├── merkle_root
│   ├── timestamp
│   ├── nBits
│   └── nonce
└── Transactions
    ├── coinbase transaction
    ├── transaction 1
    └── transaction N
```

### 7.1 前ブロックへの参照

`previous_block_hash`は親となるブロックのヘッダーを参照する。これによりブロックが連結され、過去の履歴を改変すると、その後のブロックのPoWも作り直す必要が生じる。

### 7.2 Merkle root

ヘッダーのMerkle rootは、順序付けられたトランザクションID（txid）の木へのコミットメントである。[BIP 141](https://github.com/bitcoin/bips/blob/master/bip-0141.mediawiki)では、witnessを含まないtxidと、witnessを含むwtxidを区別する。witnessを含む場合、そのコミットメントはCoinbaseの出力を使って別途結び付けるため、ヘッダーのMerkle rootだけでwitnessデータを直接まとめているわけではない。

Merkle proofでは、信頼できるルートに対して特定のtxidの包含を確認できるが、それだけで全取引の有効性を検証したことにはならない。

```text
                 Merkle Root
                 /         \
             Hash AB      Hash CD
             /   \         /   \
           txidA txidB    txidC txidD
```

### 7.3 PoWターゲット

Bitcoinでは、ブロックヘッダーのハッシュ値が難易度に対応するターゲット以下である必要がある。`nBits`はそのターゲットをコンパクトに表現するフィールドである。

```text
SHA256(SHA256(serialized_header)) <= target
```

ヘッダーは[規定の80バイト形式](https://developer.bitcoin.org/reference/block_chain.html#block-headers)にシリアライズする。`nBits` が表すtargetにも、有効範囲や親の履歴に応じた期待難易度の検証が必要であり、自由に易しいtargetを指定できるわけではない。

ハッシュ値は規定のバイト順で整数として解釈して比較する。マイナーはNonceやブロック内の変更可能な情報を調整し、条件を満たすヘッダーを探索する。ハッシュ値が条件を満たしても、ブロックのほかの規則が満たされていなければ有効とはならない。

---

## 8. マイニングとProof of Work

### 8.1 マイナーの責務

マイナーはブロック候補を組み立て、PoWを計算する。一般に、候補にはCoinbaseトランザクションと、選択された通常のトランザクションが含まれる。

### 8.2 フルノードの責務

フルノードは、マイナーが提案したブロックを独立に検証する。マイナーが「有効だ」と主張しても、ノードは自分の検証ルールに反するブロックを受け入れない。

### 8.3 インセンティブ

採用チェーンに含まれるブロックのCoinbaseでは、ブロック補助金とそのブロックの手数料合計を上限とする額を請求できる。検証を通るブロックでも採用チェーンから外れれば、そのブロックのCoinbase報酬は採用中のUTXO状態には残らない。ただし、Coinbase出力には成熟期間などの制約がある。

### 8.4 セキュリティ上の意味

PoWは、履歴を書き換えるために計算資源が必要となる構造を作る。しかし、攻撃耐性はハッシュレート分布、ネットワーク状況、経済的インセンティブなどにも依存する。PoWだけで全種類の攻撃を防げるわけではない。

## 9. P2PネットワークとMempool

BitcoinのノードはP2Pネットワークを通じてブロックやトランザクションを交換する。

```text
       Node A ───── Node B
        /  \         /  \
       /    \       /    \
    Node C ───── Node D ─── Miner
```

通信遅延や接続関係が異なるため、ノードが同じ情報を受け取る時刻は一致しない。

### 9.1 Mempoolはグローバルな共有キューではない

Mempoolは各ノードが保持する未承認トランザクションの候補集合である。ノードごとのリレーポリシー、手数料条件、容量制約などによって内容が異なりうる。

### 9.2 ブロック伝播

新しいブロックが発見されると、ノード間で情報が伝播する。その間にも別のマイナーがブロックを発見することがある。この**通信遅延と並行ブロックの関係**が、後にKaspaを理解する重要な入口になる。

## 10. フォークとチェーン選択

### 10.1 同時発見による分岐

```text
           Block A
           /     \
       Block B   Block C
          |
       Block D
```

図は上が親、下が子であり、実際の参照は子から親へ向かう。Block BとBlock Cが同じ親を参照している場合、ネットワークの一部ではBが、別の一部ではCが先に観測される可能性がある。

### 10.2 最も多くの累積PoWを持つ有効チェーン

Bitcoinのチェーン選択は、単純な「ブロック数の長さ」だけではなく、**累積チェーンワーク（chainwork）**に基づく。ノードは検証規則を満たすチェーン候補から、最も多くの累積作業量を持つものを選ぶ。ここでのworkはtargetから計算する指標であり、マイナーが実際に何回ハッシュを試したかを計測・合算するものではない（[固定版のGetBlockProof](https://github.com/bitcoin/bitcoin/blob/110183746150428e6385880c79f8c5733b1361ba/src/chain.cpp)）。

### 10.3 再編成（Reorganization）

新しいチェーンが優勢になれば、以前は採用されていたブロックが最良チェーンから外れる場合がある。これをチェーン再編成という。

再編成が起きると、ブロックに含まれていたトランザクションの確認状態も変化しうる。そのため、**ブロックに含まれたことと、絶対に取り消されないことは同じではない。**

### 10.4 確認数の考え方

採用チェーン内でトランザクションを含むブロックを1確認と数え、その上へブロックが追加されるにつれて確認数が増える。確認数が増えるほど、通常は履歴を覆すために必要な作業量が増える。ただし安全性は確率的であり、固定の確認数で数学的な絶対保証が得られるわけではない。

---

## 11. Bitcoinの主要な不変条件

設計を理解するには、「絶対に破ってはいけないルール」を明文化するとよい。

| ID     | 不変条件                                      | 破れた場合の問題     |
| ------ | --------------------------------------------- | -------------------- |
| INV-01 | 存在しないUTXOを消費できない                  | 無からの価値生成     |
| INV-02 | 同じUTXOを有効な状態で二重消費できない        | 二重支払い           |
| INV-03 | 支出条件を満たさない入力は無効                | 無権限の支出         |
| INV-04 | 通常トランザクションの出力総額は入力総額を超えない        | 不正な価値生成       |
| INV-05 | ブロックはPoW・構文・コンセンサス規則を満たす | 無効なブロックの採用 |
| INV-06 | Coinbase報酬は許可された範囲を超えない        | 過剰発行             |
| INV-07 | ノードは受信データを独立に検証する            | 不正なピアへの依存   |
| INV-08 | チェーン状態は採用チェーンと整合する          | UTXO状態の破損       |

実際のBitcoinには、これ以外にも詳細な規則が多数存在する。ここではアーキテクチャを理解するための代表例を示した。

## 12. Bitcoin Coreのソースコードを読む

Bitcoin CoreはBitcoinの代表的なフルノード実装であり、主にC++で記述されている。

本章の読み取り対象は **Bitcoin Core v28.0 / `110183746150428e6385880c79f8c5733b1361ba`** とする。これは説明を固定する参照版であり、最新版や運用推奨版という意味ではない。Kaspaの基準版とは別に管理する。

| 入口 | 確認する内容 |
|---|---|
| [src/consensus/tx_check.cpp](https://github.com/bitcoin/bitcoin/blob/110183746150428e6385880c79f8c5733b1361ba/src/consensus/tx_check.cpp) | 空の入出力・金額・重複入力などの文脈に依存しない検査 |
| [src/consensus/tx_verify.cpp](https://github.com/bitcoin/bitcoin/blob/110183746150428e6385880c79f8c5733b1361ba/src/consensus/tx_verify.cpp) | UTXO入力・Coinbase成熟・金額差の検査 |
| [src/pow.cpp](https://github.com/bitcoin/bitcoin/blob/110183746150428e6385880c79f8c5733b1361ba/src/pow.cpp) | targetの範囲・PoWの比較と難易度計算 |
| [src/chain.cpp](https://github.com/bitcoin/bitcoin/blob/110183746150428e6385880c79f8c5733b1361ba/src/chain.cpp) | targetから各ブロックのworkを求める処理 |
| [src/validation.cpp](https://github.com/bitcoin/bitcoin/blob/110183746150428e6385880c79f8c5733b1361ba/src/validation.cpp) | ブロック適用・Coinbase額の上限・チェーン切り替えの入口 |

### 12.1 調査の順序

1. `README.md`とビルド手順を読む
2. `src/`配下の主要ディレクトリを確認する
3. トランザクション検証とUTXO管理の実装を探す
4. Mempoolとブロック検証の責務を比較する
5. 関連テストを読む
6. 変更前後の状態遷移を図にする

ファイル配置や関数名はバージョンによって変わる。**書籍では、特定のコミットを固定してからソースコードを説明する**ことが重要である。

### 12.2 読み方の問い

- この関数はコンセンサス規則を実装しているのか、ローカルポリシーを実装しているのか。
- このデータはMempoolに属するのか、現在採用しているチェーン状態に属するのか。
- エラーが起きたとき、どこまで状態変更が巻き戻されるのか。
- 変更の影響はウォレット、P2P、ストレージに波及するのか。

これらの問いは、後の`rusty-kaspa`のコードリーディングにもそのまま応用できる。

## 13. 実践：Rustで最小UTXO検証器を考える

Bitcoinの構造を理解するために、UTXOの消費と生成だけを再現する教育用モデルを作る。

**注意：以下はBitcoin互換の実装ではない。署名、スクリプト、Coinbase、手数料ポリシー、ブロック検証などを省略している。**

### 13.1 最小モデル

```rust
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct OutPoint {
    txid: u64,
    index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Output {
    value: u64, // 学習用の最小単位（satoshi相当）
}

#[derive(Debug)]
struct Transaction {
    txid: u64,
    inputs: Vec<OutPoint>,
    outputs: Vec<Output>,
}

#[derive(Debug, PartialEq, Eq)]
enum ApplyError {
    EmptyInputs,
    EmptyOutputs,
    OutputIndexOverflow,
    MissingInput,
    DuplicateInput,
    ValueOverflow,
    Overspend,
    OutputAlreadyExists,
}

fn apply_transaction(
    utxos: &mut HashMap<OutPoint, Output>,
    tx: &Transaction,
) -> Result<u64, ApplyError> {
    if tx.inputs.is_empty() {
        return Err(ApplyError::EmptyInputs);
    }
    if tx.outputs.is_empty() {
        return Err(ApplyError::EmptyOutputs);
    }

    let mut seen = HashSet::new();
    let mut input_total = 0_u64;

    for input in &tx.inputs {
        if !seen.insert(*input) {
            return Err(ApplyError::DuplicateInput);
        }
        let output = utxos.get(input).ok_or(ApplyError::MissingInput)?;
        input_total = input_total
            .checked_add(output.value)
            .ok_or(ApplyError::ValueOverflow)?;
    }

    let mut output_total = 0_u64;
    for output in &tx.outputs {
        output_total = output_total
            .checked_add(output.value)
            .ok_or(ApplyError::ValueOverflow)?;
    }

    if output_total > input_total {
        return Err(ApplyError::Overspend);
    }

    // 作成するOutpointも、状態変更前にすべて検査する。
    let mut new_outputs = Vec::with_capacity(tx.outputs.len());
    for (index, output) in tx.outputs.iter().enumerate() {
        let outpoint = OutPoint {
            txid: tx.txid,
            index: u32::try_from(index).map_err(|_| ApplyError::OutputIndexOverflow)?,
        };
        if utxos.contains_key(&outpoint) {
            return Err(ApplyError::OutputAlreadyExists);
        }
        new_outputs.push((outpoint, *output));
    }

    for input in &tx.inputs {
        utxos.remove(input);
    }
    utxos.extend(new_outputs);

    Ok(input_total - output_total) // 手数料に相当する差額
}
```

このモデルでは`txid`を便宜上の数値とし、呼び出し側が新しいトランザクションへ一意に割り当てる前提を置く。過去に完全消費されたtxidの再利用を検出する履歴管理や、実際のハッシュ計算は実装していない。Bitcoinの金額上限・支出不能出力の扱い・署名等の規則も省略しており、**本番の台帳処理に流用してはならない**。

### 13.2 確認したいテストケース

| ケース                         | 期待結果         |
| ------------------------------ | ---------------- |
| 存在するUTXOを一度消費         | 成功し、差額を返す |
| 入力または出力が空 | `EmptyInputs` / `EmptyOutputs` |
| 既存Outpointと同じ出力を作成 | `OutputAlreadyExists`。同じ入力を消費する場合も拒否 |
| 同じ入力を一つのトランザクションで二度指定 | `DuplicateInput` |
| 消費済みUTXOを再利用           | `MissingInput`   |
| 出力合計が入力合計を超える     | `Overspend`      |
| 金額加算が`u64`を超える        | `ValueOverflow`  |
| 失敗時にUTXO集合が変更されない | 状態維持         |

本文のRustコードを抽出し、拒否時の集合の不変性を含むテストを実行した。記録と再実行手順はリポジトリの `experiments/chapter-05-review-2026-10-31.md`、テスト本体は `experiments/chapter-05-utxo-tests.rs` を参照する。標準ライブラリだけを使う教材であり、Bitcoin Coreとの実行時差分テストではない。

**設計上のポイント：検証を完了してから状態を変更する。** これはBitcoinやKaspaに限らず、一般的なバックエンド開発でも重要な原則である。

### 13.3 発展課題

1. 所有者を表す公開鍵情報を出力に追加する
2. 署名検証を抽象化した`ScriptVerifier`を導入する
3. ブロック内の複数トランザクションを順序どおり適用する
4. ブロック適用を巻き戻すUndoデータを設計する
5. Mempoolの競合トランザクションを別の状態として扱う
6. 最後に、一本のチェーンをBlockDAGへ置き換えると何が変わるかを考える

---

## 14. BitcoinとKaspaの対応関係

| 観点                | Bitcoin            | Kaspa                 | 学習上の意味                   |
| ------------------- | ------------------ | --------------------- | ------------------------------ |
| 台帳モデル          | UTXO               | UTXO                  | 支出と検証の基本概念を共有     |
| コンセンサス資源    | PoW                | PoW                   | マイニングの経済的基盤が共通   |
| ブロックの参照      | Genesis以外は1つの親       | 複数の親を参照可能    | 並行ブロックの扱いが異なる     |
| 履歴の構造          | Blockchain         | BlockDAG              | 状態と順序の決め方が変わる     |
| チェーン／DAGの選択 | 累積チェーンワーク | GHOSTDAGなどの規則    | 同じロジックを流用できない     |
| ブロックの順序      | チェーン上の位置   | DAGから導出される順序 | 順序付けの複雑さが増す         |
| 未承認トランザクション          | Mempool            | Mempool               | ノードごとのローカル管理がある |
| 実装言語の代表例    | Bitcoin Core：C++  | rusty-kaspa：Rust     | コードベースの構造が異なる     |

### 14.1 Kaspaで再設計が必要になる部分

Bitcoinでも最良チェーン以外の枝を管理する。各ブロックが一つの親を参照し、チェーン切り替えでは旧枝のUTXO変更を戻して新枝を適用する構造を持つ。Kaspaでは、複数の親を持つDAG、並行ブロック、GHOSTDAGによる順序付け、Virtual Stateなどを扱う必要がある。

したがって、**BitcoinのUTXOやPoWの知識は有用だが、Bitcoinのチェーン処理をそのままKaspaに当てはめることはできない。**

### 14.2 BlockDAGを導入する理由への接続

Bitcoinは、ネットワーク遅延が存在する環境で、競合ブロックの一部を最良チェーンから外す方式を採る。ブロック生成頻度を大幅に高めると、この競合が増えやすくなる。

Kaspaは、並行ブロックをDAGとして保持・参照し、コンセンサスに利用する設計を採用する。次章では、なぜ従来型ブロックチェーンでブロック生成頻度を上げることが難しいのかを詳しく検討する。

## 15. よくある誤解

**誤解1：ウォレットがコインを保管している。**  
ウォレットが主に管理するのは鍵や支払い情報である。価値の状態は、台帳上のUTXOとして表現される。

**誤解2：マイナーが正しい履歴を自由に決められる。**  
マイナーはブロックを提案するが、フルノードはコンセンサス規則に従って独立に検証する。

**誤解3：Mempoolに入ったら送金は確定する。**  
Mempoolへの受理は未承認トランザクションとしてのローカルな取り扱いであり、ブロックへの取り込みとは異なる。

**誤解4：最長チェーンとはブロック数が最多のチェーンである。**  
Bitcoinの選択基準は累積チェーンワークである。

**誤解5：署名さえ正しければ支払いは有効である。**  
UTXOの存在、支出条件、金額、ブロック規則など複数の条件を満たす必要がある。

## 16. 理解度チェック

1. ウォレット、フルノード、マイナーはそれぞれ何を担当するか。
2. UTXOを識別するOutpointは、何の組み合わせか。
3. Mempoolに存在するトランザクションは、なぜ確定済みとはいえないか。
4. Bitcoinで最良チェーンを選択するとき、ブロック数だけを比較しないのはなぜか。
5. Merkle rootとPoWターゲットは、それぞれ何を保証・検証するために使われるか。
6. Bitcoinの構造のうち、Kaspaへ引き継がれるものと再設計されるものを挙げよ。

<details>
<summary>解答例を見る</summary>

1. ウォレットは鍵・支払いを管理し、フルノードは規則を検証し、マイナーはブロック候補を生成してPoWを計算する。
2. トランザクションIDと出力インデックス。
3. ノードごとのローカルな未承認トランザクション集合にすぎず、ブロックへの取り込みやチェーン選択を経ていないため。
4. ブロックの難易度が異なりうるため、累積作業量を比較する必要がある。
5. Merkle rootはトランザクション群へのコミットメント、PoWターゲットはブロックヘッダーのハッシュが満たすべき条件を表す。
6. PoW・UTXO・P2Pなどは共通する一方、単一チェーン中心の構造と順序付けはBlockDAG・GHOSTDAGに置き換わる。

</details>

## 17. まとめ

Bitcoinのアーキテクチャは、署名、UTXO、スクリプト、Mempool、P2P、PoW、ブロック検証、チェーン選択といった構成要素の連携によって成り立つ。

重要なのは、各要素の責務を分けることである。ウォレットはトランザクションを作り、フルノードは検証し、マイナーはブロックを提案する。UTXO集合は台帳の現在状態を表し、ブロックチェーンはその状態へ至る履歴を記録する。

Kaspaを学ぶ際にも、この役割分担は土台となる。ただし、Kaspaは一本のチェーンではなくBlockDAGを扱うため、ブロックの参照関係、順序付け、コンセンサス状態の導出に新たな設計が必要となる。

> **本章の核心：Bitcoinの構成要素を理解することは、Kaspaで「変わらない部分」と「根本から変わる部分」を見分けるための準備である。**

**次章：[Chapter 06 — Blockchain Limitations（ブロックチェーンの限界）](chapter-06.md)**

---

## 参考資料

1. Satoshi Nakamoto, _Bitcoin: A Peer-to-Peer Electronic Cash System_ — <https://bitcoin.org/bitcoin.pdf>
2. Bitcoin Core 公式リポジトリ — <https://github.com/bitcoin/bitcoin>
3. Bitcoin Developer Reference — <https://developer.bitcoin.org/reference/>
4. Bitcoin Developer Guide — <https://developer.bitcoin.org/devguide/>
5. Andreas M. Antonopoulos, David A. Harding, _Mastering Bitcoin_ — <https://github.com/bitcoinbook/bitcoinbook>
6. Kaspa 公式ドキュメント — <https://docs.kaspa.org/>
7. rusty-kaspa — <https://github.com/kaspanet/rusty-kaspa>

### 確認範囲（2026-10-31）

Bitcoin Core v28.0の該当ソースと一次資料で、入出力検査・PoW/chainwork・Coinbase・Merkleの説明を照合した。UTXO教材はRust 1.90.0で実行確認した。Bitcoin Coreのビルド・ノード実行・全規則の監査を行った記録ではない。詳細はリポジトリの `experiments/chapter-05-review-2026-10-31.md` を参照する。
