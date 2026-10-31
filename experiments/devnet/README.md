# Devnetノード起動とwRPCのスモークテスト

本書の基準版とSDKの方針は[対象バージョン](../../src/baseline.md)を参照。以下は既存イメージで観測した結果であり、ソースからの再ビルド検証ではない。

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
単一ノードでの送金とvirtual受理は末尾の実習で確認済み。2ノードのブロック伝播・停止復帰は末尾の実習で確認済み。分断時の両側採掘やPruning後の同期は未検証。採掘と状態保持の検証は以下を参照。

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

## 実署名・送金・コンセンサス受理の確認

`send.cjs` は公式v1.0.1リリースのNode.js/WASM SDKを使用する。依存のSDKはリポジトリへコピーしない。
Node.js 20.0.0で確認した。以下はリポジトリのルートから実行する。

```bash
mkdir -p experiments/devnet/.sdk
curl -fL https://github.com/kaspanet/rusty-kaspa/releases/download/v1.0.1/kaspa-wasm32-sdk-v1.0.1.zip -o experiments/devnet/.sdk/sdk.zip
# 次のSHA-256が一致することを確認してから展開する
shasum -a 256 experiments/devnet/.sdk/sdk.zip
# 70679fe150689eedd588a475ae93f1135cfb99bd7b205dc2661d104d5db564d0
unzip -q experiments/devnet/.sdk/sdk.zip -d experiments/devnet/.sdk
npm install --prefix experiments/devnet/.sdk --ignore-scripts --no-audit --no-fund websocket@1.0.34

docker compose -f experiments/devnet/compose.yaml -f experiments/devnet/mining.yaml up -d node
KASPA_SDK_ROOT="$PWD/experiments/devnet/.sdk" node experiments/devnet/send.cjs
# 終了時はノードも停止する
docker compose -f experiments/devnet/compose.yaml -f experiments/devnet/mining.yaml stop node
```

SDKのJSON接続には `Encoding.SerdeJson` を使う。`Encoding.Json` はこの版には存在せず、指定を取り違えると接続に失敗する。
WebSocket依存の推移依存まで固定する再現性改善は今後の課題。

スクリプトはDevnet・ノード版・UTXOインデックスを検査した後、新しい送金元・宛先の秘密鍵をメモリ内でランダム生成する。専用マイナーを最大1CPUで起動し、Coinbaseの成熟をSDKのDAA期間で確認する。1件の成熟UTXOから1 KASを署名・送信し、宛先UTXO・入力消費・virtual chainの受理IDを照合する。実験の待機は180秒、個々のRPCは10秒を上限とする。

マイナーは成功・失敗・SIGINT/SIGTERMで停止する。強制終了やDocker自体の障害時は `docker ps --filter name=mastering-kaspa-send-` で残存を確認し、該当する実験コンテナを停止する。既存の採掘プロファイルのminerは先に停止しておく。

秘密鍵やシード、署名中のトランザクション全体を出力しない。出力は公開アドレス・TX ID・受理結果等に限定する。実験終了後の鍵復旧手段は提供しないため、実資産の保管・送金には使えない。今回のメモリ内生成と解放は、ランタイムの全コピーを含む鍵消去の保証ではない。

実測結果は `send-observed-2026-10-31.json`。これは観測時点のvirtual受理であり、不可逆な決済確定・2ノード収束・再編成耐性を保証しない。Mempool掲載、3種類の異常系と算術上の手数料・お釣りの検算は、下記の追加モードで確認した。ブロック包含の独立観測とタイムアウト再試行は継続課題。

## 手数料・お釣りと異常系

同じ環境で次を実行する。

```bash
KASPA_SDK_ROOT="$PWD/experiments/devnet/.sdk" node experiments/devnet/send.cjs --checks
```

このモードは成熟資金の準備後に採掘を止め、SDKの残高不足、署名を変更した送金のノード拒否、正常送金のMempool掲載、同じ入力の競合送金の拒否を確認してから採掘を再開する。Mempool照会では `filterTransactionPool: false` を指定する。不正署名用の複製にはsafe JSON復元APIを使う。

どちらのモードでも、入力−出力による手数料とSDK値、送金元に返った同じTX IDのお釣りUTXOを比較する。実測値と検証範囲は `../devnet-send-checks-2026-10-31.md`、公開JSONは `send-checks-observed-2026-10-31.json`。
送信成功応答を失ったケースの再試行は次のモードで検証済み。ブロック包含の独立照会、mass手数料の別実装での再計算は未検証。

## 応答喪失と再試行・二重計上防止

```bash
KASPA_SDK_ROOT="$PWD/experiments/devnet/.sdk" node experiments/devnet/send.cjs --retry-checks
```

同じ起動手順でノードのみを開始してから実行する。`--checks` とは別の試験モードとして使う。
ローカルプロキシがsubmitTransactionの成功応答を1件落とし、送信クライアントの500ms期限切れを再現する。TX IDによるMempool照会で状態を復旧し、同じ署名済みTXの再送が重複として拒否されることを確認した。

受取UTXOの同じ情報を2回処理し、Outpointをキーに計上すると1回分の1 KASに保たれる。これはインメモリの検証であり、DBの一意制約、再起動、並行ワーカー、再編成対応まで保証するものではない。

結果は `send-retry-observed-2026-10-31.json`、故障条件と限界は `../devnet-send-retry-2026-10-31.md`。
プロキシは127.0.0.1だけで待ち受け、正常終了時に停止する。Mempoolに入る前に失敗したケースや、照会前に採掘済みとなったケースの復旧は未検証。

## 2ノードの伝播と停止復帰

Python環境は上記と同じ。単一ノードのComposeとは独立した構成を使う。

```bash
experiments/devnet/.venv/bin/python experiments/devnet/two-node.py
```

`two-node.yaml` の専用プロジェクトでA/Bを起動し、双方の接続を確認する。短時間の採掘後にDAG・指定報酬先のUTXOを比較し、Bを停止してAだけで追加採掘した後、再開したBが追従することを確認する。終了時は本試験のサービスを停止し、ボリュームは保持する。
接続先にはサービス名でなく、スクリプトが取得したAのネットワーク内IPを使用する。

中断後の停止確認：

```bash
docker compose -f experiments/devnet/two-node.yaml --profile mining stop -t 2
docker compose -f experiments/devnet/two-node.yaml --profile mining ps -a
```

RPCは127.0.0.1:19310/19320だけへ公開。1CPU以内の採掘、段階ごと90秒上限の待機を行う。再実行時もノードIPを取得し直すため、手動で固定IPを埋め込む必要はない。

結果は `two-node-observed-2026-10-31.json`、実測条件と制約は `../devnet-two-node-2026-10-31.md`。
停止復帰後に27ブロック・指定報酬先の26件のUTXOが一致した。全UTXO集合の照合、2ノードでの送金受理、両側採掘の分断・reorg、Pruning後の同期は未検証。
