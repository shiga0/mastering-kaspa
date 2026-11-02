# gen1 HD鍵・アドレス復旧とUnicode入力の検証

実施日：2026-11-02（日本時間）。公式v1.0.1 Node.js/WASM SDK、Node.js 20.0.0、macOS ARM64。

## 固定したものと再実行

[実行方法](hd-recovery/README.md)、[ハーネス](hd-recovery/run.cjs)、[公開結果](hd-recovery/observed-2026-11-02.json)。SDK ZIPのSHA-256は `70679fe150689eedd588a475ae93f1135cfb99bd7b205dc2661d104d5db564d0`。実際に読み込むJS/WASM/型定義のハッシュをZIP内の同じファイルとも照合し、runnerでも固定値との一致を要求する。SDKとニーモニック等の復旧素材は収録しない。

経路の説明には基準ソース `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` の以下を使用し、ファイルのSHA-256を結果JSONへ保存した。

- [`wallet/keys/src/derivation/gen1/hd.rs`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/wallet/keys/src/derivation/gen1/hd.rs)：purpose 44、coin type 111111、account、受取/変更の経路。
- [`wallet/keys/src/privkeygen.rs`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/wallet/keys/src/privkeygen.rs)、`pubkeygen.rs`、`xprv.rs`、`xpub.rs`：SDKからの鍵導出の入口。
- [`wallet/bip32/src/mnemonic/phrase.rs`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/wallet/bip32/src/mnemonic/phrase.rs)：SDK `toSeed` と内部 `to_seed`。
- [BIP-39「From mnemonic to seed」](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki)（2026-11-02参照）：ニーモニック/追加パスフレーズのUTF-8 NFKD、PBKDF2-HMAC-SHA512、2048回、64 bytes。

今回は公式ソースからSDKを再ビルドしてバイナリ一致を証明したわけではない。公式配布ZIPの実行結果と、同じリリース基準ソースの読解を分ける。

## オフライン復旧の方法

12語と24語の英語ニーモニックを各1件ランダム生成し、空/ASCII/Unicode合成形/Unicode分解形の追加パスフレーズ4条件で実行した。各条件で作成用と復旧用の独立プロセスを順に起動し、作成用を終了した後に復旧用を開始する。親は復旧素材をメモリから匿名パイプへ渡すだけで、seed/xprv/鍵キャッシュは共有しない。合計16子プロセス、各60秒上限。

gen1の単一署名パス `m/44'/111111'/account'/branch/index` を使い、account 0/1、branch 0(receive)/1(change)、index 0/1/20/21/2147483647の各組み合わせを検査した。2147483647は通常導出の最大indexで、hardened指定ではない。

## 結果

8条件×20個の**160鍵・アドレス**が再構成できた（作成用/復旧用を合わせた主導出320個）。同一ニーモニックでも4種類の追加パスフレーズ間では各アドレスが変わった。

| 比較・検査 | 結果 |
|---|---|
| 作成用/復旧用の公開鍵・Devnet/Mainnet/Schnorr/ECDSAアドレス | 全160個で一致 |
| PrivateKeyGeneratorと明示XPrvパスの秘密鍵（メモリ内比較） | 両プロセスで一致 |
| アカウントxpub由来とmaster由来の公開鍵生成 | 両プロセスで一致 |
| ネットワーク/アドレス形式を変えたアドレス | 意図どおり異なる |
| 元の160メッセージ署名を復旧後の公開鍵で検証 | 全件成功 |
| 署名対象メッセージ変更、別indexの公開鍵 | 検証失敗 |
| 通常index/accountの2147483648、xpubからのhardened導出、不正パス | 拒否 |
| 不正checksumのMnemonic.validate / constructor | false / 拒否 |

メッセージ署名は固定の実験文言に `noAuxRand: true` を指定し、再現性も照合した。トランザクション署名・残高探索・送金の代替とはしない。SDKの複数API間の照合であり、BIP32/曲線演算全体を独立実装と比較した意味ではない。

## 見つかったUnicode正規化の差

固定版ソースの `to_seed` は `mnemonic` と追加パスフレーズを連結したsalt、およびニーモニック本体のバイト列を、そのままPBKDF2へ渡す。SDKでも以下を実測した。

- 英語ニーモニックのseedは、Node cryptoのraw入力PBKDF2と全条件で一致した。
- ASCII/空/分解形では、Node cryptoでNFKD化したBIP-39参照値とも一致した。
- 合成形 `caf\u00e9` では、NFKD参照値と一致しなかった。同じ表示の分解形 `cafe\u0301` と別seed・別アドレスになった。
- SDKへ渡す前に追加パスフレーズを明示的にNFKD化すると、Node cryptoのNFKD参照値と一致した。

BIP-39はNFKDを指定するため、これは固定版SDKを呼び出す際の互換性の差として記録する。テストの `allChecksPassed` はこの既知の差を再現できたことも含み、全面的なBIP-39適合を表さない。現行SDKやウォレットUIが事前に正規化するかまでは確認していない。

既存のraw入力で作ったウォレットを復旧する際、正規化を後から無条件に追加すると元とは別の鍵になる。作成時の版・正規化を含む入力処理と既知アドレスを照合する必要がある。第24章24.4.3へこの条件を追記した。

## 公開物と限界

親の公開出力は集計・SDK/ハーネス識別値・公開結果のハッシュだけ。ニーモニック、seed、秘密鍵、xprv、xpub、メッセージ署名そのものも公開結果には含めない。復旧素材をargv/env/ディスクへ書かず、子のstderrも親から公開しない。WASMオブジェクトのfreeや一部Bufferのゼロ化は行うが、JavaScript文字列を含む全コピーの消去保証ではない。ランダム素材を保存しないため、公開結果ハッシュは再実行で変わる。

未検証：legacy/gen0、マルチシグ/cosigner、機器対応、Wallet APIの暗号化DB/バックアップ、残高・取引履歴の再探索、gap limit/アカウント探索の停止条件、HD復旧鍵でのトランザクション署名・送金/受理。既知indexへの直接導出を、index 20/21をまたぐ実際の探索成功とは数えない。ネットワークprefixやアドレス形式の違いの検査も、対応ネットワークでの送金成功ではない。

第24章の冒頭・パスフレーズ・復旧節・残作業、および進捗表/ROADMAPを部分成功の範囲に合わせて更新した。C24全体をVerifiedにはしない。Docker/ノード/RPCを使わず、過去のランダム鍵Devnet送金は再実行していない。

## 収録時の検査

node --check、mdBook build、既存2教材の本文/ソース同期、git diff --checkが成功。結果JSONのハーネスハッシュ、SDKの固定値、公式ZIP内のファイルハッシュも一致した。内部workerへ不正な復旧素材を渡す負例では終了コード1、stdout空、stderrは固定メッセージのみで、入力や例外スタックを出力しないことを確認した。

後続の限定した実ノード試験として、[HD復旧鍵のDevnet送金](hd-send-review-2026-11-02.md)で受取鍵・変更鍵による2件の送金受理を確認した。本記録の「未検証」は上記オフライン試験の範囲を表す。
