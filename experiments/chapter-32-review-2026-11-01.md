# SUMMARY順レビュー 36：第32章「Silverscriptとスマートコントラクト設計」

確認日：2026-11-01。本文、固定公式文書/CLIの主要箇所、独自Rust教材を確認した。途中のユーザーの導入相談に応じ、既定を維持してRust 1.94.0を追加し、公式CLIのビルド・コンパイルも確認した。実Script VM・Devnet/本番利用・暗号方式の全面監査ではない。

## 参照版と実行環境

- [公式Silverscript v1.0.0](https://github.com/kaspanet/silverscript/releases/tag/v1.0.0)を確認。ls-remoteとタグ指定cloneのGit HEADは3ed973335b59269293564805cc2c58a14595ec03、コミット日2026-09-09。
- [固定Cargo.toml](https://github.com/kaspanet/silverscript/blob/3ed973335b59269293564805cc2c58a14595ec03/Cargo.toml)はedition 2024 / rust-version 1.94.0を指定。Kaspa依存はa41a333b08848f41bf737b72592e463a6011b8acのGit revision。本書のノード基準版v1.0.1 / fcd9c28とは別であり、基準版Devnetとの互換性を主張しない。
- 元のstableはrustc 1.90.0、既存nightlyは1.92.0-nightly。承認されたrustup toolchain install 1.94.0 --profile minimalで追加し、導入後も既定のrustc --versionは1.90.0。rustc +1.94.0は1.94.0 (4a4ef493e 2026-03-02)。default/overrideを変更していない。
- 通常のsandboxではGitの名前解決に失敗したため、承認された公開ネットワーク読取で取得。最初のoffline CargoはGit依存が未取得で失敗。承認されたCargo実行で固定依存を取得し、その後はofflineで再実行した。clone・target・成果物はすべて本書外へ配置した。

## 公式CLIの確認

固定README、Tutorial、CONSTRUCTOR_ARGS、silverc.rs、言語バージョン定数とABI型の主要箇所を確認した。CLIの-o/-c/--constructor-args/--ast-onlyを、架空のコマンドとしてではなく実装に結び付けた。

```bash
cargo +1.94.0 run --locked -p silverscript-lang --bin silverc -- --help
```

ビルド/ヘルプは終了コード0。以下のソースを本書外へ保存してコンパイルした。本文の第32.10節と実行ソースが一致することも確認。

```text
pragma silverscript ^0.1.0;

contract TwoOfTwoVault(pubkey ownerA, pubkey ownerB) {
    entry spend(sig signatureA, sig signatureB) {
        require(ownerA != ownerB);
        require(checkSig(signatureA, ownerA));
        require(checkSig(signatureB, ownerB));
    }
}
```

constructor引数にはsecp256k1の公開点G/2Gのx座標32-byteを使用した。以下は公開点で、秘密鍵・署名ではない。資金投入/鍵生成/署名/送信は行っていない。

- G: 79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798
- 2G: c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5

owners.jsonは、この順に各公開点のhexをデコードしたbyte配列を、kind: bytes / value: [0〜255の整数]の2要素配列へ格納した。pubkeyという型名をJSONのkindに使わない。

```bash
cargo +1.94.0 run --offline --locked -p silverscript-lang --bin silverc -- two-of-two.sil --constructor-args owners.json -o two-of-two.json
```

実際にはソース・引数・出力の絶対一時パスを指定した。上記はラボ内のファイル名で示した同じ引数関係。結果は終了コード0。ビルド済みsilvercバイナリーで同じ引数を再実行してJSONのbytesが一致した。JSON内contracts.TwoOfTwoVault.compiled.bytecodeをデコードしたScriptは168 bytes。

- リリース/パッケージはv1.0.0だが、固定版のCOMPILER_VERSION定数と生成物のcompiler_versionは0.1.0、schema_versionは1だった。
- 最初に^1.0.0を指定した試行はfuture majorを含むpragmaとして拒否された。公式例と定数に合わせ^0.1.0へ訂正するとコンパイルに成功した。リリース番号から言語pragmaを推測しない。
- 最初のkind: pubkeyはunknown variantとして拒否された。固定CONSTRUCTOR_ARGSを照合してkind: bytesへ訂正した。誤った試行を成功として扱っていない。
- 不正文法、kind: pubkey、^1.0.0をそれぞれ再実行し、期待した診断・終了コード1と出力ファイル不在を確認。
- --ast-onlyのAST JSON出力は成功。ASTの成功を型検査/コンパイル成功と同一視しない。
- 同一環境での2回の出力一致であり、別環境での再現ビルドや生成Scriptの意味の正しさを検証したものではない。

## 独自Rust承認モデル

元のVaultPolicyはowners/thresholdがpubで、newを使わず閾値0の構造体を作ると空承認でも許可し得た。フィールドを非公開にして生成経路をnewへ限定した。また、HashSetを入力にすると重複設定が事前に失われるため、Vecから重複・空IDを検査して集合化するようにした。

- constructorで閾値0、所有者数を超える閾値、空集合、重複/空IDを拒否。
- authorizeは閾値成立後も証拠を最後まで確認し、未知・重複IDを拒否する独自の厳格な方針。
- 検証済みIDという呼出し側の仮定を、同じ取引・SIGHASH/規則・鍵との結合まで明示。文字列の違いだけでは異なる鍵を保証しない。
- これは暗号署名の検証器でも実VMのcheckMultiSigの受理条件を再現した関数でもない。実VMと差分比較するには入力領域の写像が必要。

本文の2つのRust断片を順につなぎ、本書外の一時Cargo libへ配置した。Rust/Cargo 1.90.0、edition 2021、外部依存なし。rustfmtを反映し本文と実行ソースの完全一致を確認。

```bash
cargo test --offline
cargo fmt --all -- --check
cargo clippy --offline --all-targets -- -D warnings
```

10テスト成功、fmt/Clippyも終了コード0。正常2-of-2、承認不足、重複、不明、過大閾値、閾値0/空集合、重複設定/空ID、空承認、閾値成立後の未知/重複証拠、2-of-3の全3組と逆順を確認した。property-basedの全入力試験や形式証明は行っていない。

## 説明・図と未検証の範囲

- 参照版・依存・CLI・言語pragmaを具体的な固定一次資料へ結び付けた。constructor引数・ABI・対象VM/ネットワークとhash対象も再現記録へ加えた。
- Mempool受付、受理/UTXO適用、確定ポリシーを分離。再編成後の再受理と状態名による永久的な重複排除を区別した。
- Compiler→Artifact→Builder/Signer→Nodeの責務を図示し、JSON全体をオンチェーンへ送る図にしない。概念図を実呼び出し順や単一の保存処理と扱わない。
- 競合支出は同じ受理状態で両方を適用しないという条件にした。単純に「どちらか一方が必ず最終受理される」とは扱わない。

公式Script VMでの実行、暗号学的な署名試験、同じ鍵を与えたScriptの挙動、実Vault/継続Covenant/回復、公式テスト全体、ノード/ネットワーク有効化、Devnet/本番利用は検証していない。既存の基準ノード・Docker構成や既定Rustは変更していない。新しいexamples/CI対象も作成していない。

## SHA-256による資料・成果物の識別

hashは主要箇所を読んだ資料と出力の識別用で、ファイル全体の監査を示すものではない。

| 資料 | SHA-256 |
|---|---|
| README.md | `80507cf9a77b102f4e65117948ea1d429a99786647b9dc2e3ad004c3f1d8b3ae` |
| Cargo.toml | `486d08b1eb458c8abef0400e6649541274e2b3aa887e3ce9497483d4b1961541` |
| Cargo.lock | `6bb45aecc03fc85e84afa6e1145735f94b5c0b9d6fe7916e2d295f8cdafe51f8` |
| docs/TUTORIAL.md | `daa9b922ccfe30e88e18a8b412e87428a2fe93ad35c90bf52055ce949543c130` |
| docs/CONSTRUCTOR_ARGS.md | `d688b5198410eb5103ccdcf77ca0b29571398659a5c22d97c5358f1a5508f455` |
| silverscript-lang/src/bin/silverc.rs | `e8c71557cc78c29c292494d3a01c45f1c93044a1efff00d4f51d58c53338671e` |
| silverscript-lang/src/compiler/mod.rs | `ba3b4af67012709d42541368eb37660d32392256337df5254f705a4326f0a5cf` |
| silverscript-lang/tests/examples/2_of_3_multisig.sil | `07e795299c57ccec6b322a7c3964fa4cd43bb9c5a9be9ffe986794430af5a6af` |
| silverscript-abi/src/lib.rs | `32a3d7e1129c60e6976f3e126cc2ac3a4ef22c4e43dd2c262abb65976f53b17c` |
| ラボ: two-of-two.sil | `053386573632516c0bbd53daaea5da73932b26116002e43b642ed7929db7c2d5` |
| ラボ: owners.json | `bdbc03af3f93a0d9764c52922b256036a96ae58a41714e580ced13164619c5a2` |
| ラボ: two-of-two.json | `b059e537cfc3f48a849418f69226a9d56da864a6032b00aba1a10058b8b90e50` |
| ラボ: ast.json | `d210d67bbccd50c5c5c6ef045f77bd69885f8b0672a002bbd64db846ba3e3533` |
| デコードしたScript bytes | `fed50341469b6e41e077f351436eccf32d10482434eaf38c356ef63598d599d9` |

## 書籍・公開対象の検査

- mdBook 0.5.4でビルド成功。生成HTML 53ページの内部ファイル・アンカー切れ0件。
- scripts/check-book-examples.pyで既存教材2組が一致。本章のCLI/参照モデルは本書外での手動検査で、CI対象を追加していない。
- SUMMARYとreview-statusの本文表49項目は同じ順序。次は第33章。git diff --check成功。
- 追跡対象・未追跡候補182ファイルの秘密鍵ブロック・既知トークン・認証付きURL・個人ホームパス等のパターン検出0件。gitleaks/trufflehogは未導入で、秘密情報の不存在を保証する監査ではない。
- 公開候補は本文・進捗・レビュー記録のMarkdown。秘密鍵・DB・生ログ・コンパイラーclone・JSON生成物・バイナリー・アーカイブを追加していない。記録にある公開点は教材のコンパイル引数であり秘密情報ではない。コミット・pushもしていない。
