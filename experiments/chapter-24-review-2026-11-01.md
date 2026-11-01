# SUMMARY順レビュー 28：第24章「ウォレットアーキテクチャと鍵管理」

確認日：2026-11-01。基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。本文・図・出典・実習への導線と入力予約教材を確認した。全ウォレットや鍵保護の監査ではない。

## 本文・固定ソースの確認

- 独立した鍵を増やす方式とHD導出を区別した。単一鍵から受取先を増やすには必ず別の秘密情報が必要、という誤読を招く説明を修正した。
- Address Prefixはkaspa / kaspatest / kaspasim / kaspadevを区別するが、完全なNetworkIdではない。kaspatestからTestnet番号を識別できないため、接続ノードの番号まで照合する。
- gen1のbuild_derivate_pathは単一署名purpose 44、multisig 45、coin type 111111とaccountを使い、必要に応じcosigner・address typeを追加する。create_addressはECDSAの圧縮公開鍵とSchnorrのx-onlyを区別する。これはgen0/legacyを含む共通仕様や復旧実測ではない。
- Signer / KeydataSignerのtry_signは、鍵配列を作り、sign_with_multiple_v2(...).fully_signed()?の後でzeroizeする。fully_signedがErrなら消去行に到達しない制御フローを補った。これは実鍵を使うメモリ残留試験ではない。鍵キャッシュ全体・解放後メモリ・すべての失敗経路まで保証しない。
- Generatorの段階分割は以前の主要経路照合と今回のモジュール説明の再確認を利用。全生成経路を新たに走査・実行したものではない。古い「ランダム鍵のDevnet送金は後続作業」という記述を、既存の実測と未実施のHD復旧へ分け直した。
- オフライン署名の確認対象に参照UTXO・変更アドレスの自分の導出経路・SIGHASHフラグを補足した。製品・ファームウェアの対応を新たに検証した結果ではない。
- 送信成否不明の状態を表へ追加。予約ID・TX ID・支払い意図を対応付け、再編成後の状態再評価と業務記録を分けた。期限切れやRPCタイムアウトだけで予約を解除しない。
- ソース探索を第1章の固定checkout再利用へ変更。入力予約の将来課題と、既に実装した予約IDが食い違わないよう整理した。

既存の固定版ソースアーカイブの該当箇所を読み、gen1・Signer・Generatorのファイルハッシュは以前の照合記録とも一致した。アーカイブはGit checkoutではなく、この回にgit rev-parseでHEADを取得したものではない。SHA-256は識別用であり、ファイル全体の監査を表さない。

| ファイル | SHA-256 |
|---|---|
| `wallet/keys/src/derivation/gen1/hd.rs` | `2c585e06a671a2b1009c06b334077d814f8930003dc4ebb54f87df256d932f5b` |
| `wallet/core/src/tx/generator/signer.rs` | `3e3195d2457abdf18bac93553fc9aadfed5e8b0a99a27ac92c882c13e4b528c5` |
| `wallet/core/src/tx/generator/generator.rs` | `9bf481fed8d22e85fe937d951e9f5990fbeefe239cffbdbf80ea4111efabf3df` |
| `crypto/addresses/src/lib.rs` | `81e3797a77caaa5170657bd327ec2fe392def9dc549cb6697f140549effb6030` |

## 復旧・公開情報に関する一次資料

- [BIP-32](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki)：xpubは非hardenedの子公開鍵を導出でき、プライバシー上の関連付けを可能にする。親xpubと対応する非hardened子秘密鍵の組合せから親秘密鍵を求められる性質もあり、xpubを無条件に公開してよいとは扱わない。xpubだけで署名できるという意味ではない。
- [BIP-39](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki)：追加パスフレーズが違っても有効な別seedを得られるため、エラーが出ないことを復旧成功の証拠にしない。ファイル暗号化・解除パスワードとは役割を分ける。
- [BIP-44](https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki)：探索は現在残高だけではなく取引履歴に基づく。gap limitや探索停止条件によって離れた使用済みindexを見落とし得る。このBIPの探索規則を全Kaspaウォレットへ一律適用しない。

仕様の文脈を確認したもので、BIPのテストベクトルや実際のseed/秘密鍵を本書へ追加せず、鍵導出・HD復旧は実行していない。

## 予約教材の修正

同一インスタンス内で再利用しない予約IDを発行し、Outpoint→予約IDの対応で所有する。releaseはIDに対応する予約だけを解除する。同じ出力の予約解除→再予約後に古い解除が遅れて届いても、新しい予約を外さない。

空入力、重複入力、不明UTXO、既存予約、合計額overflow、ID上限を更新前に検査する。insertは初期投入用とし、同じOutpointを黙って上書きしない。ErrでUTXO・予約・IDカウンターが維持されることを比較した。

IDは別インスタンス・別ネットワーク・再起動をまたいで一意ではなく、認証トークンでもない。Cloneは状態比較のためで、複製した送金ワーカーの正しさを保証しない。&mut selfによる直列呼び出しの教育用モデルであり、排他制御・DB・再編成の反映・入力成熟・Script・NetworkId・署名を実装しない。通常のErrでのメモリ内状態維持は、panic・クラッシュ時の原子性の保証ではない。

## 実行した検査

本文のコードを本書のリポジトリ外の一時Cargoプロジェクトへ抽出。Rust/Cargo 1.90.0、edition 2021、外部依存なし。rustfmtを本文へ反映した。

```bash
cargo test --offline
cargo fmt --all -- --check
cargo clippy --offline --all-targets -- -D warnings
```

11テスト成功、fmt / Clippyも終了コード0。

1. 1回目の予約の合計と二重予約拒否。
2. 重複入力の失敗で全状態を維持。
3. 不明入力の失敗で全状態を維持。
4. 空入力で予約IDも消費しない。
5. 解除後に新IDで再予約。
6. 古い解除要求が新しい予約を外さない。
7. 複数入力の予約を解除しても別の予約を維持。
8. 一部が既に予約済みの入力集合で全状態を維持。
9. 合計額overflowでも既存予約を維持。
10. 予約済み出力への重複insertで金額を変更しない。
11. IDカウンターの上限で循環せず全状態を維持。

## 既存実測と今回の制限

`devnet-send-review-2026-10-31.md`、`devnet-send-checks-2026-10-31.md`、`devnet-send-retry-2026-10-31.md`は、公式SDKのランダム鍵による単一署名送金、所定の拒否、成功応答喪失と再照会などの過去の検査である。gen1 HD導出や本教材の予約APIを実ノードへ組み込んだ結果ではない。

今回は鍵・シード・xpubの生成、署名、送金、ノード起動・採掘を行っていない。gen0/legacy互換性、HD復旧、探索範囲の実動、鍵キャッシュの寿命や全経路の消去、任意のGenerator経路、マルチシグ・ハードウェア機器は未検証。ウォレットの会計台帳・多プロセス・再起動をまたぐ予約復元も未実装。

## 書籍・公開対象の検査

- mdBook 0.5.4でビルド成功。生成HTML 53ページの内部ファイル・アンカー切れ0件。
- 既存の教材2組がscripts/check-book-examples.pyで一致。手動教材をCIへ追加していない。
- SUMMARYとreview-statusの本文表49項目が同じ順番。次は第25章。
- 実行したRustと本文のコードが一致。git diff --check成功。
- 追跡対象・未追跡候補174ファイルの秘密鍵ブロック、既知トークン、認証付きURL、個人ホームパス等のパターン検出0件。拡張公開/秘密鍵の長いBase58文字列も追加検索し0件。gitleaks/trufflehogは未導入で、秘密情報の不存在を保証する監査ではない。
- 新規公開候補はレビュー記録のMarkdown。鍵・シード・xpub・DB・生ログ・生成物は追加せず、examplesに新規フォルダーも作っていない。コミット・pushはしていない。
