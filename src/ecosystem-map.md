# Kaspa 技術・エコシステムマップ

KaspaのL1プロトコル、アプリケーション、開発ツール、外部の実行環境を区別するための案内です。分類と資料の説明を2026年10月31日に確認しました。各プロジェクトの稼働・互換性・安全性を実測したページではありません。

## 全体像

次の図は分野の分類です。線は分類上の枝を表し、通信経路・実装の依存関係・メインネットでの有効化を表しません。

```text
Kaspaに関係する分野
├─ L1プロトコルとノード
│  ├─ トランザクション：KAS / UTXO / Kaspa Script / 署名検証
│  ├─ コンセンサス：PoW / BlockDAG / GHOSTDAG
│  └─ ノード：P2P / 同期 / Mempool / ストレージ / rusty-kaspa
├─ アプリケーションの用途
│  └─ ウォレット / 決済 / エクスプローラー / インデクサー
├─ 開発ツールと接続手段
│  └─ RPC・wRPC / SDK / SilverScript / Argent
├─ 外部の実行環境と接続
│  └─ Igra / Kasplex / ブリッジ
├─ プログラマビリティ
│  └─ Covenants / Based Apps / Inline ZK / vProgs
└─ コンセンサス研究・改善提案
   └─ DAGKnight / KIPs
```

同じ技術が複数の分野に関係することもあります。例えばSilverScriptは開発ツールですが、生成するScriptが利用する機能はL1側の検証規則に依存します。ZKも証明技術であり、ZKを使うというだけで外部L2に分類されるわけではありません。

L1の説明は本書の[基準版 v1.0.1](baseline.md)を起点に読みます。以下で参照する開発プロジェクトや現在のドキュメントは別の版を対象とするため、基準版でそのまま利用できるとは限りません。

## 混同しやすい概念

| 項目 | 位置づけ・資料で確認した範囲 | 関連章 |
|---|---|---|
| **Kaspa Script** | UTXOの支出条件を検証するL1の仕組み。基準版の説明・実測範囲は本文と[レビュー進捗](review-status.md)を参照します | [第16章](part-03-transactions/chapter-16.md) |
| **Covenants** | 支出トランザクションの出力などに制約を設ける仕組み。[公式資料](https://docs.kaspa.org/programmability)ではL1の資産ルール・状態を持つ出力の選択肢として説明されています | [第31章](part-05-advanced/chapter-31.md) |
| **SilverScript** | Kaspa Scriptを生成する言語・コンパイラ。[v1.0.0の公開リリース](https://github.com/kaspanet/silverscript/releases/tag/v1.0.0)を確認しました | [第32章](part-05-advanced/chapter-32.md) |
| **Argent** | SilverScriptを基盤とするActorベースの言語・コンパイラ。[確認したREADME](https://github.com/argent-lang/argent/blob/2796809f167144902f04a4b3333a4c144f91fdaf/README.md)では開発中で、リリース準備は未完了とされています | [Argentの独立解説ページ](part-05-advanced/argent.md) |
| **Igra / Kasplex** | Kaspaに関連する外部実行環境。[Igraの資料](https://igra-labs.gitbook.io/igralabs-docs)はEVM互換を、[KasplexのL2資料](https://docs-kasplex.gitbook.io/l2-network)はL2としての位置づけを説明しています。構成・ブリッジ・信頼条件は個別に確認します | [第33章](part-05-advanced/chapter-33.md) |
| **Based Apps / Inline ZK** | [公式資料](https://docs.kaspa.org/programmability)が区別する実行モデルです。Based Appsは共有アカウント状態を扱い、Inline ZKは各アクションの証明を個別に検証・決済する方式として説明されています | [第34章](part-05-advanced/chapter-34.md) |
| **vProgs** | [公開リポジトリ](https://github.com/kaspanet/vprogs/blob/f9b84a863a7c7c20586a9cf947550475e894f72e/README.md)は初期開発・プロトタイプ段階と明記しています。一方、[公式資料](https://docs.kaspa.org/programmability)の **Full vProgs** は独立アプリ間の合成を目指す将来方向です。公開コードの存在と構想全体の完成は分けて読みます | [第34章](part-05-advanced/chapter-34.md) |
| **DAGKnight** | [論文の要旨](https://eprint.iacr.org/2022/1494)で説明されるコンセンサス研究です。論文の存在だけでは、特定ネットワークでの採用・有効化を判断できません | [第29章](part-05-advanced/chapter-29.md) |
| **KIPs** | [Kaspa Improvement Proposals](https://github.com/kaspanet/kips)で扱う改善提案です。提案文書、実装、有効化の状態は提案ごとに確認します | [第30章](part-05-advanced/chapter-30.md) |

ArgentのREADMEでは、`.ag` からSilverScriptの `.sil` と実行用の成果物を生成する関係が説明されています。Argentを外部L2として扱ったり、SilverScriptと独立したL1機能として数えたりしないようにします。

また、[今回確認したSilverScriptのREADME](https://github.com/kaspanet/silverscript/blob/15b7bc27680bdf0d83b294fbbfda55ecb59958ca/README.md)はKaspa依存に `rusty-kaspa 2.1.0` のクレートを使用すると記載しています。これは本書の基準版 v1.0.1 と異なります。リリースの公開や言語の存在だけで、基準版ノードとの互換性や任意のコントラクトの安全性を確認したことにはなりません。

このREADMEのコミットと、[第32章のv1.0.0に固定したVM実習](part-05-advanced/chapter-32.md#32151-同じソースを公式コンパイラーとscript-vmで検査する)も別の参照軸です。実習を再現するときは、章内のコンパイラー・依存Kaspaの完全SHAを使います。

## 用途から技術を選ぶ

最初に「資金の支出条件を定めたいのか」「複数の状態を協調して動かしたいのか」「別の実行環境が必要なのか」を分けます。下表は設計の比較であり、すべてが同じ成熟度・ネットワークで使えるという一覧ではありません。ToccataのL1機能と本書の固定Devnet、公開言語の版は[対象バージョン](baseline.md#リリース済み実装とネットワークの区別)で区別します。

| 選択肢 | 解決する問題・代表例 | 実行場所・状態 |
|---|---|---|
| Kaspa Script | 通常送金、署名・時間による支出条件 | ノードのL1 VMで支出条件を検証。状態の単位はUTXO |
| Covenants | 後継出力の制約、エスクロー、トークンの遷移 | L1 Scriptで入力・出力を検査。状態は支出されるUTXOから後継UTXOへ移す |
| SilverScript | 上の支出条件を読みやすく記述し、生成物を検査する | コンパイラーはローカルで動き、生成ScriptはL1で検証。言語自体が別の台帳を持つわけではない |
| Argent | 複数の状態付きUTXOを協調させる交換・チケット等 | ローカルのコンパイラーと取引構築ライブラリを使い、L1が生成Scriptを検証。Actorは状態付きUTXOで、常駐する非同期プロセスではない |
| Full vProgs | 独立アプリ間の合成を目指す研究 | アプリ側の実行と証明、L1との結び付きを設計する構想。公開プロトタイプと構想全体の完成は別 |
| 個別のL2 | EVM等の別の実行環境でアプリを動かす | 各L2のVM・状態。L1へのデータ公開・検証・資産移動は製品ごとに異なる |

| 選択肢 | 信頼・安全性の前提と運用上の制約 | 確認した実装・ネットワークの範囲 |
|---|---|---|
| Kaspa Script | 鍵、署名対象、参照UTXO、資源・手数料を検査 | 本書の単一署名実測はv1.0.1 Devnet。後続v1取引の差は第15・16章 |
| Covenants | IDの一致に加え、プログラム・出所・遷移・回復経路が必要 | Toccataの基盤は公式案内上Mainnet有効。各契約の監査・互換性・運用可否は個別に確認 |
| SilverScript | コンパイラー、ABI（呼び出し形式）、初期化引数と生成Scriptを固定して検査 | 固定v1.0.0のコンパイル／ローカルVM実習あり。基盤のMainnet対応だけで生成物の送信成功としない |
| Argent | 依存する成果物・承認・後継状態・Genesisの認証を検査 | 固定READMEは開発中。ローカル例の確認と、Mainnetでの利用確認を分ける |
| Full vProgs | 証明対象の業務ルール、DA（データ可用性）、運用者停止・退出、証明費用 | 研究・開発段階。本書で証明生成・本番退出は未検証。通常のL2と同じ信頼モデルとは限らない |
| 個別のL2 | ブリッジ、運用者、DA、更新権限、引き出し条件を評価 | Igra等の対応ネットワーク・現行構成は各公式資料で確認。本書の固定Devnet実測には含まれない |

通常のKAS入金監視なら、まず[RPC・SDKの実習](part-04-development/chapter-28.md#2864-公式sdkを使った最小監視サンプル)へ進めます。支出先・次状態までL1で拘束する必要が生じたらCovenantを検討し、記述手段としてSilverScriptやArgentを選びます。共有状態の同時更新を扱う場合は、[公式の選択ガイド](https://docs.kaspa.org/programmability)と[第34章](part-05-advanced/chapter-34.md)のBased Appsも参照します。ZK Proofの検証成功だけでアプリ全体の安全性は判断できません。

## 本書を読む順序

初めて読む場合は本書の目次の順に進みます。分野を調べ直すときは、次の表から関連する部へ移れます。

| 確認したいこと | 読む範囲 |
|---|---|
| デジタルマネー、暗号、分散システム、Bitcoin、BlockDAGの基礎 | 第1部：[第1章](part-01-foundations/chapter-01.md)〜[第8章](part-01-foundations/chapter-08.md) |
| Kaspaの構造、ノード、ブロック検証、GHOSTDAG、DAA、仮想状態 | 第2部：[第9章](part-02-consensus/chapter-09.md)〜[第14章](part-02-consensus/chapter-14.md) |
| UTXO、Script、Mempool、採掘、セキュリティと確定性 | 第3部：[第15章](part-03-transactions/chapter-15.md)〜[第20章](part-03-transactions/chapter-20.md) |
| Pruning・同期、RPC・SDK、ウォレット、実装、Devnet実習 | 第4部：[第21章](part-04-development/chapter-21.md)〜[第28章](part-04-development/chapter-28.md) |
| DAGKnight、KIPs、Covenants、SilverScript・Argent、L2、vProgs | 第5部：[第29章](part-05-advanced/chapter-29.md)〜[第34章](part-05-advanced/chapter-34.md) |
| アプリケーション構築と貢献 | 第6部：[第35章](part-06-contributing/chapter-35.md)〜[第36章](part-06-contributing/chapter-36.md) |

第5部の各テーマは別々に調査できます。章の掲載順を、Covenants・開発言語・L2・vProgsの必須の実装順序と解釈する必要はありません。

## 一次資料を読むときの確認事項

個別の技術を詳しく扱う際は、出典のURLと参照日、対象のコミット・リリース・ネットワークを記録します。開発段階は提供者の説明と確認した実装・有効化条件を区別し、実行した場合はコマンド・入力・期待値・実測を残します。信頼モデルや未確認事項も併記します。

このページの資料照合記録は、リポジトリの `experiments/ecosystem-map-review-2026-10-31.md` にあります。今回の確認には、掲載プロジェクトのビルド、コントラクトの実行、外部ネットワークやブリッジへの接続は含みません。関連章の確認範囲は[レビュー進捗](review-status.md)を参照してください。
