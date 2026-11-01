# SUMMARY順レビュー 26：第22章「Kaspa RPC・wRPCとアプリケーション統合」

確認日：2026-11-01。基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。本文・図・教材・出典と実習への導線を確認した。全RPC/SDKの検証ではない。

## 本文の修正

- 接続・購読・同期判定・コンセンサス受理・業務上の確定を区別した。Readyは一度通れば永続する資格ではない。
- 通知の少なくとも1回の配送を前提にする表現を修正。再接続・再購読の成功と、欠落通知の再生は別である。現在UTXOの照会だけでは、照会間に作成され消費された出力の履歴を復元できない。
- 現在の出力のキーを完全なNetworkId・TX ID・output indexとした。event_kindを足した永久重複排除では追加→削除→再追加を捨て得ることを説明し、現在状態・評価世代・履歴を分けた。実会計台帳を実装したわけではない。
- 送信結果が不明なときの新規取引の構築と、同一署名済み取引の再送を区別した。UTXO使用済みでも以前の送信成否を先に調べる。エラー表・理解度チェックの曖昧な再試行表現も修正した。
- 固定版の読み取りAPI・UTXO index要件、エンコーディング、Testnet suffixを補足した。固定ソースを再利用する検索手順と公開記録の範囲を整理した。

## 固定ソースとの照合

これまで基準に使った固定版ソースアーカイブを読み、以下の主要箇所を確認した。アーカイブはGit checkoutではなく、今回git rev-parseでHEADを取得したものではない。

- GetServerInfoResponse：RPC API version/revision、server version、network_id、has_utxo_index、is_synced、virtual_daa_score。Rustのフィールド名と生JSON/SDKの表記を混同しない。
- get_server_info_call / get_sync_status_call：MiningRuleEngineのis_sink_recent_and_connectedを用いる。これをネットワーク全体・全インデックスの完全性証明へ広げない。
- get_utxos_by_addresses_call：UTXO indexが無効ならNoUtxoIndex。要求アドレス順にエントリが並ぶとは限らない。get_info_callのis_utxo_indexedとGetServerInfoResponseのhas_utxo_indexも区別した。
- NetworkId：network typeとOption<u32>のsuffix。Testnetの番号を照合する。教材のNetworkは独自の簡易モデルである。
- wRPC server connection：Borsh / SerdeJsonの通知エンコーディングを分岐する。HTTP JSON-RPCと同じプロトコルとは扱わない。

SHA-256はファイルの識別用であり、ファイル全体・すべてのAPIを監査した意味ではない。

| ファイル | SHA-256 |
|---|---|
| `rpc/core/src/model/message.rs` | `24a5fb7c14606e6284c00ca729a960f97a6b5c9d76f22659c8887ff254c6ea98` |
| `rpc/service/src/service.rs` | `b07acf50884a1d4c0361f0227952293012a7af6cf49663be5d8b0aad022ad43f` |
| `consensus/core/src/network.rs` | `8a1f167b10617aa8d3cc1eb53a11fe6ce1c7e685f8a119048d31066eb6910a7b` |
| `rpc/wrpc/server/src/connection.rs` | `40f141ab0c865bf1dd5851c350fbe83e4b510e5db17d21a46cc4079cffa142a6` |

外部資料は [gRPC Introduction](https://grpc.io/docs/what-is-grpc/introduction/) のサービス・stub・Protobufの説明と、[RFC 6455 §7.2.3](https://www.rfc-editor.org/rfc/rfc6455#section-7.2.3) の異常切断後のバックオフを参照した。RFC 9110はHTTPの意味論であり、Kaspa固有の送信/通知保証を規定するものではない。

## 教材と実行

本文のCargo.tomlとRustコードをリポジトリ外の一時プロジェクトへ抽出。edition 2021、Rust/Cargo 1.90.0、外部依存なし。rustfmtを本文へ反映した。examples配下へ新規プロジェクトは作成していない。

```bash
cargo test --offline
cargo fmt --all -- --check
cargo clippy --offline --all-targets -- -D warnings
```

8テスト成功、fmtとClippyも終了コード0。

1. 想定Network・同期済み・新しい応答・必要なindexありでReady。
2. 異なるNetworkを拒否。
3. 未同期を拒否。
4. Testnet-10とTestnet-11を区別し、一致時は通す。
5. index必須時の欠如を拒否し、不要時は許可。
6. fresh=falseの応答を拒否。
7. 同じGatewayのモック応答をReady→未同期→通信エラー→Readyと変え、毎回取り直す。
8. 通信エラーを呼び出し元へ返す。

freshは独自アダプターが判断する値で、実際のGetServerInfoResponseのフィールドではない。時計・TTL・API互換性検査・TLS・実接続・インデックス追従の証明を実装した教材ではない。

## 既存の実測と今回の範囲

- `devnet-send-retry-2026-10-31.md`：成功応答を1件破棄し、Mempool照会・同一署名済みTXの重複拒否と、その後の受理ID・入力消費・宛先UTXOを確認した過去の検査。ブロック入り済みの成否不明TXの検索経路や全通信障害を網羅しない。
- `devnet-notifications-2026-10-31.md`：実通知・意図的な重複反映・購読停止中の欠落補完・SQLite再読み込みの過去の検査。
- `devnet-live-reconciliation-2026-10-31.md`：実通知をヒントに完全照会し、単一プロセス内の世代で古い結果を拒否した過去の検査。SDKの無期限自動再接続や複数ワーカーの検証ではない。

今回これらの実ノード試験は再実行していない。ノード起動・採掘・鍵生成・署名・送金・通知購読も行っていない。全RPC/SDK、自動再接続、Pruning後の復元、切断中の全履歴、TLS/認証、再編成を扱う会計台帳の保証は未検証である。

## 書籍・公開対象の検査

- mdBook 0.5.4でリポジトリ外へビルド成功。生成HTML 53ページの内部ファイル・アンカー切れ0件。
- scripts/check-book-examples.py：既存の教材2組が一致。手動教材をCI対象へ追加していない。
- SUMMARYとreview-statusの本文表49項目が同じ順番。次は第23章。
- git diff --check成功。本文のRustコードは実行した一時プロジェクトと一致。
- 追跡対象・未追跡候補172ファイルの秘密鍵ブロック、既知トークン、認証付きURL、個人ホームパス等のパターン検出0件。gitleaks/trufflehogは未導入で、秘密情報の不存在を保証する監査ではない。
- 新規公開候補はレビュー記録のMarkdown。鍵・シード・DB・生ログ・アーカイブ・ビルド成果物は追加していない。コミット・pushは行っていない。
