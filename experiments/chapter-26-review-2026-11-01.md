# SUMMARY順レビュー 30：第26章「Rustによるコンセンサス実装」

確認日：2026-11-01。基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。本文・図・GHOSTDAG主要経路、ヘッダー保存/Virtual更新の境界、型断片・既存教材への課題を確認した。全コンセンサスの検証ではない。

## 本文・固定ソースの確認

- 概念図を実際の呼び出し順序と区別し、GHOSTDAGがヘッダー処理中に計算されることを明示した。
- 通常validate_headerはヘッダー単独検査（check_pow_and_calc_block_levelを含む）・親関係・文脈作成・GHOSTDAGの後、pre_pow_validation / post_pow_validationへ進む。関数名だけでPoW未検査と推測しない。trusted headerの経路は通常と検査範囲が異なる。
- processorのghostdagは既存メタデータを再利用でき、なければGhostdagManagerを呼ぶ。分類関数自体の戻り値と、呼び出し元による保存を区別した。
- find_selected_parentとSortableBlockの比較、mergesetの列挙と昇順、初期mergeset_bluesのselected parent、selected parent DAAによるk、候補と既存blueのanticone上限を再確認した。非空の親・ストア等の前提とassert/unwrap、Genesis / ORIGINの特例も補った。
- scoreはselected parentのscore + mergeset_bluesの長さ。workはselected parentのwork + 各mergeset blueのcalc_work(bits).max(level_work)の合計。先頭のselected parentを含み、計算対象自身のPoWを加える式ではない。任意のk減少への対応を保証しない。
- commit_headerはGHOSTDAG・ヘッダー・DAA除外/depth・親/Reachability・StatusHeaderOnly等をWriteBatchへ積み、関係するwrite guardをdb.writeまで保持する。window cacheの更新もあり、全キャッシュの巻き戻しやI/O失敗・電源断後の耐久性を実証したとは扱わない。
- commit_virtual_stateはUTXO差分・Virtual state・selected chainの変更を別のWriteBatchへ積む。ヘッダーとVirtualの更新が一つのバッチという意味ではない。全通知・索引・Pruningまでまとめたトランザクションとも扱わない。
- pick_virtual_parentsには候補選択のランダム化がある。親・初期UTXO・適用順・規則/activation等を固定した評価の決定性と、同じDAGを持つノードの一時的なVirtualの完全一致を区別した。複数RPCの観測を同一スナップショットとも見なさない。
- strictなPast/Futureと、自己ancestorを含む公式Reachability APIの違いを明記。Anticone集合内の要素同士が祖先関係を持てることも補った。
- 課題Aの未知/同一ID・共有親だけでは不十分な条件、課題Bの逆適用に必要な削除値・依存順・状態の照合を補足した。追加問い合わせや逆差分は今回実装した機能ではない。
- ソース調査手順は第1章の固定checkout再利用へ変更。metadataの生JSON・絶対パスを公開しない。第25章で確認した--no-depsの範囲と区別する。

既存の固定版アーカイブから該当箇所を読み取った。Git checkoutではないため、この回にgit rev-parseでHEADを取得したものではない。SHA-256はファイルの識別用であり、全行・全経路の監査を意味しない。

| ファイル | SHA-256 |
|---|---|
| `consensus/src/processes/ghostdag/protocol.rs` | `97fe5e1db8afae9d6a3074cf6b6e56ed72e453db9edcbf9b539a4469cc2918ee` |
| `consensus/src/processes/ghostdag/mergeset.rs` | `8a86df5cf864a17c963be9cd1cf08961c040ea28e4684b238506ec0a51671122` |
| `consensus/src/processes/ghostdag/ordering.rs` | `8b6716afbaddd2becd1910e8d80108747daaa97970669cb50c22ff0fca0dc47f` |
| `consensus/src/model/stores/ghostdag.rs` | `c23fa656f30227291ee75e17cb0634c71a7a02dcea17fdba435ceef61cea1e18` |
| `consensus/src/model/services/reachability.rs` | `737aeed155f7d35692fbc8a08eda9c10889bb7279ec4a48ce7ebd1f4d02414c2` |
| `consensus/src/pipeline/header_processor/pre_ghostdag_validation.rs` | `946a6762302a9b8bdc7b8c782d0fc1470e97ad102d9394ee21663addc14fb58f` |
| `consensus/src/pipeline/header_processor/processor.rs` | `27d5e5964e41ad4fd47df07a7c02fe5506386a5f91b13d3288578638cfba443c` |
| `consensus/src/pipeline/virtual_processor/processor.rs` | `e1e92b1925016cba6e032b245e42e5e5e41775e4e41ab10f663110e44617511e` |

## 過去の公式差分比較

`ghostdag-comparison-2026-10-31.md`、`ghostdag/README.md`、`ghostdag/observed-2026-10-31.json`を確認した。24ケース、各親配列順712ブロック、2順で1424比較の過去の記録であり、今回再実行していない。JSONの基準SHA・ケース数・全一致フラグと、各ケースのcomparedBlocks合計712を照合した。

公式側の分類・親選択・mergeset・score/workを実行している一方、到達可能性は両側で同じDFSアダプターを使う。ヘッダー・hash・bits/DAAは合成値であり、公式Reachability・PoW・DAA再計算・全ノード・永続化・任意のDAGの同値性を証明しない。過去の報告にある「章全体レビューは未完了」はその実験時点の範囲であり、今回のページ通読と全技術の検証も別である。

## 型・既存DAG教材の実行

Rust/Cargo 1.90.0、edition 2021、外部依存なし。本書のリポジトリ外の2つの一時libへ、26.3の型断片とexamples/dag-relations/src/lib.rsをそれぞれコピーした。examples側のコードやフォルダー構成は変更していない。

- 型断片：cargo checkとClippy成功。BlockId / InsertErrorは独自の説明用型で、挿入・検証処理は実装していない。
- 既存DAG教材：cargo test --offlineで8テスト成功。strict Past、並行枝、Anticone要素間の祖先関係、拒否時の状態維持、登録順、k-cluster境界等。公式GHOSTDAGの再実行ではない。
- 両方でcargo fmt --all -- --check / cargo clippy --offline --all-targets -- -D warnings成功。

課題のis_ancestor / is_in_anticoneとUtxoDiffの逆適用、新しいストレージ・ノードは実装していない。型だけのコンパイル確認をコンセンサスの実行テストとして数えない。

## 外部資料と制限

[ePrint 2018/104](https://eprint.iacr.org/2018/104)の題名をPHANTOM and GHOSTDAG: A Scalable Generalization of Nakamoto Consensusへ合わせた。書誌の確認であり、原論文の証明を今回全面レビューしたものではない。[RustonomiconのSend and Sync](https://doc.rust-lang.org/nomicon/send-and-sync.html)はスレッド間での転送/共有の性質に関する資料であり、コンセンサスの論理・処理順・障害復旧まで保証するものではない。

ノード起動・採掘・鍵生成・署名・送金は行っていない。全規則、公式Reachability、全キャッシュ/全ストア、全feature、I/O障害・クラッシュの注入、Pruning後の同期、原論文の証明は未検証。今回の保存境界のソース読み取りを、永続化の耐障害性の実証へ広げない。

## 書籍・公開対象の検査

- mdBook 0.5.4でビルド成功。生成HTML 53ページの内部ファイル・アンカー切れ0件。
- 既存教材2組がscripts/check-book-examples.pyで一致。教材・CIの対象を追加していない。
- SUMMARYとreview-statusの本文表49項目が同じ順番。次は第27章。
- コンパイルした型断片と本文、実行したDAGコピーと既存ソースが一致。git diff --check成功。
- 追跡対象・未追跡候補176ファイルの秘密鍵ブロック、既知トークン、認証付きURL、個人ホームパス等のパターン検出0件。gitleaks/trufflehogは未導入で、秘密情報の不存在を保証する監査ではない。
- 新規公開候補はレビュー記録のMarkdown。鍵・DB・生ログ・metadata・アーカイブ・生成物は追加していない。examplesへのフォルダー追加、コミット・pushもしていない。
