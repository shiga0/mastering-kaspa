# Devnet実習の事前確認

- 日付：2026-10-31（Asia/Tokyo）
- 環境：macOS / arm64
- 現在の状態：起動・読み取りRPC・再起動後のRPC復帰はPASS。送金は未実行。

以下はDocker起動前の記録。解消後の結果は末尾に追記した。

`docker version` の実行結果：

```text
Client: Version 27.4.0
Context: desktop-linux
Cannot connect to the Docker daemon at unix://<DOCKER_SOCKET>. Is the docker daemon running?
```

この結果からデーモンに接続できないことを確認した。デーモン停止とソケット等の設定不備は切り分けていない。`kaspad` もPATH上には見つからなかった。

再開条件：Dockerデーモンに接続でき、`docker version` でServer情報を取得できること。その後、公式イメージのバージョン・digest・アーキテクチャ、Devnetフラグとデータ保存先、RPC方式・クライアントを固定する。ノード起動、ネットワーク識別、資金準備、送金、受理確認を順に実行し、実測ログを保存する。現時点で起動や送金に成功したとは記録しない。

## Docker起動後の再検証

Docker Desktop 4.37.2 / Engine 27.4.0（linux/arm64）へ接続できた。
固定イメージ `kaspanet/rusty-kaspad@sha256:5bc3195c2cd2d171885781ed55ead9485d93c7bd7b784d46e9e393c96757fe22` を使い、kaspad 1.0.1を起動した。初期の接続不可は解消した。

- Compose設定：`experiments/devnet/compose.yaml`
- 再現手順：`experiments/devnet/README.md`
- 読み取り専用wRPC検査：`experiments/devnet/probe.py`
- 再起動後の実測応答：`experiments/devnet/observed-2026-10-31.json`

`getInfo`、`getCurrentNetwork`、`getBlockDagInfo` に応答し、ネットワークdevnet、バージョン1.0.1、UTXOインデックス有効を確認した。ノード再起動後も検査は終了コード0で成功した。初期のinternalネットワーク設定ではホストのRPCポートへ接続できず、専用bridgeネットワークに変更して解消した。

外部ピア・採掘を使わないため、isSyncedはfalse、blockCountは0。これは同期・送金の成功を意味しない。第12章の照合コミットと、このイメージのソースの対応関係も未確認。起動と再起動時のRPC復帰までをPASSとし、資金準備・採掘・送金・受理確認・2ノード同期は未実施とする。

## 採掘・状態保持の追加検証

固定digestのCPUマイナーを1スレッドで実行し、ブロックの受理後に停止した。Devnetの未同期採掘フラグを使用。151ブロック、virtual DAA score 151、報酬UTXO 150件を観測した。ノード再起動後に、ブロック数・DAA score・tips・UTXO内容が一致することを `persistence.py --compare` で確認し、終了コード0で成功した。

実測は `devnet/persistence-before-2026-10-31.json` と `devnet/persistence-after-2026-10-31.json` に保存。初回のゼロブロックでのRPC復帰確認より、検証範囲を拡張した。送金・受理確認・2ノード同期は未実施。

イメージのソースラベルはDocker包装用リポジトリを指す。宣言するノード版v1.0.1のタグのコミットは確認したが、バイナリとソースの一致は未検証。本文のGHOSTDAG照合対象とは版が異なる。`devnet/image-provenance.json` に根拠と限界を保存した。
