# Kaspa 技術・エコシステムマップ

> **Draft / 調査用の俯瞰図**  
> 本ページは技術分野と学習経路の整理を目的とする。プロジェクトの稼働状況・メインネット対応・互換性は未検証であり、採用前に一次資料で確認すること。

## 目的

Kaspaの**L1プロトコル**、その上で動く**アプリケーション**、開発者向け**ツール**、外部の**実行環境・L2**を区別し、本書の各章と対応させる。

## 全体像

```text
                          KASPA ECOSYSTEM
                                  |
        +-------------------------+--------------------------+
        |                         |                          |
   APPLICATIONS              DEVELOPER TOOLS          EXTERNAL SYSTEMS
   Wallet / Payment          SDK / RPC / wRPC         L2 / Bridges
   Explorer / Indexer        SilverScript*            Igra* / Kasplex*
   Games / DeFi              Argent*                  ZK-related projects*
        |                         |                          |
        +-------------------------+--------------------------+
                                  |
                       KASPA L1 PROTOCOL
                                  |
               +------------------+------------------+
               |                  |                  |
         TRANSACTIONS         CONSENSUS           NETWORK / NODE
         KAS / UTXO           PoW                 P2P / Synchronization
         Kaspa Script         BlockDAG            rusty-kaspa
         Signature checks     GHOSTDAG            Mempool / Storage
               |
         ADVANCED SCRIPTING
         Covenants* / Script extensions*

         RESEARCH / PROPOSALS (not necessarily activated)
         DAGKnight* / vProgs* / other KIPs*
```

`*` はプロジェクト・提案・研究などを含む領域。**図中の配置は機能分類であり、L1への統合、稼働、公式認定を意味しない。** 特にSilverScriptとArgentは実装リポジトリを特定してから技術的関係を確定する。

## 混同しやすい概念

| 項目               | 位置づけ                                                   | 本書での扱い                                 |
| ------------------ | ---------------------------------------------------------- | -------------------------------------------- |
| **Kaspa Script**   | UTXOの支出条件を検証するL1の仕組み                         | [第16章](part-03-transactions/chapter-16.md) |
| **Covenants**      | 支出先や後続トランザクションなどへの制約を表す概念・機能群 | [第31章](part-05-advanced/chapter-31.md)     |
| **SilverScript**   | 高水準のスクリプト／コントラクト記述を扱う開発者向け技術   | [第32章](part-05-advanced/chapter-32.md)     |
| **Argent**         | 同名プロジェクトの対象・機能・Kaspaとの関係を要調査        | 第32章の調査候補（位置づけ未確定）           |
| **Igra / Kasplex** | L1外部の実行環境やエコシステム関連プロジェクト             | [第33章](part-05-advanced/chapter-33.md)     |
| **vProgs**         | 検証可能なプログラム実行に関する研究・構想                 | [第34章](part-05-advanced/chapter-34.md)     |
| **DAGKnight**      | コンセンサスの研究・発展                                   | [第29章](part-05-advanced/chapter-29.md)     |

**注意：** Covenantという考え方を説明できることと、任意のCovenantがKaspaメインネットで有効であることは同義ではない。また、開発言語の存在は基盤プロトコルの機能追加を保証しない。

## 本書を読む順序

```text
BitcoinとPoW（第5〜6章）
        ↓
BlockDAG（第7〜8章）
        ↓
Kaspaアーキテクチャ・GHOSTDAG（第9〜14章）
        ↓
UTXOとKaspa Script（第15〜16章）
        ↓
RPC / SDK / ノード開発（第22〜28章）
        ↓
Covenants → SilverScript → L2 → vProgs（第31〜34章）
        ↓
アプリケーション構築と貢献（第35〜36章）
```

## 一次資料と検証のルール

個々の技術について、以下を記録してから本文の記述を確定する。

1. **出典**：論文・KIP・公式ドキュメント・ソースコードのURL。
2. **対象バージョン**：参照したコミット、リリース、ネットワーク。
3. **成熟度**：メインネット有効／テストネット・Devnet／開発中／提案・研究のいずれか。
4. **検証**：再現可能なコマンド、テスト、期待結果。
5. **制約**：前提条件、信頼モデル、既知の未解決事項。

### 参照先

- [Kaspa Wiki](https://wiki.kaspa.org/en/home)
- [rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)
- [Kaspa Improvement Proposals](https://github.com/kaspanet/kips)

> このページは全体像を掴むための入口であり、個別プロジェクトの採用推奨や安全性評価ではない。
