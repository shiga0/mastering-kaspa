# SUMMARY順レビュー 06：第2章「デジタルマネーの基礎」

確認日：2026-10-31。対象は `src/part-01-foundations/chapter-02.md`。文章・図・計算例・中央台帳の疑似コードを確認し、一次資料と固定ソースの該当箇所へ照合した。

## 主な修正

- 貨幣の三つの機能に出典を付け、交換手段の説明を自然な語にした。現金も含む表に合わせて見出しを変更し、自己管理と事業者への管理委託を区別した。
- 所有権・本人の意図と、プロトコル上の支出条件・署名の検証を区別した。署名が保護する範囲は方式・設定によることを補った。
- データやトランザクションをコピーすることと、競合する支出を重ねて受理することを区別。全参加者が常に同じ最新状態を持つという読み方を避けた。
- UTXOを未使用出力の集合として説明し、IDと出力番号による識別、未使用でも即座には使えない条件を補った。
- 100の入力から30・69の出力を作る図で、手数料1を独立した出力のように描く誤りを修正。通常送金の差額とし、実ネットワークの必要手数料の値とは区別した。
- 台帳の残高例は発行・償却・手数料を除いた保存の例として限定。PoWへの図もBitcoin/Kaspaの学習経路として説明した。
- 疑似コードの残高・重複確認が更新処理の外にあった競合を修正。確認と更新・処理済み記録を、直列実行と同じ結果になるトランザクションの範囲へまとめた。
- 再送キーを送金元の範囲で扱い、同じ内容の再送は保存済み結果を返す。同じキーで内容が変わる場合は拒否し、更新途中の失敗はまとめて取り消す設計とした。
- 正の整数・送金権限・既存の異なる口座を前提とし、DBの分離・一意制約・範囲確認・競合時の再試行を説明。期待結果の表と実測を区別した。

## 根拠

| 資料 | 照合した範囲 |
|---|---|
| [ECB：What is money?](https://www.ecb.europa.eu/ecb-and-you/explainers/tell-me-more/html/what_is_money.en.html) | 交換手段・価値尺度・価値保存の三機能。国別の法制度の調査ではない |
| [Bitcoin開発者ガイド：Transactions](https://developer.bitcoin.org/devguide/transactions.html) | UTXOの支出・お釣り・手数料差額、署名ハッシュ種別が保護する範囲。古い手数料水準や運用推奨は転用しない |
| [Ethereum accounts](https://ethereum.org/en/developers/docs/accounts/) | 残高とnonce、同じnonceの重複実行を防ぐ説明。アカウント型の一例として使用 |
| [PostgreSQL 18：Transactions](https://www.postgresql.org/docs/18/tutorial-transactions.html) | 更新をまとめて反映・取り消す性質 |
| [PostgreSQL 18：Transaction Isolation](https://www.postgresql.org/docs/18/transaction-iso.html) | 原子性だけでは競合の扱いを決められないこと、Serializableの直列実行と同じ結果・失敗時の全体再試行 |
| 基準版 `consensus/core/src/tx.rs` | UTXOの金額・Script等の項目、Outpointのtransaction ID・index |
| 基準版 `consensus/src/processes/transaction_validator/tx_validation_in_utxo_context.rs` | `validate_populated_transaction_and_get_fee` の入力合計・出力合計・差額と、Coinbase成熟条件の検証 |

HTTP資料は上記確認日に参照。Kaspa基準版はv1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。上記固定ソース2件のSHA-256が既存の `consensus-state-review-2026-10-31.md` の値と一致することを確認した。

## 確認の限界

設計上の期待結果を追加したもので、疑似コードをDBへ実装して試験したものではない。並行送金・再送・障害復旧・応答喪失の実測、実際のDB制約とロック設計は対象外。本番の決済台帳やKaspaのコンセンサス実装として提供するコードではない。法的な所有権の判断、最新Mainnetの性能や必要手数料、論文の全安全性証明も今回の対象外。

次はSUMMARYの7ページ目、第3章 `src/part-01-foundations/chapter-03.md`。

## 修正後の確認

固定ソース2件のハッシュ一致、残高例の1,200単位の保存、通常送金例の100=30+69+1を確認。mdBookビルド、生成HTML 53ページの内部リンク・アンカー、SUMMARY全49ページと進捗表の順・ラベル、既存2教材の本文一致、差分の空白検査は成功。公開候補151ファイルの秘密情報・個人ホームパスのパターン検査で検出なし。並行処理・再送・ロールバックの表は設計上の期待結果であり、これらの実行試験が成功したという記録ではない。
