# SUMMARY順レビュー 47：付録F「論文・KIP・実装対応表」

確認日：2026-11-01。全文を通読し、固定KIP一覧/主要metadata、基準版の探索入口と書籍管理教材を確認。全KIP仕様とコードの対応、全論文の証明や現行ネットワーク状態を全面照合したものではない。

## 資料の版

- KIPは第30章と同じe4ae2332117b5cb68bd6188e065ef885b6d17939。既存の調査コピーでgit rev-parse HEADが一致することを今回確認。新規cloneや最新HEADの取得はしていない。
- [固定README](https://raw.githubusercontent.com/kaspanet/kips/e4ae2332117b5cb68bd6188e065ef885b6d17939/README.md)はWebでも参照。ローカルREADMEの15行の番号/Statusと付録表が一致し、各本文ファイルが存在することを検査。
- ノードの参照軸は既存archiveのv1.0.1 / fcd9c28f9b211b9872a0c90a8c94f79466ffb063。KIPの固定版と時点が異なり、すべてのActiveなKIPをこのノードが実装しているという説明にしない。

## 修正した対応関係

- 変化するmasterのKIP索引/ノード参照を固定版へ整理。表の番号から固定KIP本文へリンクを追加。
- 15本文の先頭Title/Statusを確認。KIP15/16はREADMEと本文のTitleが異なると注記。表題の要旨・番号だけで仕様を同定しない。READMEのStatusも本文のStatusも本番有効化の証明ではない。
- 証拠レベルのTをT-read/T-runへ分割。テストを読むことと実行して成功することを分け、実行SHA/環境/日時/入力/結果を記録する。
- 基準版にないBlockrateParams/blockrate図をParamsのprior_* / crescendo: CrescendoParams / ForkedParamへ変更。値の取得に使うDAA scoreの文脈とnetworkの違いを付録E/第12章へ接続。
- GHOSTDAG/DAAの入口を固定ソースのprotocol.rs/window.rs/difficulty.rsへ具体化。ただし探索候補の表であり、全関数を精査したものではない。
- 論文の書誌確認は第29章へ接続。付録Hの全書誌が正式に固定/精査済みであるとはしない。
- ソースcloneはv1.0.1固定の読者用手順。depth 1のlog/blameでは過去の全履歴を調べられないため、歴史調査には別の完全な調査コピーを用意する。作業中のコピーをresetする指示はしていない。
- 研究→KIP→実装→検証の対応を必ず1対1の直線的な段階としない。UTXOの生成と即支出可能性も区別。

## 記録不足チェック教材

旧コードのis_reproducibleはcommit/testがSomeならtrueとなり、placeholderや空欄も通した。can_claim_mainnetという名前も、手動boolとこの検査だけで稼働を主張できるように見えた。

has_test_record / has_mainnet_recordへ変更し、40桁16進SHAと空白でない記録参照の有無のみを検査する。実在commit/ファイル・記録内容・テスト成功・再現性・network/activationの正しさは検査しない。未精査の文字列があってもtrueになるテストで、その限界を明示。公開可否やMainnet稼働の自動判定にしない。

既存Rust/Cargo 1.90.0、edition 2021、外部依存なしの一時Cargo libへ掲載コードを抽出。

```bash
cargo fmt --all
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
cargo fmt --all -- --check
```

4テスト成功：提案のみでは記録なし、テスト記録のみではMainnet記録なし、placeholder SHA/空白参照の拒否、記録欄の存在と内容確認の区別。doc-test0件を件数へ加えない。整形後の実ソースを本文へ戻し、一致確認。新規toolchain導入や新しいexamples/CI追加はしていない。

## 資料のSHA-256

識別用であり全仕様/経路を監査した印ではない。

| 資料 | SHA-256 |
|---|---|
| 固定KIP/README.md | `16d52aee9a0d07437e71b2532c16eabd6dc27087dacc8f06dce99d4924ab88fb` |
| 固定KIP/kip-0015.md | `d5a6aef73c3d594a30703e8af44b2176f7a2ca85984e4b68f3f0582d7304d401` |
| 固定KIP/kip-0016.md | `c70ef07766732677a72ff3e6c5c9c639104c6b2df6fa726ffc6752671ef5e1d4` |
| 基準版/consensus/core/src/config/params.rs | `9b9a88195e530888de6543004a241c136c142a0bf606e27de4c527901b17ffea` |
| 基準版/consensus/src/processes/ghostdag/protocol.rs | `97fe5e1db8afae9d6a3074cf6b6e56ed72e453db9edcbf9b539a4469cc2918ee` |
| 基準版/consensus/src/processes/window.rs | `fe553a9a8701a3b0fb3826f0cee6890fe166435c0c9fa871b27ac7f3f0ad182c` |
| 基準版/consensus/src/processes/difficulty.rs | `749d4ec53a38ee2cd2e9f25a395ddd69ab019d6f813f6cc3ffb3b988395566ff` |

既存の公式関数比較・ノードビルド/起動・Devnet採掘/鍵生成/署名/送金は再実行していない。KIP本文の全数式や暗号方式、論文の全証明、最新PR/リリースと全ネットワーク有効化は今回の範囲外。

## 統合・公開確認

- mdBook 0.5.4のビルド成功（警告なし）、53 HTMLのローカルリンク/アンカー不整合0。
- SUMMARY49件と主表、前段1〜48番の順序が一致。47番の付録Fを今回確認、48番の付録Gを次の対象とした。
- scripts/check-book-examples.pyで既存2教材の本文/実装が一致。
- git diff --check成功。
- 追跡対象/非ignore候補193ファイルの既知token prefix・秘密鍵・credential URL・私有home path等のパターン検出0。gitleaks/trufflehogは未導入で、未知の機密やGit全履歴の完全監査ではない。
- 既存付録A〜Eの未コミット差分を保持。今回の新規ファイルは本記録のみで、生ログ・私有パス・秘密鍵/DB・バイナリー・新しいexamplesは追加していない。commit/push/実ノード操作はしていない。
