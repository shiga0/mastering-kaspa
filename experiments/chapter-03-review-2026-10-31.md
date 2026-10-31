# SUMMARY順レビュー 07：第3章「暗号技術の基礎」

確認日：2026-10-31。対象は `src/part-01-foundations/chapter-03.md`。暗号概説と基準版の該当ソースを照合し、SHA-256教材を本文から抽出して実行した。

## 修正点

- 正しい命題を「誤解」としていた見出しを、「異なる入力は必ず異なるハッシュ値になる」へ修正。同じハッシュ関数・設定という比較条件を補った。
- 原像は指定ハッシュになる入力を探す問題とし、信頼できる照合先ハッシュの必要性、エンコード・暗号化・署名の区別を補った。
- ドメイン分離の連結図と実バイト列を区別。基準版のBLAKE2b用途タグ・出力長、ECDSA用SHA-256、専用PoWの入口を示した。
- ECCの秘密スカラーの範囲、署名検証に必要なメッセージ・署名・公開鍵、署名nonceとPoW/アカウントnonceの違いを補った。
- アドレスの図を単一鍵の例に限定し、ScriptHash等を一律に同じ導出としないようにした。外部要求で署名する場合の操作権限・対象確認を説明した。
- PoWの擬似式をtarget以下へ揃え、ブロックIDとの区別を補った。Merkle Proofは兄弟ハッシュ・左右位置・木の構成・照合先ルートの確認が必要とした。
- SHA-256教材の依存を固定し、出力例を追加。自己比較・長さだけのテストを既知の具体値との比較へ変更した。
- 実装探索を第1章で取得する基準版に揃え、grepによる限定したファイルの検索へ変更した。

## 一次資料

- [FIPS 180-4](https://csrc.nist.gov/pubs/fips/180-4/upd1/final)：SHA-256の標準資料。
- [NIST SHA-256計算例](https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Standards-and-Guidelines/documents/examples/SHA256.pdf)：abcの既知の期待値。
- [RFC 7693](https://www.rfc-editor.org/rfc/rfc7693)：BLAKE2bの出力長・パラメータの概説。
- [BIP 340](https://github.com/bitcoin/bips/blob/master/bip-0340.mediawiki)：Schnorrの鍵・署名・nonce管理の参考。KaspaのSighash/Script仕様へそのまま転用しない。
- [sha2 0.10.9 API](https://docs.rs/sha2/0.10.9/sha2/)：教材のDigest/Sha256のAPI。

いずれも確認日に参照した。標準・BIPの全証明、実装の安全性監査を完了したものではない。

## 固定ソース

Kaspa基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。以下は読んだファイルのSHA-256。呼び出し経路・全暗号処理の実行比較ではない。

| ファイル | SHA-256 |
|---|---|
| [crypto/hashes/src/hashers.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/crypto/hashes/src/hashers.rs) | `d866185c789662a1cddec239043a7775e85c31ffb0abce04f7ce143c9444be88` |
| [crypto/hashes/src/pow_hashers.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/crypto/hashes/src/pow_hashers.rs) | `9db0fa500c78a93ee9b5ef50082eb092ef851cd02d49a714a3265fb3a6de0d0d` |
| [consensus/pow/src/lib.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/pow/src/lib.rs) | `eddc2cbf4118d63c57009345444ebdaaa8422f9d141e920bdd1f75cbe96324dc` |
| [crypto/addresses/src/lib.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/crypto/addresses/src/lib.rs) | `81e3797a77caaa5170657bd327ec2fe392def9dc549cb6697f140549effb6030` |

hashers.rsの用途別宣言・BLAKE2bマクロ、pow_hashers.rsのcSHAKE用処理、PoW Stateの計算・target比較、アドレスのVersion定義を確認した。本文の二つのgrepも既存の基準版ソースで実行し、該当箇所を出力することを確認した。

## SHA-256教材の実行

本文のRustコードブロック2件を連結し、一時プロジェクトのmain.rsとして実行。リポジトリへ別の実装・生成物・依存コードは追加していない。Cargo.tomlはedition 2021、sha2 = "=0.10.9"。Rust/Cargo 1.90.0、macOS ARM64のローカル環境で確認した。

- cargo test --offline：2テスト成功（空入力・abc・Kaspa・kaspaの計4入力）。
- cargo run --offline --quiet：本文の標準出力と一致。
- cargo fmt --check：成功。
- Python hashlib.sha256でも計4入力の期待値を確認。特に表示する2行はRustの標準出力と機械比較して一致した。

既知のabcの値はNIST例とも対応する。少数入力の成功は、暗号関数の安全性や衝突がないことの証明ではない。ライブラリを監査したという意味でもない。

ロックされた間接依存を含む版：block-buffer 0.10.4, cfg-if 1.0.5, cpufeatures 0.2.17, crypto-common 0.1.7, digest 0.10.7, generic-array 0.14.7, libc 0.2.191, sha2 0.10.9, typenum 1.20.1, version_check 0.9.5

一時Cargo.lockのSHA-256：`e81ab754294d4f9587cc7e38506ef09b5699c1a66a95241c5d890d4882d8f764`。そのlock自体は公開物へ追加しておらず、依存の完全な再現用配布物ではない。実習時に生成したlockを保持するよう本文へ記載した。

新規のcargo new/addやソースcloneを別環境で通した試験ではなく、既存のキャッシュで本文コードを実行した記録。教材は手動検査であり、CIの同期検査対象を増やしていない。

## 残る範囲

Kaspa固有の署名・ハッシュ・PoW・Merkleの実行時差分比較、HD復元、暗号方式の全安全性証明、鍵管理監査、他OS/CPU・新規環境での再現は今回の対象外。秘密鍵・シード・署名nonceの実データは生成・掲載していない。

次はSUMMARYの8ページ目、第4章 `src/part-01-foundations/chapter-04.md`。

## 本文反映後の確認

mdBookビルド、生成HTML 53ページの内部リンク・アンカー、SUMMARY全49ページと進捗表の順・ラベル、既存2教材の本文一致、差分の空白検査は成功。公開候補152ファイルの秘密情報・個人ホームパスのパターン検査で検出なし。Rust教材の抽出コードが最終本文と一致することも確認した。
