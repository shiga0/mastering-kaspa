# SUMMARY順レビュー 38：第34章「vProgsとKaspaのプログラマビリティ」

確認日：2026-11-01。公開案内・研究投稿・固定プロトタイプの主要箇所と、独自Rust教材を確認した。Part V第29〜34章の目次順本文レビューは一区切りだが、全研究/公式実装/本番機能の検証完了ではない。

## 公開案内と研究資料

- [Programmability Overview](https://docs.kaspa.org/programmability)はCovenants/Based Apps/Inline ZK/Full vProgsを区別する。[Based Apps](https://docs.kaspa.org/programmability/based-apps)は単一アプリの組み込みアカウント・共有状態、構築中という説明。[Full vProgs](https://docs.kaspa.org/programmability/full-vprogs)は独立アプリ間の合成を将来の方向性として説明する。宣伝上の名称から完成度を推定しない。
- [Toccata Dev Guide](https://docs.kaspa.org/toccata)は2026-06-30、DAA score 474_165_565でMainnet有効化と記載する。元の稼働日を削るのではなく、確認日に読んだ公式文書の記載へ結び付けた。本番ノードで独立観測したという意味ではない。
- [Computation DAGの形式モデル](https://research.kas.pa/t/zoom-in-a-formal-backbone-model-for-the-vprog-computation-dag/407)は2025-08-17の投稿。read/write集合・状態Witnessを持つ操作列と計算依存scopeの縮減を論じる。BlockDAG、DA/永久アーカイブ、全体コストの単調減少と同一視しない。
- [全体構想](https://research.kas.pa/t/concrete-proposal-for-a-synchronously-composable-verifiable-programs-architecture/387)は2025-08-07の投稿。[Proof Stitching](https://research.kas.pa/t/a-basic-framework-for-proofs-stitching/323)は2025-04-02のproof of conceptで、logic zoneのsubtransaction・index・入出力/call stack・証明を扱う。異なる研究投稿を単一の完成した仕様と混ぜない。
- 上記は確認時点のWeb内容で、固定snapshotを収録していない。Yellow PaperはGitHub上のファイル表示までで、今回PDF本文・全証明を照合していない。参考資料でも追加調査先と明示した。

## 固定プロトタイプ

公開kaspanet/vprogsのls-remote HEADを確認し、本書外へshallow clone。Git HEADはf9b84a863a7c7c20586a9cf947550475e894f72e、コミット日は2026-07-28。既存checkoutや本書のGit設定を変更していない。調査後の追跡ソース差分もなかった。

- READMEはearly development / prototypeと明記し、based computationのframeworkと説明する。リポジトリ名からFull vProgsの完成を推定しない。
- core/types/src/lib.rsの再exportはSchedulerTransaction等。README中のTransactionという略記をそのまま実API名と扱わない。
- storage/typesのReadStore/Store/WriteBatch、state/versionのStateVersion、scheduling/schedulerのScheduledBatch/StateDiff等、transaction-runtime/programのProgramType/ELF bytes、l1/bridgeの接続責務、zk/abiの構造を主要入口から確認。
- scheduling/scheduler/src/rollback.rsにpub struct Rollbackはあるが、lib.rsはrollbackをprivate moduleとして扱い、Rollbackの公開再exportをしていない。型の存在と外部からimport可能なAPIを区別した。rollbackの全ストア更新・障害時整合性を監査したものではない。
- node/vm/src/lib.rsのVM::process_transactionはtodo!を含み、tx_image_id/batch_image_idは0を返す。この経路を完成した汎用VMとしない。一方、別のrunner等があるため、この1か所から全コードが機能しないとも一般化しない。
- Cargo.tomlにl1/zk/runner/sim等もあるため、古いREADMEの6層だけで実コード配置を断定しない。
- Cargo.tomlはKaspa依存をbranch=masterで記述するが、固定Cargo.lockのGit revisionは9fdbaf1b1886fb5742783d2da73c36239bd6f272。本書の基準ノードfcd9c28、Silverscript依存a41a333、固定KIP e4ae233と別の参照軸。

```bash
cargo +1.94.0 metadata --offline --locked --no-deps --format-version 1
```

成功し61パッケージを列挙、resolveはnull。no-depsであり依存解決・ビルド/公式テスト成功ではない。生JSONは個人ローカルパスを含むので本書外に保持し、収録していない。core/atomics/src/atomic_ring.rs、storage/canonical-chain/tests/chain.rs、state/proof-receipt/tests/roundtrip.rs、scheduling/scheduler/tests/e2e.rs、node/framework/tests/e2e.rs、runner/src/config.rs等にテストの所在を確認したが、公式テストは実行していない。実ノードや証明用toolchain等を使う試験を、無条件に一括実行する指示にもしていない。

## 設計の境界と独自教材

- proofのoperation commitmentとcontextの拘束、old root/sequence/programの検証側照合を補足。公開入力を列挙するだけでは回路の拘束を意味しない。
- アプリ間のall-or-nothingは同じ論理操作/整合した観測の安全性であり、進行性は別要件。メモリー上のcloneによる原子性を、複数アプリ/ノード・DB・外部作用の原子性と同一視しない。
- 図は責務の概念整理で、全呼び出し順や単一の保存処理ではない。研究の計算依存DAGとブロックのDAGも区別した。
- 教材は空のfrom/toラベルを拒否し、総量のテストをu128へ変更。実行不要な空mainを足すbinから、公開型/関数のlibへ整理した。
- 架空IDによるアカウント残高モデルであり、公式vProgの状態所有権、認証/署名、nonce/Replay、L1順序や永続化・証明/commitmentを実装しない。残高0のentryが残るため、経済的等価性と辞書/rootの同一性も別の仕様とした。

本文を本書外の一時Cargo libへ抽出。Rust/Cargo 1.90.0、edition 2021、外部依存なし。rustfmtを反映し、本文と実行ソースの完全一致を確認した。

```bash
cargo test --offline
cargo fmt --all -- --check
cargo clippy --offline --all-targets -- -D warnings
```

9テスト成功、fmt/Clippyは終了コード0。

1. 通常移転の各残高・総量保存。
2. 残高不足で元状態を維持。
3. 自己送金で増額しない。
4. 受取人overflowを拒否。
5. 金額0・空の送信元/宛先を拒否。
6. 自己送金でも残高不足・未知アカウントを拒否。
7. 送金順を変えると成否が変わる具体例。
8. 両残高/金額を各0・1・10・u64::MAXとする64ケースで、成功時の総量保存と全ケースの入力不変。
9. 同じ操作の再適用で2回移転でき、Replay防止が未実装であること。

有限グリッドの性質確認であり、全入力の形式証明やproptest実行ではない。演習のバッチ・rollback・複数アプリ等は実装済みにしていない。

## 今回行っていないこと

公式vprogs/全ノードのビルド・テスト、ZK証明生成/暗号方式の全面照合、Full vProgsの実相互運用、実アプリ間原子性、DA/復旧/退出、現行Mainnetの独立観測、採掘・鍵生成・署名・送金は行っていない。基準版Dockerや既定Rustは変更せず、examples/CI対象を増やしていない。

## 読んだ主要ファイルのSHA-256

該当箇所を含むファイルの識別用であり、全ファイルの実装/仕様監査の印ではない。

| ファイル | SHA-256 |
|---|---|
| README.md | `0f171c036bb2c62dcf100eda76867b42fc1bdba203045e8cde6238be0fb8c7d9` |
| Cargo.toml | `0303e247ae3662b3b1eff42f61cfc0b6457d9a6431e1952531e89176737aebf8` |
| Cargo.lock | `98cc62c70291ec83ba8ed9cc490e358f5b52e6b09a2b4b5433e1e392af74344e` |
| core/types/src/lib.rs | `992e74e356a6414b4a2f77da5dc9d6bd5deb665256f5b54c411d4b324b54070c` |
| core/types/src/scheduler_transaction.rs | `551f5bf96ccf02f4d399fb7de5204c50f66fd75c25a971a6ea6afed8cf794662` |
| storage/types/src/lib.rs | `1b286fb6af8dbd8409148747b1fbe48f6bc77e5bde13c504506dad75a91c12af` |
| state/version/src/lib.rs | `6da59de11ef44d7e88e20f38a94fff30a096fbf237dbc234fa4a21d164ca6251` |
| scheduling/scheduler/src/lib.rs | `d651257329977d38d5b51257df885fd00104d7309f7b48c548654f23845a5bd6` |
| scheduling/scheduler/src/rollback.rs | `07230789ad3d5959430263d3cf27a645354c751a3c1def881d95b78df48c7429` |
| transaction-runtime/program/src/lib.rs | `702087395619933e02009c8703159a6cbf27022c05d550115a3dfd058ec00fd3` |
| node/vm/src/lib.rs | `3f49fc48f9c4b157ca56dfaae76e6dd20a3171b7431c30bd717757a593c8a79f` |
| l1/bridge/src/lib.rs | `dfe945f37af7307831d0156616b3012f99ab6c5deb4d517ad1b495c0e038afa9` |
| zk/abi/src/lib.rs | `0271477663e6f3b8446643baa3bcef776a5813ecb1a7b0a43de59f22b9959248` |

## 書籍・公開対象の検査

- mdBook 0.5.4でビルド成功。生成HTML 53ページの内部ファイル・アンカー切れ0件。
- scripts/check-book-examples.pyで既存教材2組が一致。本章の独自教材は本書外での手動検査で、examples/CI対象を増やしていない。
- SUMMARYとreview-statusの本文表49項目が同じ順番。次は第35章。git diff --check成功。
- 追跡対象・未追跡候補184ファイルの秘密鍵ブロック・既知トークン・認証付きURL・個人ホームパス等のパターン検出0件。gitleaks/trufflehogは未導入で、秘密情報の不存在を保証する監査ではない。
- 公開候補は本文・進捗・レビュー記録のMarkdown。鍵・DB・生ログ・metadata生JSON・ソースclone・PDF・アーカイブ・ビルド成果物を追加していない。コミット・pushも行っていない。
