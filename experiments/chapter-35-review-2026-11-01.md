# SUMMARY順レビュー 40：第35章「Kaspaアプリケーションの構築」

確認日：2026-11-01。基準版の主要RPC入口・独自の請求集計モデル/型断片とSQLite制約を確認した。Payment Tracker全体を作成したものではなく、実アダプター・入金監視E2E・期限/会計台帳の検証完了ではない。

## 目次順の訂正

SUMMARYの第32章と第33章の間にあるArgent補足が未照合のまま、番号付き章だけ進めていたことを確認した。次の対象をArgentへ戻し、その後に第36章を確認する。第33/34/35章の実際の目次順は38/39/40番目で、進捗表・第33/34章記録の番号・ROADMAPを訂正した。Part V全ページを完了扱いにしない。既存章の技術確認結果を撤回する訂正ではない。

## 固定版RPCとの対応

参照は既存のv1.0.1 / fcd9c28f9b211b9872a0c90a8c94f79466ffb063のソースアーカイブ。今回Git checkout/HEADを取得したという意味ではない。

- rpc/wrpc/examples/simple_client/src/main.rsはKaspaRpcClient::new、get_server_infoでnetwork_id/is_synced/has_utxo_indexを取得し、DAG照会と切断を行う。既定はMainnetの公開Resolver。隔離Devnetへ無変更で実行する例にはしていない。
- [現行Getting Started](https://docs.kaspa.org/integrate/getting-started)のRust例ではnew_with_args等が使われる。現行Webのサンプルと固定版のAPIを混ぜず、引用を固定ソースへ変更した。固定GitHubのWeb取得はCache missだったため、ソースアーカイブの該当箇所を読んだ。実RPC接続は実行していない。
- rpc/core/src/model/message.rsのGetServerInfoResponseを確認。ネットワーク種別だけでなく完全なNetworkId、接続世代・応答の鮮度、UTXO indexの有無が必要。
- rpc/service/src/service.rsのget_utxos_by_addresses_callはutxoindex無効時にNoUtxoIndexを返す。UtxosChangedNotificationはadded/removedを持つが、教材のrevisionや支払い取消理由を直接表すものではない。
- rpc/core/src/convert/notification.rsの主要変換も確認。UTXO集合の差分、VirtualChainの受理情報、アプリで正規化した支払い適格状態を区別した。通知の永続履歴・全APIや再接続経路を監査したものではない。
- 受取済みUTXOの支出と、元の入金の受理取消を区別する。現在集合から消えただけで入金を取り消さない。接続断の間に受理/支出された出力は、通知をローカル保存する設計だけでは復元できないこともある。
- experiments/devnet/utxo-store.pyは現在UTXOの観測ストアであり、過去の支払い台帳の実装ではない。今回は変更・再実行していない。

## 集計教材の修正

- 生のAcceptedを足してPaidにするように読めた状態をEligibleへ改名。確認ポリシー達成・集計対象と正規化済みという呼出し側の前提であり、関数が確定を検証するわけではない。
- 独自Network enumでTestnetの番号も区別し、請求と観測の一致を検査。実RPCのネットワーク識別の真正性・アドレス所属は検証しない。
- ingesterが付与するrevisionを導入し、古い版の上書きと同じ版の別状態を拒否。より新しい再受理は反映できる。RPCの標準フィールドでもDAA scoreでもなく、接続世代・永続化・応答順序を含む正規化が別途必要。
- proposed cloneで集計を検査してから置換し、overflow時も全状態を保持する。これはメモリー上の局所的な更新で、DB・外部作用・複数ワーカーの原子性は実装しない。
- 0額の候補を拒否し、請求をObservedへ進めない。期限/Review・業務会計は教材外と明記した。

本文のドメインとChainHealth/ChainReaderの2断片を順につなぎ、本書外の一時Cargo libへ配置。Rust/Cargo 1.90.0、edition 2021、外部依存なし。rustfmtを反映し、本文と実行ソースの完全一致を確認した。

```bash
cargo test --offline
cargo fmt --all -- --check
cargo clippy --offline --all-targets -- -D warnings
```

12テスト成功、fmt/Clippyは終了コード0。ChainReaderは型が整合しただけで、実RPCアダプターではない。

1. 0額請求の拒否。
2. 同一観測の再配送を重複加算しない。
3. CandidateだけではPaidにしない。
4. 新しい版の取消で再集計。
5. 同一出力の金額不整合を拒否。
6. 集計overflowの観測を拒否し、全状態を保持。
7. Mainnet/Testnet-10/DevnetをTestnet-11の請求へ混ぜない。
8. 古いEligibleが新しい取消を上書きしない。
9. 同じ版の異なる状態を拒否し、全状態を保持。
10. 新しい版で同一出力が再適格になった場合に反映。
11. 0額候補はObservedにしない。
12. 60+40の分割入金、100回の重複配送、片方の取消を正しく集計。

## SQLiteの制約検査

本文DDLをPython標準sqlite3のインメモリーDBへ適用した（SQLite 3.43.2）。foreign_keys=ONも照合。請求/観測の複合FK、ネットワーク単位の専用アドレス一意性、outpointの一意性、整数/範囲・状態の制約を加えた。

- 正常挿入、output_index=u32::MAX、amount=2**63−1の挿入に成功。
- 別ネットワークの請求への紐付け、存在しない請求、0/負/小数の観測金額、負/u32超過のindex、負/小数のrevision、不正な状態、専用アドレス重複、出力重複の12ケースがIntegrityErrorで拒否された。
- Python bindingはamount=2**63の入力をOverflowErrorで拒否。u64全域がSQLiteへ格納できたという結果ではない。Rust側のu64::MAXテストとは範囲が異なる。

DDLとRustは自動接続していない。UTC形式・アドレス/txid・完全NetworkIdの文字列検査、範囲変換、状態の版付きupsert、監査履歴/チェックポイントの保存、並行更新・復旧・実マイグレーションは未実装。単なるDDLの制約検査をDB統合試験へ読み替えない。

## 説明・CI・未実装範囲

- 請求アドレスの再割当て、遅延入金/期限、支払い後の再評価を整理した。
- 観測/進捗を一緒にcommitする設計と、再取得可能なcursorが実RPCに存在するかは別の確認。Webhook等の外部作用にはoutbox・同じ版の冪等キー・成否不明の照合が必要。
- CI例にcontents: read、Rust 1.90.0、lockedコマンドを加えた。[GitHubのSecure use](https://docs.github.com/en/actions/reference/security/secure-use)を権限/固定方法の入口として追加した。本書の実workflowは変更・実行していない。GitHubでの成功記録ではない。
- ディレクトリ図とPayment Trackerは配置/設計の提案であり、examples/payment-domainやpayment-tracker等を作成していない。

実ノード接続、鍵生成・署名・送金、実Payment Tracker、履歴インデクサーからの復旧、複数ワーカー/DBの統合、E2E、期限/業務会計は検証していない。

## 照合した主要ファイルのSHA-256

主要箇所を読んだ資料の識別用で、全ファイルの監査を示すものではない。

| ファイル | SHA-256 |
|---|---|
| rpc/wrpc/examples/simple_client/src/main.rs | `80725f9f77f30f1503eb8152c1199a68d630a31f46edb4b25c997efd26184a2d` |
| rpc/core/src/model/message.rs | `24a5fb7c14606e6284c00ca729a960f97a6b5c9d76f22659c8887ff254c6ea98` |
| rpc/core/src/convert/notification.rs | `5f63ca99d1175f3c792241b32c4468cc28064c83defca7d8e81787a47fb55fd4` |
| rpc/service/src/service.rs | `b07acf50884a1d4c0361f0227952293012a7af6cf49663be5d8b0aad022ad43f` |

## 書籍・公開対象の検査

- mdBook 0.5.4でビルド成功。生成HTML 53ページの内部ファイル・アンカー切れ0件。
- scripts/check-book-examples.pyで既存教材2組が一致。今回の集計/API/SQL検査は本書外での手動確認で、CI対象を増やしていない。
- SUMMARYとreview-statusの本文表49項目が同じ順序。さらに進捗表1〜40のページ位置・番号をSUMMARYへ照合した。Argent補足を次の対象とし、その後に第36章へ進む。git diff --check成功。
- 追跡対象・未追跡候補185ファイルの秘密鍵ブロック・既知トークン・認証付きURL・個人ホームパス等のパターン検出0件。gitleaks/trufflehogは未導入で、秘密情報の不存在を保証する監査ではない。
- 公開候補は本文・進捗・レビュー記録のMarkdown。秘密鍵・DB・生ログ・ソースclone・アーカイブ・ビルド成果物は追加していない。コミット・pushも行っていない。
