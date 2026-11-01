# SUMMARY順レビュー 43：付録B「Kaspa用語集」

確認日：2026-11-01。用語集全体を通読し、主要な定義を本書の既存レビュー済み章と、手元の基準版ソース資料に照合した。文章・参照の修正であり、新しいプログラム実行やノードの動作検証ではない。

## 主な修正

- Merge setの旧定義はselected parentを含むかが曖昧だった。第12章と合わせ、Pastが自身を含まない場合のPを除く候補を `Past(B) \ (Past(P) ∪ {P})` と明示。列挙関数はPを除き、保存するmergeset_bluesはPを含む。
- blue/redは評価対象に相対的な分類で、ブロック検証の有効/無効や中の全取引の受理を意味しない。blue scoreは親の値+mergeset_blues要素数、blue workは親の値+各要素のwork（下限付き）と説明。親選択はblue_work/hashの比較。
- Anticone/Past/Futureの自己包含、Tipsの観測範囲、Virtualの親選択を整理。全tipを無条件に採用する説明にしない。
- 第20章に合わせ、ノード実装のfinality point/選択規則と、業務上の入金確定を区別。
- トランザクション/Outputの作成・観測と受理/適用を区別。UTXO集合にあってもmaturityや支出条件を満たさなければ即使用可能とはしない。outpointの存在と未使用性は別検証。
- 通常取引のTXIDがsignature script/sigop count/massを除くことを補足。transaction hash・sighashとは区別。
- 署名が身元や取引全体の有効性を単独で証明する説明を避けた。冪等性は一度の適用と同じ効果と定義し、更新の原子性を補足。保存則ではcoinbase発行や手数料を含む式の前提を明示する。
- Dockerのtagは変更可能で、digest固定とソース/バイナリー対応の証明を分けた。
- 第13/17章等へのDAA score・Massの導線とArgent補足への項目を追加。研究/提案・ローカルruntimeとMainnetでの有効化を区別。
- 汎用トップページだけだった根拠の一部を固定ソースと第29章の書誌へ変更。付録Cをリンクにした。

## 照合範囲と資料

rusty-kaspa v1.0.1、基準SHA fcd9c28f9b211b9872a0c90a8c94f79466ffb063として以前取得した手元のソースarchiveを読み取った。新規cloneやGit HEADの取得はしていない。GHOSTDAGのprotocol.rs/mergeset.rs、GhostdagData::new_with_selected_parent、hashing/tx.rsのid/write_transactionを確認。

Webでは固定GitHubソースがcache miss、PHANTOM/GHOSTDAG論文PDFが403で取得できなかった。今回オンラインで再確認した資料とは扱わず、ソースは手元の固定資料、書誌は既存第29章への案内を使った。最新のKIP/実装・Mainnet有効化状態を再調査したものではない。

| 手元の基準版ソース | SHA-256 |
|---|---|
| consensus/src/processes/ghostdag/protocol.rs | `97fe5e1db8afae9d6a3074cf6b6e56ed72e453db9edcbf9b539a4469cc2918ee` |
| consensus/src/processes/ghostdag/mergeset.rs | `8a86df5cf864a17c963be9cd1cf08961c040ea28e4684b238506ec0a51671122` |
| consensus/src/model/stores/ghostdag.rs | `c23fa656f30227291ee75e17cb0634c71a7a02dcea17fdba435ceef61cea1e18` |
| consensus/core/src/hashing/tx.rs | `2f5bb331827f62d816aca1686c4fad1e8c9ba20c70aaca9e71eac934b04b5582` |

第12/13/16/17/20章、Argent補足との本文対応も確認した。計算式の実行比較、全取引検証・UTXO規則、暗号方式の監査、数学的証明、全用語の網羅的な一次資料照合を完了したという意味ではない。既存実験・Rust/SDK/Devnet試験は今回再実行していない。

## 統合・公開確認

- mdBook 0.5.4のHTMLビルド成功。53 HTMLのローカルリンク/アンカー検査で不整合0。
- SUMMARY49件と主表が一致し、前段1〜44番の順序が一致。43番を今回確認、44番の付録Cを次の対象とした。
- scripts/check-book-examples.pyで既存2教材の本文/実装一致を確認。Rustのテストを再実行した意味ではない。
- git diff --check成功。
- 追跡対象/非ignore候補189ファイルの既知token prefix・秘密鍵・credential URL・個人home path等のパターン検出0。gitleaks/trufflehogは未導入で、未知の機密や全Git履歴の完全監査ではない。
- 追加は本記録のみ。私有パス・秘密鍵・生ログ・DB・バイナリーや新しいexamplesは追加せず、既存付録Aの未コミット差分も保持。commit/pushは実施していない。
