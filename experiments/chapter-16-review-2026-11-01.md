# SUMMARY順レビュー 20：第16章「Kaspa Scriptとトランザクション署名」

確認日：2026-11-01。基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。本文全体・図・教材・過去の検証記録・実習導線を確認した。

## 本文の修正

- 署名前の入力データであるpreimageと、ハッシュ結果のsighashを区別。ブロック識別子のハッシュをPoW値と同一視する表を修正した。
- SchnorrとECDSAのどちらも署名用ノンスの扱いが重要と整理。比較用の [BIP-340](https://bips.dev/340/) も参照したが、Bitcoinの署名方式とKaspaのSighash・Script・Taproot規則を一括りにしない。
- SIGHASH_ALLでの出力変更と、NONE/SINGLE等の承認範囲を区別。「どの変更でも必ず署名が無効」と一般化しないようにした。
- SigHashReusedValuesの集約ハッシュのキャッシュは同一取引の入力間で再利用し、取引の変更後は作り直すと説明した。
- 署名のライフサイクル図の意味のない迂回線を除き、UTXOの解決、ダイジェスト、署名データ、ノード検証へ整理。呼び出し順とは区別し、Mempoolの全段階を各ノードが必ず観測するものではないとした。
- 既存のDevnetの拒否・手数料等の検算・応答喪失の検査を冒頭から案内。未確認だったとの古い包括的な記述を、所定のケースの実測と全異常系の未完了に分けた。
- 状態モデルに送信応答の喪失による成否不明の文脈を補足。ノードでの検証とウォレットの署名前検証も言葉を整理した。
- lib/editionとリポジトリ外の作業場所を明記。ソース探索は第1章の固定版を再利用し、前後章・確認範囲への案内を更新した。

## 教材の変更と確認範囲

SignatureVerifierのインターフェースは以前signerとsignatureだけを受け取っていた。取引・入力位置・参照UTXOも渡せる境界へ変更した。これは互換Sighash APIではなく、トランザクション型・UTXO型も独自モデルである。

FakeVerifierは文脈の引数を無視して文字列approved:<signer>だけを比較する。したがって、同じ文字列のまま送金先と金額を変更しても検証を通る。新テストはその限界を示し、本物の署名の改変検出に成功した証拠には数えない。単に暗号関数を差し替えて完成するモデルでもない。

空入力・空出力は明示的なエラーとした。UTXO集合は共有参照で受け取り、検証が成功しても入力の消費や出力の登録を行わない。再検証と、コンセンサスでの再支出の受理は区別する。

本文の唯一のRustブロックを一時Cargo libプロジェクトへ抽出し、rustfmtの結果を本文へ反映。Rust/Cargo 1.90.0、macOS ARM64、edition 2021、外部依存なしでcargo test --offline、cargo fmt --check、cargo clippy --offline --all-targets -- -D warningsを実行し、成功した。

9テスト成功：

1. 一致するモック応答で手数料10を返す。
2. UTXOが要求するsignerと異なるclaimを拒否する。
3. 金額超過を拒否する。
4. 同じ入力の重複を拒否する。
5. 不正なモック応答でUTXO集合を変更しない。
6. 空入力・空出力をそれぞれ拒否する。
7. 未知のOutpointと、入力合計・出力合計のオーバーフローを拒否する。
8. 複数入力で後ろのモック応答だけ不正でも取引全体を拒否し、集合は変えない。
9. 出力の金額・支払い先を変更してもモックを通るという反例。戻り値は10から20へ変わる。

既存5件に6〜9を追加。鍵・署名を暗号学的に検証するテストではない。呼び出された文脈を記録するモックの追加は発展課題とし、今回の実行済みのテストには数えない。cargo newから新規環境で手入力した試験でもない。

新しいexamplesフォルダーや生成物は公開物へ追加せず、CI同期対象も変更していない。

## 固定ソースの再確認

| ファイル | SHA-256 |
|---|---|
| [consensus/core/src/hashing/sighash.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/hashing/sighash.rs) | `6bad7741130a188ca57c6a79fbf08b245aec8c747abd2c5e10b046a7ea918c3e` |
| [consensus/core/src/hashing/sighash_type.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/hashing/sighash_type.rs) | `658396593b26001af712d1762f36842d9685bdaf620cb93e7a050c7cfc2fb09d` |
| [crypto/txscript/src/standard.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/crypto/txscript/src/standard.rs) | `85af86284faf79ae746ff6ac600043c7e2b92fefb7e448edfa107255c258cc65` |

Sighashの構成・集約ハッシュの分岐・キャッシュ、6種類のSIGHASH値、アドレスversionから標準Scriptへの分岐を再確認した。上記のSHA-256は `signing-wallet-review-2026-10-31.md` の既存値と一致した。全opcode・全Script・全Sighashのバイト単位の独立実装や公式既知値試験は今回実行していない。

## 既存の実測と今回の限界

- `signing-wallet-review-2026-10-31.md`：主要ソース・HD導出/Generator/Signerの部分レビュー。
- `devnet-send-review-2026-10-31.md`：ランダム鍵の単一署名送金、virtual受理とUTXO観測。
- `devnet-send-checks-2026-10-31.md`：SDKの残高不足、変更した署名の拒否、Mempool内の競合、手数料/お釣りの検算。
- `devnet-send-retry-2026-10-31.md`：所定の送信成功応答を落とした際の照会と同一取引の再送・重複計上の検査。

これらは以前の実行記録であり、今回再実行していない。本文の課題Dの署名後の送金先変更は実験計画であり、署名バイト変更の既存検査とも区別する。

今回は鍵生成・実際の署名・送金・採掘・ノード起動を行っていない。ページ通読とモック教材の実行を、HD復旧・全方式・全異常系・不可逆な決済確定・現在のMainnetとの一致の証明へ拡張しない。検証用Dockerバイナリと固定ソースの由来の完全一致も既存の限界を維持する。

次はSUMMARYの21ページ目、第17章 `src/part-03-transactions/chapter-17.md`。

## 統合・公開物の確認

既存の本文とRustソースの同期対象2件が一致。mdbook build成功、生成HTML53ページの内部ファイル・アンカーのリンク切れ0件。SUMMARYとreview-statusの49項目は名称・順序一致。git diff --checkも成功した。

追跡済み・追加候補166ファイルへの既存の秘密情報パターン検査は検出0件。gitleaks/trufflehogは未導入であり、秘密情報が一切ないとの保証ではない。新しいexamplesフォルダー、生成物、秘密鍵、個人パスは追加していない。
