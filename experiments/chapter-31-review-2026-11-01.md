# SUMMARY順レビュー 35：第31章「Covenantsと高度なスクリプティング」

確認日：2026-11-01。本文・図・独自教材を確認し、固定KIPの主要説明と基準版ソースの該当箇所を照合した。全仕様・暗号方式・Vault・現行Mainnet有効化の検証完了を示すものではない。

## 固定資料と実装の境界

KIP文書は第30章で取得した公開cloneを再利用し、Git HEADがe4ae2332117b5cb68bd6188e065ef885b6d17939であることを再確認した。KIP-10/17/20の本文にも固定URLでアクセスした。ノード側は既存v1.0.1 / fcd9c28f9b211b9872a0c90a8c94f79466ffb063のソースアーカイブを読む。今回こちらのGit checkout/HEADを取得したわけではない。両資料を同時点の仕様集合として扱わない。

- [KIP-10](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0010.md)の主要仕様と基準版opcode定義を照合。入力/出力数、現在の入力index、入力/出力のamount・SPKの7命令はkip10_enabledで分岐する。
- tx_validation_in_utxo_context.rsのcheck_scriptsはblock_daa_scoreでcrescendo_activationを評価し、KIP-10の有効/無効をエンジンへ渡す。全ネットワークの値・実運用状態を確認したのではない。
- Amount命令はsompiの値をScript数値へ変換する。SPK命令はtxscriptのSpkEncoding::to_bytesを通し、versionのbig-endian bytesとscript bytesを連結する。アドレス文字列との単純比較ではない。数値全域/全エンコード/実行資源上限の試験ではない。
- [KIP-17](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0017.md)の主要説明は内省・バイト列操作等を拡張し、Toccata/toccata_activationの条件を記述する。一方、基準版のOpTxVersion/OpTxLockTimeはOpcodeReservedを返す。名前が定義されていることを機能の実装済みと読み替えない。
- [KIP-20](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0020.md)はcovenant_idによる系譜/出力bindingを記述する。固定ソースのUtxoEntry/TransactionOutputにはそのフィールドがない。IDの仕組みだけで期待する次状態・Script・継続条件の正しさが得られるわけではなく、正しいCovenantの検査も必要という本文の条件を補足した。
- 上記3文書のStatusは固定版でActiveだが、本書のSDK/Devnetや現行Mainnetで全機能を利用可能という意味にはしていない。全命令・実装PR・リリース/activationの網羅照合は未実施。

## 金額教材の修正

旧教材は出力合計+feeが入力額以下であれば許可し、残額を未使用額としていた。しかし固定版のvalidate_populated_transaction_and_get_feeは入力総額−出力総額をfeeとして計算する。残額を見逃すと予定以上の手数料を許可し得る。

- feeを呼出し側の予定値expected_feeへ改名。独立したwireフィールドではないと明記した。
- 出力合計+予定手数料=入力額を要求し、予定を超える差額をUnexpectedFeeで拒否する。釣り銭は明示的な出力にする。
- 空ラベルは許可リストにあっても拒否。全出力の許可先、空出力、加算overflow、予算超過も検査する。
- input_amountは正しいUTXOからの解決/合計済みという仮定、destinationは架空の識別ラベル。実アドレス/Script、単一消費、署名、時間・再帰制約、供給上限、Mempoolの最小手数料を検証するものではない。

本書外の一時Cargo libへ本文コードを抽出した。Rust/Cargo 1.90.0、edition 2021、外部依存なし。rustfmtを本文へ反映し、実行ソースとの完全一致を確認。

```bash
cargo test --offline
cargo fmt --all -- --check
cargo clippy --offline --all-targets -- -D warnings
```

11テスト成功、fmt/Clippyは終了コード0。

1. 許可先と予定手数料どおりの支出。
2. 未許可先の拒否。
3. 出力+予定手数料による予算超過の拒否。
4. 出力合計のoverflow拒否。
5. 残額を未使用として見逃す過剰手数料の拒否。
6. 2出力のうち片方だけが未許可の場合の拒否。
7. 複数許可先/複数出力と、その並び替えの許可。
8. 空の出力集合と空の許可リストの拒否。
9. 空の受取先ラベルの拒否。
10. 出力+予定手数料のoverflow拒否。
11. 予定手数料0の厳密な収支一致。

Property-basedの全入力試験や形式証明ではない。proptestや公式Scriptのテストは実行していない。

## 説明・図の整理

- 署名は鍵による承認だけでなく署名対象への拘束を持つ。Covenantの説明から署名のコミットメントを落とさず、SIGHASHの範囲と第16章を参照した。
- UTXO型でもコントラクト状態を持てる。Covenantとスマートコントラクトを排他的な分類にしない。
- 再帰制約では金額・Script version/bytes・次状態・他出力・複数入力の集約を考慮する。回復可能性は安全性の不変条件だけでなく、鍵/データ・競合・時間・ネットワーク条件を含む進行性の検討が必要。
- DDDの図でDomainの純粋な検査と外側のアダプターを分けた。RPC受付からそのままUTXO更新まで進むように見えていた図を、Mempool検査/伝播とBlock Processing/Virtualでの状態適用へ分けた。
- オフチェーンのポリシーは攻撃者の別クライアントを制限しない。実際に署名する取引から同じ情報を評価する必要も補足した。

実Vault・再帰的Script・回復競合・現行本番/全SDKの利用検証、公式ノードの再ビルド/実行、採掘・鍵生成・署名・送金は行っていない。発展課題のフォルダー・実装やCI対象も追加していない。

## 照合対象ファイルのSHA-256

以下は読んだ該当箇所を含むファイルの識別用であり、全ファイルの仕様/証明/実装監査の印ではない。

| 固定資料 | ファイル | SHA-256 |
|---|---|---|
| KIPs e4ae233 | kip-0010.md | `da9774f32605c54a61dfc7204be92198fab3e6b68d3c59c149d74e6897c9b98e` |
| KIPs e4ae233 | kip-0017.md | `345a0a02f0590f4fe12704e82d25ee2fe7e703001b7c089ed6e3a639eea5fb90` |
| KIPs e4ae233 | kip-0020.md | `94610881c7ef905fe5346c7514b920f21a0e1a10e462776ba45eda73ac179cc8` |
| rusty-kaspa fcd9c28 | crypto/txscript/src/opcodes/mod.rs | `db9c5564a5f7e38c4d9c01a167839d10cc1d8bfba2f688f8cf26f46e4d78c12c` |
| rusty-kaspa fcd9c28 | crypto/txscript/src/lib.rs | `e237f8a433c404c609ae00a28d1acc34e137e7070aae99863f78a65dc17ca96a` |
| rusty-kaspa fcd9c28 | consensus/core/src/tx.rs | `52c5fe7980c80ced32861b6ba54a6374581d5fc5d8bd738ce8e380ab04289f65` |
| rusty-kaspa fcd9c28 | consensus/src/processes/transaction_validator/tx_validation_in_utxo_context.rs | `43c445db82fbd046cd2cbcdb90646ea84d5df732985163483ff7f8a004b25ce7` |

## 書籍・公開対象の検査

- mdBook 0.5.4でビルド成功。生成HTML 53ページの内部ファイル・アンカー切れ0件。
- scripts/check-book-examples.pyで既存教材2組が一致。第31章教材は本書外の手動検査で、examplesやCI対象を増やしていない。
- SUMMARYとreview-statusの本文表49項目が同じ順序。次は第32章。git diff --check成功。
- 追跡対象・未追跡候補181ファイルの秘密鍵ブロック・既知トークン・認証付きURL・個人ホームパス等のパターン検出0件。gitleaks/trufflehogは未導入で、秘密情報の不存在を保証する監査ではない。
- 新規公開候補はこのレビュー記録のMarkdown。鍵・DB・生ログ・ソースclone・アーカイブ・ビルド成果物は追加していない。コミット・pushもしていない。
