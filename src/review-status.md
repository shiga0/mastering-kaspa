# 技術レビューの進捗

更新日：2026年10月31日。本書の[対象バージョン](baseline.md)を基準に、ページの確認と個別の検証結果を記録します。

## SUMMARY順のレビュー

`src/SUMMARY.md` の上から、文章・構成・技術的根拠・リンク・実習への導線を確認しています。既存のソース照合や実行結果を使い、同じ検証を繰り返すことは避けます。

| 目次順 | ページ | 状況 | レビュー記録 |
|---|---|---|---|
| 1 | [はじめに](introduction.md) | 読者案内・概説を確認 | `experiments/introduction-review-2026-10-31.md` |
| 2 | [本書の対象バージョン](baseline.md) | 版・配布物・確認範囲を確認 | `experiments/baseline-review-2026-10-31.md` |
| 3 | [技術レビューの進捗](review-status.md) | 進捗と検証記録の対応を確認 | `experiments/review-status-review-2026-10-31.md` |
| 4 | [Kaspa 技術・エコシステムマップ](ecosystem-map.md) | 分類・出典・確認範囲を確認 | `experiments/ecosystem-map-review-2026-10-31.md` |
| 5 | [第1章](part-01-foundations/chapter-01.md) | 入門概説・基準版との対応を確認 | `experiments/chapter-01-review-2026-10-31.md` |
| 6 | [第2章](part-01-foundations/chapter-02.md) | 基礎概説・台帳教材を確認 | `experiments/chapter-02-review-2026-10-31.md` |
| 7 | [第3章](part-01-foundations/chapter-03.md) | 基礎概説・SHA-256教材を確認 | `experiments/chapter-03-review-2026-10-31.md` |
| 8 | [第4章](part-01-foundations/chapter-04.md) | 基礎概説・伝播教材を確認 | `experiments/chapter-04-review-2026-10-31.md` |
| 9 | [第5章](part-01-foundations/chapter-05.md) | 基礎概説・UTXO教材を確認 | `experiments/chapter-05-review-2026-10-31.md` |
| 10 | [第6章](part-01-foundations/chapter-06.md) | 基礎概説・確率計算教材を確認 | `experiments/chapter-06-review-2026-10-31.md` |
| 11 | [第7章](part-01-foundations/chapter-07.md) | 集合の定義・DAG教材を確認 | `experiments/chapter-07-review-2026-10-31.md` |
| 12 | [第8章](part-01-foundations/chapter-08.md) | 総まとめ・基準版の処理経路を確認 | `experiments/chapter-08-review-2026-10-31.md` |
| 13 | [第9章](part-02-consensus/chapter-09.md) | 次の対象 | — |

この順番でページを確認したことと、ページに関係する全技術・全コードを検証したことは別です。後続章には、先に重要箇所だけを照合・実行したものがあります。

## 判定の読み方

- **ページの確認**：目次順で通読し、説明・案内・根拠との対応を確認した範囲です。
- **資料・実装との照合**：固定ソースや論文を確認した範囲です。下表の「未照合」は、このページの資料照合をまだ進めていない状態を表し、個別の実行記録があれば右欄に記載します。
- **実行検証**：記録にある版・入力・環境で実行した範囲です。教材のテスト、公式関数の比較、Dockerノードの実測は分けて読みます。

`Source-checked` は、左欄に示す範囲のソース照合を表します。実行済みの個別検査があっても、その結果は右欄に限定して記録し、章全体の `Verified` へは広げません。個々の主張の判定基準は[付録H](appendices/appendix-h.md)を参照してください。

## 主な検証結果と限界

以下は既存の記録を整理したものです。今回のページレビューで、プロトコルやDevnetの実験を新たに実行したわけではありません。詳細な入力・失敗試行・環境・再実行手順は、後半の記録一覧を参照してください。

| 対象 | 確認した範囲 | 主な限界・対象外 |
|---|---|---|
| 教育用Rust教材 | DAG解析器5テスト、最小ノード6テスト、書式・Clippy。第3章SHA-256抜き出し教材は実行・書式・2テスト | Kaspa互換性や公式ノードの全規則ではありません。第3章教材は手動検査で、CI対象には追加していません |
| GHOSTDAG | 基準版への再照合、公式分類関数と集合モデルの24ケース比較 | 小規模な合成DAG。到達可能性は共通DFS。全DAG・論文の全証明は対象外 |
| DAA | 算術27ケース、Full窓120ケース、Sampled/Dual288ケースでモデルと一致 | GHOSTDAGメタデータを入力に使う条件付き比較。全ネットワーク・全境界は未検証 |
| ブロック本体検証 | 公式テスト2件で単独検査・BadMerkleRoot/親不足の扱いを確認 | 親不足等の回帰テストはPoWを無効化。全ブロック検証やPoW込みE2Eではありません |
| Devnetの起動・保持 | 起動・読み取りwRPC・採掘・再起動後のDAG/報酬UTXO保持 | 確認したmacOS/ARM64構成に限定 |
| 単一署名送金 | 送金・Mempool/virtual受理・入力消費・出力・手数料/お釣り、所定の拒否と成功応答喪失からの復旧 | HD復旧・全Script・ブロック包含の独立照会・不可逆な決済確定は未検証 |
| 2ノード・分断 | 伝播・片側停止復帰・両側採掘後の収束・報酬UTXOの消失 | 指定報酬先のUTXOを観測。全UTXO集合・競合送金の勝敗・会計処理は対象外 |
| 通知と観測ストア | 重複適用・購読中断・SQLite再読込・再照会補完・古い結果の拒否・観測UTXOの取り消し | 単一書き込み主体の現在UTXO観測。複数ワーカーや会計台帳ではありません |
| Pruning前提 | 固定ソースの条件と、実測pointがGenesisのままであることを確認 | Pruningは未達。実際のデータ削減・Pruning後IBDは未検証 |

ソースをビルドした関数比較と、由来の完全一致を確認していないDockerバイナリの実測は別の証拠です。版・イメージ・依存・確認環境は[対象バージョン](baseline.md)へ集約しています。

## 章・付録別の状況

表はSUMMARYの全49ページと同じ順です。資料照合の範囲と実行記録を分け、目次順でページを確認するたびに更新します。

| 原稿 | 資料・実装との照合 | 実行検証・主な残作業 |
|---|---|---|
| [はじめに](introduction.md) | 読者案内・概説を確認（2026-10-31） | 目次・前提知識・基準版・サンプル区分を整理。PoW/BlockDAGの概説を一次資料で確認。後続章の正確性は各章の判定を参照 |
| [本書の対象バージョン](baseline.md) | 版・配布物・確認範囲を確認（2026-10-31） | 固定ソース/SDK/lockの識別値、依存・CI設定、記録時点の実測環境を照合。MSRV・Dockerバイナリのソース一致・現行Mainnetは未確認 |
| [技術レビューの進捗](review-status.md) | 進捗と証拠の対応を確認（2026-10-31） | 目次49ページ、既存の検証記録・CI設定との対応を確認。新たなプロトコル検証は行っていない |
| [Kaspa 技術・エコシステムマップ](ecosystem-map.md) | 分類・出典・確認範囲を確認（2026-10-31） | 分類図と読む順を整理。固定README・公式資料で位置づけを照合。各プロジェクトの実行・互換性・稼働・安全性は未検証 |
| [Chapter 01 — Introduction to Kaspa（Kaspa入門）](part-01-foundations/chapter-01.md) | 入門概説・基準版との対応を確認（2026-10-31） | 図・分類・順序とTX検証を整理。固定ソースと閲覧課題を照合。論文の全証明・現行Mainnet・新規取得実行は未検証 |
| [Chapter 02 — Foundations of Digital Money（デジタルマネーの基礎）](part-01-foundations/chapter-02.md) | 基礎概説・台帳教材を確認（2026-10-31） | 署名と支出権限、UTXO手数料、疑似コードの並行処理・再送を整理。固定ソース等と照合。DB実装・並行実行・障害復旧は未実測 |
| [Chapter 03 — Cryptographic Foundations（暗号技術の基礎）](part-01-foundations/chapter-03.md) | 基礎概説・ハッシュ/PoW/アドレス型の該当箇所を確認（2026-10-31） | SHA-256教材をsha2 0.10.9・Rust 1.90.0で実行、2テスト成功。Pythonと出力一致。Kaspaの暗号処理の実行比較・安全性証明・鍵管理監査は対象外 |
| [Chapter 04 — Distributed Systems & P2P（分散システムとP2Pネットワーク）](part-01-foundations/chapter-04.md) | 基礎概説・接続管理/アドレス交換の該当箇所を確認（2026-10-31） | 伝播教材の出力・重複受信・書式と近似式の計算例を確認。Kaspaの通信・合意を再現した実験ではない。新たな分断実測は行っていない |
| [Chapter 05 — Bitcoin Architecture（Bitcoinのアーキテクチャ）](part-01-foundations/chapter-05.md) | 基礎概説・Bitcoin Core v28.0の該当箇所を確認（2026-10-31） | UTXO教材の空入出力・既存Outpoint再作成を修正。成功/拒否/状態維持を2テストで確認。Coreの実行・全規則・Bitcoin互換性は未検証 |
| [Chapter 06 — Blockchain Limitations（ブロックチェーンの限界）](part-01-foundations/chapter-06.md) | 基礎概説・近似モデル・書誌情報を確認（2026-10-31） | 追加発見と別枝/ステール率、TPSの平均と容量上限を区別。確率教材5テスト・表示4行と高精度計算を確認。実ネットワークの性能・安全性は未検証 |
| [Chapter 07 — Introduction to BlockDAG（BlockDAG入門）](part-01-foundations/chapter-07.md) | 集合の定義・基準版APIの祖先判定規約を確認（2026-10-31） | Anticoneと反鎖を区別。教材の未知ID照会・表示順を修正し、祖先・拒否時不変性・登録順など5テスト成功。Kaspa互換性・GHOSTDAG全体は未検証 |
| [Chapter 08 — From Blockchain to Kaspa（ブロックチェーンからKaspaへ）](part-01-foundations/chapter-08.md) | 総まとめ・基準版の処理経路を確認（2026-10-31） | 図・集合/score/work・PHANTOMとの関係を整理。第6章の確率教材と既存の主要経路照合を参照。新たなコード実行・ノード実験・安全性証明は対象外 |
| [Chapter 09 — Kaspa Architecture（Kaspaのアーキテクチャ）](part-02-consensus/chapter-09.md) | 未照合 | 目次順で本文・出典・図・コードを確認予定 |
| [Chapter 10 — Kaspa Node & P2P Network（KaspaノードとP2Pネットワーク）](part-02-consensus/chapter-10.md) | 未照合 | 目次順で本文・出典・図・コードを確認予定 |
| [Chapter 11 — Block Processing & Validation（ブロック処理と検証）](part-02-consensus/chapter-11.md) | 主要経路のみSource-checked（v1.0.1） | 検証順・保存状態・親不足/BadMerkleRootを照合。公式本体テスト2件と教材5テスト成功。P2P全体・全規則・永続化の障害検証は未完了 |
| [Chapter 12 — GHOSTDAG Internals（GHOSTDAGの内部構造）](part-02-consensus/chapter-12.md) | 主要規則のみSource-checked（v1.0.1再照合） | DAG解析器5テスト成功。別ハーネスで公式分類関数と24ケースの親選択・blue/red・score/workが一致。大規模DAG・章全体は未検証 |
| [Chapter 13 — Difficulty Adjustment Algorithm（難易度調整アルゴリズム）](part-02-consensus/chapter-13.md) | 主要経路のみSource-checked（v1.0.1） | Full/Sampled・整数target計算・DAA scoreを照合。教材5テスト成功。Full/Sampled算術27ケース・計54回でbits一致。Full窓/non-DAA/DAA scoreは120ケースで一致。Sampled/Dualは288ケースで一致。全ネットワーク・全境界・Pruning後IBDは未完了 |
| [Chapter 14 — Consensus Ordering & Virtual State（コンセンサス順序と仮想状態）](part-02-consensus/chapter-14.md) | 主要経路のみSource-checked（v1.0.1） | 合意順序・virtual・UTXO処理の主要経路を照合。教材3テスト成功。公式実行との差分・クラッシュ耐久性・章全体は未完了 |
| [Chapter 15 — UTXO Model & Transaction Processing（UTXOモデルとトランザクション処理）](part-03-transactions/chapter-15.md) | 主要経路のみSource-checked（v1.0.1） | UTXO・TX検証の主要経路を照合。教材4テスト成功。公式実行との差分・全subnetwork・章全体は未完了 |
| [Chapter 16 — Kaspa Script & Transaction Signing（Kaspa Scriptとトランザクション署名）](part-03-transactions/chapter-16.md) | 主要経路のみSource-checked（v1.0.1） | 署名/Script・gen1導出・Generator/Signerの一部を照合。単一署名送金・所定の拒否・応答喪失を実測。HD復旧・全Script・全異常系・章全体は未完了 |
| [Chapter 17 — Mempool & Transaction Propagation（Mempoolとトランザクション伝播）](part-03-transactions/chapter-17.md) | 未照合 | 目次順で本文・出典・図・コードを確認予定 |
| [Chapter 18 — Mining & Block Template Construction（マイニングとブロックテンプレート構築）](part-03-transactions/chapter-18.md) | 未照合 | 目次順で本文・出典・図・コードを確認予定 |
| [Chapter 19 — Mining Economics & Network Security（マイニング経済とネットワークセキュリティ）](part-03-transactions/chapter-19.md) | 未照合 | 目次順で本文・出典・図・コードを確認予定 |
| [Chapter 20 — Kaspa Network Security & Finality（Kaspaのネットワークセキュリティとファイナリティ）](part-03-transactions/chapter-20.md) | 主要経路のみSource-checked（v1.0.1） | finality point・sink候補・Devnet/Crescendo定数を照合。教材4テスト成功。公式Finality実行比較・攻撃モデルの証明は未完了 |
| [Chapter 21 — Pruning, Storage & Node Synchronization（Pruning・ストレージ・ノード同期）](part-04-development/chapter-21.md) | 主要経路のみSource-checked（v1.0.1） | Devnetの進行条件・削減・証明/UTXO検査をソース照合。実測pointはGenesisのまま。実削減とPruning後IBDは未検証 |
| [Chapter 22 — Kaspa RPC, wRPC & Application Integration（Kaspa RPC・wRPCとアプリケーション統合）](part-04-development/chapter-22.md) | 未照合 | 固定版の読み取り・送金・成功応答喪失・実通知補完を実測。生wRPCとSDKの形式、全エラー、章全体の照合は未完了 |
| [Chapter 23 — Rust SDK & WASM（Rust SDKとWASM）](part-04-development/chapter-23.md) | 未照合 | 公式Node.js/WASM配布SDKで送金・実通知を実測。Rust SDKアダプター、独自WASM教材の実ビルド/ブラウザー実行、章全体は未完了 |
| [Chapter 24 — Wallet Architecture & Key Management（ウォレットアーキテクチャと鍵管理）](part-04-development/chapter-24.md) | 主要経路のみSource-checked（v1.0.1） | 鍵導出・署名・Generator/Signerの一部を照合。単一署名送金と所定の拒否・応答喪失を実測。HD復旧・鍵管理の監査・全異常系は未完了 |
| [Chapter 25 — Exploring the rusty-kaspa Codebase（rusty-kaspaのコードベースを読む）](part-04-development/chapter-25.md) | 未照合 | 目次順で本文・出典・図・コードを確認予定 |
| [Chapter 26 — Consensus Implementation in Rust（Rustによるコンセンサス実装）](part-04-development/chapter-26.md) | 26.7の経路のみSource-checked（v1.0.1再照合） | 分類関数を24ケースで実行比較。永続化・Virtual State・並行処理の経路は未照合 |
| [Chapter 27 — Building a Minimal Kaspa-like Node in Rust（Rustによる最小Kaspa型ノードの構築）](part-04-development/chapter-27.md) | 未照合 | 収録サンプル6テスト成功。伝播・受信順の発展課題は未検証 |
| [Chapter 28 — Devnet & Integration Testing（Devnetと統合テスト）](part-04-development/chapter-28.md) | 未照合 | 起動・送金・所定の異常系・2ノード・分断・通知補完・報酬UTXO取り消しを限定条件で実測し本文へ反映。全シナリオ、新規環境、Pruning後IBDは未完了 |
| [Chapter 29 — GHOSTDAG Security & DAGKnight（GHOSTDAGのセキュリティとDAGKnight）](part-05-advanced/chapter-29.md) | 未照合 | 目次順で本文・出典・図・コードを確認予定 |
| [Chapter 30 — Kaspa Improvement Proposals（Kaspa改善提案・KIP）](part-05-advanced/chapter-30.md) | 未照合 | 目次順で本文・出典・図・コードを確認予定 |
| [Chapter 31 — Covenants & Advanced Scripting（Covenantsと高度なスクリプティング）](part-05-advanced/chapter-31.md) | 未照合 | 目次順で本文・出典・図・コードを確認予定 |
| [Chapter 32 — Silverscript & Smart Contract Design（Silverscriptとスマートコントラクト設計）](part-05-advanced/chapter-32.md) | 未照合 | 目次順で本文・出典・図・コードを確認予定 |
| [Argent — Actorベースのアプリケーション言語](part-05-advanced/argent.md) | 未照合 | 目次順で本文・出典・図・コードを確認予定 |
| [Chapter 33 — Layer 2, Bridges & Zero-Knowledge Proofs（レイヤー2・ブリッジ・ゼロ知識証明）](part-05-advanced/chapter-33.md) | 未照合 | 目次順で本文・出典・図・コードを確認予定 |
| [Chapter 34 — vProgs & Programmability（vProgsとKaspaのプログラマビリティ）](part-05-advanced/chapter-34.md) | 未照合 | 目次順で本文・出典・図・コードを確認予定 |
| [Chapter 35 — Building a Kaspa Application（Kaspaアプリケーションの構築）](part-06-contributing/chapter-35.md) | 未照合 | 目次順で本文・出典・図・コードを確認予定 |
| [Chapter 36 — Contributing to Kaspa（Kaspaへのコントリビュート）](part-06-contributing/chapter-36.md) | 未照合 | 目次順で本文・出典・図・コードを確認予定 |
| [Appendix A — Rust環境・Docker環境の構築](appendices/appendix-a.md) | 未照合 | 記録したARM64環境でDevnet起動・送金等を実測し手順を反映。新規環境、他OS/CPU、付録全体は未確認 |
| [Appendix B — Kaspa用語集](appendices/appendix-b.md) | 未照合 | 目次順で本文・出典・図・コードを確認予定 |
| [Appendix C — 数学・確率・暗号技術の補足](appendices/appendix-c.md) | 未照合 | 目次順で本文・出典・図・コードを確認予定 |
| [Appendix D — RPC・CLIリファレンス](appendices/appendix-d.md) | 未照合 | 起動・読み取り・送金・通知で使ったAPI名と経路を反映。全RPC/CLIの要求・応答・エラーは未照合 |
| [Appendix E — プロトコルパラメータ一覧](appendices/appendix-e.md) | 未照合 | 目次順で本文・出典・図・コードを確認予定 |
| [Appendix F — 論文・KIP・実装対応表](appendices/appendix-f.md) | 未照合 | 目次順で本文・出典・図・コードを確認予定 |
| [Appendix G — 実験結果・テストベクトル](appendices/appendix-g.md) | 未照合 | G-005は24ケース、G-006は算術27・Full窓120・Sampled/Dual288ケースを実行。ほかのIDと既存実験の対応・完了条件の照合は未完了 |
| [Appendix H — 参考文献・変更履歴](appendices/appendix-h.md) | 未照合 | 目次順で本文・出典・図・コードを確認予定 |

## 検証記録の場所

検証記録はリポジトリの `experiments/` にあります。下記のファイル名は同ディレクトリからの相対パスです。記録中の「未実施」は、その実験時点の範囲を示す場合があります。後続検査がある項目は、本ページと記録の追記を併せて確認してください。

| 対象 | 記録ファイル |
|---|---|
| 第8章基礎の総まとめと処理経路 | `chapter-08-review-2026-10-31.md` |
| 第7章BlockDAG概説とDAG教材 | `chapter-07-review-2026-10-31.md` |
| 第6章性能概説と確率計算教材 | `chapter-06-review-2026-10-31.md` |
| 第5章Bitcoin概説とUTXO教材 | `chapter-05-review-2026-10-31.md`、`chapter-05-utxo-tests.rs` |
| 第4章伝播教材と分散システム概説 | `chapter-04-review-2026-10-31.md` |
| 第3章SHA-256教材と暗号概説 | `chapter-03-review-2026-10-31.md` |
| DAG解析器と初期GHOSTDAG照合 | `ghostdag-review-2026-10-31.md` |
| 最小ノード | `minimal-node-2026-10-31.md` |
| 基準版GHOSTDAG再照合 | `ghostdag-baseline-review-2026-10-31.md` |
| 公式GHOSTDAGとの実行比較 | `ghostdag-comparison-2026-10-31.md` |
| ブロック検証・DAA・Finalityの主要経路 | `core-validation-review-2026-10-31.md` |
| DAA算術 / Full窓 / Sampled・Dual | `daa-comparison-2026-10-31.md` / `daa-window-comparison-2026-10-31.md` / `daa-transition-comparison-2026-10-31.md` |
| 第14・15章の主要経路 | `consensus-state-review-2026-10-31.md` |
| 第16・24章の主要経路 | `signing-wallet-review-2026-10-31.md` |
| Devnet環境と起動 | `devnet-environment-2026-10-31.md`、`devnet/README.md` |
| 送金 / 手数料・所定の拒否 / 成功応答喪失 | `devnet-send-review-2026-10-31.md` / `devnet-send-checks-2026-10-31.md` / `devnet-send-retry-2026-10-31.md` |
| 2ノード伝播 / P2P分断 | `devnet-two-node-2026-10-31.md` / `devnet-partition-2026-10-31.md` |
| 通知重複・欠落 / 稼働中の再照会 | `devnet-notifications-2026-10-31.md` / `devnet-live-reconciliation-2026-10-31.md` |
| 分岐合流時のUTXO観測補正 | `devnet-reorg-observer-2026-10-31.md` |
| Pruning条件と未達の実測 | `devnet-pruning-review-2026-10-31.md` |

教材コードは `examples/`、公式関数の比較ハーネスは `experiments/ghostdag/` と `experiments/daa*/`、実ノードの検査は `experiments/devnet/` を参照してください。付録GのG-005・G-006には、今回までの実行結果を限定した範囲で反映しています。ほかの実験IDへ反映するときは、既存の実験と完了条件を個別に対応づけます。

## CIの設定と実行確認

`.github/workflows/deploy.yml` には、次の検査を設定しています。

- mdBookのHTMLビルド。
- 第12・27章の収録済みRust教材2件の書式・Clippy・テスト。
- `scripts/check-book-examples.py` による、その2教材の本文と実装の一致検査。
- `experiments/devnet/test_utxo_store.py` のSQLite観測ストア7テスト。

公式GHOSTDAG/DAAの比較、Dockerを使うDevnet試験、他章の抜き出しRustコードの検査は、このworkflowでは実行しません。設定内容とローカルの成功記録を、GitHub上の実ジョブの成功へ読み替えないでください。GitHub上のCI・Pages公開は未確認です。

## 次に確認するページと残作業

目次順の次は[第9章](part-02-consensus/chapter-09.md)、その次は[第10章](part-02-consensus/chapter-10.md)です。以前の重要章優先の一覧ではなく、この順番でページを確認します。

個別検証として残る主なものは、Finalityの実行時境界、HDウォレット復旧、Pruning後IBD、複数ワーカー/会計処理、新規環境での再現、未照合の説明・数値・出典です。全残作業と完了条件は、リポジトリの `ROADMAP.md` に記録しています。
