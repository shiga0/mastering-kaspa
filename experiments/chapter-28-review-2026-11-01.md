# SUMMARY順レビュー 32：第28章「Devnetと統合テスト」

確認日：2026-11-01。本文・設定・主要スクリプト・既存記録・独自Rust教材を確認した。実ノードの採掘・送金・分断を今回再実行した結果ではない。

## 本文・手順の修正

- 過去の固定版実測、概念図/障害計画、今回の静的検査・単体テストを区別した。すべてのコマンドを未実行の雛形とする古い注意を修正した。
- probe.pyの成功条件はDevnet・server version 1.0.1・UTXO indexと疎通で、isSyncedは観測値である。同期完了・E2E成功へ読み替えない。
- mining.yamlでcommandが置き換わり、--nogrpcが外れ--enable-unsynced-miningが入ることを説明した。隔離Devnetの資金生成の例外と、一般の同期済み要件を区別した。
- send.cjsがgetServerInfoでDevnet/版/indexを確認する一方、同期済みを開始条件にしていないことを照合。既存profileのminerを先に止める具体的な手順、モードを1つずつ選ぶこと、既定プロジェクト名への依存を補った。
- 名前付きボリュームを停止時に消さず、再実行で以前の状態を再利用することを明記した。two-node.pyが実IPを取得する手順と、YAMLを単独upすることも区別した。
- 署名の検証対象をUTXO・金額・Script/署名対象として記載し、NetworkIdは接続検査として分けた。
- 観測ストアの世代は単一プロセス内の要求開始順で、分散ロックやノードのDAAではない。完全な成功スナップショットだけを反映し、エラー/部分取得を空集合として全件無効化しない。現在UTXOから切断中の全取引履歴を復元できるとは扱わない。
- 分断中の双方採掘は順に行った過去の検査であり、同時採掘を実測したと読み替えない。
- CI図を設計例と明記。現行deploy.ymlは本文同期・書籍・SQLite観測ストア単体テスト・2つのRust教材を検査し、Docker/SDK送金/実ノード障害注入は実行しない。
- 実Rust RPCアダプターが未実装であることと、固定版Node.js/WASM SDKを使う既存実習を区別した。
- Devnet READMEの「第12章の照合版は別」「タイムアウト再試行は未実施」という現在の状況と合わない記述を更新した。イメージのバイナリ/ソース一致は依然未証明である。

## Composeとスクリプトの検査

Docker Compose v2.31.0-desktop.2。config --format jsonの結果をメモリ内で検査し、次の5構成すべて終了コード0。

1. compose.yaml
2. compose.yaml + mining.yaml
3. two-node.yaml
4. two-node.yaml + partition.yaml
5. two-node.yaml + partition.yaml + reorg.yaml

検査時はプロジェクト名の環境上書きを外し、接続先IPの変数には検査用の値を与えた。コンテナ内で本当に接続できるかの検査ではない。全サービスのイメージdigest、ノードのDevnet/--appdir=/data/linux/arm64、ホスト公開が127.0.0.1のwRPCだけであることを確認した。単一ノードの元構成とmining上書きでgRPC/未同期採掘のフラグも検査した。

[Composeのmerge](https://docs.docker.com/compose/how-tos/multiple-compose-files/merge/)でcommandの上書き・相対パス基準を、[profiles](https://docs.docker.com/compose/how-tos/profiles/)でサービス選択の範囲を確認した。設定検査の成功はイメージ取得・起動・通信・採掘の成功ではない。

experiments/devnetの全cjsをnode --checkで構文確認した。実行やSDK接続は行っていない。SDK ZIPは現在の作業領域になく、今回ダウンロード/展開/チェックサムの再計測はしていない。本文の既知ハッシュは過去の取得記録に基づく。

識別用SHA-256。ファイル全体・全実行経路の監査を意味しない。

| ファイル | SHA-256 |
|---|---|
| `experiments/devnet/compose.yaml` | `ff3bd152d0afefff506f8d1a1d068e187639e2d6d55146abbe3c13ff437f894a` |
| `experiments/devnet/mining.yaml` | `625e3501b55f7be8610a9c29e4fe90a8d2b470e0ae3fb7167f3bee7d4fa999f4` |
| `experiments/devnet/two-node.yaml` | `727e581a370b76f3019bd8a78427ab75f2b5f862446f4f7c6355c753288c2cd5` |
| `experiments/devnet/partition.yaml` | `b91671fa6355078b3650faa9a2e4f0b121adfe6fdfaaf46d19419406ff3d4803` |
| `experiments/devnet/reorg.yaml` | `75bdfabe15ec3fad272f6703496f06e10910576c2d4b1504c4bcaffba3475720` |
| `experiments/devnet/probe.py` | `f625ab53f46b382e3feeb1a34fc0c322aabc1b1d5297ed8574f0777fa68c816c` |
| `experiments/devnet/send.cjs` | `175b59ae28dd59e19a58025fba3685c7383d9f1d7015360d33a2a93fe3d693c3` |
| `experiments/devnet/utxo-store.py` | `48b0b462123f71e0f4b7361e06908e376a652053e24a3987aec63f8656e31b09` |
| `.github/workflows/deploy.yml` | `9d6219c0b34546c8d63f224ba6951202582d27b58d8d93aef8e9b79a2a8b732a` |

## 独自Rust教材

本書外の一時Cargo libへ28.15/28.16を抽出。Rust/Cargo 1.90.0、edition 2021、外部依存なし。通常のRequireSyncedと、明示的なAllowUnsyncedDevnetLabを分け、どちらもDevnetと必要なUTXO indexを要求する。

```bash
cargo test --offline
cargo fmt --all -- --check
cargo clippy --offline --all-targets -- -D warnings
```

7テスト成功、fmt / Clippyも終了コード0。

1. 同期済みDevnetを通常の条件で許可。
2. Mainnet・Testnet-10/11・Simnetを両条件で拒否。
3. 通常条件は未同期を拒否。
4. 明示した隔離Devnet条件だけが未同期を許可。
5. Unreachable / Timeout / ProtocolRejectedを区別して保持。
6. 例外条件でもindex欠如を許可しない。
7. 同じGatewayの応答を取り直し、未同期・別Networkへの変化を見逃さない。

これは独自モデルであり、採掘設定を変更するコードや同期完了の証明ではない。server version・応答鮮度・TLS・index追従等も実装していない。実SDKの型へ直接対応する構造体ではない。

## 観測ストアの単体テストと実測記録

python3 -m unittest discover -s experiments/devnet -p 'test_*.py'で既存7テスト成功。SQLiteはTemporaryDirectoryを使い削除された。重複/削除・再読込・完全照会・古い結果の拒否等の検査であり、実RPC通知を今回受けたものではない。

本文の151ブロックの再起動保持、2ノードの27ブロック/報酬先26出力、分断後の62ブロック/指定出力46件、removedの10件等は2026-10-31の各記録に限定した過去の観測である。再実行時に同じ件数やhashが得られる保証ではない。全UTXO・実Pruning・Pruning後IBD・会計台帳・HD復旧を達成した結果へ広げない。

## 今回行っていないこと

Dockerコンテナの起動/停止、採掘、鍵生成・署名・送金、RPC/購読接続、分断・reorg、SDK取得・新規Python依存の導入は行っていない。ボリュームを削除せず、Devnet用CIも追加していない。ノードバイナリのソース一致、別OS/CPU、全Script/HD復旧、複数ワーカー/会計、実Pruning後IBD、全障害注入は未検証である。

## 書籍・公開対象の検査

- mdBook 0.5.4でビルド成功。生成HTML 53ページの内部ファイル・アンカー切れ0件。
- 既存教材2組がscripts/check-book-examples.pyで一致。実行したRustと本文の2断片も一致。新規examples/CI対象は追加していない。
- SUMMARYとreview-statusの本文表49項目が同じ順番。次は第29章。
- git diff --check成功。追跡対象・未追跡候補178ファイルの秘密鍵ブロック、既知トークン、認証付きURL、個人ホームパス等のパターン検出0件。gitleaks/trufflehogは未導入で、秘密情報の不存在を保証する監査ではない。
- 新規公開候補はレビュー記録のMarkdown。SDK・鍵・DB・生ログ・configの生出力・ビルド成果物は追加せず、コミット・pushもしていない。
