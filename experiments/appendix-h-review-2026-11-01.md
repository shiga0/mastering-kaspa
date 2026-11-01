# SUMMARY順レビュー 49：付録H「参考文献・変更履歴」

確認日：2026-11-01。全文を通読し、主要な公開書誌・参照版・Git履歴と検証管理の対応を確認。これでSUMMARYの49ページの本文レビューは一巡したが、全主張・全引用節・全実装/暗号・全ネットワークの検証が完了した意味ではない。

## 主要書誌

- [ePrint 2016/1159](https://eprint.iacr.org/2016/1159)：公開題名をSPECTRE: A Fast and Scalable Cryptocurrency Protocolへ合わせ、Sompolinsky/Lewenberg/Zoharを記載。公開履歴の最終改訂2018-01-15を確認。学会出版版等の題名との違いを曖昧に混ぜない。
- [ePrint 2018/104](https://eprint.iacr.org/2018/104)：第29章の既存確認記録に合わせ、PHANTOM and GHOSTDAGの題名とSompolinsky/Wyborski/Zoharを記載。今回の公開ページ取得は2回ともtimeout。PDF版/hashや改訂日を新たに固定できたとはしない。
- [ePrint 2022/1494](https://eprint.iacr.org/2022/1494)：The DAG KNIGHT Protocol: A Parameterless Generalization of Nakamoto ConsensusとSompolinsky/Sutton、公開ページ最終改訂2023-02-24を確認。GHOSTDAGという別表題へ置換しない。
- [Bitcoin PDF](https://bitcoin.org/bitcoin.pdf)：題名・著者と表紙を確認。[原告知](https://www.metzdowd.com/pipermail/cryptography/2008-October/014810.html)の2008-10-31も確認し、年の根拠を分離した。
- [FIPS 180-4](https://csrc.nist.gov/pubs/fips/180-4/upd1/final)の2015-08公開情報/改訂予定注記、[FIPS 186-5](https://csrc.nist.gov/pubs/fips/186-5/final)の2023-02-03公開情報/修正案内を確認。規格本文とerrata全体やKaspa署名の実装を監査したものではない。
- ThalerのZK資料は付録Cで参照した範囲へ接続し、本付録で全証明を精査したとはしない。

公開書誌・要旨/表紙の確認と、実際に読んだPDF版/hash・引用節/定理の精査を区別。未知のDOIや年・改訂日を推測していない。

## 固定版と変更履歴

- ノードの基準はv1.0.1 / fcd9c28f9b211b9872a0c90a8c94f79466ffb063。KIPは第30章/付録Fのe4ae2332117b5cb68bd6188e065ef885b6d17939。時点が違う資料であり、索引Activeと基準ノード実装・Mainnet有効化は別。
- 手元の固定KIP READMEに対し、H.5の11行の番号/Status・本文ファイルの存在を検査し一致。
- [v1.0.1リリースページ](https://github.com/kaspanet/rusty-kaspa/releases/tag/v1.0.1)でタグと短縮SHA fcd9c28の表示を確認。新規ソースcloneやバイナリー再ビルド/動作確認ではない。
- 旧稿のv2.1.0を最新とする記述やv2.0.0/Toccataとの関連は、固定版の根拠と結び付かなかったため本文の基準説明から削除。別版の存在/機能・最新版を否定または認定した調査ではない。
- 読者用のソース取得はv1.0.1タグとSHA照合へ変更。新規調査コピー向けで、既存コピーのresetを指示しない。shallow cloneに全履歴がないことも明示。
- 書籍Gitの6コミットの存在とlogの日付/題名を確認。ed762c1、d80cc68、f61b100、3fd455b、3a002f8、58e80d2の区切りを記載。付録A〜Hは本レビュー時点の未コミット作業として分離。書籍タグ/リリース/公開を作成した意味ではない。

## 管理基準・進捗の修正

- Metadata-checkedを追加し、Title/著者/Status等の確認だけで内容を精査したとはしない。
- Verifiedは個別の主張の条件/限界に限定。Source-checkedには部分実行の有無を別記する定義を与えた。READMEもこの定義へ同期し、既存進捗の実行記録と矛盾させない。
- Claim表は形式例として未確認状態を初期値にし、表の存在を検証実績へ読み替えない。
- SUMMARY49件（導入4/36章/Argent/付録8）に対応し、最後の付録Hを確認済みに変更。本文の通読完了と残る技術検証を分けた。
- ROADMAPの日付を更新し、Finalityの実行境界、HD復旧、Pruning後IBD、複数ワーカー/会計、新規環境再現、未照合の出典/数値等を継続項目として維持。
- mdBook previewは127.0.0.1:3000を明示。今回HTTPサーバー/GUIは起動しない。

## 照合した資料の識別

| 資料 | SHA-256 |
|---|---|
| 固定KIP README | `16d52aee9a0d07437e71b2532c16eabd6dc27087dacc8f06dce99d4924ab88fb` |
| 第29章の書誌確認記録 | `099301e4541192b642560d2a9b1af3f2985869baaa239d11b0c50145c66fffe6` |
| 基準版一覧 | `1078f60fde8880d1468447106eac789fb2230d3dd99043d7cd8d9a30a9e6af3a` |

## 統合・公開確認

- mdBook 0.5.4のビルド成功（警告なし）、53 HTMLのローカルリンク/アンカー不整合0。
- SUMMARY49件と進捗主表/前段1〜49番が一致。全49行にレビュー記録があり、「次の対象」は残らない。
- scripts/check-book-examples.pyで既存2教材の本文/実装が一致。
- git diff --check成功。
- 追跡対象/非ignore候補195ファイルの既知token prefix・秘密鍵・credential URL・私有home path等のパターン検出0。gitleaks/trufflehogは未導入で、未知の機密やGit全履歴の完全監査ではない。
- 既存付録A〜Gの差分を保持。今回の追加は本記録のみ。README・ROADMAP・進捗/本文の整合を更新し、生ログ・私有パス・秘密鍵/DB・バイナリー・新しいexamplesを追加していない。

新しいRust教材がないため、無関係な既存テストや公式関数比較・Devnet実測は繰り返さない。全外部リンク、全論文本文、全ライセンス、全引用IDの全面照合は未完。新規インストール・ノード操作・commit/push/Pages公開もしていない。
