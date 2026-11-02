# Part IV — Kaspa Development & Implementation（Kaspaの開発と実装）

保存・同期の仕組みからRPC・SDK・ウォレットへ進み、公式実装の読み方と教育用ノードの自作をつなぎます。最後にDevnetで実ノードを動かし、期待した結果と実測を照合します。

## 読む前に

Part II・IIIの処理の流れに加え、Rust・ターミナル・ネットワークの基本を使います。実習ごとのPython・Node.js・Docker等の前提は各章と付録Aを参照してください。

## 目的に合わせた読み進め方

| 目的 | 読み進め方 |
|---|---|
| アプリケーションを開発する | [第22章](chapter-22.md)のRPC → [第23章](chapter-23.md)のSDK・WASM → [第24章](chapter-24.md)のウォレット → [第28章](chapter-28.md)の実ノード実習 |
| ノード内部の実装を学ぶ | [第21章](chapter-21.md)の保存・同期 → [第25章](chapter-25.md)のコード調査 → [第26章](chapter-26.md)のコンセンサス実装 → [第27章](chapter-27.md)の教育用ノード → [第28章](chapter-28.md)の実ノード実習 |

各章から第28章の実習へ進む場合は、先に[28.6の環境準備](chapter-28.md#286-ローカル環境の準備)を確認してください。第27章の教育用ノードを完成させなくても、公式ノードとSDKを使う実習に取り組めます。自作教材と公式実装の違いを理解したい場合は、第27章と第28章を続けて読むと比較しやすくなります。

## この部の各章

- [Chapter 21 — Pruning, Storage & Node Synchronization（Pruning・ストレージ・ノード同期）](chapter-21.md)
- [Chapter 22 — Kaspa RPC, wRPC & Application Integration（Kaspa RPC・wRPCとアプリケーション統合）](chapter-22.md)
- [Chapter 23 — Rust SDK & WASM（Rust SDKとWASM）](chapter-23.md)
- [Chapter 24 — Wallet Architecture & Key Management（ウォレットアーキテクチャと鍵管理）](chapter-24.md)
- [Chapter 25 — Exploring the rusty-kaspa Codebase（rusty-kaspaのコードベースを読む）](chapter-25.md)
- [Chapter 26 — Consensus Implementation in Rust（Rustによるコンセンサス実装）](chapter-26.md)
- [Chapter 27 — Building a Minimal Kaspa-like Node in Rust（Rustによる最小Kaspa型ノードの構築）](chapter-27.md)
- [Chapter 28 — Devnet & Integration Testing（Devnetと統合テスト）](chapter-28.md)

## 読み終えたときの目標

公式ソースの処理と自作教材の省略を区別し、Devnet実習の結果・環境・限界を記録できることを目指します。
