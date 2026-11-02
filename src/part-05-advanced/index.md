# Part V — Advanced Kaspa（Kaspaの高度な技術）

GHOSTDAGの安全性とDAGKnight、KIP、Covenants、Silverscript、Argent、L2・ZK・vProgsを扱います。研究・提案・公開実装・ネットワークでの有効化を区別しながら、Kaspaのプログラマビリティを読み解きます。

## 読む前に

Part II・IIIのコンセンサスとトランザクション、およびPart IVの実装・SDKの基礎を使います。個々の分野を調べる場合は、関連する章から読むこともできます。

## 目的に合わせた読み進め方

| 目的 | 読み進め方 |
|---|---|
| コンセンサスの安全性と研究を読む | [第29章](chapter-29.md)のGHOSTDAG・DAGKnight → [第30章](chapter-30.md)の提案・実装・有効化の調査 |
| Script・コントラクト開発を学ぶ | [第30章](chapter-30.md)のKIP → [第31章](chapter-31.md)のCovenants → [第32章](chapter-32.md)のSilverScript・VM実習 → [Argent](argent.md)のActorモデル |
| L2やアプリ間連携を設計する | [第33章](chapter-33.md)のL2・ブリッジ・ZK → [第34章](chapter-34.md)のvProgs・計算依存・原子的合成 |

本書のノード基準版、SilverScript、Argent、vProgsは、それぞれ異なる版・依存関係を参照しています。同じ環境で一続きに動く実習ではありません。各ページに示したコミット・ツールチェーン・実行環境を使い、ローカル検査の成功と対象ネットワークでの受理・有効化を区別してください。

## この部の各章

- [Chapter 29 — GHOSTDAG Security & DAGKnight（GHOSTDAGのセキュリティとDAGKnight）](chapter-29.md)
- [Chapter 30 — Kaspa Improvement Proposals（Kaspa改善提案・KIP）](chapter-30.md)
- [Chapter 31 — Covenants & Advanced Scripting（Covenantsと高度なスクリプティング）](chapter-31.md)
- [Chapter 32 — Silverscript & Smart Contract Design（Silverscriptとスマートコントラクト設計）](chapter-32.md)
- [Argent — Actorベースのアプリケーション言語](argent.md)
- [Chapter 33 — Layer 2, Bridges & Zero-Knowledge Proofs（レイヤー2・ブリッジ・ゼロ知識証明）](chapter-33.md)
- [Chapter 34 — vProgs & Programmability（vProgsとKaspaのプログラマビリティ）](chapter-34.md)

## 読み終えたときの目標

各技術の目的・信頼の前提・実装段階を、固定した論文・KIP・ソースと結び付けて説明できることを目指します。

各技術の状態は参照した版・資料の範囲で扱います。現在のMainnetで利用できるかは、各章の対象と根拠を確認してください。
