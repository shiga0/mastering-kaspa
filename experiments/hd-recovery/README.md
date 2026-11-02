# gen1 HD鍵・アドレスのオフライン復旧

公式v1.0.1 Node.js/WASM SDKで生成した一時的なニーモニックを、独立したNodeプロセスへ渡し、gen1の鍵・アドレスとメッセージ署名を再現する。実行ごとにテスト素材を生成し、ニーモニック・seed・秘密鍵・xprv・xpubは公開結果へ含めない。

## 実行

[Devnet実習のSDK取得手順](../devnet/README.md)で公式v1.0.1 SDK ZIPを取得・SHA-256照合・展開する。この実験はWebSocket依存を使わず、Docker/ノード/RPCも不要。確認環境はNode.js 20.0.0 / macOS ARM64。

```bash
KASPA_SDK_ROOT="$PWD/experiments/devnet/.sdk" node experiments/hd-recovery/run.cjs
```

runnerはSDK packageの版とJS/WASM/型定義のSHA-256を検査する。標準出力は集計JSONのみ。`--worker` は内部IPC用で、個人の復旧素材を入力するインターフェースとして使わない。

## 条件

- ランダムな英語12語/24語ニーモニックを各1件。
- パスフレーズ4条件：空、ASCII、Unicode合成形、Unicode分解形。
- account 0/1、receive/change、index 0/1/20/21/2147483647。
- gen1単一署名パス：`m/44'/111111'/account'/branch/index`。
- 各条件で作成用と復旧用の2プロセスを順に起動し、合計16プロセスで実行。各プロセスは60秒上限。

復旧素材は親プロセスのメモリから子の標準入力の匿名パイプへ渡し、引数・環境変数・ファイルへ書かない。子の標準出力には公開鍵/アドレス/署名と検査結果のみを返し、親は比較後、集計と公開結果のハッシュだけを出力する。SDKの例外内容やスタックは出力しない。WASMオブジェクトをfreeしても、JavaScript文字列やランタイムの全コピーを消去できた保証にはならない。

160個の復旧鍵について、以下を比較する。

1. 作成用プロセスと復旧用プロセスの鍵由来の公開結果が一致する。
2. `PrivateKeyGenerator` と明示パスの `XPrv.derivePath` の秘密鍵が一致する（メモリ内比較のみ）。
3. アカウントxpubからの `PublicKeyGenerator.fromXPub` と、master由来の公開鍵生成が一致する。
4. ネットワークprefix/アドレス形式を変えると符号化したアドレスが変わる。
5. 元の鍵で署名した固定メッセージを、復旧した公開鍵で検証できる。メッセージ変更・別indexの公開鍵では検証に失敗する。
6. 通常indexの上限超過、accountの上限超過、xpubからのhardened導出、不正パス、不正checksumのvalidate/constructorが拒否される。

SDK以外の暗号実装として比較するのはNode cryptoのPBKDF2-HMAC-SHA512。EC/BIP32全体の独立実装比較ではない。index 20/21等は既知indexへの直接導出で、gap limitの探索試験ではない。

## Unicodeの結果

固定版 `Mnemonic.toSeed` は、追加パスフレーズのNFKD正規化を行わず、入力バイト列をPBKDF2へ渡した。`caf\u00e9` と `cafe\u0301` はNFKDでは同じになるが、そのまま渡すと別seed・別アドレスになった。Node cryptoでraw入力から求めたseedとは一致し、合成形ではBIP-39のNFKD参照値と一致しない。明示的にNFKD化してからSDKへ渡すと参照値と一致した。

これは**固定版の既知の差を再現できたこと**を成功条件に含めている。`allChecksPassed` はSDKのBIP-39全面適合を意味しない。既存のウォレットがraw入力で作られていた場合、復旧時に正規化を追加すると別の鍵になるため、作成時の版・入力処理・既知アドレスとの照合が必要。

実測は [observed-2026-11-02.json](observed-2026-11-02.json)、出典と本文への反映は [検証記録](../hd-recovery-review-2026-11-02.md)。ランダム素材を公開しないため、公開結果ハッシュは実行ごとに変わる。

## 残る範囲

legacy/gen0、マルチシグ/cosigner、機器対応、Wallet APIのDB/暗号化バックアップ、残高・履歴の再探索、gap limit/未使用アカウントの停止条件、トランザクション署名・送金/受理はオフライン `run.cjs` の対象外で、下記の `send.cjs` による別試験を参照。過去のランダム鍵Devnet送金とも別の検査。CI対象も変更しない。

## 復旧した鍵でDevnet送金する

`send.cjs` は、このオフライン試験とは別の実ノード検証。作成用プロセスを終了してから資金を準備し、別プロセスで復元した受取鍵で1 KASを送る。お釣りを変更枝へ戻し、さらに新しい復旧プロセスで変更鍵を復元して、そのお釣りからもう1 KASを送る。宛先も同じ一時HDルートの別アカウントで、テスト用の内部送金である。

前提：Dockerが起動済み、SDKと `websocket@1.0.34` は [Devnet実習](../devnet/README.md) の手順で導入済み。実行場所はリポジトリのルート。既定の単一ノード実習が動いている場合は起動前に拒否する。他の実習と同時実行しない。

```bash
KASPA_SDK_ROOT="$PWD/experiments/devnet/.sdk" KASPA_FRESH_VOLUME=mastering-kaspa-fresh-20261102 node experiments/hd-recovery/send.cjs
```

`KASPA_FRESH_VOLUME` は使用する実習用ボリューム名。名前が存在すれば保持済みデータを使い、なければ作成する。`fresh.yaml` と `--force-recreate` でマウントを確定し、`/data` の実ボリューム名も検査する。空DBが前提の試験ではない。既存ボリュームは削除しない。

- 24語・ASCII追加パスフレーズを内部生成。受取はaccount 1 / index 21、変更はaccount 1 / index 20、宛先はaccount 0 / receive index 0。
- 作成用workerは公開アドレスだけを返し終了する。親はその後に採掘・Coinbase成熟待機・未署名TX構築を行う。
- 復旧workerが同じ素材から鍵を再構成し、既知アドレスを照合後に `signTransaction(..., true)` で署名/ローカル検証する。誤った追加パスフレーズはアドレス不一致として署名前に拒否する。
- 親は署名以外のTX項目が元の未署名TXと一致することを確認し、送信・Mempool・virtual受理・入力消費・宛先UTXO・変更UTXOと手数料を照合する。
- 二回目の入力Outpointが、一回目の変更出力Outpointそのものであることも確認する。

条件待機は実験開始から240秒、RPCは10秒、各workerは30秒、Docker呼び出しは20秒を上限とする。採掘は最大1CPUで、成熟待機と各TX受理待機時だけ起動する。通常終了・例外・SIGINT/SIGTERMでは専用マイナーと実習ノードを停止する。SIGKILLやDocker障害では自動停止できないため、以下で該当する実験だけを停止・確認する。

```bash
docker ps --filter name=mastering-kaspa-hd-send-
# 残った該当マイナーだけを停止する
docker stop --time=2 <表示された実験マイナー名>
KASPA_FRESH_VOLUME=mastering-kaspa-fresh-20261102 docker compose -f experiments/devnet/compose.yaml -f experiments/devnet/mining.yaml -f experiments/devnet/fresh.yaml stop -t 2 node
```

ノードのボリュームは終了後も保持する。送金署名済みTXはworkerから匿名パイプで親へ返すが、公開結果へは収録しない。このworkerは本番向けの承認ポリシーを備えた署名デバイスではない。秘密情報・生ログの扱いとJavaScriptメモリ消去の限界は上記と同じ。

再現記録は [HD復旧鍵のDevnet送金](../hd-send-review-2026-11-02.md)、公開結果は `send-observed-2026-11-02.json`。残高/履歴の未知index探索やgap limit、暗号化バックアップ、複数ノード/再編成、最終決済確定の検証とは区別する。

## アドレス/アカウント探索の境界

`discovery.cjs` は公式 `Wallet.accountsDiscovery` を、127.0.0.1の固定UTXO応答サーバーに接続して実行する。Docker・採掘・送金は不要。SDKとWebSocket依存は上記のものを使う。

```bash
KASPA_SDK_ROOT="$PWD/experiments/devnet/.sdk" node experiments/hd-recovery/discovery.cjs
```

公式APIの14ケースを `discovery-vectors.json` に保存した。ランダムな英語12語ニーモニックからgen1のaccount 0〜4、受取/変更index 0〜21の公開アドレスを作成し、公式APIが実際に照会した範囲・順序・戻り値を手で付けた期待値と比較する。合成UTXOの所在を公式APIへ教えることはしない。resident=trueを指定し、追加パスフレーズは省略する。

- `addressScanExtent=20` は両枝のindex 0〜19。20/21は含まれず、21/22へ広げると拾える。
- `accountScanExtent=2` の初期探索はaccount 0/1。2にだけ資金があるfixtureは見落とし、extent 3へ広げると検出する。
- 資金のある最後のaccountが1なら、account extent 2では0/1/2を照会する。終了条件は `account_index < last_account_index + account_scan_extent` で、単純に「指定数の空accountを最後に調べる」とは言わない。
- UTXOがないfixtureとaccount 0にUTXOがあるfixtureは、戻り値がどちらも0。spent-onlyの履歴ラベルを付けても、RPCが返す現在UTXOが空なら検出しない。
- 0のaddress/account extentも0を返す。空のアドレス要求ではaccountの識別ができないため、公開記録はaccount=nullと要求数で表す。

公式APIは固定範囲の現在UTXO探索であり、BIP-44の履歴ベースのgap limitと同じ処理ではない。別の `discovery-model.cjs` で教育用の連続未使用アドレスの停止方針を実装し、手動期待値8例と、空区間を列挙する別モデルとの12,288比較を行う。使用済みindex 0の残高がゼロでも、履歴の使用記録を残せばgapをリセットできる例を含む。このモデルは履歴を外から与える前提で、実履歴の取得は実装しない。

RPCの要求IDはu64でJavaScriptの安全な整数範囲を超え得るため、テストサーバーはIDの元のJSONトークンを保って返す。Numberへ変換したIDをJSON.stringifyで返すと、丸めによって対応する要求へ返せなくなる。runnerはこの負例と、実際の大きなIDの返送も検査する。

各実行は子プロセス内で45秒、親から60秒の上限。サーバーは127.0.0.1の空きポートにだけbindし、終了時にWallet/RPC接続とサーバーを閉じる。親はSDKの生ログを捨てて集計JSONだけを出力する。復旧素材、公開鍵/xpub、アドレスの実値は公開結果に含めない。

結果は `discovery-observed-2026-11-02.json`、[検証記録](../hd-discovery-review-2026-11-02.md)を参照。これは**公式SDKを実行した固定RPC試験**で、実ノードの支出済み履歴復元、Indexerとの接続、Pruning後の履歴保持、実ウォレットの完全復旧を検証したものではない。
