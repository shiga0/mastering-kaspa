# Part VI — Building & Contributing（アプリケーション開発と貢献）

これまでの知識を使って、Kaspaアプリケーションの責務とデータの扱いを設計します。最後に、公式リポジトリを調べ、再現可能な報告や変更を開発コミュニティへ届ける方法を学びます。

## 読む前に

Part IVのRPC・SDK・ウォレット・Devnet実習を前提にします。アプリケーション設計にはDBとバックエンド開発の基礎が役立ちます。通常の入金監視には、Part Vで扱うCovenants・Argent・vProgsは必須ではありません。Part IVから第35章へ進めます。

## この部の各章

- [Chapter 35 — Designing and Partially Implementing a Kaspa Application（Kaspaアプリケーションの設計と部分実装）](chapter-35.md)
- [Chapter 36 — Contributing to Kaspa（Kaspaへのコントリビュート）](chapter-36.md)

## 第35章で実装する範囲

第35章はPayment Trackerの設計と部分実装を扱います。実行できるのは、支払い集計のRust教材と、公式SDKの現在UTXOをJSON経由でRustの入金候補へ取り込む[サンプル](chapter-35.md#35111-公式sdkの現在utxoをrustの入金候補へ変換する)です。

請求作成のUI・API、DB永続化、履歴の再照合、確認ポリシーを統合した完成アプリは収録していません。それらは実アプリへ進む際の設計・実装課題として区別します。

## 読み終えたときの目標

受理・再編成・重複処理を意識したアプリケーションの設計と、根拠・再現手順を添えた技術的な提案ができることを目指します。
