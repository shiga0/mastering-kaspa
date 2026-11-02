# 公式SDKでDevnetのUTXOを監視する

第22・23章の境界設計から、実際のKaspa APIへ進むための小さなサンプルです。
独自のモックではなく、公式v1.0.1 Node.js/WASM SDKの`RpcClient`を使います。
RustのネイティブRPCアダプターではありません。

接続とサーバー情報取得 → UTXO購読・照会 → 実通知受信 → 切断 → 再接続・再購読・再照会を実行します。
接続先はローカルDevnet、監視先は既存の採掘実習用アドレスに固定しています。秘密鍵は不要です。

## 準備

コマンドは本書リポジトリのルートで実行します。
[第28章28.6](../../src/part-04-development/chapter-28.md#286-ローカル環境の準備)の手順で公式SDKを取得し、SHA-256を確認してWebSocket依存を導入してください。
DockerとNode.jsが必要です。既存のCompose構成はlinux/arm64専用です。

```bash
export KASPA_SDK_ROOT="$PWD/experiments/devnet/.sdk"
docker compose -p mastering-kaspa-sdk-demo -f experiments/devnet/compose.yaml -f experiments/devnet/mining.yaml up -d node
```

この実習は専用のComposeプロジェクトとデータボリュームを使います。
ホストの18310番ポートは既存の単一ノード実習と共通なので、そちらが動いている場合は先に停止します。
ノードの起動直後に接続を拒否された場合は、ログでwRPC待受を確認してから再実行してください。

## 実行

ターミナルAで監視を開始します。

```bash
export KASPA_SDK_ROOT="$PWD/experiments/devnet/.sdk"
node examples/sdk-monitor/monitor.cjs
```

`"step":"subscribed","session":1`が出たら、ターミナルBで採掘を開始します。
採掘報酬によるUTXO追加が通知の入力になります。送金や請求処理は行いません。

```bash
docker compose -p mastering-kaspa-sdk-demo -f experiments/devnet/compose.yaml -f experiments/devnet/mining.yaml --profile mining up -d miner
```

サンプルは通知を受けてUTXOを再照会し、自分で接続を閉じます。2秒後に新しいクライアントで再接続し、購読と照会をやり直します。
採掘を継続しておけば、2回目の通知を受信した後に`"step":"complete","sessions":2`で終了します。
接続・照会は各10秒、通知待ちは各60秒で打ち切り、失敗時は終了コード1になります。

### 出力の読み方

| step | 確認する内容 |
|---|---|
| `connected` | サーバーがDevnet v1.0.1でUTXO index有効。`isSynced`も観測値として表示 |
| `snapshot` | 現在UTXOの件数とsompi合計。`bigint`は文字列で出力 |
| `subscribed` | 購読要求と初回照会が完了 |
| `notification` | 実通知の追加・削除件数。前後の照会と同じ瞬間の値とは限らない |
| `disconnected` / `offline-gap` | 意図的な切断と待機 |
| `complete` | 2セッションとも接続・購読・通知・照会が完了 |

`after-reconnect`の照会は、その時点の状態を取り直します。切断中に届かなかった通知を再配送するものではありません。
UTXO合計には未成熟な採掘報酬も含むため、利用可能残高や入金確定額として使いません。

## コードと設計の対応

| 独自Rust教材で学んだ責務 | このサンプルで使う公式SDK |
|---|---|
| 接続先の確認 | `getServerInfo()`の`networkId`・`serverVersion`・`hasUtxoIndex` |
| ノードの同期状態の観測 | `getServerInfo()`の`isSynced` |
| 接続と購読 | `connect()` → `subscribeUtxosChanged([address])` |
| 状態の取得 | `getUtxosByAddresses({ addresses: [address] })` |
| イベントを更新のヒントとして扱う | `utxos-changed`受信後に現在UTXOを再照会 |
| 再接続時の初期化 | 新しいクライアントで同じ検査・購読・照会を実行 |

実装全体は[monitor.cjs](monitor.cjs)です。通知ハンドラーは購読前に登録し、各セッションの終了時に解除・切断します。
`bounded`は待ち時間を制限する関数で、SDKの処理自体をキャンセルするものではありません。

## 終了

成功・失敗にかかわらず採掘とノードを停止します。サンプル終了だけでは採掘は止まりません。

```bash
docker compose -p mastering-kaspa-sdk-demo -f experiments/devnet/compose.yaml -f experiments/devnet/mining.yaml --profile mining stop -t 2 miner node
```

データボリュームは残ります。再実行時のUTXO件数は0とは限りません。

## 次に進むには

この例は手動で切断・再接続する有限の実習です。突然の通信断からの自動復旧、永続化、通知と照会の競合制御、履歴の復元は実装していません。
継続的な監視では[既存の再照合実験](../../experiments/devnet/live-reconciliation.py)と[第28章](../../src/part-04-development/chapter-28.md)を参照してください。
第35章へ進む場合は、正規化・履歴保存・請求との対応・確認ポリシーを追加します。現在UTXOの合計を、そのまま請求の支払い額へ渡さないでください。
