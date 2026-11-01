# SUMMARY順レビュー 48：付録G「実験結果・テストベクトル」

確認日：2026-11-01。全文を通読し、実験索引と既存記録の対応、独自UTXO教材、掲載SHA/DAG例を確認。ノード起動/採掘/送金・既存公式比較は再実行していない。

## 実行した最小例（G-001〜004）

今回のrun識別子はG-001〜004-20261101-review（実体は本記録で、別の結果フォルダーを作成していない）。Darwin/arm64、Rust/Cargo 1.90.0、edition 2021、Python 3.11.0、shasum 6.02。

- G-001：shasum -a 256のstdinへ改行なしの空bytes/abcを渡し、掲載値とPython hashlibの両方に一致。SHA-256一般の2既知値であり、Kaspa PoW/署名ハッシュの試験ではない。
- G-002/003：本文src/lib.rsを外部依存のない一時Cargo libに抽出。cargo test --offlineで8テスト成功、cargo fmt --all -- --checkとcargo clippy --offline --all-targets -- -D warningsも成功。
- G-004：本文のedges_child_to_parent JSONを直接parse。Pythonの再帰的集合モデルで親の存在・閉路なし・Past(D)={A,B,C}、Past(E)={A,C}、Anticone(D)={E}を確認。別途Rust祖先例が実行済みの付録Cの記録と対応付けたが、Rust祖先例を今回再実行していない。

UTXO教材は元の4テストに、消費済み出力の再消費、既存outpointとの衝突、空入力、入出力合計overflowの4テストを追加。入力内の重複を「再消費」と同一視しない名前へ変更し、MissingInputを含め拒否後の台帳不変を確認。doc-test0件は件数へ加えない。

正常例は100入力→60/35出力でfee5、台帳95。固定ケースの確認であり、全入力空間の性質証明やfuzzingではない。署名/Script/Mass/Coinbase/合意順序を実装するモデルではない。整形後ソースと本文が一致し、掲載ソースのSHA-256は以下。

`5042258915d62210bacb95c105b68513781c830e0c65b2f1b8ad8816cdc1f77e`

新規toolchain導入や新しいexamplesフォルダーは作成していない。本文の+1.90.0は読者用版指定で、実行には既存の同版toolchainを使った。

## 既存実測との対応（今回の再実行ではない）

| ID | 照合した記録 | 範囲と限界 |
|---|---|---|
| G-005 | ghostdag-comparison-2026-10-31.md | 24ケース/親順ごと712ブロック。合成ヘッダー・共通DFS等の条件を維持 |
| G-006 | daa-comparison / daa-window-comparison / daa-transition-comparison-2026-10-31.md | 算術27、Full窓120、Sampled/Dual288ケース。全sample rate/PoW/Pruning後IBDへ広げない |
| G-007 | devnet-two-node-2026-10-31.md | 固定ARM64構成で2ノード伝播・B停止後復帰、27ブロック/DAAと指定アドレス26 UTXOの一致。全UTXO・Pruning後IBD/同期時間測定ではない |
| G-008 | devnet-live-reconciliation-2026-10-31.md | 購読終了/再購読と現在UTXO再照会、古い応答拒否。単一プロセス/単一アドレスで全履歴再生や自動継続運用ではない |
| G-009 | devnet-notifications-2026-10-31.md | 受けた実通知をアプリ側で二重適用しSQLiteの値不変。ネットワークの重複配信試験ではない。reorg removedは別記録を案内し、会計取消/複数ワーカーは未完 |
| G-010 | devnet-environment / devnet-pruning-review-2026-10-31.md | 通常再起動の保持は既存実測。Pruning pointはGenesisのままで前提未達、実削減・削減後再起動/新規ノードIBDは未実施 |

G-007の「構成と版未固定でBLOCKED」は現状と合わず撤回。IDごとの限定した成功とテーマ全体の完了を区別。G-010はINCONCLUSIVEとし、通常再起動の成功でPruning試験を完了扱いにしない。

## 文書/公開形式の修正

- 提案フォルダー群を実在ファイルの図と保存形式の例へ分離。experiments/rust/vectors/resultsやexperiments.ymlは作っていない。
- 独自UTXO教材の作業先は本書外。CI例は参考とし、実際のdeploy.ymlが付録教材を自動検査済みとしない。版1.90/contents: readを示したがworkflowは変更・実行していない。
- DAG図はJSONにない辺とも読めたため、D→B/C、E→Cと祖先Aの経路列挙へ変更。
- NOT_RUN/RUNNING/PASS等の実行状態を明示し、NOT_RUNのJSON/YAMLは未実行テンプレートと実測結果を分離。
- 冪等性ではadded→removed→addedを永久的な既処理集合で潰さず、現在状態と履歴の同一性を分ける。
- 生ログ・私有パス・peer識別子・DB等の保存と公開を区別。公開用集計/版/hashを使い、生stdout/stderrをGitへ追加することを必須にしない。
- 未コミットの教材はソースhashと未コミット状態も記録し、実行回数/CI/独立性を単一の保証序列へ読み替えない。

[Cargo test公式説明](https://doc.rust-lang.org/cargo/commands/cargo-test.html)を参照。本文のコマンドは本書の実測Rust版で検査した範囲に限定する。

## 統合・公開確認

- mdBook 0.5.4のビルド成功（警告なし）、53 HTMLのローカルリンク/アンカー不整合0。
- SUMMARY49件と主表、前段1〜49番の順序が一致。48番の付録Gを今回確認、49番の付録Hを次の対象とした。
- scripts/check-book-examples.pyで既存2教材の本文/実装が一致。
- git diff --check成功。
- 追跡対象/非ignore候補194ファイルの既知token prefix・秘密鍵・credential URL・私有home path等のパターン検出0。gitleaks/trufflehogは未導入で、未知の機密やGit全履歴の完全監査ではない。
- 既存付録A〜Fの未コミット差分を保持。今回の新規ファイルは本記録のみ。生ログ・私有パス・秘密鍵/DB・バイナリー・新しいexamplesは追加していない。commit/push/実ノード操作はしていない。
