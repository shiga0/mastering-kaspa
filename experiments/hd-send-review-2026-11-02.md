# HD復旧鍵によるDevnet署名送金

実施日：2026-11-02（日本時間）。公式v1.0.1 Node.js/WASM SDK / kaspad 1.0.1 / Devnet。Node.js 20.0.0、macOS ARM64、Docker Engine 27.4.0。

## 目的と対象

[先行するオフライン復旧](hd-recovery-review-2026-11-02.md)の160鍵/アドレス再構成とメッセージ署名に続き、復旧したgen1鍵で実際にUTXOを支出できるかを確認した。対象は新しい一時24語ニーモニック・ASCII追加パスフレーズ・単一署名の限定構成。

- 受取：`m/44'/111111'/1'/0/21`
- 変更：`m/44'/111111'/1'/1/20`
- 宛先：`m/44'/111111'/0'/0/0`

宛先も同じ一時HDルートに属する別アカウントで、外部サービスへの支払ではなくテスト用の内部送金である。残高探索は既知のパス/アドレスを直接照会しており、未知indexの再発見ではない。

[実行手順](hd-recovery/README.md)、[ハーネス](hd-recovery/send.cjs)、[公開結果](hd-recovery/send-observed-2026-11-02.json)を保存した。SDKの固定JS/WASM/型定義ハッシュは先行実験と同じ。ハーネスのSHA-256とノード/マイナーのdigestを結果JSONへ記録する。

## 実行した経路

1. 親が一時ニーモニックを生成し、作成用workerへ匿名パイプで渡す。workerは公開アドレスだけを返して終了。親はHD秘密鍵やxprvを作成しない。
2. 停止中だった単一ノードを `fresh.yaml` と `--force-recreate` で起動し、実際の `/data` のマウント先を確認。既存の新規環境試験用ボリュームを保持して再利用した。空DBでの実験ではない。
3. Devnet/1.0.1/UTXO indexを検査し、受取アドレスへ最大1CPUで採掘。Coinbase成熟100 DAAを満たす入力を、virtual DAA score 352で取得した。採掘を停止する。
4. 受取から1 KASを宛先へ送り、お釣りを変更アドレスへ戻す未署名TXを親が構築。誤った追加パスフレーズのworkerは、既知アドレスとの不一致で署名前に拒否された。
5. 正しい復旧素材を別workerへ渡す。`Mnemonic.toSeed` → `XPrv.derivePath` → `toPrivateKey` で受取鍵を復元し、アドレス一致後に `signTransaction(tx, [key], true)` で署名/ローカル検証する。署名済みTXだけを親へ返して終了。
6. 親が署名以外のTX項目を未署名TXと照合し、署名済みTXを送信。Mempool掲載を確認後に採掘再開。virtual chain受理ID・宛先UTXO・変更UTXO・入力消費を確認し、再び採掘を停止する。
7. 第一TXの変更UTXOだけを入力に、さらに1 KASを送る未署名TXを構築。新しい復旧workerで変更鍵を復元し、同じ照合・署名・送信・受理・入力消費・お釣りを確認した。
8. ノードと専用マイナーを停止し、終了後にノードRunning=falseと専用マイナー稼働0件を確認した。既存/実験ボリュームは削除していない。

署名APIは基準版 [`wallet/core/src/wasm/signer.rs`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/wallet/core/src/wasm/signer.rs) と照合した。verify_sig=trueはPopulatedTransactionに対するローカル署名検証を要求する。今回ノード側の実受理も別途観測したが、全Signer経路や鍵消去の監査ではない。

## 観測値

| 項目（sompi） | 復旧受取鍵の第一TX | 復旧変更鍵の第二TX |
|---|---:|---:|
| 入力 | 50,000,000,000 | 49,899,990,000 |
| 宛先 | 100,000,000 | 100,000,000 |
| 手数料 | 10,000 | 10,000 |
| 変更出力 | 49,899,990,000 | 49,799,980,000 |

入力＝宛先＋変更＋手数料が一致し、手数料はSDK値と算術値、お釣りは算術値と実UTXOで一致。massを独立実装で再計算した試験ではない。第二TXの入力Outpointが第一TXの変更Outpointそのものであることも照合した。公開JSONにTX ID・Outpoint・virtualのaccepting block hashを保存した。

両TXで、復旧した署名のローカル検証、Mempool掲載、virtual受理、入力消費、宛先/変更UTXOを確認。これは観測時点の受理で、不可逆な決済確定ではない。

## 終了と公開物

条件待機は開始から240秒、RPCは10秒、workerは30秒、Docker呼び出しは20秒。専用マイナーは各段階で停止し、finallyとSIGINT/SIGTERMでもノード/マイナーを停止する構成。今回の正常終了では停止状態まで確認したが、全失敗分岐・SIGKILL・Docker障害の後片付けを検証した意味ではない。

復旧素材は匿名パイプとメモリで扱い、argv/env/ディスクへ書かない。署名済みTXの全体も収録せず、公開アドレス・ID・数値・検査結果だけを残す。例外スタックや子stderr・Docker生ログを公開しない。WASMのfreeはJavaScript文字列等を含む全コピーの消去保証ではない。workerは実験用で、本番の承認ポリシーを備えた署名デバイスではない。

## 残る範囲

未知indexの残高/履歴探索、gap limitとアカウント探索、暗号化DB/バックアップ復旧、legacy/gen0・マルチシグ・機器、全パス/全パスフレーズ条件、複数ノード・再編成・Finality・Pruning後IBDは未検証。Unicode正規化の差は先行オフライン実験の結果を維持し、今回のASCII試験からUnicodeでの送金復旧全体へ広げない。

第24章・進捗表・ROADMAPの「復旧鍵での送金が未実施」を、この2件の限定成功へ更新した。C24全体のVerifiedやE8全失敗経路の完了にはしない。CIの実行対象も変更しない。

## 収録時の検査

node --check、mdBook build、既存2教材の本文/ソース同期、git diff --checkが成功。オフライン/送金の両結果JSONが現在のハーネスSHA-256と一致することを確認した。別途Dockerを照会してnodeがexited、HD送金用マイナーの稼働0件、新旧の単一ノード用ボリュームが保持されていることも確認した。
