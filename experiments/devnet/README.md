# Devnetノード起動とwRPCのスモークテスト

検証環境：macOS Apple Silicon、Docker Engine 27.4.0、Python 3.11、websockets 14.1。
このComposeは **linux/arm64専用の検証構成**。macOS Apple Siliconで確認済みであり、Linux ARM64自体での実行とamd64では未検証。amd64でそのまま動くとは扱わない。

`kaspanet/rusty-kaspad` の公開イメージをdigestで固定し、kaspad 1.0.1で確認した。公開されたイメージラベルのsourceはDocker包装用の `supertypo/docker-rusty-kaspa`、revisionは `639a036ff577ede2a6e871473a2b47c010b8f0fa`。これはノード本体のコミットではない。
公開ビルド変数は `REPO_URL=kaspanet/rusty-kaspa`、`RUSTY_VERSION=v1.0.1` を宣言する。GitHubでこのタグが指すコミットは `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` と確認した。ただし、バイナリがそのソースと一致することをビルド証明・再ビルドで確認したわけではない。第12章の照合コミットは別の版であり、この実習のバイナリと同一とは扱わない。出典と確認範囲は `image-provenance.json` に記録する。

外部ピア探索・接続とUPnPを無効化し、専用bridgeネットワークを使用する。
ネットワーク自体は外向き通信を禁止する設定ではない。
RPCはホストの127.0.0.1:18310だけに公開する。
イメージの既定entrypointはDNS照会するため、ノード本体のkaspadを直接起動する。

リポジトリのルートで実行：

```bash
python3 -m venv experiments/devnet/.venv
experiments/devnet/.venv/bin/python -m pip install -r experiments/devnet/requirements.txt
docker compose -f experiments/devnet/compose.yaml config
docker compose -f experiments/devnet/compose.yaml up -d
docker compose -f experiments/devnet/compose.yaml logs --tail=30
experiments/devnet/.venv/bin/python experiments/devnet/probe.py
```

ログの `WRPC Server starting` を確認してからprobeを実行する。
成功時にはDevnet・serverVersion 1.0.1・UTXOインデックス有効のJSONを表示する。同期状態は観測値として出力し、採掘後もfalseであるとは仮定しない。
接続直後にconnection refusedとなる場合はログで起動状態を確認する。
初回の採掘前には `isSynced: false` とブロック数0を観測した。保持したボリュームに採掘済みデータがあれば、ブロック数0には戻らない。

再起動の確認：

```bash
docker compose -f experiments/devnet/compose.yaml restart node
# ログでRPC待受を確認してから再実行
experiments/devnet/.venv/bin/python experiments/devnet/probe.py
```

終了時：

```bash
docker compose -f experiments/devnet/compose.yaml stop
```

データは名前付きボリュームに保存する。停止で削除しない。
送金、取引受理、2ノード同期は未検証。採掘と状態保持の検証は以下を参照。

## 短時間採掘と再起動後の状態比較

追加の `mining.yaml` はノード内部だけでgRPCを有効化し、Devnetで未同期採掘を許可する。gRPCポートはホストへ公開しない。
CPUマイナーもdigest固定、ARM64、1スレッド・最大1CPU。プロファイル指定時だけ起動する。
報酬先は公開のテスト用アドレスであり、この構成をMainnetに変更して使用しない。秘密鍵を保管するウォレットや送金処理は含まない。

```bash
# 別ターミナルですぐに停止できる状態で実行する
docker compose -f experiments/devnet/compose.yaml -f experiments/devnet/mining.yaml --profile mining up -d
# 1ブロック以上の受理をログで確認したら、必ずマイナーを停止する
docker compose -f experiments/devnet/compose.yaml -f experiments/devnet/mining.yaml --profile mining logs --tail=20 miner
docker compose -f experiments/devnet/compose.yaml -f experiments/devnet/mining.yaml --profile mining stop miner

# RPC待受と採掘停止を確認してから、状態を保存する
experiments/devnet/.venv/bin/python experiments/devnet/persistence.py > /tmp/kaspa-devnet-before.json
docker compose -f experiments/devnet/compose.yaml -f experiments/devnet/mining.yaml restart node
# ログでRPC復帰を確認してから比較する
experiments/devnet/.venv/bin/python experiments/devnet/persistence.py --compare /tmp/kaspa-devnet-before.json

# 実験終了時は両サービスを停止する
docker compose -f experiments/devnet/compose.yaml -f experiments/devnet/mining.yaml --profile mining stop
```

マイナーは自動停止しないため、ブロックの受理を確認したら停止する。途中失敗時も上記のstopで終了する。
マイナーはIP指定を要求するため、`network_mode: service:node` でノードと同じ名前空間の127.0.0.1に接続する。

2026年10月31日の実測：151ブロック、virtual DAA score 151、報酬UTXO 150件。採掘を停止した後、再起動前後でブロック数・DAA score・tips・UTXO内容がすべて一致した。実測JSONは `persistence-before-2026-10-31.json` と `persistence-after-2026-10-31.json`。
これはプロセス再起動時の保持検証であり、クラッシュ耐性、送金可能な成熟残高、Pruning後の挙動を検証したものではない。

## 公開するデータ

収録するJSONは本実験で生成したDevnetの公開ブロック・報酬UTXO情報のみ。初回のRPC記録から、検証に不要なランダムな `p2pId` は省略している。ローカルパスも匿名化する。
秘密鍵・シード・実ウォレット・認証情報・生ログ・ノードDBは収録しない。`probe.py` の実行時出力には `p2pId` が含まれるため、生の出力をそのまま追加せず公開に必要な項目だけを残す。採掘報酬先は公開の教材用アドレスで、実資産の保管先として使用しない。
