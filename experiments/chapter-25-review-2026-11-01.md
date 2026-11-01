# SUMMARY順レビュー 29：第25章「rusty-kaspaのコードベースを読む」

確認日：2026-11-01。基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。本文・図・探索コマンド・主要入口・型/trait断片・DAG教材を確認した。全ソース・全実行経路の監査ではない。

## ワークスペースと調査手順

固定版の既存ソースアーカイブで実行：

```bash
cargo metadata --locked --offline --no-deps --format-version 1
```

終了コード0。packages / workspace_membersはともに59件、resolveはnull。本文の地図9行のディレクトリとpackage名は、この出力と照合した。例としてprotocol/flowsのpackage名はkaspa-p2p-flowsであり、ディレクトリ名をそのままcrate名とは扱わない。

metadata JSONは一時領域にだけ保存した。package ID・manifest_path等は絶対パスを含むため、生JSONや内部パスは公開記録へ追加していない。ソースアーカイブはGit checkoutではなく、この回にgit rev-parseでHEADを取得したものではない。本文の取得手順は第1章の固定checkoutを再利用し、ローカル変更を消す手順を追加していない。

[Cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html)で--no-depsはワークスペース情報のみで依存を取得せず、resolveもnullになることを確認した。--lockedと--offlineの役割も分けた。[Cargo tree](https://doc.rust-lang.org/cargo/commands/cargo-tree.html)はtarget・feature等に依存する依存グラフであり、実行時の呼び出しグラフではない。今回cargo treeによる外部依存全体の再解決はしていない。

広すぎる探索語やheadで切る例を、固定版の具体的なファイル・識別子へ置き換えた。テスト絞り込みには存在を確認した名前を示し、0 testsの成功を対象検証と数えないよう補った。

## 主要な経路と失敗時の更新

- kaspad/src/main.rsはparse_argsとdaemon::create_coreを使う。ノード起動の入口をkaspadとして示し、概念図と実クレートの地図を分けた。
- HandleRelayInvsFlowはsession.validate_and_insert_blockを呼び、block_taskとvirtual_state_taskを分けて扱う。MissingParentsではorphan処理・再投入の分岐がある。
- validate_and_insert_block_implはBlockProcessingMessageと2つのoneshot結果チャネルを使う。block_taskだけの完了をVirtual更新全体の完了へ読み替えない。
- submit_transaction_callはRPC transactionをtry_intoで変換し、FlowContext::submit_rpc_transactionへ渡す。allow_orphanはunsafe_rpcで制限される。FlowContextではMiningManager::validate_and_insert_transactionをawaitし、結果のaccepted集合のIDをbroadcast_transactionsへ渡す。要求IDの返却・Mempool処理・伝播・Virtual受理は別の境界である。
- get_block_template_callからMiningManagerのasync側・spawn_blocking先を追った。同期側はVirtualの近似IDとキャッシュを使い、毎回全候補を再計算するとは限らない。
- 本体処理は多くのエラーでStatusInvalidを保存するが、BadMerkleRoot / MissingParents / PrunedBlockは例外である。Errならヘッダーや索引まで一切変わらないという前提を置かない。
- bounded queueの図は概念例であり、全キューがboundedとは断定しない。本体処理の通知にはunbounded channelを前提にした箇所もあり、チャネル・ワーカー・呼び出し元ごとに調べる。

以前に実行した公式本体テスト2件は `core-validation-review-2026-10-31.md` を参照。今回、validate_body_in_isolation_test / merkle_root_missing_parents_known_invalid_testのソース中の存在と文脈を確認したが、公式コンセンサスの再ビルド・テスト・実ノードの経路は再実行していない。

SHA-256は該当ファイルの識別用であり、全行・全feature・全経路を照合した意味ではない。

| ファイル | SHA-256 |
|---|---|
| `Cargo.toml` | `a4ac0081533750ca27a929b80575463ee59439c412cae748212e24b3bc07cada` |
| `kaspad/src/main.rs` | `1c7e8cd4258a515c1a11c3dc0daff637eaa93fb51cefd92ece15f64ab2b314f8` |
| `protocol/flows/src/v5/blockrelay/flow.rs` | `f6574e5c5b943277660768105f82228b9af1e3c6047d2724fd2e2526d836cabc` |
| `protocol/flows/src/flow_context.rs` | `6f1ed18826f7d9c261ee8bb65e6602f919bfbdca5314673fb733776a4e737014` |
| `consensus/src/consensus/mod.rs` | `0bbe4c56fa696460fcadb3b431b240d7a5f3186d19c0a1b88438bbbb54eec01c` |
| `rpc/service/src/service.rs` | `b07acf50884a1d4c0361f0227952293012a7af6cf49663be5d8b0aad022ad43f` |
| `mining/src/manager.rs` | `b839ccb5b70879b0b91ab63de47f61d085e2358361fa6fe6b19147d53640c348` |
| `consensus/src/pipeline/body_processor/processor.rs` | `5158abde7814277ddab817737b8577073803ea4e0f43faf8c1d05d410c0947c1` |
| `consensus/src/pipeline/body_processor/body_validation_in_isolation.rs` | `a292d8eaa6a5748dceba7f93a20b0a93cbe0469d915b4e6e8df689ca0755110a` |

## 独自教材の修正と実行

DagStoreは既知IDの集合だけでなく親の辺を保持し、明示したrootから始める。新規の空親、重複親、自分自身の親、不明親、既知IDを拒否し、検証後に更新する。整数IDの大小や親配列の順序をKaspaのトポロジー/canonical orderingと同一視しない。

本文からリポジトリ外の一時Cargoプロジェクトへ抽出。Rust/Cargo 1.90.0、edition 2021、外部依存なし。rustfmt結果を本文へ反映した。

```bash
cargo test --offline
cargo fmt --all -- --check
cargo clippy --offline --all-targets -- -D warnings
```

8テスト成功、fmt / Clippyも成功。

1. 親の後に子を追加し、辺が保存される。
2. 一部が不明な親の場合、既存の非空状態を維持。
3. 同じIDを異なる親で再投入しても既存の辺を置き換えない。
4. 明示したroot以外は空親を拒否。
5. 重複親を拒否して全状態を維持。
6. 自分自身を親にせず、全状態を維持。
7. 不明だった親の到着後に再投入できる。
8. 複数親と、IDが数値順に到着しない例を扱える。

25.8のBlockId / BlockHeader / BlockValidatorは別の一時libでcargo check / Clippyを実行して成功。Validatorの実装やPoW検証は含まないため、実行時の検証テストとは数えない。

教材の通常のErrでのメモリ内状態維持と、実ノードの無効ステータス保存、DBバッチ、クラッシュ耐性は別の性質である。PoW・GHOSTDAG・UTXO・ネットワーク・永続化・同期処理をこの教材で実装したものではない。

## 今回の範囲

全呼び出し・外部依存の完全グラフ・全feature/target・全キューの容量・キャンセル/停止・クラッシュ時の耐久性・公式テスト全体は未検証。ノード起動・採掘・鍵生成・署名・送金は行っていない。以前の個別実行記録は、その記録の版・入力・条件に限定して扱う。

## 書籍・公開対象の検査

- mdBook 0.5.4でビルド成功。生成HTML 53ページの内部ファイル・アンカー切れ0件。
- 既存の教材2組がscripts/check-book-examples.pyで一致。手動教材をCIへ追加していない。
- SUMMARYとreview-statusの本文表49項目が同じ順番。次は第26章。
- 実行した2つのRust断片と本文が一致。git diff --check成功。
- 追跡対象・未追跡候補175ファイルの秘密鍵ブロック、既知トークン、認証付きURL、個人ホームパス等のパターン検出0件。gitleaks/trufflehogは未導入で、秘密情報の不存在を保証する監査ではない。
- 新規公開候補はレビュー記録のMarkdown。生metadata JSON、鍵・DB・ログ・ビルド成果物は追加していない。examplesへのフォルダー追加、コミット・pushもしていない。
