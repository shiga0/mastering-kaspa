# 第16・24章：署名・ウォレットの照合記録

確認日：2026-10-31。基準版：v1.0.1 / `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。Source-checkedの部分レビュー。秘密鍵・シードの生成、署名・送金は今回実行していない。

## 固定ソースと照合ファイルのハッシュ

- [consensus/core/src/hashing/sighash.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/hashing/sighash.rs) — SHA-256 `6bad7741130a188ca57c6a79fbf08b245aec8c747abd2c5e10b046a7ea918c3e`
- [consensus/core/src/hashing/sighash_type.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/hashing/sighash_type.rs) — SHA-256 `658396593b26001af712d1762f36842d9685bdaf620cb93e7a050c7cfc2fb09d`
- [wallet/keys/src/derivation/gen1/hd.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/wallet/keys/src/derivation/gen1/hd.rs) — SHA-256 `2c585e06a671a2b1009c06b334077d814f8930003dc4ebb54f87df256d932f5b`
- [wallet/keys/src/derivation_path.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/wallet/keys/src/derivation_path.rs) — SHA-256 `b19ffbb8695425ef29e641c487109ea5399734e8807ebe172ae45262643da09f`
- [wallet/core/src/tx/generator/signer.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/wallet/core/src/tx/generator/signer.rs) — SHA-256 `3e3195d2457abdf18bac93553fc9aadfed5e8b0a99a27ac92c882c13e4b528c5`
- [crypto/txscript/src/standard.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/crypto/txscript/src/standard.rs) — SHA-256 `85af86284faf79ae746ff6ac600043c7e2b92fefb7e448edfa107255c258cc65`
- [wallet/core/src/tx/generator/generator.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/wallet/core/src/tx/generator/generator.rs) — SHA-256 `9bf481fed8d22e85fe937d951e9f5990fbeefe239cffbdbf80ea4111efabf3df`

## 確認範囲

- SIGHASHの構成、許可する6種類、出力・入力集約の分岐、Schnorr/ECDSAのダイジェストの違い。
- 標準PubKey/PubKeyECDSA/ScriptHashのScript生成。
- gen1のpurpose/coin type/account、公開鍵形式とアドレス化。gen0全体の照合は対象外。
- Generatorのmassに応じた段階分割と、Signerの呼び出し・一時鍵配列の消去。

## 未確認

ハッシュのドメインと全エンコード・既知ベクトルの完全照合、公式実装の実行、鍵導出と復旧のE2E、鍵管理の監査、送金受理・手数料・massの実測、legacy/gen0、全Script opcode、マルチシグ・ハードウェア機器の対応は未確認。章全体をVerifiedにはしない。

## 第16章の既存教材

Rust 1.90.0、edition 2021、`rustc --test`で掲載教材を再実行。実署名・公式SDKの検証ではない。

```text
running 5 tests
test tests::duplicate_input_is_rejected ... ok
test tests::overspend_is_rejected ... ok
test tests::wrong_signer_is_rejected ... ok
test tests::failed_validation_does_not_modify_utxo_set ... ok
test tests::valid_authorization_returns_fee ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 第24章の既存教材

Rust 1.90.0、edition 2021、`rustc --test`で掲載教材を再実行。実署名・公式SDKの検証ではない。

```text
running 4 tests
test tests::duplicate_input_does_not_mutate_state ... ok
test tests::missing_input_does_not_mutate_state ... ok
test tests::release_allows_retry ... ok
test tests::reserve_once ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
