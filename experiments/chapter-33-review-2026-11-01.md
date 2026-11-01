# SUMMARY順レビュー 37：第33章「レイヤー2・ブリッジ・ゼロ知識証明」

確認日：2026-11-01。本文・主要一次資料・固定KIPの説明と独自状態遷移教材/型断片を確認した。実ブリッジ・暗号方式・回路/証明・現行本番有効化の全面監査ではない。

## 一次資料と用語

- [EthereumのZK-rollups](https://ethereum.org/en/developers/docs/scaling/zk-rollups/)と[Validium](https://ethereum.org/en/developers/docs/scaling/validium/)のDA説明を確認。L1へroot/証明だけを置くことと、状態再構築に必要なデータをL1のDAで公開するRollupを区別した。公開するデータは圧縮/差分でもよく、元の取引の全バイトと同一とは限らない。Ethereum側の構成をKaspaで実装済みと扱わない。
- [EthereumのBridges](https://ethereum.org/en/bridges/)はコード・運用・カストディ/検閲等のリスクを説明する。ブリッジの分類は排他的でなく、証明・ヘッダー・署名者・管理者の複合構成として読む。
- 元のNIST参照はZKPと表示しながらMulti-Party Threshold Cryptographyのページを指していた。[PEC / ZKProof](https://csrc.nist.gov/projects/pec/zkproof)へ修正し、公開命題に含まれないWitness情報と、公開入力自体の開示を区別した。健全性等とゼロ知識性も別の要件とした。
- [ePrint 2018/046](https://eprint.iacr.org/2018/046)の公開要旨・書誌を確認。題名Scalable, transparent, and post-quantum secure computational integrity、著者Eli Ben-Sasson / Iddo Bentov / Yinon Horesh / Michael Riabzev。透明なSTARKの研究上の性質を、実装・設定や別のSNARKを使うラッパーまで自動的に一般化しない。PDF/全証明の照合はしていない。
- 上記Webページは確認時点の内容で、固定snapshotを収録していない。引用した公開要旨/説明をプロトコルの全面監査へ読み替えない。

KaspaのKIPは第30章の公開cloneを再利用し、HEADがe4ae2332117b5cb68bd6188e065ef885b6d17939であることを再確認した。[KIP-16](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0016.md)の主要説明/OpZkPrecompileと、[KIP-21](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0021.md)の要旨/レーン別commitmentを読んだ。文書のStatusはActiveだが、任意のL2の正当性・裏付けを自動検証するという意味ではない。基準版ノードv1.0.1 / fcd9c28および第32章のSilverscript依存a41a333とも別の参照軸である。この回にノードの全実装や有効化設定を照合したのではない。

## 安全性の境界を整理

- 検証するプログラム/検証鍵・公開入力のエンコードを特定し、old rootが受理済み状態と一致すること、batch commitmentとWitnessの取引列が結び付くことも検査する。入力を列挙するだけでは回路が拘束したことにならない。
- Merkle包含は信頼するrootがあって初めて意味を持つ。任意のrootへの包含だけで、正しい状態遷移や資産ロックを証明したことにしない。
- DAと永久アーカイブを区別し、Pruning等の保持期間をまたぐ復旧/退出用データの保存主体を定める。現行UTXOの保持と古い履歴の削除も区別した。
- SourceFinalizedはポリシー判断であり、無条件に覆らない状態名ではない。送信後タイムアウトはOutcomeUnknown等として相手側と照合する。失敗/期限切れだけで即返金すると、遅延実行と二重成立し得る。
- ReplayのIDはネットワーク/bridge/資産/イベント・outpoint等の論理イベントを宛先・受取人・金額・版へ結び付ける。再編成でブロックhashが変わったことだけで別の支払いへ変えない。
- claim_onceの予約/完了の意味を仕様化する。ローカルDBの一意制約・トランザクションは外部チェーンの実行と原子的に一体化できない。宛先の重複防止と、送信・成否不明・記録失敗・照合/再送を含む設計が必要。
- 回復は安全性の不変条件だけでなく、進行性・運用要件として扱う。概念図は全呼び出し順や単一の更新の再現ではない。

## 独自状態遷移教材

空のfrom/toラベルを拒否し、残高総量のテストをu128へ変えてテスト自身の加算overflowを避けた。受取人overflow・自己送金の残高不足・完全な入力保持などを検査した。from/toは架空のIDで実アドレスではない。

これはアカウント残高の教材で、KaspaのUTXO、署名/認証、nonce/Replay防止、手数料、資産裏付け、rootを実装するものではない。意図的に同じ送金を2回与えて2回移転できることもテストし、欠けた保証を隠さない。残高0のentryが残る点も、状態commitmentを設計する場合の別課題として明記した。

本書外の一時Cargo libへ本文を抽出。Rust/Cargo 1.90.0、edition 2021、外部依存なし。rustfmtを本文へ反映して実行ソースとの一致を確認した。

```bash
cargo test --offline
cargo fmt --all -- --check
cargo clippy --offline --all-targets -- -D warnings
```

9テスト成功、fmt/Clippyは終了コード0。

1. 通常移転の各残高と総量保存。
2. 残高不足の拒否と入力不変。
3. 自己送金で残高不変。
4. 金額0の拒否。
5. 受取人加算overflowで全入力状態を維持。
6. 空のfrom/toを拒否。
7. 自己送金でも残高不足や未知の送信元を許可しない。
8. 送信元/受取人残高と金額を各0・1・10・u64::MAXとする64ケースで、成功時の総量保存と全ケースの入力不変を検査。
9. 同一送金の2回適用でReplay対策が存在しないことを明示。

有限グリッドの性質確認であり、全入力の証明やproptest等の実行ではない。合計保存だけで認証・裏付けを証明できるという意味でもない。

SourceFinality/MessageVerifier/ReplayProtectionの3traitはpubとし、別の一時libへ抜き出してcargo check --offlineとcargo fmt --all -- --checkが成功した。本文と型検査ソースの一致も確認。型が整合するだけで、finalityの実測・永続/並行ReplayGuard・外部作用のexactly-onceを実装/試験したわけではない。

## 今回行っていないこと

公式ノード/Script VM・Silverscriptの再実行、回路化/証明生成・暗号系の監査、実ブリッジ/署名・送金、ネットワーク有効化・分断/再編成・実DA/退出・複数ワーカー/DBの統合試験はしていない。個別L2プロジェクトの稼働調査もしていない。新規examplesやCI対象は追加していない。

## 固定KIPのSHA-256

主要説明を読んだファイルの識別用であり、全ファイルの仕様・証明を検証した印ではない。

| ファイル | SHA-256 |
|---|---|
| kip-0016.md | `c70ef07766732677a72ff3e6c5c9c639104c6b2df6fa726ffc6752671ef5e1d4` |
| kip-0021.md | `ce1606e74e20a9652a0c67ddd349d84b8b2ab8265df5a1d10b27d35f7a95cf26` |

## 書籍・公開対象の検査

- mdBook 0.5.4でビルド成功。生成HTML 53ページの内部ファイル・アンカー切れ0件。
- scripts/check-book-examples.pyで既存教材2組が一致。本章の状態遷移とAPI断片は手動確認で、examples/CI対象を増やしていない。
- SUMMARYとreview-statusの本文表49項目が同じ順番。次は第34章。git diff --check成功。
- 追跡対象・未追跡候補183ファイルの秘密鍵ブロック・既知トークン・認証付きURL・個人ホームパス等のパターン検出0件。gitleaks/trufflehogは未導入で、秘密情報の不存在を保証する監査ではない。
- 公開候補は本文・進捗・レビュー記録のMarkdown。鍵・DB・生ログ・アーカイブ・ソースclone・ビルド成果物は追加していない。コミット・pushも行っていない。
