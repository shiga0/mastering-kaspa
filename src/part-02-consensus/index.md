# Part II — Kaspa Consensus & Architecture（コンセンサスとアーキテクチャ）

Kaspaノードの全体像を押さえ、受信したブロックが検証され、GHOSTDAG・難易度調整・Virtual Stateの更新へ進む流れを追います。ブロックの分類とトランザクションの受理を、別の処理として結び付けます。

## 読む前に

Part IのPoW・UTXO・DAGの基礎を前提にします。ソースコードの参照にはRustの基本文法が役立ちます。

## この部の各章

- [Chapter 09 — Kaspa Architecture（Kaspaのアーキテクチャ）](chapter-09.md)
- [Chapter 10 — Kaspa Node & P2P Network（KaspaノードとP2Pネットワーク）](chapter-10.md)
- [Chapter 11 — Block Processing & Validation（ブロック処理と検証）](chapter-11.md)
- [Chapter 12 — GHOSTDAG Internals（GHOSTDAGの内部構造）](chapter-12.md)
- [Chapter 13 — Difficulty Adjustment Algorithm（難易度調整アルゴリズム）](chapter-13.md)
- [Chapter 14 — Consensus Ordering & Virtual State（コンセンサス順序と仮想状態）](chapter-14.md)

## 読み終えたときの目標

ブロック受信からVirtual Stateまでの主要な処理と、その入力・出力・検証の役割を説明できることを目指します。
