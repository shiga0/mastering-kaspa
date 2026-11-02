# HDアカウント探索APIとgap limitの境界

実施日：2026-11-02（日本時間）。公式Node.js/WASM SDK v1.0.1、Node.js 20.0.0、macOS ARM64。固定ソースは `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。

## 対象と方法

[オフライン鍵復旧](hd-recovery-review-2026-11-02.md)、[復旧鍵によるDevnet送金](hd-send-review-2026-11-02.md)に続き、既知のパスを直接導出する段階から、アドレス/アカウントの探索範囲と停止条件へ進めた。

[実行手順](hd-recovery/README.md)、[公式API用ハーネス](hd-recovery/discovery.cjs)、[入力14ケース](hd-recovery/discovery-vectors.json)、[結果と識別値](hd-recovery/discovery-observed-2026-11-02.json)を収録する。

公式 `Wallet.accountsDiscovery` をresident=true / Devnetのアドレス設定で実行し、127.0.0.1の固定RPCサーバーへ接続する。サーバーは `getUtxosByAddresses` だけを受け付け、fixtureで指定した公開アドレスに合成UTXOを返す。ノード・採掘・送金は使わない。公式APIのループ・鍵導出・RPC呼び出しは実行するが、コンセンサスや本番Indexerは対象外。

新しい英語12語ニーモニックから、account 0〜4、受取/変更、index 0〜21の計220アドレスを事前生成。公式APIが照会したアドレスをこの表へ照合し、accountの順序・各枝の範囲・重複なし・戻り値を手で付けた期待値と比較した。公式APIへfixtureの資金所在や期待するaccountは渡していない。

## 固定ソースと仕様の区別

- [`scan_bip44_accounts`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/wallet/core/src/wallet/mod.rs)：現在UTXOによるaccount探索、最後に検出したaccountとextentによる継続条件。
- [`Bip32::get_address_range_for_scan`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/wallet/core/src/account/variants/bip32.rs)：受取と変更の両枝で `0..address_scan_extent` を列挙。
- `wallet/core/src/api/message.rs`、`wallet/core/src/wasm/api/message.rs`：入力と戻り値の境界。上記ファイルのSHA-256も結果へ保存した。
- [BIP-44 Account discovery / Address gap limit](https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki)（2026-11-02参照）：外部枝の履歴を基準にしたaccount探索と、連続未使用20アドレスの停止条件。

BIP-44の履歴ベースの方針を、固定版Kaspa APIの現在UTXOによる固定範囲探索へ読み替えない。API名のBip44だけで同じ探索方式と断定しない。

## 公式APIの結果

14ケースが期待した要求範囲・account順序・戻り値と一致した。

| 条件 | 観測 |
|---|---|
| address extent 20、account 1 receive/change index 19にUTXO | account 1を検出 |
| 同じ範囲でreceive/change index 20にだけUTXO | 検出せず0 |
| address extent 21へ拡大 | index 20を検出 |
| address extent 22、receive index 21 | 検出 |
| account extent 2、account 2にだけUTXO | 0/1で止まり検出せず0 |
| account extent 3へ拡大 | 0〜4を照会しaccount 2を検出 |
| 全UTXO空 / account 0にUTXOあり | 両方とも戻り値0 |
| 支出済み履歴ラベルあり、現在UTXO空 | 空の場合と同じ0 |
| address extent 0 / account extent 0 | 0。前者は空アドレス要求2回、後者は要求なし |

空アドレス要求からaccountを識別することはできないため、ゼロaddress extentの公開結果ではaccount=nullにした。ラベルだけから「0/1が照会された」とは記録しない。

`account_index < last_account_index + account_scan_extent` が継続条件。last=1、extent=2なら照会は0/1/2で終了し、最後の資金accountより後の空accountは1個。extentをそのまま「最後に連続して調べる空accountの件数」と説明しない。

戻り値0には未発見/初期値とaccount 0を検出した場合が含まれ、0だけで残高不存在を証明できない。spent-only fixtureの履歴ラベルは比較用の既知条件で、公式RPCへ履歴データを返したわけではない。実際の支出済みウォレットや履歴Indexerでの検証と区別する。

## gap limitの別モデル

[教育用モデル](hd-recovery/discovery-model.cjs)は使用済みindexの集合を外から受け取り、連続未使用数を数えて止める。公式SDKの探索関数の代わりとして扱わない。

- gap=20で使用済みindex 19は拾うが、0〜19が未使用なら20/21へ進まない。
- index 0が支出済み、index 20にUTXOがある場合、現在UTXOだけの集合 `[20]` では20に達しない。履歴による使用済み集合 `[0,20]` なら0でgapをリセットし、20を拾える。
- `[0,21]` では1〜20の20連続未使用で止まり、21を見落とす。
- 探索上限22、gap 23なら21を拾えるが、停止理由はgapではなく明示上限。範囲外に資金がない証拠にはしない。

手動期待値8ケースが一致。さらにindex 0〜7の全256使用パターン×gap 1〜4×上限1〜12の12,288比較で、カウンター方式が空区間列挙モデルと一致した。不正なgap/上限/indexの4ケースは拒否した。履歴取得を実装した試験や全BIP-44適合テストではない。

## 実行・公開物

SDKのJS/WASM/型定義を前回と同じSHA-256で検査。RPC IDはu64のJSON数値トークンをそのまま返し、JavaScript Numberで丸めない。`9007199254740993` をNumberへ変換すると別の値になる負例を検査し、実際の大きな要求IDでも公式APIが完了することを確認した。

復旧素材は子のメモリのみ。resident Walletを使い、ニーモニック・seed・秘密鍵・xpub、公開アドレスの実値、生ログは収録しない。子45秒・親60秒の上限を設け、Wallet接続とRPCサーバーを閉じて終了。親が公開するのは専用prefixの集計JSONのみ。

## 残る範囲

実ノード上の未知index探索、支出済み履歴を保持するIndexerとの照合、Pruning後の履歴復元、暗号化バックアップ/Wallet DBの復旧、全account範囲・legacy/機器対応・再編成時の探索は未検証。第24章へ今回の条件・0の解釈・固定範囲とgapの違いを追記するが、ウォレット全体の復旧完了とはしない。

## 収録時の検査

公式14ケースの戻り値・account要求順序が入力の期待値と一致し、ハーネス/モデル/入力のSHA-256が公開結果と一致することを再確認した。node --check（2ファイル）、mdBook build、既存2教材の本文/ソース同期、git diff --checkが成功。RPCサーバー閉鎖後に集計を出力し、子プロセスの終了コード0まで確認した。
