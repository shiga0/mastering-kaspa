# SUMMARY順レビュー 13：第9章「Kaspaのアーキテクチャ」

確認日：2026-10-31。対象は `src/part-02-consensus/chapter-09.md`。第2部の入口として、構造・責務・固定版との対応・コード教材を確認した。

## 主な修正

- 参照版をv1.0.1・完全SHAへ固定し、後発版のP2P v11/チャンク化IBDやToccata文書を基準版の説明に混ぜないようにした。
- コンテキスト図を厳密なC4図とせず、Mempoolが永続化へ直結するように見える内部図を機能分類へ変更。起動時に各サービスを構成するソースへ対応づけた。
- PoWを実際の資源投入者・試行回数の計測とみなす表現を修正。祖先の自己包含の規約と閉路の不在を区別した。
- 同じ検証結果にはブロック・パラメータ以外にDAG/UTXO履歴・評価条件も必要と明記した。
- Anticone、Virtualの親の選択、全tipsの無条件採用ではないことを前章までと揃えた。
- ライフサイクル・受信処理・まとめの図を、ヘッダー内のGHOSTDAG、本体、Virtualでの順序・UTXO処理へ揃えた。保存・検証・受理を同じ状態として扱わない。
- Pruning proofと受信UTXO集合の検証を区別し、既存DevnetではPruning未達であるという限界を参照した。
- Mempoolは全未確定取引や永続ウォレット履歴ではないと説明。手数料ポリシーとの境界の出典を固定版の検査コードへ変更した。
- wRPCの説明を固定READMEへ対応づけ、ソース探索は第1章で取得する基準版へ揃えた。教材をリポジトリ外で作る手順を追加した。
- DAG教材で重複親を拒否し、拒否時に全状態が変わらないテストを追加。tipsは構造上の先端であり、Virtualの親選択ではないことを明記した。

## 固定ソースと出典

基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。今回の主な入口とファイルのSHA-256は以下の通り。

| ファイル | SHA-256 |
|---|---|
| [README.md](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/README.md) | `8fe16a2428a0cd33524bf5b7842fcb3670e6227d367e1cc1778bc5f5d5d76d31` |
| [kaspad/src/daemon.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/kaspad/src/daemon.rs) | `72c6db27589315a87f18912835e384c1cf0b70a8fca6403c9be909c98f396528` |
| [mining/src/mempool/mod.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/mining/src/mempool/mod.rs) | `d16e17f455ecd23e545c4b057c71914c41d4cb9f1c161513e93459ac251bc085` |
| [rpc/service/src/service.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/rpc/service/src/service.rs) | `b07acf50884a1d4c0361f0227952293012a7af6cf49663be5d8b0aad022ad43f` |
| [rpc/grpc/core/proto/rpc.proto](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/rpc/grpc/core/proto/rpc.proto) | `c152f1f74f47ac9e7353eac50a667a54629394e55478413cd621bfdaf480f281` |

- kaspadの起動処理でConsensusFactory、MiningManager、RpcCoreService、WrpcService等の生成の入口を確認。本文のgrepとディレクトリ表が参照版で成立することを確認した。
- Mempoolのtransaction/orphan/acceptedの管理と、採掘候補として保持する説明を確認。全Mempool経路や再起動時の耐久性の試験ではない。
- 固定READMEのwRPCの任意設定、JSON/Borsh、JSON-RPCとの差の説明を確認。API全体の互換性試験ではない。
- [v1.0.1リリースノート](https://github.com/kaspanet/rusty-kaspa/releases/tag/v1.0.1)のMainnet/P2P v6接続変更を確認。現在のMainnetの接続条件を調査した記録ではない。
- 本文の固定ソースリンクと、ディレクトリ表8件の存在を確認した。

既存の主要検証経路・Virtual・Pruningの照合は、それぞれ `core-validation-review-2026-10-31.md`、`consensus-state-review-2026-10-31.md`、`devnet-pruning-review-2026-10-31.md` を参照した。祖先の自己包含は第7章の記録、親選択の規則はGHOSTDAG基準版の記録とも整合させた。既存の実測を今回の新規実行へ読み替えていない。

## DAG教材の実行

本文のCargo.tomlとRustブロックを一時プロジェクトへ抽出。Rust/Cargo 1.90.0、macOS ARM64、edition 2021、外部依存なしでcargo test --offlineを実行した。

3テスト成功：

1. 並行したa/bがtipsとなり、両親を持つcを追加するとtipsがcだけになること。
2. 未知の親に対応するエラーを返すこと。
3. 重複ID・自己参照・重複親・未知の親を各エラーで拒否し、登録済みブロック全体が変更されないこと。

cargo fmt --checkも成功。本文の最終コードを使った検査であり、別実装をテストしたものではない。一時プロジェクト・ビルド生成物は公開物へ追加せず、CI同期対象も変更していない。cargo newからの新規環境手順全体を別途実行したものではない。

## 対象外

kaspad全体のビルド・起動・再起動、全RPCの型・形式・通知、署名・PoW・GHOSTDAGの公式実装との差分、Pruning後IBD、現行Mainnetの仕様、全モジュールの依存関係・安全性監査は今回の対象外。教材もGenesisの一意性・最大親数・P2Pの遅延到着を実装していない。

次はSUMMARYの14ページ目、第10章 `src/part-02-consensus/chapter-10.md`。

## 本文反映後の確認

mdBookビルド、生成HTML 53ページの内部リンク・アンカー、SUMMARY全49ページと進捗表の順・ラベル、既存2教材の本文一致、差分の空白検査は成功。公開候補159ファイルの秘密情報・個人ホームパスのパターン検査で検出なし。実ノードへの新たな接続・採掘・送金は行っていない。
