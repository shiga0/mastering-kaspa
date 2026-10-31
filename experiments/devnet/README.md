# Devnetノード起動とwRPCのスモークテスト

## 実習の入口

まず下記のPython環境と単一ノードのprobeを準備する。送金と実通知には「公式SDKでの単一署名送金」のSDK導入も必要。すべてリポジトリのルートで、一つずつ実行する。

| 段階 | 実行入口 | 観測するもの |
|---|---|---|
| 起動 | `probe.py` | Devnet・ノード版・読み取りRPC |
| 状態保持 | `persistence.py` | 短時間採掘後の再起動での保持 |
| 送金 | `send.cjs`、`--checks`、`--retry-checks` | 単一署名・主要な拒否・応答喪失 |
| 伝播 | `two-node.py` | 2ノードの停止復帰 |
| 分断 | `partition.py` | 両側採掘後の収束 |
| 通知補完 | `notifications.py`、`live-reconciliation.py` | 重複・欠落・古い照会結果の拒否 |
| UTXO取り消し | `reorg-observer.py` | 枝の報酬の消失と観測ストアの復旧 |
| Pruning前提 | `pruning-check.py` | pointの確認。実際のPruningは未達 |

単一ノードは18310、2ノードは19310/19320を使う。reorg実習も19310/19320を使うため、2ノード実習を停止してから実行する。現在UTXOの観測ストアは、会計台帳や成熟した利用可能残高ではない。各節末尾の検証記録に成功範囲と未確認事項を記す。

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
単一ノードでの送金とvirtual受理は末尾の実習で確認済み。2ノードのブロック伝播・停止復帰は末尾の実習で確認済み。分断時の両側採掘・通知補完・枝の報酬の取り消しも後述の別実習で確認した。Pruning後の同期は未検証。採掘と状態保持の検証は以下を参照。

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
停止復帰後に27ブロック・指定報酬先の26件のUTXOが一致した。この停止復帰試験では、全UTXO集合の照合、2ノードでの送金受理、Pruning後の同期は未検証。両側採掘の分断と枝の報酬の取り消しは、後述の別試験を参照する。

## 両ノード稼働中のP2P分断

2ノードのボリュームを使うため、前の2ノード試験を一度実行してから、同時実行を避けて開始する。

```bash
experiments/devnet/.venv/bin/python experiments/devnet/partition.py
```

`partition.yaml` と `relay.conf` を追加して、B→relay→AのP2P接続を作る。relayだけ停止し、A/Bが稼働したままピア数0になることを確認する。それぞれで順に短時間採掘し、異なるsinkができた後にrelayを再開して収束を比較する。最大同時1CPU、各段階90秒上限。

中断後の停止確認：

```bash
docker compose -f experiments/devnet/two-node.yaml -f experiments/devnet/partition.yaml --profile mining stop -t 2
docker compose -f experiments/devnet/two-node.yaml -f experiments/devnet/partition.yaml --profile mining ps -a
```

新しいrelayイメージもdigest固定し、ホストへP2Pポートを公開しない。実行時にrelayのIPを取得するため固定IPの編集は不要。データは停止時に削除しない。
結果は `partition-observed-2026-10-31.json`、条件と制約は `../devnet-partition-2026-10-31.md`。
今回、分断前27、分断中A=42/B=47、復旧後62ブロックとなり、指定報酬先のUTXO内容も両ノードで一致した。

通知の取りこぼし・重複の復旧は後述の別試験を参照する。競合送金の取り消しを伴う台帳処理、全UTXO集合・Pruning後の同期は未検証。

## 通知欠落・重複とSQLiteの再読込

```bash
KASPA_SDK_ROOT="$PWD/experiments/devnet/.sdk" experiments/devnet/.venv/bin/python experiments/devnet/notifications.py
python3 -m unittest discover -s experiments/devnet -p 'test_*.py'
```

公式SDKで実UTXO通知を1件受け、Outpoint一意制約のあるSQLiteへ2回適用する。その後は購読を終了して採掘を続け、別プロセスで既存DBを再読込し、RPCスナップショットで欠落を補完する。採掘は最大1CPUで、条件待機と通知受信には期限を設け、終了時にノード・マイナー・購読子プロセスを停止する。

DBは一時領域に作成して終了時に削除する。公開結果は `notifications-observed-2026-10-31.json`、実測条件は `../devnet-notifications-2026-10-31.md`。
現在のUTXO観測ストアであり、会計上の売上や成熟残高ではない。実測ではremoved通知は0件。通知順序の逆転、稼働中の補完との競合、複数ワーカー、再編成の会計処理は未検証。

## 稼働中の再照会と再購読

```bash
KASPA_SDK_ROOT="$PWD/experiments/devnet/.sdk" experiments/devnet/.venv/bin/python experiments/devnet/live-reconciliation.py
```

`watch-once.cjs --hints` で実通知を継続して受け、通知本文の差分を直接適用せずに再照会のきっかけとする。`LatestSnapshot` が要求開始順に世代番号を発行し、古い結果や二重コミットによる上書きを拒否する。照会中の通知は次の照会へつなぎ、通知がなくても周期的に照会する。
購読前後の穴は、購読開始後の完全照会で補完する。実験では購読を止めた間にも採掘し、再購読直後の照会で全Outpointと金額が一致することを確認した。

実測は `live-reconciliation-observed-2026-10-31.json`、詳細は `../devnet-live-reconciliation-2026-10-31.md`。DBは一時領域で削除する。単一プロセス内の観測状態向けで、並行ワーカーや会計上のreorg取り消しは未検証。

## 分岐合流後のUTXO取り消し

```bash
KASPA_SDK_ROOT="$PWD/experiments/devnet/.sdk" experiments/devnet/.venv/bin/python experiments/devnet/reorg-observer.py
```

`reorg.yaml` を追加し、Bの報酬先だけ別の公開テスト用アドレスへ変える。専用プロジェクト `mastering-kaspa-reorg-small` を使うが、RPCポート19310/19320は他の2ノード試験と共有するため同時実行しない。
最大0.5CPUで順に採掘し、再接続後に統合ブロックを作る。Bの実削除通知・再照会・SQLiteの無効化・別プロセス再読込を比較する。DBは一時領域で削除し、キーや生通知は公開しない。

実測ではB固有の報酬UTXO10件が消え、全10件が削除通知にも含まれた。履歴は削除せず無効化し、再適用・再読込でも有効額0を維持した。これは観測UTXOの補正であり、売上や決済済み請求書の取り消しではない。

結果：`reorg-observed-2026-10-31.json`。失敗試行と限界：`../devnet-reorg-observer-2026-10-31.md`。
停止確認：

```bash
docker compose -p mastering-kaspa-reorg-small -f experiments/devnet/two-node.yaml -f experiments/devnet/partition.yaml -f experiments/devnet/reorg.yaml --profile mining stop -t 2
```

再実行は既存のボリュームを使うため、ハッシュ・件数は変わる。条件が成立しなければ成功扱いにせず停止する。Pruning後の同期・競合する送金の勝敗・会計の取り消しは未検証。

## Pruningの前提検査（未達を成功扱いにしない）

ノードを起動した状態で、採掘せずに読む。

```bash
experiments/devnet/.venv/bin/python experiments/devnet/pruning-check.py --require-advanced
```

pointがGenesisのままならJSONを表示して終了コード2を返す。今回の実測も未達だった。pointが進んでも、データ削減・Pruning ProofによるIBDの検証済みフラグは立てない。
基準版Devnetの従来pruning depthは185798、finality depthは86400。深さはblue scoreの規則で扱い、DAA scoreやブロック数をそのまま代用しない。

実測：`pruning-observed-2026-10-31.json`。固定ソースと次の完了条件：`../devnet-pruning-review-2026-10-31.md`。
長時間の生成、削減後の新規ノード同期、不正な集合の注入は今回実行していない。
