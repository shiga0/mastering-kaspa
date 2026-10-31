# Chapter 01 — Introduction to Kaspa（Kaspa入門）

> **Mastering Kaspa** / Part I — Foundations（基礎編）  
> 原稿バージョン：v0.1（初稿）

## 1. はじめに

Bitcoinは、中央管理者を必要とせず、インターネット上で価値を移転する仕組みを実現した。その中心にあるのが、Proof of Work（PoW）とブロックチェーンである。

しかし、ネットワーク上ではブロックの伝播に時間がかかる。複数のマイナーがほぼ同時にブロックを発見すると、一時的な分岐が発生する。従来のブロックチェーンでは、最終的に一つの有効なチェーンを選択し、それ以外のブロックは主チェーンから外れる。

**Kaspaは、この課題に対してBlockDAGを採用したProof of Work型の暗号資産ネットワークである。** 複数のブロックが並行して生成されることを前提に、それらの参照関係を有向非巡回グラフ（DAG）として扱い、GHOSTDAGによってコンセンサス上の順序を導く。

本章では、Kaspaの基本思想、技術的特徴、Bitcoinとの違い、本書の全体像を整理する。

## 2. この章で学ぶこと

- Kaspaとは何か
- Kaspaが解決しようとしている問題
- BlockchainとBlockDAGの違い
- Proof of Workを採用する理由
- GHOSTDAGの基本的な役割
- Kaspaの主要な構成要素
- BitcoinとKaspaの共通点・相違点
- 本書で学ぶ技術の全体像

## 3. Kaspaとは何か

Kaspaは、Proof of Workを利用する分散型暗号資産ネットワークである。Bitcoinと同様に、参加ノードがトランザクションやブロックを検証し、マイナーが計算資源を使ってブロックを生成する。

一方、Bitcoinが基本的に一本のチェーンを形成するのに対して、KaspaはBlockDAGを採用する。

| 項目                   | Kaspa                                                     |
| ---------------------- | --------------------------------------------------------- |
| ネットワーク           | 分散型P2Pネットワーク                                     |
| コンセンサスの基盤     | Proof of Work                                             |
| ブロック構造           | BlockDAG                                                  |
| 順序付け               | GHOSTDAG                                                  |
| ネイティブ通貨         | KAS                                                       |
| トランザクションモデル | UTXO                                                      |
| 主要ノード実装         | rusty-kaspa（Rust）                                       |
| 目指す方向             | PoWの安全性を維持しながら高頻度のブロック生成を可能にする |

重要なのは、単にブロック生成間隔を短くすることではない。**並行して生成されたブロックを、コンセンサスに活用する設計**に特徴がある。

## 4. なぜKaspaが必要なのか

### 4.1 従来のブロックチェーンの課題

Bitcoinのブロック生成間隔は平均約10分となるよう調整される。ブロック生成間隔を極端に短くすると、ネットワーク伝播が追いつかず、競合するブロックが増える可能性がある。

```text
Genesis
   |
 Block A
   |
 Block B
  /   \
 C1   C2
  |
 Block D
```

この例では、`C1`と`C2`が競合している。Bitcoinでは、最も多くの累積Proof of Workを持つ有効なチェーンが選ばれ、選択されなかったブロックは主チェーンに含まれない。

### 4.2 Kaspaのアプローチ

Kaspaでは、複数の親ブロックを参照することができる。

```text
         Genesis
            |
         Block A
         /     \
    Block B   Block C
         \     /
         Block D
         /     \
    Block E   Block F
         \     /
         Block G
```

`Block B`と`Block C`は並行して生成され、`Block D`は両方を参照する。このように並行ブロックをDAGに取り込める。

ただし、**BlockDAGというデータ構造だけで安全性が保証されるわけではない**。ネットワーク全体で一貫した順序を導くアルゴリズムが必要である。

## 5. BlockDAGとは何か

DAG（Directed Acyclic Graph）は、有向非巡回グラフを意味する。Kaspaでは、ブロック間の参照関係をこの構造で表す。

| 観点         | Blockchain                     | BlockDAG                   |
| ------------ | ------------------------------ | -------------------------- |
| 基本構造     | 一本のチェーン                 | 有向非巡回グラフ           |
| 親ブロック   | 原則1つ                        | 複数を参照可能             |
| 並行ブロック | 主チェーンから外れる場合がある | DAGに取り込める            |
| 順序の決定   | チェーンの順序                 | コンセンサスによる順序付け |
| 設計上の特徴 | 比較的単純                     | 並行性を扱えるが複雑       |

どちらかが無条件に優れているわけではない。BlockDAGは並行性を扱える代わりに、順序付けや状態管理が複雑になる。

## 6. GHOSTDAGとは何か

GHOSTDAGは、KaspaのBlockDAGに対してブロックの分類と順序付けを行うアルゴリズムである。

例えば、同じUTXOを使用する二つのトランザクションが存在した場合、両方を有効にはできない。そのため、ネットワーク参加者が共通の順序を導ける必要がある。

| 用語            | 概要                                                        |
| --------------- | ----------------------------------------------------------- |
| blue block      | GHOSTDAGのルールに従ってblueに分類されたブロック            |
| red block       | blueに分類されなかったブロック                              |
| blue score      | 選択された過去のblueブロック数に基づく指標                  |
| Selected Parent | 親候補から選ばれる基準となる親ブロック                      |
| Virtual Block   | DAGの先端を統合して現在の状態を考えるための仮想的なブロック |

**blueは「正しい」、redは「不正」という意味ではない。** これらはアルゴリズム上の分類である。詳しくは第12章で扱う。

## 7. Proof of Workを採用する理由

PoWでは、マイナーが計算資源を使ってブロック生成を試み、ノードが結果を検証する。

```text
ブロック候補を作成
       ↓
Nonceなどを変更してハッシュ計算
       ↓
難易度条件を満たす？
   ├─ No → 再計算
   └─ Yes
        ↓
    ブロックを送信
        ↓
    ノードが検証
        ↓
    BlockDAGに追加
```

PoWはブロック生成に現実の資源消費を要求し、検証は比較的容易である。一方で、電力消費やマイニングの集中化といった課題もある。

KaspaはPoWを維持しつつ、BlockDAGによって高頻度のブロック生成を目指している。

## 8. Kaspaの主要な構成要素

```text
┌────────────────────────────┐
│ Applications               │
│ Wallet / Explorer / Apps   │
└─────────────┬──────────────┘
              │
┌─────────────▼──────────────┐
│ RPC / SDK                  │
│ Rust / WASM / wRPC         │
└─────────────┬──────────────┘
              │
┌─────────────▼──────────────┐
│ Kaspa Node                 │
│                            │
│  Transaction / UTXO        │
│  Mempool                   │
│  Consensus / GHOSTDAG      │
│  BlockDAG / Storage        │
└─────────────┬──────────────┘
              │
┌─────────────▼──────────────┐
│ P2P Network / Other Nodes  │
└────────────────────────────┘
```

> この図は学習用の概念図であり、`rusty-kaspa`の実際のモジュール依存関係を正確に示したものではない。

| コンポーネント | 役割                                 |
| -------------- | ------------------------------------ |
| P2P Network    | ノード間の情報交換                   |
| BlockDAG       | ブロックの参照関係を保持             |
| GHOSTDAG       | ブロックの分類と順序付け             |
| Consensus      | ネットワーク全体で共通の状態を導く   |
| UTXO           | 使用可能なトランザクション出力を管理 |
| Mempool        | 未確定トランザクションを管理         |
| RPC            | 外部アプリケーションとの通信         |
| Wallet         | 鍵管理とトランザクション作成         |
| Miner          | PoWによるブロック生成                |

## 9. BitcoinとKaspaの比較

| 項目                   | Bitcoin                             | Kaspa                  |
| ---------------------- | ----------------------------------- | ---------------------- |
| コンセンサスの基盤     | PoW                                 | PoW                    |
| ブロック構造           | Blockchain                          | BlockDAG               |
| ブロック生成間隔       | 平均約10分                          | 高頻度                 |
| トランザクションモデル | UTXO                                | UTXO                   |
| ブロックの参照関係     | 原則1つの親                         | 複数の親               |
| 順序付け               | 最も多くの累積PoWを持つ有効チェーン | GHOSTDAGによる順序付け |
| 主なノード実装         | Bitcoin Core（C++）                 | rusty-kaspa（Rust）    |
| ネイティブ通貨         | BTC                                 | KAS                    |

PoWとUTXOは共通するが、BlockDAGとGHOSTDAGによってブロックの順序付けやコンセンサス状態の扱いが異なる。

## 10. Kaspaを学ぶための3つの視点

### 10.1 データ構造

**何を、どのような形式で保持するのか。**

- Block
- Transaction
- UTXO
- BlockDAG
- Mempool

### 10.2 アルゴリズム

**保持したデータを、どのようなルールで処理するのか。**

- Proof of Work
- GHOSTDAG
- Difficulty Adjustment
- Transaction Validation
- Block Validation

### 10.3 システム

**実際の分散ネットワークでどう動かすのか。**

- P2P通信
- Node Synchronization
- Storage
- RPC
- Wallet
- Mining
- Monitoring

この三つを分けて考えると、論文とソースコードの対応関係を調査しやすくなる。

## 11. 本書の全体構成

| Part                                      | 内容                                      |
| ----------------------------------------- | ----------------------------------------- |
| Part I — Foundations                      | 暗号技術、分散システム、Bitcoin、BlockDAG |
| Part II — Consensus & Architecture        | Kaspaノード、GHOSTDAG、コンセンサス       |
| Part III — Transactions & Mining          | UTXO、Script、Mempool、マイニング         |
| Part IV — Node Architecture & Development | RPC、Rust SDK、ノード実装、Devnet         |
| Part V — Advanced Kaspa                   | DAGKnight、KIP、Covenants、ZK、vProgs     |
| Part VI — Building & Contributing         | アプリケーション構築、OSSへの貢献         |

本書の目標は、技術を説明できるだけでなく、論文と実装を照合し、Rustコードを読み、Devnetで検証し、OSSへ改善を提案できるようになることである。

## 12. 実践：Kaspaのコードを見てみる

主要なRust実装はGitHubで公開されている。

- [rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)

```bash
git clone https://github.com/kaspanet/rusty-kaspa.git
cd rusty-kaspa
ls
cat Cargo.toml
```

この段階ではすべてを理解する必要はない。まずワークスペースの構成を確認し、どのようなコンポーネントが存在するかを把握する。

## 13. よくある誤解

**誤解1：KaspaはBitcoinより単純に高速なだけ。**  
ブロック生成頻度だけでなく、BlockDAGとGHOSTDAGを採用している点が本質的に異なる。

**誤解2：BlockDAGならブロックの競合が発生しない。**  
並行ブロックは発生する。Kaspaはそれらを前提に設計されている。

**誤解3：blue blockは正しく、red blockは不正である。**\
blueとredはGHOSTDAGによる分類であり、有効・無効と同義ではない。

**誤解4：PoWなら必ず安全である。**  
安全性はハッシュパワーの分布、ネットワーク条件、攻撃者の能力、コンセンサスルールなどに依存する。

**誤解5：高頻度のブロック生成は即時の不可逆性を意味する。**  
ブロック生成頻度と、トランザクションの確定に対する安全性は別の問題である。

## 14. 理解度チェック

**Q1. KaspaとBitcoinの最も重要な構造上の違いは何か。**  
A. BitcoinはBlockchain、KaspaはBlockDAGを採用する。

**Q2. Kaspaで複数の親ブロックを参照できる理由は何か。**  
A. 並行して生成されたブロックの関係をDAGとして扱うため。

**Q3. GHOSTDAGの主な役割は何か。**  
A. BlockDAG内のブロックを分類し、コンセンサスに利用する順序を導くこと。

**Q4. Kaspaが採用するコンセンサスの基盤は何か。**  
A. Proof of Work。

**Q5. BlockDAGを採用すると、すべてのセキュリティ問題が解決するか。**  
A. 解決しない。順序付け、ネットワーク遅延、攻撃耐性などを引き続き考慮する必要がある。

## 15. まとめ

Kaspaは、Proof of Workを採用しながら、BlockDAGによって複数のブロックを並行して扱う暗号資産ネットワークである。

従来のブロックチェーンでは一本のチェーンを中心にコンセンサスを形成する。一方、Kaspaでは複数のブロックが形成するDAGを扱い、GHOSTDAGによって順序付けを行う。

本書では、技術を個別に暗記するのではなく、**データ構造・アルゴリズム・システムの依存関係を理解する**ことを重視する。

次章では、Kaspaの前提となるデジタルマネーの基礎を学ぶ。

## 参考資料

- [Kaspa公式サイト](https://kaspa.org/)
- [Kaspa公式ドキュメント](https://docs.kaspa.org/)
- [rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)
- [PHANTOM and GHOSTDAG 論文](https://eprint.iacr.org/2018/104.pdf)
- [Bitcoin Whitepaper](https://bitcoin.org/bitcoin.pdf)

---

> **原稿ステータス：初稿 v0.1**  
> 本章は入門原稿である。出版前に、論文・KIP・実装との照合、出典の追加、最新のネットワーク仕様の確認を行う。
