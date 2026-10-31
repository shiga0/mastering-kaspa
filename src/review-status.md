# 技術レビューの進捗

更新日：2026年10月31日。章全体の正確性、固定ソースとの照合、教材の動作確認は別々に記録する。ここで「未レビュー」は、原稿が存在しないという意味ではなく、章全体の一次資料との照合を完了していないという意味である。

基準版は[本書の対象バージョン](baseline.md)を参照。第12・26章の既存照合は基準版と別のコミットであり、基準版への再照合は未完了。

## 現在の確認範囲

- 第12章と第26章のGHOSTDAG主要規則は、公式実装の基準コミット `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` へ再照合した（Source-checked）。通常managerのkがselected parentのDAA scoreに依存することを反映した。公式分類関数と集合モデルの差分を24ケース・各親配列順712ブロックで比較し一致。原論文の証明・全ケースの網羅・章全体レビューは未完了。
- 第12章のDAG解析器は5テスト、第27章の最小ノードは6テスト、書式検査・Clippyを確認した。Kaspa互換性の検証ではない。
- それ以外の掲載コードについて、過去の抜き出し検査の成功を章全体のVerified判定には使わない。継続的なCI検査は収録済みの2教材が対象。
- 固定digestのkaspad 1.0.1でDevnet起動・読み取りwRPC・採掘と再起動後のDAG/報酬UTXO保持を確認した。単一ノードで公式WASM SDKの署名・送金・virtual受理とUTXO変化も確認した。手数料/お釣りの検算、残高不足・不正署名・競合拒否も実測。応答喪失時の照会復旧・同一TX再送・インメモリの重複計上防止も実測。2ノードのブロック伝播・停止後のDAG/指定報酬先UTXOの追従も実測。両ノード稼働中のP2P分断・各枝での採掘・再接続後の収束も実測。通知の重複適用・購読停止中の欠落・SQLite再読込・停止状態での照会補完も実測。単一プロセス内で通知をヒントとした稼働中の再照会・旧結果拒否・再購読補完も実測。分岐合流時の実UTXO削除とSQLite観測状態の取り消しも実測。複数ワーカー・会計台帳・Pruning・GitHub上のCI実行は未確認。

## 章・付録別の状況

| 原稿 | 一次資料との照合 | 実行検証・残作業 |
|---|---|---|
| [はじめに](introduction.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Kaspa 技術・エコシステムマップ](ecosystem-map.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 01 — Introduction to Kaspa（Kaspa入門）](part-01-foundations/chapter-01.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 02 — Foundations of Digital Money（デジタルマネーの基礎）](part-01-foundations/chapter-02.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 03 — Cryptographic Foundations（暗号技術の基礎）](part-01-foundations/chapter-03.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 04 — Distributed Systems & P2P（分散システムとP2Pネットワーク）](part-01-foundations/chapter-04.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 05 — Bitcoin Architecture（Bitcoinのアーキテクチャ）](part-01-foundations/chapter-05.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 06 — Blockchain Limitations（ブロックチェーンの限界）](part-01-foundations/chapter-06.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 07 — Introduction to BlockDAG（BlockDAG入門）](part-01-foundations/chapter-07.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 08 — From Blockchain to Kaspa（ブロックチェーンからKaspaへ）](part-01-foundations/chapter-08.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 09 — Kaspa Architecture（Kaspaのアーキテクチャ）](part-02-consensus/chapter-09.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 10 — Kaspa Node & P2P Network（KaspaノードとP2Pネットワーク）](part-02-consensus/chapter-10.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 11 — Block Processing & Validation（ブロック処理と検証）](part-02-consensus/chapter-11.md) | 主要経路のみSource-checked（v1.0.1） | 検証順・保存状態・親不足/BadMerkleRootを照合。公式本体テスト2件と教材5テスト成功。P2P全体・全規則・永続化の障害検証は未完了 |
| [Chapter 12 — GHOSTDAG Internals（GHOSTDAGの内部構造）](part-02-consensus/chapter-12.md) | 主要規則のみSource-checked（v1.0.1再照合） | DAG解析器5テスト成功。別ハーネスで公式分類関数と24ケースの親選択・blue/red・score/workが一致。大規模DAG・章全体は未検証 |
| [Chapter 13 — Difficulty Adjustment Algorithm（難易度調整アルゴリズム）](part-02-consensus/chapter-13.md) | 主要経路のみSource-checked（v1.0.1） | Full/Sampled・整数target計算・DAA scoreを照合。教材5テスト成功。Full/Sampled算術27ケース・計54回でbits一致。Full窓/non-DAA/DAA scoreは120ケースで一致。Sampled/Dualは288ケースで一致。全ネットワーク・全境界・Pruning後IBDは未完了 |
| [Chapter 14 — Consensus Ordering & Virtual State（コンセンサス順序と仮想状態）](part-02-consensus/chapter-14.md) | 主要経路のみSource-checked（v1.0.1） | 合意順序・UTXO処理の主要経路を照合。公式実装との実行時差分検証・章全体レビューは未完了 |
| [Chapter 15 — UTXO Model & Transaction Processing（UTXOモデルとトランザクション処理）](part-03-transactions/chapter-15.md) | 主要経路のみSource-checked（v1.0.1） | 合意順序・UTXO処理の主要経路を照合。公式実装との実行時差分検証・章全体レビューは未完了 |
| [Chapter 16 — Kaspa Script & Transaction Signing（Kaspa Scriptとトランザクション署名）](part-03-transactions/chapter-16.md) | 主要経路のみSource-checked（v1.0.1） | 署名/Script・gen1導出・Generator/Signerの該当範囲を照合。単一署名のDevnet送金・virtual受理を実測。HD復旧・異常系・章全体レビューは未完了 |
| [Chapter 17 — Mempool & Transaction Propagation（Mempoolとトランザクション伝播）](part-03-transactions/chapter-17.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 18 — Mining & Block Template Construction（マイニングとブロックテンプレート構築）](part-03-transactions/chapter-18.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 19 — Mining Economics & Network Security（マイニング経済とネットワークセキュリティ）](part-03-transactions/chapter-19.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 20 — Kaspa Network Security & Finality（Kaspaのネットワークセキュリティとファイナリティ）](part-03-transactions/chapter-20.md) | 主要経路のみSource-checked（v1.0.1） | finality point・sink候補・Devnet/Crescendo定数を照合。教材4テスト成功。公式Finality実行比較・攻撃モデルの証明は未完了 |
| [Chapter 21 — Pruning, Storage & Node Synchronization（Pruning・ストレージ・ノード同期）](part-04-development/chapter-21.md) | 主要経路のみSource-checked（v1.0.1） | Devnetの進行条件・削減・証明/UTXO検査をソース照合。実測pointはGenesisのまま。実削減とPruning後IBDは未検証 |
| [Chapter 22 — Kaspa RPC, wRPC & Application Integration（Kaspa RPC・wRPCとアプリケーション統合）](part-04-development/chapter-22.md) | 未レビュー | 固定版の起動・送金・主要な拒否・応答喪失・実通知補完を実測し、第28章と付録へ反映。2ノード/分断/報酬UTXO取り消しは限定条件で実測。章全体レビュー・HD復旧・Pruning後IBDは未完了 |
| [Chapter 23 — Rust SDK & WASM（Rust SDKとWASM）](part-04-development/chapter-23.md) | 未レビュー | 固定版の起動・送金・主要な拒否・応答喪失・実通知補完を実測し、第28章と付録へ反映。2ノード/分断/報酬UTXO取り消しは限定条件で実測。章全体レビュー・HD復旧・Pruning後IBDは未完了 |
| [Chapter 24 — Wallet Architecture & Key Management（ウォレットアーキテクチャと鍵管理）](part-04-development/chapter-24.md) | 主要経路のみSource-checked（v1.0.1） | 署名/Script・gen1導出・Generator/Signerの該当範囲を照合。単一署名のDevnet送金・virtual受理を実測。主要な拒否と応答喪失も実測。HD復旧・全異常系・章全体レビューは未完了 |
| [Chapter 25 — Exploring the rusty-kaspa Codebase（rusty-kaspaのコードベースを読む）](part-04-development/chapter-25.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 26 — Consensus Implementation in Rust（Rustによるコンセンサス実装）](part-04-development/chapter-26.md) | 26.7の経路のみSource-checked（v1.0.1再照合） | 分類関数を24ケースで実行比較。永続化・Virtual State・並行処理の経路は未照合 |
| [Chapter 27 — Building a Minimal Kaspa-like Node in Rust（Rustによる最小Kaspa型ノードの構築）](part-04-development/chapter-27.md) | 未レビュー | 収録サンプル6テスト成功。伝播・受信順の発展課題は未検証 |
| [Chapter 28 — Devnet & Integration Testing（Devnetと統合テスト）](part-04-development/chapter-28.md) | 未レビュー | 固定版の起動・送金・主要な拒否・応答喪失・実通知補完を実測し、第28章と付録へ反映。2ノード/分断/報酬UTXO取り消しは限定条件で実測。章全体レビュー・HD復旧・Pruning後IBDは未完了 |
| [Chapter 29 — GHOSTDAG Security & DAGKnight（GHOSTDAGのセキュリティとDAGKnight）](part-05-advanced/chapter-29.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 30 — Kaspa Improvement Proposals（Kaspa改善提案・KIP）](part-05-advanced/chapter-30.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 31 — Covenants & Advanced Scripting（Covenantsと高度なスクリプティング）](part-05-advanced/chapter-31.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 32 — Silverscript & Smart Contract Design（Silverscriptとスマートコントラクト設計）](part-05-advanced/chapter-32.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Argent — Actorベースのアプリケーション言語](part-05-advanced/argent.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 33 — Layer 2, Bridges & Zero-Knowledge Proofs（レイヤー2・ブリッジ・ゼロ知識証明）](part-05-advanced/chapter-33.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 34 — vProgs & Programmability（vProgsとKaspaのプログラマビリティ）](part-05-advanced/chapter-34.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 35 — Building a Kaspa Application（Kaspaアプリケーションの構築）](part-06-contributing/chapter-35.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Chapter 36 — Contributing to Kaspa（Kaspaへのコントリビュート）](part-06-contributing/chapter-36.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Appendix A — Rust環境・Docker環境の構築](appendices/appendix-a.md) | 未レビュー | 起動・読み取りwRPCの最小構成は検証済み。公式Node.js/WASM SDKで単一署名送金を実測。HDウォレット・異常系・章全体は未完了 |
| [Appendix B — Kaspa用語集](appendices/appendix-b.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Appendix C — 数学・確率・暗号技術の補足](appendices/appendix-c.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Appendix D — RPC・CLIリファレンス](appendices/appendix-d.md) | 未レビュー | 起動・読み取りwRPCの最小構成は検証済み。公式Node.js/WASM SDKで単一署名送金を実測。HDウォレット・異常系・章全体は未完了 |
| [Appendix E — プロトコルパラメータ一覧](appendices/appendix-e.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Appendix F — 論文・KIP・実装対応表](appendices/appendix-f.md) | 未レビュー | 出典・対象バージョン・主張を照合する |
| [Appendix G — 実験結果・テストベクトル](appendices/appendix-g.md) | 未レビュー | 2教材の検証記録あり。G-001〜G-010は未実行 |
| [Appendix H — 参考文献・変更履歴](appendices/appendix-h.md) | 未レビュー | 出典・対象バージョン・主張を照合する |

第14・15章の照合は `experiments/consensus-state-review-2026-10-31.md` に記録している。

第16・24章の照合は `experiments/signing-wallet-review-2026-10-31.md` に記録している。

送金実習の実測は `experiments/devnet-send-review-2026-10-31.md` を参照。

手数料と異常系の追加検証は `experiments/devnet-send-checks-2026-10-31.md` を参照。

応答喪失時の復旧と重複処理は `experiments/devnet-send-retry-2026-10-31.md` を参照。

2ノードの伝播・片側停止復帰は `experiments/devnet-two-node-2026-10-31.md` を参照。

P2P分断と再接続は `experiments/devnet-partition-2026-10-31.md` を参照。

実通知とSQLite観測ストアの検証は `experiments/devnet-notifications-2026-10-31.md` を参照。

稼働中の再照会と世代管理は `experiments/devnet-live-reconciliation-2026-10-31.md` を参照。

分岐合流後の実UTXO削除と観測補正は `experiments/devnet-reorg-observer-2026-10-31.md` を参照。

Pruningの前提条件と未達の実測は `experiments/devnet-pruning-review-2026-10-31.md` を参照。

## 検証記録の場所

リポジトリの `experiments/minimal-node-2026-10-31.md` と `experiments/ghostdag-review-2026-10-31.md` に、実行条件・コマンド・結果・未確認事項を記録している。教材コードは `examples/`、同期検査は `scripts/check-book-examples.py` を参照する。

GHOSTDAGの固定版再照合は `experiments/ghostdag-baseline-review-2026-10-31.md`、公式分類関数との24ケースの実行比較は `experiments/ghostdag-comparison-2026-10-31.md`、再実行入口は `experiments/ghostdag/README.md` を参照する。

第11・13・20章の主要経路と公式本体検証テスト2件・教材14テストは `experiments/core-validation-review-2026-10-31.md` に記録している。

## 次のレビュー順

1. 第12章は24ケースの差分一致を基盤に、本文の図との対応と追加ケースを確認する。
2. 第14・15・20・21章の状態遷移、トランザクション受理、Finality、Pruningを照合する。
3. 第22〜24・28章の固定バージョンによるDevnet実習を通す。
4. 高度な技術について、提案・開発中・ネットワーク有効化の区別を出典で確認する。

DAA算術の27ケースの公式実行比較は `experiments/daa-comparison-2026-10-31.md`、再実行入口は `experiments/daa/README.md`。Full窓/non-DAA/DAA scoreの別検査は `experiments/daa-window-comparison-2026-10-31.md`、実行入口は `experiments/daa-window/README.md`。Sampled/Dualの288ケース比較は `experiments/daa-transition-comparison-2026-10-31.md`、実行入口は `experiments/daa-transition/README.md`。全ネットワーク・中央値時刻窓・Pruning後IBDは未検証。
