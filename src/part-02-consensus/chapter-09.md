# Chapter 09 — Kaspa Architecture（Kaspaのアーキテクチャ）

> **Part II — Kaspa Consensus & Architecture**  
> 構造・基準版との対応を確認：2026-10-31。参照版は[本書の基準版](../baseline.md)のv1.0.1、コミット `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。
> 本章は責務と処理経路の入口を扱う。全規則・全モジュールの検証ではない。

## 9.1 この章で学ぶこと

第1部では、Bitcoinの線形チェーンが抱える制約と、BlockDAGが並行生成ブロックを扱う理由を説明した。本章では視点を一段下げ、**Kaspaのフルノードを、役割と依存関係を持つソフトウェアシステム**として捉える。

学習目標は次のとおり。

1. Kaspaのネットワーク・コンセンサス・状態管理・外部APIを区別できる。
2. 「ブロックをDAGに格納すること」と「トランザクションを有効な状態に反映すること」の違いを説明できる。
3. GHOSTDAG、選択された親、仮想ブロック、仮想状態の関係を理解する。
4. `rusty-kaspa` のディレクトリから主要な責務を追跡できる。
5. ノードの正常性を同期・P2P・DAG・UTXO・Mempoolに分けて診断できる。

本章は**全体設計**を扱う。GHOSTDAGの具体的な計算は第12章、仮想状態と順序付けは第14章、Rust実装の精読は第25〜27章で掘り下げる。

---

## 9.2 Kaspaを一文で表す

Kaspaは、Proof of Work（PoW）で生成された複数のブロックを**有向非巡回グラフ（BlockDAG）**として扱い、コンセンサス規則によってトランザクションを処理する順序と共有状態を決める分散型台帳である。

この説明には、混同してはいけない三つの要素がある。

| 要素 | 問い | 主な役割 |
|---|---|---|
| PoW | 規定のtargetを満たす計算の証拠があるか | ブロック生成のコストと重みの基礎 |
| BlockDAG | どのブロックがどのブロックを参照しているか | 並行生成された履歴の表現 |
| コンセンサス | 共有状態をどう決定するか | 順序・受理・UTXO状態の決定 |

**BlockDAGはデータ構造であって、それ単体でコンセンサスを実現するわけではない。** 同様に、PoWだけでトランザクションの順序が一意に決まるわけでもない。

## 9.3 システムコンテキスト：外側から見る

以下は外部の利用者・ピアと保存先を整理する概念図である。厳密なC4のContext図ではなく、接続の用途を学ぶために描いている。

```text
   ┌──────────────┐            ┌───────────────┐
   │ ウォレット /  │   RPC      │ Explorer /    │
   │ アプリケーション├──────┐ ┌──┤ Indexer       │
   └──────────────┘      │ │  └───────────────┘
                         ▼ ▼
                   ┌──────────────┐
                   │ Kaspa Full   │
   ┌──────────┐    │ Node         │    ┌──────────┐
   │ Miner /  ├───►│ (kaspad)     │◄───┤ Peer     │
   │ Pool     │    └──────┬───────┘    │ Nodes    │
   └──────────┘           │            └──────────┘
                          ▼
                    ┌──────────┐
                    │ Local DB │
                    └──────────┘
```

- **ウォレット**は鍵を保持・署名し、ノードに送信・照会を行う。一般的なフルノードが利用者の秘密鍵を管理するわけではない。
- **マイナー**はブロックテンプレートを取得し、PoWを探索して候補ブロックを提出する。
- **他のフルノード**はP2P通信でブロック・トランザクション・同期情報を交換する。
- **Indexer / Explorer**はノードの情報を検索しやすい形に加工する。インデクサはコンセンサスそのものではない。
- **ローカルDB**はDAGや状態の永続化を担う。DB内の値が正しいかどうかを最終的に判断するのはコンセンサス規則である。

> **設計上の境界**：RPCはアプリケーションとの接点、P2Pはノード間の接点。両者を同一の信頼境界として扱わない。

## 9.4 ノード内部の論理コンポーネント

実装上のモジュール境界と完全一致するものではないが、理解のため次のように分けられる。

```text
kaspad：起動時の構成とサービスの接続
├─ P2P / 同期：ブロック・取引・同期データの交換
├─ Consensus：ヘッダー / GHOSTDAG / DAA / 本体 / Virtual / UTXO
├─ Mining：Mempool・取引ポリシー / ブロックテンプレート
├─ RPC：外部からの照会・提出・通知
└─ 保存：コンセンサスのデータストア / オプションの索引等
```

線は機能の分類であり、呼び出し順序や依存方向を表さない。[基準版の起動処理](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/kaspad/src/daemon.rs)では、コンセンサス、MiningManager、RPC等を構成している。Mempoolはコンセンサスから独立した別世界ではなく、トランザクション検証の規則や現在の状態を参照する。実装には非同期処理、キャッシュ、通知、複数のデータストアがあり、単純な上下関係には収まらない。

### 責務の一覧

| サブシステム | 入力 | 出力・副作用 | 守るべき条件 |
|---|---|---|---|
| P2P | 未信頼のネットワークメッセージ | 配信・同期候補 | 入力サイズ・形式・リソース制限 |
| Block validation | ブロック・親参照・PoW情報 | 検証結果 | 無効なブロックを有効とみなさない |
| DAG管理 | 親子関係 | DAGの関係・tip集合 | 閉路を許さず参照整合性を保つ |
| GHOSTDAG | DAGの過去・選択親候補 | 選択親・青/赤分類等 | 決定論的な計算 |
| 順序・Virtual | DAGのコンセンサス情報 | 仮想的な順序・選択状態 | ノード間で同じ入力なら同じ結果 |
| UTXO処理 | 順序付けされたトランザクション | UTXO集合の更新 | 二重消費を認めない |
| Mempool | 未確定トランザクション | ブロック候補のトランザクション群 | ノードポリシーとコンセンサスを区別 |
| 永続化 | DAG・状態・索引 | 再起動可能な記録 | 整合性・復旧可能性 |
| RPC | アプリからの要求 | 状態照会・トランザクション提出 | 公開範囲とアクセス制御 |

## 9.5 BlockDAGはどのように表現されるか

BitcoinではGenesis以外のブロックが一つの親を参照する。一方Kaspaでは、ブロックが**複数の親**を参照できる。

```text
           Genesis
             │
             A
           ╱   ╲
          B     C       ← 互いに並行して作られたブロック
           ╲   ╱
             D         ← BとCの両方を参照できる
             │
             E
```

上を親、下を子として配置した概念図であり、実際のブロックヘッダーの参照は子から親へ向かう。親が複数あるからといって、**すべての親のトランザクションを無条件で採用する**わけではない。

### DAGを理解するための用語

- **Parent（親）**：そのブロックが直接参照するブロック。
- **Ancestor（祖先）**：親参照をたどって到達できる過去のブロック。
- **Descendant（子孫）**：あるブロックを祖先に持つ後続ブロック。
- **Anticone**：対象自身を除き、対象と祖先・子孫関係にない既知ブロックの集合。集合自体が反鎖になるとは限らない。
- **Tip**：現在知られているDAGで子を持たないブロック。
- **Selected parent**：GHOSTDAG規則で選ばれる親。単純な「最初の親」ではない。

### 不変条件の例

1. 親参照を1回以上たどって自分自身へ戻る閉路がない。本章の祖先・子孫は自身を除くが、基準版APIには自身も祖先とみなすものがある（[第7章](../part-01-foundations/chapter-07.md)）。
2. 参照先の親は、対象ブロックより論理的に過去のブロックである。
3. 同じブロック、必要なDAG・UTXOの履歴、パラメータ、評価条件を与えれば、同じ論理的な検証結果を導く。ブロック単体だけが同じでも、親不足や参照状態が違えば処理結果は異なり得る。
4. DAGに存在することと、トランザクションが最終的にUTXO状態へ反映されることは同義ではない。

## 9.6 GHOSTDAGの位置づけ

GHOSTDAGはBlockDAG上で選択親やブロックの分類・順序付けに必要な情報を計算するコンセンサス上の仕組みである。ブロックが並行して発生する状況でも、ノードが共有できる決定論的な基準を提供する。

ここで重要なのは、**blue / red は valid / invalid の言い換えではない**ということ。ブロックの色付けはDAG上のコンセンサス上の分類であり、ヘッダーの形式検証やPoW検証、トランザクション検証とは異なる次元の話である。

また、ブロックの分類だけでUTXO集合は決まらない。実際にどのトランザクションをどの順序で適用するかという処理が必要になる。

## 9.7 Virtual BlockとVirtual State

Kaspaのアーキテクチャで特に重要なのが**Virtual（仮想）**という概念である。

現時点のDAGには複数のtipが存在しうる。新しいブロックが届くたびに「現在のDAG全体をどう解釈するか」を求める必要がある。そこで、規則に従って選ばれた親群を参照する**概念上の仮想ブロック**を考え、そこから得られる順序・状態を計算する。

```text
      B       C       D      ← 選ばれたVirtualの親（全tipsとは限らない）
       ╲      │      ╱
        ╲     │     ╱
         [ Virtual ]         ← 概念上の計算対象
               │
               ▼
      Consensus ordering
               │
               ▼
         Virtual UTXO
```

**Virtual Blockは、ネットワークでマイニングされる通常のブロックではない。** 全tipsを無条件に親として取り込むわけでもない。ノードが現在のDAGに対するコンセンサス状態を計算するための概念である。

### なぜVirtualが必要なのか

Bitcoinの説明では「現在のチェーン先端」を中心に状態を説明できる。しかしBlockDAGでは先端が一つとは限らない。Virtualは、複数tipを持つ現在のDAGから**一つの参照可能な状態**を導くための考え方である。

### Virtualの更新で起こりうること

新しいブロックを受け取ると、選択された順序やUTXO状態に変化が生じることがある。そのため、アプリケーションは「一度見えたトランザクションが常に同じ状態に留まる」と仮定してはいけない。承認数、安定性、再編成に相当する変化を考慮する必要がある。

## 9.8 トランザクションのライフサイクル

ウォレットから送信したトランザクションが状態に反映されるまでを、責務ごとに分けてみる。

```text
 Wallet
   │ 署名済み取引
   ▼
 RPC submitTransaction
   │
   ▼
 Mempool admission ────► P2P relay
   │
   ▼
 Mining template
   │
   ▼
 Mined block
   │
   ▼
 Header validation / GHOSTDAG
   │
   ▼
 Body validation
   │
   ▼
 Virtual ordering / UTXO update
   │
   ▼
 RPC notification / indexer / wallet
```

この図は典型的な流れを示す。実際には、他ノードが生成したブロックの受信、同期中の処理、Mempoolの再評価など複数経路がある。

### 四つの異なる状態

| 状態 | 意味 | 誤解しやすい点 |
|---|---|---|
| RPCが受け付けた | API要求の処理結果 | ネットワーク全体で採用されたとは限らない |
| Mempoolにある | ノードが候補として保持 | ブロックに入ったとは限らない |
| ブロックに含まれた | あるDAGブロックに記録 | そのトランザクションが有効なUTXO更新を生んだとは限らない |
| Virtual stateに反映 | 現在のコンセンサス状態に反映 | 永久に覆らない保証と同義ではない |

**アプリケーションの残高表示や入金判定は、どの状態を採用基準にするか明示しなければならない。**

## 9.9 ブロック受信から状態更新まで

概念的な処理フローをもう少し詳しく見る。

```text
Receive block
  ├─ メッセージ形式・サイズの検査
  ├─ 不足する親データの取得／同期
  ├─ ヘッダー処理（PoW・親関係・GHOSTDAG・履歴依存検査等）
  ├─ 本体処理（Merkle root・取引構造・親本体等の検査）
  ├─ Virtualの親選択・順序に沿ったUTXO/取引の検証
  ├─ UTXO状態の更新
  └─ イベント通知／必要に応じたMempool再評価
```

GHOSTDAGは本体検証より前のヘッダー処理に含まれる。ヘッダー保存・本体保存・UTXO検証の状態は区別する（[第11章](chapter-11.md)）。この図は**概念モデル**であり、実装の厳密な呼び出し順序ではない。実際には段階的検証、非同期パイプライン、重複検知、キャッシュ、状態更新の最適化が存在する。

### 何を信頼しないか

- P2Pで届いたブロックの主張。
- RPC利用者が提出したトランザクション。
- 他ノードが報告した同期状況。
- アプリケーションが推定した承認状態。

**検証済みのコンセンサス結果と、外部から与えられた入力を分離する**ことが重要である。

## 9.10 同期とPruning

新規ノードは、ネットワークに接続しただけではすぐに最新状態を利用できない。既知のDAG・ヘッダー・必要なブロックデータ・状態を取得し、整合性を検証する必要がある。これがInitial Block Download（IBD）を含む同期処理である。

Kaspaには古い履歴を無制限にすべて保持する前提を避けるため、**pruning**とそれに対応する証明・同期手順がある。

| 概念 | 目的 |
|---|---|
| IBD | 新規ノードをネットワーク状態へ追いつかせる |
| Pruning | 一定の条件の下で古いデータの保持量を抑える |
| Pruning point | 履歴を整理する際の基準点 |
| Pruning proof | Pruning pointの候補を複数レベルのヘッダー等から検査する仕組み。受信UTXO集合の検証は別の経路 |

「データを削除する＝安全性を無視する」ではない。一方で、何を保持し、何を証明で置き換えるかは**コンセンサスの安全性とストレージ効率のトレードオフ**である。

詳しいアルゴリズムは[第21章](../part-04-development/chapter-21.md)で扱う。既存Devnet実測はPruning pointがGenesisのままであり、実際の削減やPruning後IBDは未検証である。

## 9.11 ノードのストレージ設計

フルノードが扱う情報は単一の「ブロック一覧」ではない。

```text
Persistent storage
├── Headers / blocks
├── DAG relations
├── Consensus metadata
├── UTXO-related state
├── Indexes / metadata
└── Pruning / synchronization data
```

実際の保存形式・キー設計・キャッシュ境界は実装バージョンに依存する。アプリケーションがノードのDBを直接参照すると内部形式に強く依存するため、原則として**公開RPCまたは専用インデクサを境界として利用する**。

### 設計で分けるべきもの

- **真実の源泉**：検証されたコンセンサス状態。
- **高速化のための表現**：キャッシュ、インデックス、集約ビュー。
- **ユーザー向けの解釈**：残高、履歴、承認状態。

これらを混ぜると、キャッシュ不整合が「残高の不整合」に見えるなど、障害調査が難しくなる。

## 9.12 Mempoolはコンセンサスそのものではない

Mempoolはノードが採掘候補として保持するトランザクションや、その依存・不足入力などを管理する。全未確定取引の一覧や、永続的なウォレット履歴ではない。ここでは、トランザクションがコンセンサス上有効かどうかに加え、**ノードの受け入れ・中継ポリシー**が関わる。

例えば、ある手数料水準のトランザクションをMempoolが拒否したとしても、それだけで「コンセンサス規則上無効」と結論づけてはいけない。基準版の[手数料率検査](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/transaction_validator/tx_validation_in_utxo_context.rs)にも、Mempoolの手数料率の検査はコンセンサス検査ではないと明記されている。

この境界は、ウォレット、取引所、マイニングプールの設計で特に重要になる。

## 9.13 P2P・RPC・wRPCの違い

| インターフェース | 主な相手 | 主な用途 |
|---|---|---|
| P2P | 他のKaspaノード | ブロック・トランザクションの伝播、同期 |
| gRPC | サーバーサイドのRPCクライアント | ノードの照会、提出、通知 |
| wRPC | WebSocketクライアント | ブラウザやアプリからの照会・通知 |

[基準版のREADME](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/README.md)では、wRPCはオプションであり、JSONまたはBorshを使う構成が説明されている。**wRPCは通常のJSON-RPC規格そのものと同一ではない**ため、既存のJSON-RPCクライアントをそのまま流用できると決めつけないこと。

### セキュリティ境界

P2PポートとRPCポートは用途が違う。特に管理系RPCをインターネットへ無制限に公開しない。ネットワーク構成、認証、ファイアウォール、アクセス元制限を検討する。

## 9.14 `rusty-kaspa` のコードベースを読む

KaspaのRust実装はワークスペース構成であり、機能ごとに複数のディレクトリ・crateへ分かれている。

以下は**調査の入口**であって、厳密な依存関係図ではない。

| ディレクトリ（代表例） | 調べる対象 |
|---|---|
| `kaspad/` | ノードの起動、構成、統合 |
| `consensus/` | コンセンサス規則、DAG、Virtual、UTXO |
| `protocol/` | P2Pプロトコル、同期、メッセージ処理 |
| `rpc/` | 外部APIとサーバー・クライアント |
| `mining/` | ブロックテンプレートやマイニング関連 |
| `database/` | 永続化とストレージ抽象 |
| `crypto/` | 暗号処理 |
| `wallet/` | ウォレット関連の機能 |

### 推奨する読み方

1. `README.md` とルートの `Cargo.toml` で全体像を確認する。
2. `kaspad/` から起動時の構成・サービス生成を追う。
3. RPCの特定メソッドを一つ選び、呼び出し先へ進む。
4. コンセンサスに入ったら「検証」「DAG関係」「Virtual」「UTXO」を分けてメモする。
5. データを書き換える箇所とイベントを通知する箇所を区別する。
6. 対応するテストを探し、どの不変条件を確認しているか読む。

```bash
# 第1章で取得した基準版の親ディレクトリから実行する
cd rusty-kaspa-v1.0.1
git rev-parse HEAD
find . -maxdepth 2 -name Cargo.toml
grep -n -E 'ConsensusFactory|MiningManager|RpcCoreService|WrpcService' kaspad/src/daemon.rs
```

未取得の場合は[第1章](../part-01-foundations/chapter-01.md#12-実践kaspaのコードを見てみる)の手順を先に実行する。コミットの期待値は本章冒頭の完全SHAである。実装は本書のリポジトリ外へ取得し、この調査ではビルドやノード起動を行わない。

### 調査ログのテンプレート

```markdown
## 調査対象
- Commit:
- Entry point:
- 目的:

## データフロー
1. 入力:
2. 検証:
3. 状態変更:
4. 通知:

## 不変条件
-

## 関連テスト
-

## 未確認事項
-
```

## 9.15 設計を理解するための簡易Rustモデル

実際のコンセンサス処理と構造管理を区別するため、**DAGの依存関係だけ**を扱う小さなプログラムを作る。以下は学習用であり、PoW、GHOSTDAG、トランザクション検証、pruningなどを実装していない。

本書のリポジトリ外の作業用ディレクトリで作成する。Rust 1.90.0で確認した、標準ライブラリのみの教材である。

```bash
cargo new --lib --edition 2021 kaspa-architecture-lab
cd kaspa-architecture-lab
```

`Cargo.toml`：

```toml
[package]
name = "kaspa-architecture-lab"
version = "0.1.0"
edition = "2021"
```

`src/lib.rs`：

```rust
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub id: String,
    pub parents: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Dag {
    blocks: HashMap<String, Block>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DagError {
    Duplicate,
    UnknownParent(String),
    SelfParent,
    DuplicateParent,
}

impl Dag {
    pub fn insert(&mut self, block: Block) -> Result<(), DagError> {
        if self.blocks.contains_key(&block.id) {
            return Err(DagError::Duplicate);
        }
        if block.parents.iter().any(|p| p == &block.id) {
            return Err(DagError::SelfParent);
        }
        if block.parents.iter().collect::<HashSet<_>>().len() != block.parents.len() {
            return Err(DagError::DuplicateParent);
        }
        for parent in &block.parents {
            if !self.blocks.contains_key(parent) {
                return Err(DagError::UnknownParent(parent.clone()));
            }
        }
        self.blocks.insert(block.id.clone(), block);
        Ok(())
    }

    pub fn tips(&self) -> HashSet<String> {
        let mut tips: HashSet<String> = self.blocks.keys().cloned().collect();
        for block in self.blocks.values() {
            for parent in &block.parents {
                tips.remove(parent);
            }
        }
        tips
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(id: &str, parents: &[&str]) -> Block {
        Block {
            id: id.to_string(),
            parents: parents.iter().map(|p| p.to_string()).collect(),
        }
    }

    #[test]
    fn parallel_blocks_can_be_merged() {
        let mut dag = Dag::default();
        dag.insert(block("genesis", &[])).unwrap();
        dag.insert(block("a", &["genesis"])).unwrap();
        dag.insert(block("b", &["genesis"])).unwrap();
        assert_eq!(dag.tips(), HashSet::from(["a".into(), "b".into()]));
        dag.insert(block("c", &["a", "b"])).unwrap();
        assert_eq!(dag.tips(), HashSet::from(["c".into()]));
    }

    #[test]
    fn invalid_insertions_preserve_all_blocks() {
        let mut dag = Dag::default();
        dag.insert(block("genesis", &[])).unwrap();
        let before = dag.blocks.clone();
        for (candidate, error) in [
            (block("genesis", &[]), DagError::Duplicate),
            (block("a", &["a"]), DagError::SelfParent),
            (
                block("a", &["genesis", "genesis"]),
                DagError::DuplicateParent,
            ),
            (
                block("a", &["missing"]),
                DagError::UnknownParent("missing".into()),
            ),
        ] {
            assert_eq!(dag.insert(candidate), Err(error));
            assert_eq!(dag.blocks, before);
        }
    }

    #[test]
    fn unknown_parent_is_rejected() {
        let mut dag = Dag::default();
        assert_eq!(
            dag.insert(block("a", &["missing"])),
            Err(DagError::UnknownParent("missing".into()))
        );
    }
}
```

実行方法：

```bash
cargo test
```

この実装では**親が先に登録される**ことを要求する。実際のP2Pでは親より先に子が到着しうるため、未解決ブロックを一時保持して不足データを要求する仕組みが必要になる。また、この学習用モデルはGenesisの一意性、ハッシュの検証、最大親数、ネットワーク敵対性を扱っていない。tipsは構造上の先端であり、Virtualの親選択を実装したものではない。教材3テストの結果と限界はリポジトリの `experiments/chapter-09-review-2026-10-31.md` に記録している。

### 追加実験

1. 親が届く前のブロックを`pending`に入れる。
2. 親が到着したときに`pending`を再評価する。
3. 複数の親なしブロックを許すモデルと、Genesisを一つに限定するモデルを比較する。
4. `tips()`を毎回全走査せず更新する方式を比較する。
5. DAGへの追加とUTXO更新を別モジュールに分ける。

最後の実験が本章の核心である。**DAG構造の管理と、トランザクションの状態遷移は別の責務**である。

## 9.16 アーキテクチャをDDD・C4で整理する

Kaspaは分散システムだが、設計を読むときは業務システムと同じように「責務」「境界」「依存」を明確にすると理解しやすい。

### コンテキストを分ける

| 概念的な境界 | 所有する関心事 | 外部へ公開すべきもの |
|---|---|---|
| Networking | Peer、メッセージ、同期 | 検証前の入力イベント |
| Consensus | DAG、順序、検証、状態 | 検証済みの状態と結果 |
| Transaction pool | 候補トランザクションとポリシー | ブロック候補・受理結果 |
| API | 外部クライアントとの契約 | 安定したリクエスト／レスポンス |
| Persistence | 永続化と再起動 | 保存・取得の抽象 |

これはDDDの厳密なBounded Contextをそのまま適用した図ではなく、**設計を読むための分割案**である。実際のcrate分割や依存方向を決める前に、コードで確認する必要がある。

### 変更影響を考える

- RPCのフィールド追加は、コンセンサス規則を変更せずに実施できる場合がある。
- コンセンサス規則の変更は、ノード間の合意に影響するため慎重な移行が必要になる。
- ストレージ最適化は、検証結果が変わらないことをテストしなければならない。
- Mempoolポリシーの変更とコンセンサス変更は別に評価する。

## 9.17 障害調査：どこから見るか

「ウォレットの残高が更新されない」という症状でも、原因は一つではない。

```text
残高が更新されない
  ├─ RPCに接続できない
  ├─ ノードが同期中
  ├─ 取引がMempoolで拒否された
  ├─ 取引は伝播したがブロックに含まれない
  ├─ ブロックを受信したが状態がまだ更新されない
  ├─ アプリ側の通知購読が切れている
  └─ Indexerの更新が遅れている
```

### 観測項目

| 観測対象 | 確認したいこと |
|---|---|
| P2P | 接続ピア数、遅延、切断、同期元 |
| Sync | IBD中か、最新状態へ追いついているか |
| DAG | 新しいブロックを受信しているか |
| Virtual | コンセンサス状態が進んでいるか |
| Mempool | 対象トランザクションが存在するか、拒否理由は何か |
| RPC | 接続、応答、通知購読が正常か |
| Indexer | ノードの情報を取り込めているか |

> **運用上の原則**：ブロック受信数だけで同期完了を判定しない。ノードの同期フラグ、状態、RPC応答、アプリ側の観測を組み合わせる。

## 9.18 バージョン差異とアップグレード

Kaspaのアーキテクチャは固定された図ではない。ネットワークアップグレードにより、コンセンサス規則、トランザクション形式、P2Pプロトコル、同期方式、APIが変化する。

例えば[基準版v1.0.1のリリースノート](https://github.com/kaspanet/rusty-kaspa/releases/tag/v1.0.1)には、MainnetでP2P v6ノードとの接続を切る変更が記載されている。これはそのリリースの記録であり、現在のMainnetの全接続条件を確認したという意味ではない。後発版の同期方式を基準版の説明へ混ぜず、参照版と対象ネットワークを明記する。

本書の実装調査では必ず次を記録する。

```text
network: mainnet / testnet / devnet
node version: x.y.z
git commit: <commit hash>
protocol version: <確認した値>
feature activation: <対象ネットワークの有効化状態>
```

特に**KIPのステータス（Draft / Proposed / Active等）と、特定ネットワークで実際に有効な機能は同一ではない**。提案文書だけで動作を断定しない。

## 9.19 Bitcoinとのアーキテクチャ比較

| 観点 | Bitcoin | Kaspa |
|---|---|---|
| ブロック参照 | Genesis以外は単一の親 | 複数の親を参照できる |
| 履歴表現 | 線形チェーンを中心に扱う | BlockDAG |
| 並行ブロック | チェーン競合として扱う | DAGの並行履歴として保持できる |
| 順序の決定 | 選択されたチェーン上の順序 | DAGに対するコンセンサス順序 |
| 現在状態の概念 | 選択チェーンに対応するUTXO | Virtualに対応するUTXO |
| 同期・保存 | チェーン同期とpruning等 | DAG同期とpruning proof等 |
| 共通点 | PoW、UTXO、P2P、検証、フルノード | PoW、UTXO、P2P、検証、フルノード |

**KaspaはBitcoinのすべてを否定して作られたものではない。** PoW、UTXO、フルノードによる検証という重要な設計思想を共有しつつ、ブロック履歴の表現と合意の計算方法を変更している。

## 9.20 よくある誤解

**Q. DAGに入ったブロックはすべて同じ扱いで有効？**  
A. いいえ。ブロックの存在、ヘッダーの妥当性、GHOSTDAG上の分類、トランザクションの状態への反映は別の問題である。

**Q. GHOSTDAGがトランザクションを署名検証する？**  
A. GHOSTDAGは主にDAGのコンセンサス上の選択・分類・順序に関係する。署名やUTXOの検証には別の処理が必要である。

**Q. Virtual Blockは実際にマイニングされる？**  
A. いいえ。現在のDAGを評価するための概念的なブロックである。

**Q. RPCが成功したら送金完了？**  
A. いいえ。提出成功、Mempool受理、ブロックへの包含、状態への反映は区別する。

**Q. NodeのDBを読めばウォレット履歴を簡単に作れる？**  
A. 内部DB形式は実装依存であり、ウォレット向け履歴検索には別のインデックスが必要になることがある。公開APIやIndexerの利用を検討する。

## 9.21 理解度チェック

1. KaspaのPoW、BlockDAG、GHOSTDAGはそれぞれ何を担うか。
2. DAGに登録されたブロックと、Virtual UTXOに反映されたトランザクションはなぜ区別されるのか。
3. Virtual Blockが必要になる理由を、複数tipという言葉を使って説明せよ。
4. Mempoolポリシーとコンセンサス規則の違いを説明せよ。
5. ウォレットで残高が更新されないとき、どの境界を順番に確認するか。
6. `rusty-kaspa` のソースコードからRPC呼び出しがコンセンサス状態を参照する経路を調べる手順を書け。
7. なぜ`git commit`を固定しない実装解説は再現性が低いのか。
8. Rustの簡易DAGモデルに親ブロックの遅延到着を追加する場合、どの状態を導入するか。

### 確認の観点

- 「データ構造」「コンセンサス規則」「アプリケーションAPI」を分離できているか。
- 「検証」「保存」「順序」「状態更新」を同じ動詞で済ませていないか。
- 入力の信頼境界と、変更時の影響範囲を説明できているか。

## 9.22 まとめ

Kaspaの全体像は、次のように整理できる。

```text
P2Pでブロックと取引を受け取る
                ↓
     ヘッダー処理でGHOSTDAG等を計算・検査する
                ↓
       本体と取引構造を検査する
                ↓
     Virtualの親・順序・取引の受理を計算する
                ↓
        Virtual UTXOを更新する
                ↓
       RPCで状態を公開する
```

実装はこれほど直線的ではないが、**どの層が何に責任を持つか**を理解するための出発点になる。

[第10章](chapter-10.md)では、P2Pネットワークとノード間同期に焦点を当て、ネットワークから届くデータをどう扱うかを詳しく見る。

---

## 参考資料

基準版を確認した出典と、更新される背景資料を分けて読む。実装の説明は本章冒頭の固定SHAを基準とする。

1. [Kaspa公式 `rusty-kaspa` リポジトリ](https://github.com/kaspanet/rusty-kaspa) — Rustフルノード、ディレクトリ構成、ビルド、RPC、シミュレータ。
2. [`rusty-kaspa` Releases](https://github.com/kaspanet/rusty-kaspa/releases) — プロトコルや実装の変更履歴。
3. [Kaspa Improvement Proposals（KIPs）](https://github.com/kaspanet/kips) — 提案の状態、設計背景、プロトコル仕様。
4. [Kaspa RPC Protocol Definition](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/rpc/grpc/core/proto/rpc.proto) — ノードAPIのメッセージ定義。
5. [基準版の取引検証](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/transaction_validator/tx_validation_in_utxo_context.rs) — コンセンサス規則とMempoolの手数料率検査の区別。
6. [Kaspa公式ドキュメント](https://docs.kaspa.org/) — ノード・開発・プロトコル資料。
7. [Bitcoin: A Peer-to-Peer Electronic Cash System](https://bitcoin.org/bitcoin.pdf) — Bitcoinとの設計比較の原点。

> **確認範囲（2026-10-31）**：基準版の起動・Mempool・RPC・主要検証経路の入口と説明を照合し、DAG教材3テストを確認した。ノード全体の実行試験、現行Mainnetの仕様、Pruning後IBD、全規則の監査は対象外。記録はリポジトリの `experiments/chapter-09-review-2026-10-31.md` を参照。
