# SUMMARY順レビュー 45：付録D「RPC・CLIリファレンス」

確認日：2026-11-01。全文を通読し、基準版の主要RPC/通知・CLI入口、既存実習コードとの対応と独自Rust教材を確認。全RPCの引数/応答/異常系・CLI実行や実ネットワーク試験ではない。

## 主要な修正

- 変化するmasterのcloneをv1.0.1と基準SHAへ固定。本書外で調査し、metadata --no-depsは列挙であり依存解決/ビルド成功ではないこと、生JSONが私有パスを含むことを明記。
- 公開記録へhostname等が出るuname -aを貼らず、OS/CPUを個別取得する形へ変更。Docker Clientの版だけではServer稼働を確認できない。
- getBlockDAGInfoという表記をgetBlockDagInfoへ統一。probe.py由来の生JSON wRPC envelopeを示し、HTTP JSON-RPC/BorshやSDKの返却形式と混同しない。
- 基準版のgetUtxosByAddressesはutxoindex無効時NoUtxoIndexとなること、getVirtualChainFromBlockは応答量が制限され一回で全差分とは限らないことを補足。
- UTXO図は実スキーマのフィールド表ではないと明示。金額のBigInt等をNumberへ丸めないことを補足。index、同期フラグ、入金確定は別の前提。
- 基準版にないSelected Tip Changedを列挙から除き、実在するSink Blue Score/Virtual DAA Score Changed等へ変更。全通知を購読試験したとはしない。
- 再接続疑似コードは以前、接続先を再検証せず古いcheckpointを使い続け、UTXO通知にstable_id/positionがあるかのように見えた。接続ごとの前提再検証・購読確認・初回/再接続/定期再照会・単一workerの現在状態置換へ変更。
- この例はSDKの実APIではなく設計の役割を示す。実行していない。完全に同時点の複数RPC snapshotや、切断中に生成後消費された出力の履歴を保証しない。並行照会には世代管理等、履歴には有効な保存位置/保持範囲を考慮した別設計が必要。
- outpoint/kindを既処理集合へ入れるだけでは、再編成のadded→removed→addedを落とす場合がある。現在状態の冪等置換とイベント履歴の同一性を分けた。
- 送信タイムアウト後に別取引を新規作成する二重送金を避ける表現へ変更。応答喪失を送信失敗と断定しない。
- Compose診断に-f experiments/devnet/compose.yamlを指定。プレースホルダーは実名へ置換し、生ログやdocker psを丸ごと公開しない。
- SDKによる既存購読成功/生RPC購読失敗は過去の実験と明示。今回の新しい接続/起動/送金/採掘/通知/Pruning試験ではない。

## CLIの参照

https://github.com/kaspanet/kaspa-cli はWebで404となったため、案内を外した。基準版rusty-kaspaのcli/Cargo.tomlにpackage kaspa-cli、lib kaspa_cli_libがあることを手元のソースで確認。crate名をそのまま実行バイナリーとみなさず、配布物/実行ターゲットを確認する説明へ変更。CLI自体は今回ビルド/実行していない。

## ソース照合

基準版v1.0.1 / fcd9c28f9b211b9872a0c90a8c94f79466ffb063として以前取得した手元のソースarchiveを読取。新規clone/HEAD取得はしていない。Webの固定cli/Cargo.toml raw URLはcache missで、オンラインで取得済みとはしない。

- rpc/wrpc/server/src/router.rs：本文表のGetInfo/GetCurrentNetwork/GetBlockDagInfo/GetBlock/GetConnectedPeerInfo/GetUtxosByAddresses/GetServerInfo/GetMempoolEntry/GetVirtualChainFromBlock、およびSubmitTransactionの登録を確認。
- rpc/service/src/service.rs：get_utxos_by_addresses_callのNoUtxoIndex、get_virtual_chain_from_block_callのbatch制限・removed/added/accepted IDの応答、get_server_info_callのnetwork/index/sync等の主要処理を確認。
- notify/src/events.rs：通知enumの名前を確認。型の存在と実際の配信/購読成功を区別。
- 既存probe.py、watch-once.cjs、live-reconciliation.pyの主要な読取/購読/再照会経路を確認。全コード経路を今回監査したという意味ではない。

| 基準版の資料 | SHA-256 |
|---|---|
| cli/Cargo.toml | `f10969163c782af9e8dd76bee28e40aef01be9ebcad47aabd7986dd0e5e5e821` |
| rpc/wrpc/server/src/router.rs | `8022ccde64382ef2a4a1ec8992b2fab3c0c527ccbdaf54604f11250681230a95` |
| rpc/service/src/service.rs | `b07acf50884a1d4c0361f0227952293012a7af6cf49663be5d8b0aad022ad43f` |
| notify/src/events.rs | `4c7cf620c36848e7e558006bf5466cc246d787aed61242569cafb5a3c1e59081` |

## Rust教材

本文のコードを一時Cargo libプロジェクトへ抽出。既存Rust/Cargo 1.90.0、edition 2021、外部依存なし。公開APIとして単独libで使えるようpubを付け、空TXIDの拒否ケースを追加。非空文字列/教材上の正額条件に限定し、実TXIDの妥当性や資金/確定性を検証するものではない。

```bash
cargo fmt --all
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
cargo fmt --all -- --check
```

正常値・空TXID・0額の3テストが成功し、Clippy/書式確認も成功。doc-testは0件で件数に加えない。整形後の実ソースと本文が一致。本文の+1.90.0は読者用の版指定で、今回は既存の同版を使い、名前付きtoolchainの追加や新規導入試験をしていない。

本書に新しいexamples/CIを追加しておらず、既存2教材の自動照合へ付録コードを含めたわけではない。

## 統合・公開確認

- mdBook 0.5.4のビルド成功（警告なし）、53 HTMLのローカルリンク/アンカー不整合0。
- SUMMARY49件と主表、前段1〜46番の順序が一致。45番の付録Dを今回確認、46番の付録Eを次の対象とした。
- scripts/check-book-examples.pyで既存2教材の本文/実装が一致。
- git diff --check成功。
- 追跡対象/非ignore候補191ファイルの既知token prefix・秘密鍵・credential URL・私有home path等のパターン検出0。gitleaks/trufflehogは未導入で、未知の機密やGit全履歴の完全監査ではない。
- 今回は付録D・進捗/ROADMAP・本記録を更新。既存A〜Cの差分を保持。秘密鍵・生ログ・DB・バイナリー・私有パス・新しいexamplesを追加せず、commit/push/実ノード操作もしていない。
