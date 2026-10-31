# 本書の対象バージョン

決定日：2026年10月31日。本書の技術レビューと実習は、以下の基準で進める。
これは再現性のための基準であり、最新版や現在のMainnetの仕様を示すものではない。

## 基準版

| 項目 | 採用する基準 | 確認範囲 |
|---|---|---|
| ノードの参照ソース | `kaspanet/rusty-kaspa` v1.0.1 | 完全なコミットは `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` |
| 実習ネットワーク | Devnet | Mainnetの実資産・秘密鍵は使わない |
| Rust教材・CI | Rust 1.90.0、edition 2021 | 収録済み2教材でテスト・Clippy・書式を確認済み |
| 基準ソースのRust要件 | `rust-version = 1.82.0` | 公式Cargo設定の宣言値。ノードのソースビルド成功を意味しない |
| Rust RPC/ウォレットSDK | 基準コミット内の `kaspa-wrpc-client`、`kaspa-rpc-core`、`kaspa-wallet-core` 等 | ワークスペース上の版は1.0.1。実習用アダプターの実装・互換性検証はこれから行う |
| WASM SDK | 公式v1.0.1リリースのNode.js用SDK（宣言版1.0.1） | アーカイブSHA-256を固定して実署名・送金・virtual受理を確認。ソースとの再ビルド一致は未確認 |
| 既存wRPC検査 | Python 3.11、websockets 14.1 | 公式Rust SDKとは別のJSONクライアントとして実行確認済み |
| mdBook | 0.5.4 | ローカルビルド確認済み |
| 実行確認環境 | macOS Apple Silicon、Docker Engine 27.4.0、linux/arm64コンテナ | Linuxホスト・amd64は未検証 |

参照元は[固定コミットのCargo.toml](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/Cargo.toml)。クレート名・版・Rust要件はここで確認した。Cargo依存を追加するときは同じ完全なGit revisionを指定し、生成したCargo.lockも保存する。公開レジストリで同名・同番号のクレートが配布されているとは、この確認だけでは判断しない。

## 実測したDockerイメージ

| 用途 | 固定参照 |
|---|---|
| ノード | `kaspanet/rusty-kaspad@sha256:5bc3195c2cd2d171885781ed55ead9485d93c7bd7b784d46e9e393c96757fe22` |
| CPUマイナー | `kaspanet/cpuminer@sha256:60f78ab2828ab24b249c99210eee5a2825303a5226154260dd021ff26d46748b` |

ノードは `1.0.1` と応答し、イメージのビルド変数もv1.0.1を宣言している。ただし、イメージのsourceラベルはDocker包装用リポジトリを指す。ノード本体のソースとの一致を、再ビルドやビルド証明で確認したものではない。

したがって、ソースで確認した規則とバイナリで観測した結果は証拠を分けて記録する。既存の実測結果は保持し、基準ソースそのものを実行した結果とは書き換えない。これはROADMAPのV2で扱う。

リポジトリ内の `experiments/devnet/image-provenance.json` にタグの参照先と包装用ソースを、`experiments/devnet/README.md` に再現手順を記録している。

## 既存原稿との違い

第12章と第26章のGHOSTDAG主要規則は、先に次のコミットを照合している。

```text
既存の照合：4edcbbccb449f84832afb1ca157904b35f37d006
本書の基準：fcd9c28f9b211b9872a0c90a8c94f79466ffb063（v1.0.1）
```

2026年10月31日に第12章・第26章の主要GHOSTDAG経路を基準版へ再照合した。親選択・候補順・blue判定・score/workの主要規則を確認し、通常managerのkが固定値からselected parentのDAA scoreに依存する `ForkedParam` へ変わった点を本文へ反映した。記録は `experiments/ghostdag-baseline-review-2026-10-31.md`。公式実装を実行した両版の動作比較と章全体のレビューは未完了である。旧照合記録は当時の証拠として保持する。

基準版の公式分類関数と独立した集合モデルの小規模DAG比較を、24ケース・各親配列順712ブロックで実施し一致した。固定アーカイブと公式Cargo.lockを使う再実行手順は `experiments/ghostdag/README.md`。合成ヘッダー・共通DFSアダプターを使い、公式ノード全体や旧版との実行比較は対象外。

付録Eなどの `master` を参照した数値、KIPや高度な言語・研究の説明も、基準版の機能と自動的に同一視しない。基準版にない内容は、対象コミット・ネットワーク・成熟度を個別に示す。

## 更新するとき

基準版を変更するときは、このページ、実習依存、イメージ情報、各章の照合範囲を併せて更新する。旧版の実測記録は履歴として残す。テスト済みの版と新しい版を混ぜてVerifiedとしない。

公式WASM SDKの実測と配布アーカイブのSHA-256は、リポジトリの `experiments/devnet-send-review-2026-10-31.md` に記録している。
