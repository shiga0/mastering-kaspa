# SUMMARY順レビュー 24：第20章「ネットワークセキュリティとファイナリティ」

確認日：2026-11-01。基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。本文全体・図・確認ポリシー教材・実習導線と既存の照合記録を確認した。

## 本文と固定ソースの確認

- PoWが実際の試行回数を直接証明するという説明を、条件を満たす計算結果と期待仕事量の説明へ修正した。
- 観測段階の図とGHOSTDAGのヘッダー内計算を区別。一時的なVirtual viewの差を、それだけで確定済み結果のSafety違反と扱わないようにした。
- calc_finality_pointのselected parentがORIGIN、blue scoreがdepth未満、Pruning pointとの距離・chain条件の境界を補足。depthは選択親DAAで選ぶが、深さ判定にはblue scoreを使う。depth増加時も選択親の既存の点から進める。
- virtual_finality_pointのPruning pointへのフォールバック、sink候補のchain条件、Pruning point一覧検査の浅い違反の見逃しに関するコメントを再確認した。
- 基準版Devnet/Crescendoの定数の説明は既存の主要経路レビューを維持。時間をそのまま入金確認の推奨値としない。
- 入金の重複管理は出力単位か取引単位かを明記し、同一TXの複数出力をTX IDだけで取りこぼさないようにした。settled後も異常の調査・訂正を考える図にした。
- 過去の分断・再接続・報酬UTXO削除の観測と、今回の教材テストを区別。Pruning pointがGenesisのままである実測を、Pruning後IBDの成功へ拡張しない。
- 固定版の再利用、lib/edition、作業場所、関連章と研究資料の導線を整理した。生ログ・秘密情報を公開しない旨も既存の公開範囲と整合させた。

今回再読した主な固定ソース：

| ファイル | SHA-256 |
|---|---|
| [consensus/src/processes/block_depth.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/block_depth.rs) | `e09e4186b20aec3fedb3dec898000a6f29da690e851a844c35edfdfda06d5111` |
| [consensus/src/pipeline/virtual_processor/processor.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/virtual_processor/processor.rs) | `e1e92b1925016cba6e032b245e42e5e5e41775e4e41ab10f663110e44617511e` |

既存の定数・派生値・主要経路と旧教材4テストは `core-validation-review-2026-10-31.md` と対応するソース識別JSONを参照した。新規の公式Finality実行テストや全フォーク境界の検査ではない。

Eclipseの背景研究には [USENIX Security 2015の原論文ページ](https://www.usenix.org/conference/usenixsecurity15/technical-sessions/presentation/heilman) を参照した。Bitcoinを対象とする研究であり、現在のKaspaの脆弱性や攻撃費用を実測した証拠とはしない。

## 教材の変更

関数は履歴や会計DBを持たないため、PaymentState/Settledという表現をPolicyDecision/PolicySatisfiedへ変更。現在の証拠が独自ポリシーに一致したという判断だけを返し、入金を自動確定・永続化しないと明記した。

閾値min_confirmationsは正の値とし、ゼロはInvestigationとする教材の方針を追加。evidence_freshを呼び出し元から受け取り、古い証拠は未同期・競合と同様に調査対象へ回す。

confirmationsは外部から与える抽象的な進行指標で、実際のRPC値から算出する処理を実装しない。evidence_freshも取得時刻・viewの整合性を証明する仕組みではない。閾値3はテスト用で、安全な運用の推奨値ではない。

前回の受理やsettledを保持しないので、revertedの検出・古い通知の排除・入金の二重計上防止はこの純粋関数で実現したことにならない。

## 教材の実行

本文のRustブロックを一時Cargo libプロジェクトへ抽出し、rustfmtの結果を本文へ反映。Rust/Cargo 1.90.0、macOS ARM64、edition 2021、外部依存なしでcargo test --offline、cargo fmt --check、cargo clippy --offline --all-targets -- -D warningsを実行し成功した。

8テスト成功：

1. 受理済みで条件を満たす場合はPolicySatisfied。
2. 未同期ならInvestigation。
3. 競合を観測したらInvestigation。
4. 現在の受理がなければ、古い正のconfirmationsだけで昇格せずObserved。
5. 閾値ゼロならconfirmationsが0でもu64::MAXでもInvestigation。
6. 正の閾値3に対して0/2/3/u64::MAXの各境界を確認。
7. 古い証拠ならconfirmationsがu64::MAXでもInvestigation。
8. 入力の進行量や受理が変わると、再評価の判断はPolicySatisfiedからConfirming/Observedへも変わり、条件が戻ればPolicySatisfiedへ戻る。

5〜8が追加テスト。8はステートマシンやDBの巻き戻しの試験ではなく、別々の証拠を同じ関数へ渡す再評価の試験。cargo newから新規環境で手入力した検査とも区別する。

## 既存の実測と今回の限界

- `devnet-partition-2026-10-31.md`：両ノードの分断・別枝生成・再接続後のDAGと指定報酬先UTXOの収束。
- `devnet-reorg-observer-2026-10-31.md`：所定の報酬UTXOの削除通知と観測SQLiteの補正。
- `devnet-pruning-review-2026-10-31.md`：Pruning pointがGenesisで、実際のPruning後IBDは未達。

これらは過去の実行記録であり、今回再実行していない。全UTXO・競合送金の勝敗・会計台帳・不可逆性・攻撃成功確率の保証へ広げない。Dockerバイナリと固定ソースの由来の完全一致も既存の限界を維持する。

今回は鍵生成・ノード起動・採掘・署名・送金を行っていない。新しいexamplesフォルダー・生成物は公開物へ追加せず、CI同期対象も変更していない。

Part III第15〜20章の本文確認は一区切りだが、全規則・全異常系・証明を検証した意味ではない。次はSUMMARYの25ページ目、Part IV第21章 `src/part-04-development/chapter-21.md`。

## 統合・公開物の確認

既存の本文とRustソースの同期対象2件が一致。mdbook build成功、生成HTML53ページの内部ファイル・アンカーのリンク切れ0件。SUMMARYとreview-statusの49項目は名称・順序一致。git diff --checkも成功した。

追跡済み・追加候補170ファイルへの既存の秘密情報パターン検査は検出0件。gitleaks/trufflehogは未導入であり、秘密情報が一切ないとの保証ではない。新しいexamplesフォルダー、生成物、秘密鍵、個人パスは追加していない。
