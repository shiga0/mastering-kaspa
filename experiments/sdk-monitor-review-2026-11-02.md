# 公式SDKの最小UTXO監視サンプル：実測記録

確認日：2026-11-02。

## 対象

- 実装：`examples/sdk-monitor/monitor.cjs`
- 読者向け手順：`examples/sdk-monitor/README.md`、第28章28.6.4
- SDK：公式v1.0.1のNode.js/WASM配布物。アーカイブのSHA-256は`70679fe150689eedd588a475ae93f1135cfb99bd7b205dc2661d104d5db564d0`と一致。
- Node.js 20.0.0、WebSocket依存`websocket@1.0.34`、Docker Engine 27.4.0、macOS Apple Silicon。
- ノードと採掘：既存の`experiments/devnet/compose.yaml`と`mining.yaml`。イメージdigest・linux/arm64指定は既存構成を維持。
- 専用Composeプロジェクト：`mastering-kaspa-sdk-demo`。新規ボリューム、ローカル18310番ポートで実行。

## 手順と判定

1. 専用プロジェクトのノードを起動し、公式SDKでDevnetへの接続が可能になるまで待機。
2. SDK監視サンプルを起動し、最初の購読・初回照会完了を待つ。
3. 専用プロジェクトのminerを起動。
4. 実UTXO通知と再照会、クライアントの切断、2秒後の新しい接続・再購読・再照会を確認。
5. 2回目の実通知と再照会、`complete`、終了コード0を確認。
6. 成功・失敗時共通の終了処理で専用minerとnodeを停止。ボリュームは保持。

自動実行用の一時ラッパーで、両セッションの`connected`・`subscribed`・`notification`・`disconnected`と、`after-reconnect`・`complete`の存在を検査した。接続結果のnetworkとserverVersionも検査した。再現する場合はサンプルREADMEの2ターミナル手順を使う。

## 結果

| 時点 | 現在UTXO件数 | 観測額（sompi） |
|---|---:|---:|
| 初回照会 | 0 | 0 |
| 最初の通知後 | 1 | 50000000000 |
| 再接続直後 | 5 | 250000000000 |
| 2回目の通知後 | 6 | 300000000000 |

両セッションでadded=1・removed=0の実通知を受けた。`isSynced`は初回false、再接続時trueだった。件数・金額・同期状態は今回の観測値であり、固定の期待値ではない。

機械可読な出力は[sdk-monitor/observed-2026-11-02.json](sdk-monitor/observed-2026-11-02.json)に保存した。秘密鍵やシードは生成・使用していない。

追加確認：`node --check`成功。SDKルート未設定時は、設定先を案内して終了コード1で終了することを確認。

## 確認範囲の限界

- 公式Node.js/WASM SDKを実際に使った読み取り専用の有限デモ。RustネイティブRPCアダプターではない。
- 切断はクライアントが意図的に行う。突然の通信断やノード再起動からの自動復旧は試験していない。
- 再接続時に現在UTXOを取り直す。切断中の全イベントや、受理後に支出された入金履歴の復元ではない。
- 採掘報酬の未成熟UTXOも集計する。成熟残高・請求入金額・決済確定の計算ではない。
- 通知と照会の原子的な整合、古い応答の排除、DB永続化、会計、署名・送金はこのサンプルの対象外。
- 他OS/CPU・別SDK版・WASMのブラウザー実行は今回検証していない。
