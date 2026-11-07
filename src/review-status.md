# 技術レビューの進捗

更新日：2026年11月7日（編集上の整理。技術検証結果の更新ではありません）。本書の[対象バージョン](baseline.md)を基準に、ページの確認と個別の検証結果を記録します。

11月3日の横断レビュー・再実行結果は[今回の記録](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/repository-review-2026-11-03.md)を参照してください。以下の個別レビュー記録は各実施時点の履歴として維持し、今回の確認を全主張・全環境の検証完了へ広げません。

同日の追加依頼による[プログラマビリティ改訂記録](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/programmability-review-2026-11-03.md)では、Covenantの出所・承認、SilverScriptの型と失敗ケース、KCC・ウォレット提案・DAGKnightの参照軸を補強しました。SilverScriptはローカル30ケースを確認し、全仕様・本番有効化は未検証です。過去の一巡レビューや実ノード実験の記録を、今回新規に実行したものへ書き換えません。

続く[最終品質レビュー](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/final-quality-review-2026-11-03.md)では、v2.1.0のTransaction v1と署名範囲を14ケースで確認し、技術選択と最初のRust実習の導線を整理しました。版・リンク・教材の横断検査と、全仕様・全ネットワークの監査は区別します。既存Covenant説明の重複追加はしていません。

## SUMMARY順のレビュー

扉ページ追加前の49ページを目次順に確認し、文章・構成・技術的根拠・リンク・実習への導線のレビューが一巡しました。既存のソース照合や実行結果を使い、同じ検証を繰り返すことは避けます。

## 章・付録別の状況

既存49ページについて、照合範囲・実行結果・個別レビュー記録を一つの表にまとめています。未検証の記載は確認範囲を示し、追加作業の指示ではありません。

| 目次順 | ページ | 資料・実装との照合 | 実行検証・主な限界 | レビュー記録 |
|---|---|---|---|---|
| 1 | [はじめに](introduction.md) | 読者案内・概説を確認（2026-10-31） | 目次・前提知識・基準版・サンプル区分を整理。PoW/BlockDAGの概説を一次資料で確認。後続章の正確性は各章の判定を参照 | `experiments/introduction-review-2026-10-31.md` |
| 2 | [本書の対象バージョン](baseline.md) | 版・配布物・確認範囲を確認（2026-10-31） | 固定ソース/SDK/lockの識別値、依存・CI設定、記録時点の実測環境を照合。MSRV・Dockerバイナリのソース一致・現行Mainnetは未確認 | `experiments/baseline-review-2026-10-31.md` |
| 3 | [技術レビューの進捗](review-status.md) | 進捗と証拠の対応を確認（2026-10-31） | 目次49ページ、既存の検証記録・CI設定との対応を確認。新たなプロトコル検証は行っていない | `experiments/review-status-review-2026-10-31.md` |
| 4 | [Kaspa 技術・エコシステムマップ](ecosystem-map.md) | 分類・出典・確認範囲を確認（2026-10-31） | 分類図と読む順を整理。固定README・公式資料で位置づけを照合。各プロジェクトの実行・互換性・稼働・安全性は未検証 | `experiments/ecosystem-map-review-2026-10-31.md` |
| 5 | [第1章](part-01-foundations/chapter-01.md) | 入門概説・基準版との対応を確認（2026-10-31） | 図・分類・順序とTX検証を整理。固定ソースと閲覧課題を照合。論文の全証明・現行Mainnet・新規取得実行は未検証 | `experiments/chapter-01-review-2026-10-31.md` |
| 6 | [第2章](part-01-foundations/chapter-02.md) | 基礎概説・台帳教材を確認（2026-10-31） | 署名と支出権限、UTXO手数料、疑似コードの並行処理・再送を整理。固定ソース等と照合。DB実装・並行実行・障害復旧は未実測 | `experiments/chapter-02-review-2026-10-31.md` |
| 7 | [第3章](part-01-foundations/chapter-03.md) | 基礎概説・ハッシュ/PoW/アドレス型の該当箇所を確認（2026-10-31） | SHA-256教材をsha2 0.10.9・Rust 1.90.0で実行、2テスト成功。Pythonと出力一致。Kaspaの暗号処理の実行比較・安全性証明・鍵管理監査は対象外 | `experiments/chapter-03-review-2026-10-31.md` |
| 8 | [第4章](part-01-foundations/chapter-04.md) | 基礎概説・接続管理/アドレス交換の該当箇所を確認（2026-10-31） | 伝播教材の出力・重複受信・書式と近似式の計算例を確認。Kaspaの通信・合意を再現した実験ではない。新たな分断実測は行っていない | `experiments/chapter-04-review-2026-10-31.md` |
| 9 | [第5章](part-01-foundations/chapter-05.md) | 基礎概説・Bitcoin Core v28.0の該当箇所を確認（2026-10-31） | UTXO教材の空入出力・既存Outpoint再作成を修正。成功/拒否/状態維持を2テストで確認。Coreの実行・全規則・Bitcoin互換性は未検証 | `experiments/chapter-05-review-2026-10-31.md` |
| 10 | [第6章](part-01-foundations/chapter-06.md) | 基礎概説・近似モデル・書誌情報を確認（2026-10-31） | 追加発見と別枝/ステール率、TPSの平均と容量上限を区別。確率教材5テスト・表示4行と高精度計算を確認。実ネットワークの性能・安全性は未検証 | `experiments/chapter-06-review-2026-10-31.md` |
| 11 | [第7章](part-01-foundations/chapter-07.md) | 集合の定義・基準版APIの祖先判定規約を確認（2026-10-31） | Anticoneと反鎖を区別。教材の未知ID照会・表示順を修正し、祖先・拒否時不変性・登録順など5テスト成功。Kaspa互換性・GHOSTDAG全体は未検証 | `experiments/chapter-07-review-2026-10-31.md` |
| 12 | [第8章](part-01-foundations/chapter-08.md) | 総まとめ・基準版の処理経路を確認（2026-10-31） | 図・集合/score/work・PHANTOMとの関係を整理。第6章の確率教材と既存の主要経路照合を参照。新たなコード実行・ノード実験・安全性証明は対象外 | `experiments/chapter-08-review-2026-10-31.md` |
| 13 | [第9章](part-02-consensus/chapter-09.md) | 構造・基準版の起動/Mempool/RPC/検証経路の入口を確認（2026-10-31） | 依存図・Virtual/祖先の規約・版の混同を修正。本文DAG教材3テスト・書式成功。ノード全体・現行Mainnet・Pruning後IBDは未検証 | `experiments/chapter-09-review-2026-10-31.md` |
| 14 | [第10章](part-02-consensus/chapter-10.md) | 基準版の接続/ハンドシェイク/relayの主要経路を確認（2026-11-01） | 伝播モデルの重複接続・切断・再接続時の未補完を含む5テストと出力を確認。全IBD・攻撃耐性・実ノードの新規通信試験は未検証 | `experiments/chapter-10-review-2026-11-01.md` |
| 15 | [第11章](part-02-consensus/chapter-11.md) | 主要経路Source-checked（v1.0.1）、本文通読確認（2026-11-01） | 既存の公式本体テスト2件に加え、修正後の教材7テスト成功。状態名/is_validと検証完了、同一ID衝突を区別。P2P全体・全規則・永続化障害は未検証 | `experiments/chapter-11-review-2026-11-01.md` |
| 16 | [第12章](part-02-consensus/chapter-12.md) | 主要規則Source-checked（v1.0.1）、本文通読確認（2026-11-01） | 改訂DAG解析器8テスト成功。別ハーネスの既存24ケースで親選択・blue/red・score/workが一致。全DAG・全規則・論文の証明は未検証 | `experiments/chapter-12-review-2026-11-01.md` |
| 17 | [第13章](part-02-consensus/chapter-13.md) | 主要経路Source-checked（v1.0.1）、本文通読確認（2026-11-01） | Full/Sampled・整数target計算・DAA scoreを照合。改訂教材7テスト・実行出力を確認。Full/Sampled算術27ケース・計54回でbits一致。Full窓/non-DAA/DAA scoreは120ケースで一致。Sampled/Dualは288ケースで一致。全ネットワーク・全境界・Pruning後IBDは未完了 | `experiments/chapter-13-review-2026-11-01.md` |
| 18 | [第14章](part-02-consensus/chapter-14.md) | 主要経路Source-checked（v1.0.1）、本文通読確認（2026-11-01） | 合意順序・virtual・UTXO処理を照合。改訂教材7テストと実行出力を確認。公式実行との差分・クラッシュ耐久性・全規則は未検証 | `experiments/chapter-14-review-2026-11-01.md` |
| 19 | [第15章](part-03-transactions/chapter-15.md) | 主要経路Source-checked（v1.0.1）、本文通読確認（2026-11-01） | UTXO・TX検証を照合。改訂教材7テスト成功。初期出力の上書き・不正入力・手数料を確認。公式実行との差分・全subnetwork・全規則は未検証 | `experiments/chapter-15-review-2026-11-01.md` |
| 20 | [第16章](part-03-transactions/chapter-16.md) | 主要経路Source-checked（v1.0.1）、本文通読確認（2026-11-01） | 既存の署名/Script照合とDevnet実測を参照。改訂教材9テスト成功。モックが出力変更を検出できない限界も確認。gen1の鍵/アドレス復旧・メッセージ署名は別実験で確認（2026-11-02）。gen1 HD復旧鍵による送金2件も別実験で成功。全Script・全異常系は未検証 | `experiments/chapter-16-review-2026-11-01.md` |
| 21 | [第17章](part-03-transactions/chapter-17.md) | 主要受付/保留/伝播経路Source-checked（v1.0.1）、本文通読確認（2026-11-01） | 改訂教材8テスト成功。RPC/P2PのRBF条件、孤児、事後再検査、更新を照合。公式実行との差分・全RBF/再編成条件は未検証 | `experiments/chapter-17-review-2026-11-01.md` |
| 22 | [第18章](part-03-transactions/chapter-18.md) | 主要生成経路Source-checked（v1.0.1）、本文通読確認（2026-11-01） | RPC・キャッシュ・取引選択・Virtualからの候補構築を照合。改訂教材8テスト成功。全選択器の分布・報酬・PoW・更新競合の実行検証は未完了 | `experiments/chapter-18-review-2026-11-01.md` |
| 23 | [第19章](part-03-transactions/chapter-19.md) | 報酬の主要分岐Source-checked（v1.0.1）、本文通読確認（2026-11-01） | Coinbase配分・発行スケジュールの入口を照合。改訂経済モデル8テスト・main実行を確認。公式報酬関数との差分・市場実測・攻撃成功確率は未検証 | `experiments/chapter-19-review-2026-11-01.md` |
| 24 | [第20章](part-03-transactions/chapter-20.md) | 主要経路Source-checked（v1.0.1）、本文通読確認（2026-11-01） | finality point・sink候補・Devnet/Crescendo定数を照合。改訂確認ポリシー教材8テスト成功。公式finality point関数/Reachabilityの合成25ケース・112回が別モデルと一致（2026-11-02）。VirtualProcessorの拒否・攻撃モデルの証明・Pruning後IBDは未検証 | `experiments/chapter-20-review-2026-11-01.md` |
| 25 | [第21章](part-04-development/chapter-21.md) | 主要経路Source-checked（v1.0.1）、本文通読確認（2026-11-01） | 進行条件・証明/UTXO・staging境界を照合。改訂教材8テスト成功。実測pointはGenesisのままで、実削減・Pruning後IBD・耐久性は未検証 | `experiments/chapter-21-review-2026-11-01.md` |
| 26 | [第22章](part-04-development/chapter-22.md) | Source-checked（主要API） | 目次順で本文・通知/再照会境界を確認。Readiness教材8テスト成功。過去の固定版送金・成功応答喪失・実通知補完とは別。全API・自動再接続・会計台帳は未検証 | `experiments/chapter-22-review-2026-11-01.md` |
| 27 | [第23章](part-04-development/chapter-23.md) | Source-checked（主要バインディング） | 目次順で本文を確認。独自教材15テスト・web向けWASMビルド・Node境界51項目成功。過去の公式SDK実測とは別。ブラウザーDOM・実Rust RPCアダプター・全SDKは未検証 | `experiments/chapter-23-review-2026-11-01.md` |
| 28 | [第24章](part-04-development/chapter-24.md) | Source-checked（主要経路） | 目次順で本文・復旧/鍵消去の境界を確認。予約ID付き教材11テスト成功。過去の単一署名実測とは別。gen1の160鍵/アドレスを独立プロセスで再構成し、メッセージ署名と明示パス/xpubを照合（2026-11-02）。Unicode入力のNFKDとの差を記録。復旧受取/変更鍵によるDevnet送金2件も受理まで確認。公式探索APIの固定RPC14ケースとgap教材12,288比較も成功。実ノード/Indexerの履歴探索・全終了経路の消去・機器/全Scriptは未検証 | `experiments/chapter-24-review-2026-11-01.md` |
| 29 | [第25章](part-04-development/chapter-25.md) | Source-checked（主要入口） | 目次順で本文・59パッケージのmetadataと主要経路を確認。DAG教材8テスト・型断片のコンパイル成功。全経路・公式テスト全体・クラッシュ耐性は未検証 | `experiments/chapter-25-review-2026-11-01.md` |
| 30 | [第26章](part-04-development/chapter-26.md) | Source-checked（GHOSTDAG/保存境界） | 目次順で本文・ヘッダー保存/Virtual更新を確認。既存DAG教材8テスト・型断片コンパイル成功。過去の公式24ケース比較は再実行せず。全ストア・障害復旧は未検証 | `experiments/chapter-26-review-2026-11-01.md` |
| 31 | [第27章](part-04-development/chapter-27.md) | 教材の実行確認 | 目次順で本文を確認し、ID検査・全状態保持を改訂。16テスト・実行表示・書式・Clippy成功。関数呼び出しの親再送/逆順受信を検査。Kaspa互換性・実P2P・永続化は対象外 | `experiments/chapter-27-review-2026-11-01.md` |
| 32 | [第28章](part-04-development/chapter-28.md) | 設定・教材の実行確認 | 目次順で本文と既存実測を確認。Compose5構成・Rust教材7/観測ストア7テスト成功。実ノード試験は今回は再実行せず。HD復旧受取/変更鍵によるDevnet送金2件は別実験で確認。全Script・会計・実Pruning後IBDは未検証 | `experiments/chapter-28-review-2026-11-01.md` |
| 33 | [第29章](part-05-advanced/chapter-29.md) | 主要出典・教材を確認 | 目次順で本文と公開要旨/書誌・固定版kの扱いを確認。数値教材9テスト成功。証明の全面照合・DAGKnight実装/本番有効化・攻撃実験は未検証 | `experiments/chapter-29-review-2026-11-01.md` |
| 34 | [第30章](part-05-advanced/chapter-30.md) | 主要文書・教材を確認 | 固定KIPの主要説明/Status・PRの区別・基準版の規則選択を照合。教材4テスト成功。全KIPの実装/証明・全ネットワーク有効化は未検証 | `experiments/chapter-30-review-2026-11-01.md` |
| 35 | [第31章](part-05-advanced/chapter-31.md) | 主要文書・教材を確認 | KIP-10/17/20と基準版の内省/有効化経路・手数料を照合。独自支出教材11テスト成功。実Vault・全命令/現行本番有効化・SDK利用は未検証 | `experiments/chapter-31-review-2026-11-01.md` |
| 36 | [第32章](part-05-advanced/chapter-32.md) | 主要文書・教材を確認 | Silverscript v1.0.0のCLIビルド/コンパイル・生成物一致/拒否を確認。独自承認教材10テスト成功。追加で2-of-2のローカルVM5件・ABI拒否1件を確認（2026-11-02）。本番/Devnet適用は未検証 | `experiments/chapter-32-review-2026-11-01.md`、`experiments/silverscript-vm-review-2026-11-02.md` |
| 37 | [Argent補足](part-05-advanced/argent.md) | 主要文書・ローカル例を確認 | 固定CLIで本文例build/inspect、Counter/2 Actorデモ・不正次状態の拒否、公式runtime単体1件を確認。ICC全体/Genesis・実ノード/E2E・署名経路の監査は未実施 | `experiments/argent-review-2026-11-01.md` |
| 38 | [第33章](part-05-advanced/chapter-33.md) | 主要資料・教材を確認 | Rollup/外部DA、公開入力・Replay/成否不明の境界を整理。状態遷移9テスト・API型検査成功。実ブリッジ/回路/本番有効化・DA/退出は未検証 | `experiments/chapter-33-review-2026-11-01.md` |
| 39 | [第34章](part-05-advanced/chapter-34.md) | 主要資料・教材を確認 | 公開案内と固定プロトタイプ/APIの境界・61パッケージmetadataを確認。独自教材9テスト成功。公式全テスト/証明・実アプリ間原子性・本番動作は未検証 | `experiments/chapter-34-review-2026-11-01.md` |
| 40 | [第35章](part-06-contributing/chapter-35.md) | 主要入口・教材を確認 | 基準版RPC/UTXO境界を照合。独自集計12テストとAPI型検査・SQLite制約検査成功。実Payment Tracker/E2E・会計台帳/複数ワーカーは未検証 | `experiments/chapter-35-review-2026-11-01.md` |
| 41 | [第36章](part-06-contributing/chapter-36.md) | 主要手順・教材を確認 | 基準版README/check/test/CI・metadata59パッケージ、公開貢献手順を確認。独自教材9テスト成功。公式スクリプト/CI全体・実PR提出は未実施 | `experiments/chapter-36-review-2026-11-01.md` |
| 42 | [付録A](appendices/appendix-a.md) | 主要手順・教材を確認 | Rust最小例2テスト・実行/書式/Clippy、Compose5構成とDocker版/ARM64 imageの読取を確認。既存Devnet記録を照合。新規導入/他OS・今回の実ノード起動・Pruningは未実施 | `experiments/appendix-a-review-2026-11-01.md` |
| 43 | [付録B](appendices/appendix-b.md) | 主要用語を照合 | 第12/13/16/17/20章等と基準版GHOSTDAG・TXIDの主要定義を確認。用語全体の完全監査や機能の有効化確認ではない | `experiments/appendix-b-review-2026-11-01.md` |
| 44 | [付録C](appendices/appendix-c.md) | 主要モデル・教材を確認 | Rust確率例7テスト・DAG例実行/書式/Clippy、数値例・集合/矢印規約・PoW比較を確認。暗号の安全性証明・公式合意の再検証ではない | `experiments/appendix-c-review-2026-11-01.md` |
| 45 | [付録D](appendices/appendix-d.md) | 主要API・設計と教材を確認 | 基準版router/service・通知型/CLI入口、既存実測との対応・Rust3テストを確認。全RPC/CLIの要求・応答・エラー/実ノード再実行は未確認 | `experiments/appendix-d-review-2026-11-01.md` |
| 46 | [付録E](appendices/appendix-e.md) | 主要定義・教材を確認 | 基準版のCrescendo前後値/導出式・Mass主要検証・定数10件/算術・Rust4テストを確認。現行Mainnet実測・全境界値・公式関数の再実行比較は未実施 | `experiments/appendix-e-review-2026-11-01.md` |
| 47 | [付録F](appendices/appendix-f.md) | 主要対応・教材を確認 | 固定KIP索引15行/主要metadata・固定版Params/探索先とRust4テストを確認。全論文/KIP仕様・実装・現行ネットワークの全面照合ではない | `experiments/appendix-f-review-2026-11-01.md` |
| 48 | [付録G](appendices/appendix-g.md) | 主要記録・教材を確認 | G-001〜004を今回のSHA2値/UTXO8テスト/固定DAGで確認。G-005〜010は既存記録と限界を対応付け、Pruning等の未完了を維持。実ノードや公式比較は再実行していない | `experiments/appendix-g-review-2026-11-01.md` |
| 49 | [付録H](appendices/appendix-h.md) | 主要書誌・管理を確認 | 主要論文の公開書誌/表紙・NIST公開情報、固定KIP表/Git履歴を確認。全PDF/引用節・暗号/ライセンス全面監査・最新ネットワークの確認ではない | `experiments/appendix-h-review-2026-11-01.md` |



ページを通読したことと、関連する全技術・全コードを検証したことは別です。

Part I〜VIと付録の扉ページ7件を、各部の概要・前提知識・章への案内として追加しました。付録の扉は目的別の参照先を示します。表紙と「おわりに」を含め、現在の目次は計58ページです。上表の49件は既存ページのレビュー記録であり、扉ページの追加で個別の技術検証の範囲は変わりません。

## 判定の読み方

- **ページの確認**：目次順で通読し、説明・案内・根拠との対応を確認した範囲です。
- **資料・実装との照合**：固定ソースや論文を確認した範囲です。上表の「未照合」は、このページの資料照合をまだ進めていない状態を表し、個別の実行記録があれば右欄に記載します。
- **実行検証**：記録にある版・入力・環境で実行した範囲です。教材のテスト、公式関数の比較、Dockerノードの実測は分けて読みます。

`Source-checked` は、左欄に示す範囲のソース照合を表します。実行済みの個別検査があっても、その結果は右欄に限定して記録し、章全体の `Verified` へは広げません。個々の主張の判定基準は[付録H](appendices/appendix-h.md)を参照してください。

## 主な検証結果と限界

第35章35.11.1には、公式SDKから取得した現在UTXOをJSON経由でRustの集計モデルへ渡す実行例を追加した（2026-11-02）。実Devnetの14件をCandidateとして取り込み、Observed・集計対象額0になることを確認。ドメイン12テストと変換7テストも成功した。履歴・確認ポリシー・DBを含む完成Payment Trackerの実装ではない。記録は`experiments/payment-candidates-review-2026-11-02.md`。

2026-11-02に[第28章28.6.4](part-04-development/chapter-28.md#2864-公式sdkを使った最小監視サンプル)へ公式SDKの監視サンプルを追加した。Node.js/WASM SDK v1.0.1で2セッションの接続・情報取得・UTXO通知・切断と再接続・再購読・再照会を実測した。記録は`experiments/sdk-monitor-review-2026-11-02.md`。意図的な切断を使う読み取り専用デモであり、Rustアダプター・自動障害復旧・完成Payment Trackerの検証とは別である。

以下は既存の記録を整理したものです。今回のページレビューで、プロトコルやDevnetの実験を新たに実行したわけではありません。詳細な入力・失敗試行・環境・再実行手順は、後半の記録一覧を参照してください。

| 対象 | 確認した範囲 | 主な限界・対象外 |
|---|---|---|
| 教育用Rust教材 | DAG解析器8テスト（2026-11-01改訂後）、最小ノード16テスト（2026-11-01改訂後）、書式・Clippy。第3章SHA-256抜き出し教材は実行・書式・2テスト | Kaspa互換性や公式ノードの全規則ではありません。第3章教材は手動検査で、CI対象には追加していません |
| GHOSTDAG | 基準版への再照合、公式分類関数と集合モデルの24ケース比較 | 小規模な合成DAG。到達可能性は共通DFS。全DAG・論文の全証明は対象外 |
| DAA | 算術27ケース、Full窓120ケース、Sampled/Dual288ケースでモデルと一致 | GHOSTDAGメタデータを入力に使う条件付き比較。全ネットワーク・全境界は未検証 |
| ブロック本体検証 | 公式テスト2件で単独検査・BadMerkleRoot/親不足の扱いを確認 | 親不足等の回帰テストはPoWを無効化。全ブロック検証やPoW込みE2Eではありません |
| Devnetの起動・保持 | 起動・読み取りwRPC・採掘・再起動後のDAG/報酬UTXO保持 | 確認したmacOS/ARM64構成に限定 |
| 単一署名送金 | 送金・Mempool/virtual受理・入力消費・出力・手数料/お釣り、所定の拒否と成功応答喪失からの復旧 | gen1 HD復旧鍵での受取/変更からの送金2件は別実験で成功（2026-11-02）。全Script・ブロック包含の独立照会・不可逆な決済確定は未検証 |
| 2ノード・分断 | 伝播・片側停止復帰・両側採掘後の収束・報酬UTXOの消失 | 指定報酬先のUTXOを観測。全UTXO集合・競合送金の勝敗・会計処理は対象外 |
| 通知と観測ストア | 重複適用・購読中断・SQLite再読込・再照会補完・古い結果の拒否・観測UTXOの取り消し | 単一書き込み主体の現在UTXO観測。複数ワーカーや会計台帳ではありません |
| Pruning前提 | 固定ソースの条件と、実測pointがGenesisのままであることを確認 | Pruningは未達。実際のデータ削減・Pruning後IBDは未検証 |

ソースをビルドした関数比較と、由来の完全一致を確認していないDockerバイナリの実測は別の証拠です。版・イメージ・依存・確認環境は[対象バージョン](baseline.md)へ集約しています。

## 検証記録の場所

検証記録はリポジトリの `experiments/` にあります。下記のファイル名は同ディレクトリからの相対パスです。記録中の「未実施」は、その実験時点の範囲を示す場合があります。後続検査がある項目は、本ページと記録の追記を併せて確認してください。

| 対象 | 記録ファイル |
|---|---|
| 第29章本文レビューと研究の前提/数値教材 | `chapter-29-review-2026-11-01.md` |
| 第30章本文レビューと固定KIP/規則選択教材 | `chapter-30-review-2026-11-01.md` |
| 第31章本文レビューとCovenantの境界/支出教材 | `chapter-31-review-2026-11-01.md` |
| 第32章本文レビューと固定Silverscript/承認教材 | `chapter-32-review-2026-11-01.md` |
| 第33章本文レビューとL2/DA/Replayの前提・状態遷移教材 | `chapter-33-review-2026-11-01.md` |
| 第34章本文レビューと固定vprogs/API・順序教材 | `chapter-34-review-2026-11-01.md` |
| 第35章本文レビューとRPC/集計教材・SQLite制約 | `chapter-35-review-2026-11-01.md` |
| Argent補足の本文レビューと固定CLI/ローカルruntime | `argent-review-2026-11-01.md` |
| 第36章本文レビューと開発用入口/不変条件教材 | `chapter-36-review-2026-11-01.md` |
| 付録A本文レビューとRust最小例/Compose設定・環境読取 | `appendix-a-review-2026-11-01.md` |
| 付録B本文レビューと集合/score・TXID等の用語照合 | `appendix-b-review-2026-11-01.md` |
| 付録C本文レビューと数値例/Rust確率・DAG教材 | `appendix-c-review-2026-11-01.md` |
| 付録D本文レビューと主要API/再照会・Rust観測検証教材 | `appendix-d-review-2026-11-01.md` |
| 付録E本文レビューと固定パラメータ/導出式・単位教材 | `appendix-e-review-2026-11-01.md` |
| 付録F本文レビューと固定KIP索引/記録不足チェック教材 | `appendix-f-review-2026-11-01.md` |
| 付録G本文レビューと実験索引/SHA・DAG・UTXO教材 | `appendix-g-review-2026-11-01.md` |
| 付録H本文レビューと主要書誌/固定版・Git履歴/管理基準 | `appendix-h-review-2026-11-01.md` |
| 第28章本文レビューとDevnet設定/教材 | `chapter-28-review-2026-11-01.md` |
| 第27章本文レビューと改訂最小ノード | `chapter-27-review-2026-11-01.md` |
| 第26章本文レビューとGHOSTDAG/保存境界 | `chapter-26-review-2026-11-01.md` |
| 第25章本文レビューとコード探索/DAG教材 | `chapter-25-review-2026-11-01.md` |
| 公式アカウント探索APIとgap limitの境界 | `hd-discovery-review-2026-11-02.md`、`hd-recovery/discovery.cjs` |
| HD復旧受取/変更鍵によるDevnet送金 | `hd-send-review-2026-11-02.md`、`hd-recovery/send.cjs` |
| gen1のHD鍵・アドレス復旧とUnicode境界 | `hd-recovery-review-2026-11-02.md`、`hd-recovery/` |
| 第24章本文レビューと入力予約教材 | `chapter-24-review-2026-11-01.md` |
| 第23章本文レビューとSDK/WASM境界教材 | `chapter-23-review-2026-11-01.md` |
| 第22章本文レビューとRPCアダプター教材 | `chapter-22-review-2026-11-01.md` |
| 第21章本文レビューとスナップショット教材 | `chapter-21-review-2026-11-01.md` |
| Finality pointの境界・連続計算 | `finality-comparison-2026-11-02.md`、`finality/` |
| 第20章本文レビューと確認ポリシー教材 | `chapter-20-review-2026-11-01.md` |
| 第19章本文レビューと経済モデル教材 | `chapter-19-review-2026-11-01.md` |
| 第18章本文レビューとテンプレート教材 | `chapter-18-review-2026-11-01.md` |
| 第17章本文レビューとMempool教材 | `chapter-17-review-2026-11-01.md` |
| 第16章本文レビューと署名検証境界の教材 | `chapter-16-review-2026-11-01.md` |
| 第15章本文レビューとUTXO・手数料教材 | `chapter-15-review-2026-11-01.md` |
| 第14章本文レビューとUTXO教材 | `chapter-14-review-2026-11-01.md` |
| 第13章本文レビューと簡易DAA教材 | `chapter-13-review-2026-11-01.md` |
| 第12章本文レビューと改訂DAG解析器 | `chapter-12-review-2026-11-01.md` |
| 第11章本文レビューと改訂教材 | `chapter-11-review-2026-11-01.md` |
| 第10章P2P概説と伝播教材 | `chapter-10-review-2026-11-01.md` |
| 第9章ノード構造とDAG教材 | `chapter-09-review-2026-10-31.md` |
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

教材コードは `examples/`、公式関数の比較ハーネスは `experiments/ghostdag/` と `experiments/daa*/`、実ノードの検査は `experiments/devnet/` を参照してください。付録GのG-001〜010は、今回の最小例と過去の実験記録を対応付けています。限定した成功と未完了を明記し、実験テーマ全体の完了へ広げていません。

## CIの設定と実行確認

`.github/workflows/deploy.yml` には、次の検査を設定しています。

- mdBookのHTMLビルド。
- ビルドしたHTML内の内部リンク・アンカーと画像等のローカル参照（`scripts/check-book-links.py`）。外部URLへのHTTP接続は行いません。
- 第12・27・35章の収録済みRust教材3件（`dag-relations`・`minimal-node`・`payment-candidates`）の書式・Clippy・テスト。
- `scripts/check-book-examples.py` による、3教材の本文と実装の一致検査。第35章は集計モデルとtrait定義が対象で、JSON変換処理全体を本文と比較するものではありません。
- `experiments/devnet/test_utxo_store.py` のSQLite観測ストア7テスト。

公式GHOSTDAG/DAAの比較、Dockerを使うDevnet試験、他章の抜き出しRustコードの検査は、このworkflowでは実行しません。設定内容とローカルの成功記録を、GitHub上の実ジョブの成功へ読み替えないでください。GitHub上のCI・Pagesを含む公開確認は、著者から完了済みと確認されました。[公開サイト](https://shiga0.github.io/mastering-kaspa/)で読めます。今回の文書更新で実ジョブや公開操作を再実行したものではありません。

## 目次順レビューの区切りと残作業

導入4ページ・第1〜36章・[Argent補足](part-05-advanced/argent.md)・付録A〜Hの計49ページを、SUMMARY順に通読・修正しました。この49ページのレビューが一巡したことは、各ページ内の全主張・実装/暗号方式・実ネットワークの検証が完了した意味ではありません。付録A〜Hと進捗/ROADMAP・個別記録はコミット `c86e88e` に保存済みです。

初版では、ここまでのレビュー・個別実験を記載した範囲で完了として閉じます。CI・Pagesを含む公開確認は完了済みです。ライセンス・貢献方針も決定・反映済みです。編集は既存の49ページのレビューで一区切りとし、一律の追加表示点検は著者との合意で必須作業から外しました。今回合意した仕上げ作業は完了です。リポジトリの `ROADMAP.md` に完了条件をまとめています。

FinalityのVirtualProcessor/Pruning検査経路、実ノード/IndexerによるHD履歴探索・暗号化バックアップ復旧、Pruning後IBD、複数ワーカー/会計、他環境での再現は、既知の制限・将来課題として残します。このページの「未検証」「残作業」は技術的な確認範囲を示し、初版までに追加実験を行う指示ではありません。新たな証拠なしにVerifiedへ変更しません。

2026年11月2日：同一macOS/ARM64ホスト上でPython/SDKとノードDBを新規作成し、初期0ブロックから採掘・送金/主要異常系・virtual受理・再起動保持・停止を再確認しました。165ブロックと報酬UTXO 40件が再起動前後で一致し、送金先UTXOも保持。新規OS/Docker導入や他環境の確認は残ります。[検証記録](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/devnet-fresh-review-2026-11-02.md)を参照してください。

2026年11月2日：固定版finality point関数の25ケース・112回を別モデルと実行比較し、全件一致しました。公式Reachabilityを使用し、depth/Pruning距離・選択親DAAのactivation境界・depth増減時の連続計算を含みます。VirtualProcessor・Pruning後IBD・安全性証明の完了には広げません。

2026年11月2日：公式v1.0.1 SDKで160個のgen1鍵/アドレスを独立プロセス間で再構成し、明示パス・アカウントxpub・メッセージ署名を照合。Node cryptoのPBKDF2と比較し、追加パスフレーズのNFKD正規化が自動では行われない差を再現しました。実ノードでの探索や復旧鍵の送金とは分けて記録しています。

2026年11月2日：gen1 HD復旧受取鍵で1 KASを送り、さらに新しいプロセスで復旧した変更鍵で第一TXのお釣りから1 KASを送金。2件とも署名/未署名項目照合、Mempool・virtual受理・入力消費・宛先/変更UTXOと手数料を確認しました。未知index/履歴探索・暗号化バックアップ復旧は残ります。

2026年11月2日：公式Wallet.accountsDiscoveryの固定RPC14ケースで、両枝の範囲・account継続条件・戻り値0の曖昧さ・現在UTXOだけでは支出済み履歴を拾えない条件を確認。別のgap教材は12,288比較で一致しました。実ノード/Indexerの履歴復元や暗号化バックアップは未検証です。

## 各ページから集約したレビュー記録

章の冒頭にあったレビュー日・作業経緯をここへ移しました。以下は元の記録であり、2026年11月7日に再検証した結果ではありません。対象版の共通条件は[本書の対象バージョン](baseline.md)、章固有の確認範囲は各ページ末尾の「検証状況を見る」に記載しています。

<a id="review-appendix-a"></a>

<details class="verification-details">
<summary>Appendix A — Rust環境・Docker環境の構築のレビュー記録</summary>

[本文と検証状況](appendices/appendix-a.md)

> **目次順の本文レビュー**：2026-11-01。独自Rust教材・Compose設定・環境の読取検査と既存実測記録を照合。全OSの新規導入/Devnet再実行ではない。

対象版の一覧は[本書の対象バージョン](baseline.md)、この回の結果は `experiments/appendix-a-review-2026-11-01.md` を参照。既存のDevnet実測と、今回新しく確認した事項を分けて記録する。

</details>

<a id="review-appendix-b"></a>

<details class="verification-details">
<summary>Appendix B — Kaspa用語集のレビュー記録</summary>

[本文と検証状況](appendices/appendix-b.md)

> 目次順の本文レビュー：2026-11-01。主要用語を既存章と基準版ソースに照合。全用語・全機能の検証ではない。\

> **編集メモ**：本付録の定義は学習用に簡潔化している。特に数理的に厳密な定義が必要な場合は、該当章と論文・実装を優先する。出版前に用語の表記揺れ、KIPの状態、実装の変更点を再確認すること。

</details>

<a id="review-appendix-c"></a>

<details class="verification-details">
<summary>Appendix C — 数学・確率・暗号技術の補足のレビュー記録</summary>

[本文と検証状況](appendices/appendix-c.md)

> 目次順の本文レビュー：2026-11-01。数式の前提・独自Rust教材を確認。暗号方式の安全性証明や公式合意の再検証ではない。\

対象版は[本書の対象バージョン](baseline.md)、今回の確認結果は `experiments/appendix-c-review-2026-11-01.md` を参照。

</details>

<a id="review-appendix-d"></a>

<details class="verification-details">
<summary>Appendix D — RPC・CLIリファレンスのレビュー記録</summary>

[本文と検証状況](appendices/appendix-d.md)

> 目次順の本文レビュー：2026-11-01。主要RPC/通知・CLIの入口・独自Rust教材を確認。\

対象版は[本書の対象バージョン](baseline.md)、今回の確認結果は `experiments/appendix-d-review-2026-11-01.md` を参照。以下の基準版はrusty-kaspa v1.0.1。

</details>

<a id="review-appendix-e"></a>

<details class="verification-details">
<summary>Appendix E — プロトコルパラメータ一覧のレビュー記録</summary>

[本文と検証状況](appendices/appendix-e.md)

> 目次順の本文レビュー：2026-11-01。基準版の主要定義/利用箇所と単位教材を確認。\

対象版は[本書の対象バージョン](baseline.md)、この回の結果は `experiments/appendix-e-review-2026-11-01.md` を参照。旧稿の別版master由来のフィールド/値を基準版へ整理した。

</details>

<a id="review-appendix-f"></a>

<details class="verification-details">
<summary>Appendix F — 論文・KIP・実装対応表のレビュー記録</summary>

[本文と検証状況](appendices/appendix-f.md)

> 目次順の本文レビュー：2026-11-01。固定KIPの主要metadata/参照先・記録教材を確認。\

確認結果は `experiments/appendix-f-review-2026-11-01.md`。今回全KIP本文・論文の証明・実装・ネットワークを全面照合したものではない。

</details>

<a id="review-appendix-g"></a>

<details class="verification-details">
<summary>Appendix G — 実験結果・テストベクトルのレビュー記録</summary>

[本文と検証状況](appendices/appendix-g.md)

> 目次順の本文レビュー：2026-11-01。実験索引/記録・独自UTXO教材・SHA/DAG例を確認。\

対象版は[本書の対象バージョン](baseline.md)、今回の結果は `experiments/appendix-g-review-2026-11-01.md` を参照。実ノードの起動・採掘・送金は今回再実行していない。

</details>

<a id="review-appendix-h"></a>

<details class="verification-details">
<summary>Appendix H — 参考文献・変更履歴のレビュー記録</summary>

[本文と検証状況](appendices/appendix-h.md)

> 目次順の本文レビュー：2026-11-01。主要書誌・参照版・Git履歴/検証索引を確認。\

基準版は[本書の対象バージョン](baseline.md)、ページごとの範囲は[技術レビューの進捗](review-status.md)、今回の記録は `experiments/appendix-h-review-2026-11-01.md` を参照。目次順の通読と、全主張・全実装の検証完了は区別する。

</details>

<a id="review-chapter-01"></a>

<details class="verification-details">
<summary>Chapter 01 — Introduction to Kaspa（Kaspa入門）のレビュー記録</summary>

[本文と検証状況](part-01-foundations/chapter-01.md)

> 入門概説・基準版との対応を確認：2026-10-31。対象は[本書の基準版](baseline.md)を参照。

</details>

<a id="review-chapter-02"></a>

<details class="verification-details">
<summary>Chapter 02 — Foundations of Digital Money（デジタルマネーの基礎）のレビュー記録</summary>

[本文と検証状況](part-01-foundations/chapter-02.md)

> 基礎概説・教材の説明を確認：2026-10-31。対象は[本書の基準版](baseline.md)を参照。

</details>

<a id="review-chapter-03"></a>

<details class="verification-details">
<summary>Chapter 03 — Cryptographic Foundations（暗号技術の基礎）のレビュー記録</summary>

[本文と検証状況](part-01-foundations/chapter-03.md)

> 基礎概説・SHA-256教材を確認：2026-10-31。対象は[本書の基準版](baseline.md)を参照。

</details>

<a id="review-chapter-04"></a>

<details class="verification-details">
<summary>Chapter 04 — Distributed Systems & P2P（分散システムとP2Pネットワーク）のレビュー記録</summary>

[本文と検証状況](part-01-foundations/chapter-04.md)

> 基礎概説・伝播教材を確認：2026-10-31。対象は[本書の基準版](baseline.md)を参照。

</details>

<a id="review-chapter-05"></a>

<details class="verification-details">
<summary>Chapter 05 — Bitcoin Architecture（Bitcoinのアーキテクチャ）のレビュー記録</summary>

[本文と検証状況](part-01-foundations/chapter-05.md)

> 基礎概説・UTXO教材を確認：2026-10-31。Kaspa側は[本書の基準版](baseline.md)を参照。

</details>

<a id="review-chapter-06"></a>

<details class="verification-details">
<summary>Chapter 06 — Blockchain Limitations（ブロックチェーンの限界）のレビュー記録</summary>

[本文と検証状況](part-01-foundations/chapter-06.md)

> 基礎概説・確率計算教材を確認：2026-10-31。Kaspa側は[本書の基準版](baseline.md)を参照。

</details>

<a id="review-chapter-07"></a>

<details class="verification-details">
<summary>Chapter 07 — Introduction to BlockDAG（BlockDAG入門）のレビュー記録</summary>

[本文と検証状況](part-01-foundations/chapter-07.md)

> 基礎概説・DAG教材を確認：2026-10-31。対象は[本書の基準版](baseline.md)を参照。

</details>

<a id="review-chapter-08"></a>

<details class="verification-details">
<summary>Chapter 08 — From Blockchain to Kaspa（ブロックチェーンからKaspaへ）のレビュー記録</summary>

[本文と検証状況](part-01-foundations/chapter-08.md)

> 基礎概説・後続章への対応を確認：2026-10-31。対象は[本書の基準版](baseline.md)を参照。

</details>

<a id="review-chapter-09"></a>

<details class="verification-details">
<summary>Chapter 09 — Kaspa Architecture（Kaspaのアーキテクチャ）のレビュー記録</summary>

[本文と検証状況](part-02-consensus/chapter-09.md)

> 構造・基準版との対応を確認：2026-10-31。参照版は[本書の基準版](baseline.md)のv1.0.1、コミット `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。

</details>

<a id="review-chapter-10"></a>

<details class="verification-details">
<summary>Chapter 10 — Kaspa Node & P2P Network（KaspaノードとP2Pネットワーク）のレビュー記録</summary>

[本文と検証状況](part-02-consensus/chapter-10.md)

> 概説・基準版の主要経路を確認：2026-11-01。参照版は[本書の基準版](baseline.md)のv1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。

</details>

<a id="review-chapter-11"></a>

<details class="verification-details">
<summary>Chapter 11 — Block Processing & Validation（ブロック処理と検証）のレビュー記録</summary>

[本文と検証状況](part-02-consensus/chapter-11.md)

> 目次順の本文レビュー：2026-11-01。主要経路は[基準版](baseline.md)v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` と照合。全規則・全経路の検証完了を意味しない。

</details>

<a id="review-chapter-12"></a>

<details class="verification-details">
<summary>Chapter 12 — GHOSTDAG Internals（GHOSTDAGの内部構造）のレビュー記録</summary>

[本文と検証状況](part-02-consensus/chapter-12.md)

> 目次順の本文レビュー：2026-11-01。主要規則は基準版と照合済み。ページの通読と全経路の検証は区別する。

> 主要なGHOSTDAG経路を[本書の基準版](baseline.md)v1.0.1で再照合した。小規模DAGの24ケースで公式実装との実行時差分を比較した。本文を目次順に確認し、説明と教材を修正した。全DAG・全規則を検証したものではない。

</details>

<a id="review-chapter-13"></a>

<details class="verification-details">
<summary>Chapter 13 — Difficulty Adjustment Algorithm（難易度調整アルゴリズム）のレビュー記録</summary>

[本文と検証状況](part-02-consensus/chapter-13.md)

> 目次順の本文レビュー：2026-11-01。基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` の主要経路と照合。全規則・全経路の検証完了を意味しない。\

</details>

<a id="review-chapter-14"></a>

<details class="verification-details">
<summary>Chapter 14 — Consensus Ordering & Virtual State（コンセンサス順序と仮想状態）のレビュー記録</summary>

[本文と検証状況](part-02-consensus/chapter-14.md)

> 目次順の本文レビュー：2026-11-01。主要経路は基準版と照合。ページの確認と全規則・全経路の検証は区別する。\

</details>

<a id="review-chapter-15"></a>

<details class="verification-details">
<summary>Chapter 15 — UTXO Model & Transaction Processing（UTXOモデルとトランザクション処理）のレビュー記録</summary>

[本文と検証状況](part-03-transactions/chapter-15.md)

> 目次順の本文レビュー：2026-11-01。主要経路は基準版と照合。ページの確認と全規則・全経路の検証は区別する。\

</details>

<a id="review-chapter-16"></a>

<details class="verification-details">
<summary>Chapter 16 — Kaspa Script & Transaction Signing（Kaspa Scriptとトランザクション署名）のレビュー記録</summary>

[本文と検証状況](part-03-transactions/chapter-16.md)

> 目次順の本文レビュー：2026-11-01。主要経路は基準版と照合。ページの確認と全規則・全経路の検証は区別する。\

</details>

<a id="review-chapter-17"></a>

<details class="verification-details">
<summary>Chapter 17 — Mempool & Transaction Propagation（Mempoolとトランザクション伝播）のレビュー記録</summary>

[本文と検証状況](part-03-transactions/chapter-17.md)

> 目次順の本文レビュー：2026-11-01。基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` の主要な受付・保留・伝播経路を照合。全規則・全経路の検証ではない。\

固定ソースの確認と教材の実行記録は `experiments/chapter-17-review-2026-11-01.md`。既存DevnetのMempool照会・競合拒否は `experiments/devnet-send-checks-2026-10-31.md`、応答喪失からの復旧は `experiments/devnet-send-retry-2026-10-31.md` を参照する。今回はノード・送金を再実行していない。

</details>

<a id="review-chapter-18"></a>

<details class="verification-details">
<summary>Chapter 18 — Mining & Block Template Construction（マイニングとブロックテンプレート構築）のレビュー記録</summary>

[本文と検証状況](part-03-transactions/chapter-18.md)

> 目次順の本文レビュー：2026-11-01。基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` の主要なテンプレート生成経路と照合。全規則・全経路の検証ではない。\

固定ソースの照合と教材の記録は `experiments/chapter-18-review-2026-11-01.md`。既存のDevnet実験とは分け、今回はノード起動・採掘・送金を実行していない。

</details>

<a id="review-chapter-19"></a>

<details class="verification-details">
<summary>Chapter 19 — Mining Economics & Network Security（マイニング経済とネットワークセキュリティ）のレビュー記録</summary>

[本文と検証状況](part-03-transactions/chapter-19.md)

> 目次順の本文レビュー：2026-11-01。報酬の主要処理は基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` と照合。経済モデルは仮定値による教材で、全規則・全市場データの検証ではない。\

確認範囲・教材の実行記録は `experiments/chapter-19-review-2026-11-01.md`。価格・機器・電力契約・現行Mainnetの報酬やハッシュレートを今回の実測値として扱わない。

</details>

<a id="review-chapter-20"></a>

<details class="verification-details">
<summary>Chapter 20 — Kaspa Network Security & Finality（Kaspaのネットワークセキュリティとファイナリティ）のレビュー記録</summary>

[本文と検証状況](part-03-transactions/chapter-20.md)

> 目次順の本文レビュー：2026-11-01。Finalityの主要経路は基準版と照合。ページの確認と全規則・全経路の検証は区別する。\

</details>

<a id="review-chapter-21"></a>

<details class="verification-details">
<summary>Chapter 21 — Pruning, Storage & Node Synchronization（Pruning・ストレージ・ノード同期）のレビュー記録</summary>

[本文と検証状況](part-04-development/chapter-21.md)

> 目次順の本文レビュー：2026-11-01。主要経路は基準版と照合。本文の確認と実Pruning・Pruning後同期の検証は区別する。\

</details>

<a id="review-chapter-22"></a>

<details class="verification-details">
<summary>Chapter 22 — Kaspa RPC, wRPC & Application Integration（Kaspa RPC・wRPCとアプリケーション統合）のレビュー記録</summary>

[本文と検証状況](part-04-development/chapter-22.md)

> 目次順の本文レビュー：2026-11-01。基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` の主要APIと照合。全API・全障害系の検証ではない。\

本文と教材の確認記録は `experiments/chapter-22-review-2026-11-01.md`。既存のDevnet実測と今回のモック教材の実行を分けて扱う。

</details>

<a id="review-chapter-23"></a>

<details class="verification-details">
<summary>Chapter 23 — Rust SDK & WASM（Rust SDKとWASM）のレビュー記録</summary>

[本文と検証状況](part-04-development/chapter-23.md)

> 目次順の本文レビュー：2026-11-01。基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。主要バインディングの照合と教材の確認であり、SDK全体の検証ではない。

確認記録は `experiments/chapter-23-review-2026-11-01.md`。公式配布SDKを使った過去のDevnet実測と、本章の独自教材の実行を区別する。

</details>

<a id="review-chapter-24"></a>

<details class="verification-details">
<summary>Chapter 24 — Wallet Architecture & Key Management（ウォレットアーキテクチャと鍵管理）のレビュー記録</summary>

[本文と検証状況](part-04-development/chapter-24.md)

> 目次順の本文レビュー：2026-11-01。主要経路を基準版と再照合し、入力予約教材を実行。全ウォレット・鍵保護の監査ではない。\

</details>

<a id="review-chapter-25"></a>

<details class="verification-details">
<summary>Chapter 25 — Exploring the rusty-kaspa Codebase（rusty-kaspaのコードベースを読む）のレビュー記録</summary>

[本文と検証状況](part-04-development/chapter-25.md)

> **目次順の本文レビュー:** 2026-11-01。基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。主要入口・metadata・教材の確認。\

確認記録は `experiments/chapter-25-review-2026-11-01.md`。全ソース・全テストの検証ではなく、既存の公式関数の実行記録とも分けて読む。

</details>

<a id="review-chapter-26"></a>

<details class="verification-details">
<summary>Chapter 26 — Consensus Implementation in Rust（Rustによるコンセンサス実装）のレビュー記録</summary>

[本文と検証状況](part-04-development/chapter-26.md)

> **目次順の本文レビュー:** 2026-11-01。主要経路・保存境界・教材の確認。全コンセンサスの検証ではない。\

> 主要なGHOSTDAG経路を[本書の基準版](baseline.md)v1.0.1で再照合した。小規模DAGの24ケースで公式実装との実行時差分を比較した。今回、本文全体を目次順で確認し、ヘッダー保存とVirtual更新の境界を補足した。過去の公式差分比較を今回再実行した意味ではない。記録は `experiments/chapter-26-review-2026-11-01.md`。

</details>

<a id="review-chapter-27"></a>

<details class="verification-details">
<summary>Chapter 27 — Building a Minimal Kaspa-like Node in Rust（Rustによる最小Kaspa型ノードの構築）のレビュー記録</summary>

[本文と検証状況](part-04-development/chapter-27.md)

> **目次順の本文レビュー:** 2026-11-01。教材のID検査・状態保持・ノード間の関数呼び出しを確認。Kaspa互換性の検証ではない。\

今回の記録は `experiments/chapter-27-review-2026-11-01.md`。2026-10-31の6テストの記録と、今回の改訂後16テストを区別する。

</details>

<a id="review-chapter-28"></a>

<details class="verification-details">
<summary>Chapter 28 — Devnet & Integration Testing（Devnetと統合テスト）のレビュー記録</summary>

[本文と検証状況](part-04-development/chapter-28.md)

> 目次順の本文レビュー：2026-11-01。設定・手順・既存実測と独自教材を確認。全実ノード試験の再実行ではない。\

今回の記録は `experiments/chapter-28-review-2026-11-01.md`。過去の実ノード実測、今回の設定検査・単体テスト、未実施の障害注入を分ける。

</details>

<a id="review-argent"></a>

<details class="verification-details">
<summary>Argent — KaspaのActorベース・アプリケーション言語のレビュー記録</summary>

[本文と検証状況](part-05-advanced/argent.md)

Part Vに置く独立した解説ページ。Argentは `2796809f167144902f04a4b3333a4c144f91fdaf`、Playgroundは `86d5453ee525e7aa743d0b3bb21d1bb8d96c3558` を取得して確認した。本書の基準ノードv1.0.1および第32章のSilverScript v1.0.0とは別の参照軸である。確認記録は `experiments/argent-review-2026-11-01.md`。

</details>

<a id="review-chapter-29"></a>

<details class="verification-details">
<summary>Chapter 29 — GHOSTDAG Security & DAGKnight（GHOSTDAGのセキュリティとDAGKnight）のレビュー記録</summary>

[本文と検証状況](part-05-advanced/chapter-29.md)

> **目次順の本文レビュー：2026-11-01。公開要旨・書誌、固定版の主要規則、確率教材を確認。安全性証明の全面照合ではない。**\

確認記録は `experiments/chapter-29-review-2026-11-01.md`。研究上の主張、基準版v1.0.1の実装、現行ネットワークでの有効化を分ける。

</details>

<a id="review-chapter-30"></a>

<details class="verification-details">
<summary>Chapter 30 — Kaspa Improvement Proposals（Kaspa改善提案・KIP）のレビュー記録</summary>

[本文と検証状況](part-05-advanced/chapter-30.md)

> **目次順の本文レビュー：2026-11-01。固定KIP文書・主要出典と規則選択教材を確認。全実装/本番有効化の照合ではない。**

確認記録は `experiments/chapter-30-review-2026-11-01.md`。文書上のStatusと、対応実装・リリース・ネットワーク有効化の証拠を分ける。

</details>

<a id="review-chapter-31"></a>

<details class="verification-details">
<summary>Chapter 31 — Covenants & Advanced Scripting（Covenantsと高度なスクリプティング）のレビュー記録</summary>

[本文と検証状況](part-05-advanced/chapter-31.md)

> **目次順の本文レビュー：2026-11-01。主要な固定KIP/基準版ソースと独自教材を確認。全仕様・本番有効化の検証ではない。**

基準版はv1.0.1 / `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`、KIP文書は第30章と同じ `e4ae2332117b5cb68bd6188e065ef885b6d17939`。両者は同時点の仕様集合ではない。確認記録は `experiments/chapter-31-review-2026-11-01.md`。

</details>

<a id="review-chapter-32"></a>

<details class="verification-details">
<summary>Chapter 32 — Silverscript & Smart Contract Design（Silverscriptとスマートコントラクト設計）のレビュー記録</summary>

[本文と検証状況](part-05-advanced/chapter-32.md)

> **目次順の本文レビュー：2026-11-01。固定Silverscript文書/CLIの主要箇所と独自Rust教材を確認。CLIコンパイルも確認。追加検証2026-11-02：2-of-2のローカルVM評価5件・ABI拒否1件を確認。本番利用は未検証。**

2026-11-03の追加確認では、32.15.2の返金・状態・系列制約まで検査した。既存2-of-2を含む30ケースの結果・識別値と、ネットワーク未検証の境界は[追加検証記録](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/programmability-review-2026-11-03.md)を参照する。

Silverscriptの参照版はv1.0.0 / `3ed973335b59269293564805cc2c58a14595ec03`。本書のノード基準版v1.0.1とは別の資料である。確認記録は `experiments/chapter-32-review-2026-11-01.md`。追加のVM検査は `experiments/silverscript-vm-review-2026-11-02.md`。

</details>

<a id="review-chapter-33"></a>

<details class="verification-details">
<summary>Chapter 33 — Layer 2, Bridges & Zero-Knowledge Proofsのレビュー記録</summary>

[本文と検証状況](part-05-advanced/chapter-33.md)

> 目次順の本文レビュー：2026-11-01。主要一次資料と独自状態遷移教材を確認。実ブリッジ・証明系・本番有効化の検証ではない。

KaspaのKIP参照版は第30〜32章と同じ `e4ae2332117b5cb68bd6188e065ef885b6d17939`、ノードの基準版はv1.0.1 / `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。両者は同時点の機能集合ではない。確認記録は `experiments/chapter-33-review-2026-11-01.md`。

</details>

<a id="review-chapter-34"></a>

<details class="verification-details">
<summary>Chapter 34 — vProgs & Programmability（vProgsとKaspaのプログラマビリティ）のレビュー記録</summary>

[本文と検証状況](part-05-advanced/chapter-34.md)

> **目次順の本文レビュー：2026-11-01。主要な公開設計・固定プロトタイプと独自Rust教材を確認。Full vProgsや本番動作の検証ではない。**

公開vprogsは `f9b84a863a7c7c20586a9cf947550475e894f72e` へ固定した。本書の基準ノードv1.0.1、固定KIP、Silverscriptと同一時点の機能集合ではない。確認記録は `experiments/chapter-34-review-2026-11-01.md`。

</details>

<a id="review-chapter-35"></a>

<details class="verification-details">
<summary>Chapter 35 — Designing and Partially Implementing a Kaspa Application（Kaspaアプリケーションの設計と部分実装）のレビュー記録</summary>

[本文と検証状況](part-06-contributing/chapter-35.md)

> **目次順の本文レビュー：2026-11-01。基準版の主要RPC入口・独自集計教材・SQLite制約を確認。完成アプリ/E2E・会計台帳の検証ではない。**

ノード参照版はv1.0.1 / `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。現行ドキュメントとAPIが異なる場合はこの固定版のソースを優先する。確認記録は `experiments/chapter-35-review-2026-11-01.md`。

</details>

<a id="review-chapter-36"></a>

<details class="verification-details">
<summary>Chapter 36 — Contributing to Kaspa（Kaspaへのコントリビュート）のレビュー記録</summary>

[本文と検証状況](part-06-contributing/chapter-36.md)

> **目次順の本文レビュー：2026-11-01。基準版の開発用入口・公開手順と独自Rust教材を確認。公式CI全体や実PR提出の検証ではない。**

ソース照合は本書の基準版v1.0.1 / `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。実際の上流への貢献では、その時点の指定base・開発ルールを別に確認する。確認記録は `experiments/chapter-36-review-2026-11-01.md`。

</details>
