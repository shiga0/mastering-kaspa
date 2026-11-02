# 新規Devnet環境からの実習再現

実施日：2026-11-02（日本時間）。対象は基準版v1.0.1 / Devnet。

## 環境と準備

macOS 15.7.3 / Apple Silicon arm64、Docker Engine 27.4.0、Python 3.11.0、Node.js 20.0.0。Docker Desktopを起動し、既存の実験コンテナがすべて停止していることを確認した。既存のOS・Docker・取得済みコンテナイメージを使うため、端末全体の新規セットアップ試験ではない。

実習READMEどおりに `.venv` を新規作成し、websockets 14.1を導入。公式v1.0.1 SDK ZIPを新規取得し、SHA-256 `70679fe150689eedd588a475ae93f1135cfb99bd7b205dc2661d104d5db564d0` の一致後に展開。websocket 1.0.34を導入した。SDK・鍵・DB・生ログは収録しない。推移依存の固定は今回の対象外。

## 見つかった問題と修正

新規ボリュームの名前だけをComposeへ上書きして `up -d node` を実行すると、既存コンテナが再利用され、実マウントは旧 `mastering-kaspa-devnet_node-data` のままだった。probeの5475ブロックによって検出。この段階の送金試験も成功したが、新規環境の結果には数えない（既存環境の再実行として扱う）。

`fresh.yaml` でボリューム名を必須指定し、`up -d --force-recreate node` で再作成した。`docker inspect` で `/data` が未使用だった `mastering-kaspa-fresh-20261102` に切り替わったこと、probeでブロック数・DAA scoreがともに0であることを確認した。旧ボリュームは削除していない。READMEへ初期状態・マウント確認・再作成・停止・旧DBへの復帰手順を追加した。

## 実測

公開結果は [fresh-observed-2026-11-02.json](devnet/fresh-observed-2026-11-02.json)。

1. `probe.py`：Devnet、serverVersion 1.0.1、UTXO index有効、初期blockCount/DAA score 0。
2. `send.cjs --checks`：ランダム一時鍵へ採掘し、成熟期間100 DAAを満たす入力をscore 102で取得。1 KASを署名送金した。
3. Mempool掲載、virtual chain受理、入力消費、送金先UTXOを確認。手数料10,000 sompi、お釣り49,899,990,000 sompiが独立算術・SDK・観測値で一致。
4. 残高不足・不正署名・競合送金を想定どおり拒否。不正署名の拒否後も入力が保たれた。
5. 送金用マイナーの自動停止後、READMEの公開報酬先へComposeマイナーで10秒追加採掘し、finally節で停止した。
6. `persistence.py` の出力を一時領域へ保存し、ノードをrestart。期限付きprobeで復帰を待ち、`persistence.py --compare` が成功。165ブロック・DAA score 165・tips・公開報酬先の40件のUTXO全内容が一致した。
7. 再起動後の `getUtxosByAddresses` で、送金結果の宛先・TX IDに対応する100,000,000 sompiのUTXOを別途確認。
8. Composeのnode/minerを停止。nodeはexit 0、minerは停止期限後exit 137。`docker ps --filter name=mastering-kaspa-send-` は0件で、実習の採掘コンテナは残っていない。ノードDBは新旧とも保持。

再実行は実習READMEの「新規ボリュームから再現する」と「短時間採掘と再起動後の状態比較」に従い、後者の各Compose呼び出しにもfresh.yamlを追加する。送金先の保持確認は、送金JSONのdestinationをRPCで照会し、transactionIdとamountを照合する。値・ハッシュ・件数は再実行で変わる。

## 範囲と残作業

同一ホスト上の新規Python/SDK環境・新規ノードDBと、既存DBでの送金を再確認した。E9/A2の部分進捗であり、新規OS/Docker導入、他OS/CPU、Pruning後IBD、Finality境界、HD復旧、複数ワーカー/会計、クラッシュ耐性、ブロック包含の独立照会は未検証。E8の共通自動実行・すべての失敗経路の後片付けも未完了。タイムアウト後再試行モードは今回は再実行していない。

進捗文書の「付録未コミット」をc86e88eでの保存済みへ修正し、READMEの異常系/2ノード未完了という古い要約も既存記録に合わせた。過去日付の実験記録は当時の状態として保持する。

検査：mdBook build、本文/サンプル2件の同期検査、SQLite観測ストア7テストが成功。
