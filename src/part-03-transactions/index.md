# Part III — Transactions, Mining & Security（トランザクション・マイニング・セキュリティ）

トランザクションが構築・署名され、Mempoolを経て採掘とコンセンサス受理へ進む過程を学びます。報酬の仕組みとネットワークの安全性を扱い、ノードのFinalityとアプリケーションの決済判断を区別します。

## 読む前に

Part IIで扱ったGHOSTDAG・DAA・Virtual Stateと、Part IのUTXO・署名の基礎を使います。

## この部の各章

- [Chapter 15 — UTXO Model & Transaction Processing（UTXOモデルとトランザクション処理）](chapter-15.md)
- [Chapter 16 — Kaspa Script & Transaction Signing（Kaspa Scriptとトランザクション署名）](chapter-16.md)
- [Chapter 17 — Mempool & Transaction Propagation（Mempoolとトランザクション伝播）](chapter-17.md)
- [Chapter 18 — Mining & Block Template Construction（マイニングとブロックテンプレート構築）](chapter-18.md)
- [Chapter 19 — Mining Economics & Network Security（マイニング経済とネットワークセキュリティ）](chapter-19.md)
- [Chapter 20 — Kaspa Network Security & Finality（Kaspaのネットワークセキュリティとファイナリティ）](chapter-20.md)

## 第15〜17章で使う共通の送金実習

第15〜17章は、同じ `experiments/devnet/send.cjs --checks` を異なる観点から読みます。[第28章の環境準備](../part-04-development/chapter-28.md#286-ローカル環境の準備)と[送金実習の起動・実行・終了手順](../part-04-development/chapter-28.md#2810-トランザクションのe2eテスト)に従って一度実行し、その結果と収録コードを各章で確認できます。各章のために同じ実習を繰り返す必要はありません。

| 章 | 共通の実習で注目すること |
|---|---|
| [第15章](chapter-15.md#15171-教材から公式sdkの送金処理へ) | UTXOの取得、取引構築、手数料・お釣りの検算、受理結果の照会 |
| [第16章](chapter-16.md#16123-fakeverifierから公式sdkの署名へ) | 一時鍵と入力の対応、署名、不正署名の拒否と入力の維持 |
| [第17章](chapter-17.md#1714-rusty-kaspa-のソースコードを読む) | 採掘停止中のMempool掲載、競合取引の拒否、採掘再開後の受理 |

この実習は固定版の隔離Devnetで行います。各章のRust教材は、それぞれの責務を理解するために別途実行するコードです。

## 読み終えたときの目標

送信・Mempool掲載・受理・決済判断の違いを説明し、署名・手数料・報酬の役割を整理できることを目指します。
