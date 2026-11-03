# Mastering Kaspa 最終品質レビュー（2026-11-03）

## A. レビュー概要

- 作業開始ブランチ：`main`。
- 開始SHA：`f43538b5d45d93f7cd347da67b88b7eccec0dd24`。
- 開始時の`git status --porcelain=v1`は空。保護が必要な既存の未コミット変更はなかった。
- 棚卸し対象：開始時の追跡267ファイル。本文の構造・版表記・参照・教材の横断検査は`src/`の59 Markdown（目次58ページ＋SUMMARY）、全36章、Argent、付録A〜Hを対象とした。README、ROADMAP、book.toml、examples、experiments、4検証スクリプトと1 workflowも確認した。
- 技術的な重点照合：第15・16章のTransaction v1、基準版ページ、技術選択マップ、第3章と付録Aの最初のRust実習。第31章とArgentの安全性説明は現行本文を読み、既存の対策・失敗ケースへの導線を維持した。
- 過去のレビュー記録は履歴として確認した。全267ファイルを同じ深さで精読・監査したという数ではなく、全章全主張・全引用を全面的に再検証したという報告でもない。

**総合評価：** 固定v1.0.1の実験結果と現行Mainnetを区別する記述、Covenantの出所・認可、独立Argentページ、日本語・目次・CIの基本構成は維持できる。残っていた主な不足は、後続Transaction v1の署名対象と実装版の比較、用途からの技術選択、最初のRust実験へ進む導線だった。そこに限定して修正した。

36章の番号・ファイル名・ディレクトリ、Argentとエコシステムマップを維持した。新規章・ファイル移動・公開設定変更・コミット・pushは行わない。

## B. 発見した問題

Critical：今回の確認範囲で確定した指摘なし。本番利用の安全性や全記述の正しさの保証ではない。

| 重要度 | 開始時のファイル・行 | 問題・理由 | 根拠・対応 |
|---|---|---|---|
| High | `src/part-03-transactions/chapter-15.md:127`、`chapter-16.md:194` | 旧版の型・署名範囲は対象版付きで正しく説明されているが、v1のcompute budget、binding、IDとALLの変更可能範囲が不足。ALLを「全バイトを拘束」と理解すると資源宣言まで署名したと誤認し得る | v2.1.0の取引型・tx/sighash・hashersを照合。15.5.1へ比較表、16章へ参照を追加。14変更ケースで確認 |
| Medium | `src/part-03-transactions/chapter-15.md:206`、`src/baseline.md:9` | 固定版と現行仕様を混同している明白な記述はなかったが、後続版のMass・Mempool手数料とネットワーク設定の違いをたどりにくい | v2.1.0のMass・Mempool検査、v2.0.1のactivation、公式Mainnet案内を照合。15.9.3と基準版の表へ集約 |
| Medium | `src/ecosystem-map.md:31` | 概念とリンクの一覧はあるが、状態・実行場所・信頼・運用制約を並べて選択できない | 既存資料と各章に基づき用途別の2表を追加。言語とL1機能、研究と個別L2を分けた |
| Low | `src/introduction.md:43`、`src/part-01-foundations/chapter-03.md:400` | 最初の入口はDocker/SDK向けで、Rustだけを試す経路が目立たない。第3章のcargoは版指定がなく、付録Aの呼び分けと揃わない | 付録A→第3章への短い案内を追加。第3章だけ`+1.90.0`と`--locked`を明示。新規一時プロジェクトで追試 |

上記の行は開始コミットでの位置。修正後の節位置・ファイルSHAは[JSON記録](final-quality-review-2026-11-03.json)にも保存する。

### 確認済み・変更不要

- 第31章31.10.1：Covenant ID、Program Identity、Genesis、Lineage、State Provenance、複製と偽造、co-spentと認可、Safety/Livenessは既存説明で区別されている。問題→攻撃→原因→対策→検証の順序を維持し、同じ説明を再追加しない。
- Argent：通常の非同期Actorとの相違、ICCの承認範囲、Genesis認証とartifactの区別、独立ページの登録は維持。
- 第32章：返金・Counter・系列制約の限界が明記されている。30ケースを再実行し、VM評価27、ABI拒否1、context拒否2の区別を維持した。
- 全36章と付録：固定版・教育用コード・既存実験の時点を横断確認した。現行Mainnetの仕様へ一括で読み替える修正や、同じ注意書きの全章追加は不要。
- `book.toml`の日本語設定、SUMMARYの36章・Argent・マップ、READMEの構成説明、ROADMAPの初版完了条件、既存CIの依存・デプロイ条件は変更不要。
- 完全一致する長い説明段落の比較では、第14/15章の固定版確認の定型文1組のみ。意味の重複を網羅的に否定する検査ではないが、削除を必要とする具体的な技術説明の重複は特定しなかった。

### 章別の読者・整合性確認

各行は今回の横断確認の範囲であり、「章全体Verified」ではない。教材のテスト成功から公式仕様との互換性を推論しない。

| 章 | 主な確認点・対応 |
|---|---|
| 01 | 入門図の親子関係、blue/redと有効性、固定ソースへの入口を維持 |
| 02 | 疑似コードの認可・再送・DBの前提を維持。実DB実装と扱わない |
| 03 | 最初のRust実験の準備・固定版・期待出力・2テストへの導線を改善 |
| 04 | P2P伝播モデルと実通信の区別、遅延の近似・重複受信を維持 |
| 05 | BitcoinとKaspaの比較を維持。最小UTXOモデルの非互換・状態保持を検査 |
| 06 | 近似確率を性能・安全性の実測とせず、数値教材を検査 |
| 07 | DAG集合と合意を区別し、Past/Anticoneの教材を検査 |
| 08 | 固定版・研究・稼働の区別とPart IIへの案内を維持 |
| 09 | ノードの責務・基準版の参照を維持。現在の全同期仕様へ一般化しない |
| 10 | 接続・再接続のモデルの範囲を維持し、伝播教材を検査 |
| 11 | 受信・検証・不足親・保存の区別と拒否時状態保持の教材を検査 |
| 12 | k・score/work・DAG解析器と公式GHOSTDAG比較の範囲を維持 |
| 13 | DAAの基準版と近似モデル、フォーク条件・数値教材を維持 |
| 14 | 順序とUTXO受理、Virtualの概念・基準版処理の区別を維持 |
| 15 | v0/v1、ID・署名対象・資源・手数料の不足を重点修正 |
| 16 | 基準版の署名範囲は維持し、v1の差へ参照を追加 |
| 17 | Mempoolの方針と合意状態を区別し、予約・競合教材を検査 |
| 18 | テンプレート教材と実PoW・報酬処理の区別を維持 |
| 19 | 経済モデルの仮定・基準版Massを維持。新しいMassは15章へ集約 |
| 20 | finalityと業務の確認ポリシーを区別し、教育用判定を検査 |
| 21 | 実Pruning/IBD未検証を維持。snapshot教材を実同期成功としない |
| 22 | RPC版・NetworkId suffix・再照会と送信の区別を維持 |
| 23 | u64精度・SDK/WASM境界・教育用アダプターの範囲を維持 |
| 24 | HD復旧とProvider Discovery、提案と基準版の区別を維持 |
| 25 | metadata・ソース閲覧・実ビルドを別の確認段階として維持 |
| 26 | 公式処理の読解と教材の役割、保存単位の注意を維持 |
| 27 | 非互換の最小ノードとして本文同期・16テストを検査 |
| 28 | 固定Devnetの環境・期待結果・停止手順への導線を維持 |
| 29 | Testnet-13設定の存在とMainnet有効化・論文証明の区別を維持 |
| 30 | KIP/KCCの状態と実装・ネットワークの区別、固定索引を維持 |
| 31 | 状態の同一性と遷移の妥当性、出所・認可・回復条件は既存で充足 |
| 32 | コンパイラー／ABI／VM／ネットワークの区別を維持。30ケース再検査 |
| 33 | L2、DA、ブリッジ、ZKとアプリ安全性の区別を維持 |
| 34 | 研究構想と公開prototype、証明対象の業務ルール・退出の注意を維持 |
| 35 | 部分実装・実データ変換・未実装の設計例を区別し、同期・テスト検査 |
| 36 | 貢献手順と合意変更の扱い、本書から次へ進む導線を維持 |

付録Aは最初の2テストとHello Worldを新規プロジェクトで追試した。Bの用語、Cの数学・確率、Dの基準版RPC、Eの基準版パラメータ、Fの固定KIP索引、Gの実験履歴、Hの証拠区分を確認した。付録内Rustも一括コンパイル対象とし、新APIや同じ比較表を各付録へ重ねて追記していない。

## C. 修正内容と一次資料

| 変更ファイル | 追加・整理した内容 |
|---|---|
| `src/part-03-transactions/chapter-15.md` | 15.5.1のv0/v1・ハッシュ比較、15.9.3のMassと手数料 |
| `src/part-03-transactions/chapter-16.md` | v1で異なる署名範囲への参照 |
| `src/baseline.md` | 再現基準・Released実装・ネットワーク設定の比較 |
| `src/ecosystem-map.md` | 実行場所・状態・信頼・制約・状況・用途の技術選択表 |
| `src/introduction.md` | Dockerを使わないRust実習ルート |
| `src/part-01-foundations/chapter-03.md` | 付録Aへの案内、Rust版・lockfileを使う実行コマンドへ置換 |
| `src/review-status.md` | 今回の確認記録への参照。過去の検証結果は維持 |
| `experiments/transaction-v1/` | `checks.rs`、`run.py`、README、14ケースの結果JSON。上流関数を直接比較する限定実験 |
| 本記録・JSON | 開始状態、対象、根拠、検査・限界、修正箇所を記録 |

章や歴史的仕様の説明は削除せず、古い実験結果を現行版の成功結果へ書き換えていない。

一次資料の確認日は2026-11-03。取得物のSHA-256はJSONへ記録する。

- [v2.1.0リリース](https://github.com/kaspanet/rusty-kaspa/releases/tag/v2.1.0)：Releasedタグとpost-Toccata実装の位置付け。SHA `01b532e8b553523216471682649693af92f0fd16`。
- 同固定版の[取引型](https://github.com/kaspanet/rusty-kaspa/blob/01b532e8b553523216471682649693af92f0fd16/consensus/core/src/tx.rs)、[ID/完全ハッシュ](https://github.com/kaspanet/rusty-kaspa/blob/01b532e8b553523216471682649693af92f0fd16/consensus/core/src/hashing/tx.rs)、[署名ハッシュ](https://github.com/kaspanet/rusty-kaspa/blob/01b532e8b553523216471682649693af92f0fd16/consensus/core/src/hashing/sighash.rs)、[ハッシュ方式](https://github.com/kaspanet/rusty-kaspa/blob/01b532e8b553523216471682649693af92f0fd16/crypto/hashes/src/hashers.rs)：v1のcompute budget、binding、BLAKE3によるID、ALLの範囲。
- 同固定版の[Mass](https://github.com/kaspanet/rusty-kaspa/blob/01b532e8b553523216471682649693af92f0fd16/consensus/core/src/mass/mod.rs)、[Mempool方針](https://github.com/kaspanet/rusty-kaspa/blob/01b532e8b553523216471682649693af92f0fd16/mining/src/mempool/check_transaction_standard.rs)：資源別評価と最低中継手数料。単純な固定手数料を新たに推奨していない。
- [v2.0.1設定](https://github.com/kaspanet/rusty-kaspa/blob/cfafeb4c093fa37a303f1b9f19c58f986b870ce3/consensus/core/src/config/params.rs)、[v2.0.0リリース](https://github.com/kaspanet/rusty-kaspa/releases/tag/v2.0.0)、[公式Toccata案内](https://docs.kaspa.org/toccata)：有効化設定、リリース時の予定と、公式案内のMainnet有効化済み記載を別の証拠として確認。実ノード未照会。
- [KIP-20](https://github.com/kaspanet/kips/blob/d832dbd89347dcb16d3d83bf6755b081714cbcb2/kip-0020.md)：Active、binding・継承／Genesis・署名／IDへのコミットメント。KIPの状態だけでネットワークを判断しない。
- [公式技術選択案内](https://docs.kaspa.org/programmability)、[Covenants](https://docs.kaspa.org/programmability/covenants)、[固定Argent README](https://github.com/argent-lang/argent/blob/2796809f167144902f04a4b3333a4c144f91fdaf/README.md)、既存の第31〜34章の固定一次資料：技術選択と、公開実装／研究・運用の区別。

新しい発展機能の実装・章の追加、ウォレット・L2の本番対応の断定はしていない。

## D. 検証結果

環境：macOS/ARM64、Python 3.11、Rust 1.90.0・1.94.0、Node.js 20、mdBook 0.5.4。依存取得済みの範囲はオフラインで実行した。

| コマンド・対象 | 結果・限界 |
|---|---|
| `mdbook build -d /tmp/mastering-kaspa-final-quality-book` | 成功 |
| `python3 scripts/check-book-links.py --book-dir /tmp/mastering-kaspa-final-quality-book` | 内部ファイル・日本語アンカー・画像参照を検査、成功。件数は最終JSON参照 |
| Markdownのローカルリンク・自リポジトリURL | ソース実体・見出しも検査。新規記録は公開前なのでローカル実体として確認 |
| SUMMARY・コードフェンス・構文 | 58参照が重複なく全ページと一致、36章・Argent・マップを維持。Python/CJS/JSONとフェンス対を検査 |
| `python3 scripts/check-book-examples.py` | 本文と第12/27/35章の収録ソース3件が一致 |
| `python3 scripts/check-rust-snippets.py` | 54 Rustブロック、233テスト、main実行が成功 |
| `python3 -m unittest discover -s experiments/devnet -p 'test_*.py'` | 7テスト成功。実ノード不要 |
| `python3 scripts/check-utxo-example.py` | 2テスト成功 |
| 収録3crateの`cargo +1.90.0 fmt --check`、`clippy --offline --locked --all-targets -- -D warnings`、`test --offline --locked` | 書式・Clippy、8＋16＋19＝43テスト成功 |
| 第32章の`experiments/silverscript-vm/run.py … --offline` | 既存30ケースを再実行、期待と一致。前回JSONのファイル識別値も照合。ネットワーク送信なし |
| 同VMハーネスのRust 1.94.0 fmt・Clippy | 成功 |
| v2.1.0公式`cargo +1.94.0 test … -p kaspa-consensus-core --locked --jobs 2 hashing::` | 5単体テスト成功。別integration targetの0件／filtered表示を成功件数へ加算しない |
| 新`experiments/transaction-v1/run.py … --offline` | integration test 1件の内部で14変更ケースをassert、すべて期待と一致。ハッシュ比較でありVM・署名検証ではない |
| 新ハーネス`rustfmt +1.94.0 --edition 2024 --check …`、公式workspace内`cargo +1.94.0 clippy … --test book_tx_v1 --no-deps -- -D warnings` | 成功。上流全workspaceのClippyではない |
| 付録A→第3章の新規プロジェクト追試 | Hello World、付録Aの2テスト、SHA-256の期待出力・2テストが成功。OS／Rust新規インストールからの試験ではない |
| `git diff --check` | 成功 |

v2.1.0の公式テストは、初回`--offline`でworkspaceの未取得Git依存により失敗した。ネットワーク取得を許可された条件で`--locked`のまま再実行し成功。その後、追加14ケースはオフラインで成功した。上流Cargo.lockや既存ソースを改変して通した結果ではない。推測した`docs/toccata.md`は404だったため使用せず、リリースが参照する`docs/toccata-guide.md`と公式案内へ戻って確認した。

CIは`build`の教材同期・Python・mdBook・リンク検査、Rust 3教材のmatrix、両ジョブ完了後のmain向けPages条件を確認した。PRではデプロイしない。今回workflowを変更せず、GitHub実ジョブ／公開設定も操作していない。Rust 1.94.0・公式workspaceのハッシュ比較とSilverScriptは手動実習のままで、CIが自動実行するとは記載しない。

## E. 残課題と公開品質

- 全論文の安全性証明、全章全引用・全規則の全面照合、本番コントラクト監査は完了していない。今回の横断レビュー・関数試験で保証できる範囲を超える。
- Mainnet／Testnetのノードは照会・送金していない。ネットワークの説明は公式案内・Releasedソースに基づく。旧固定Devnetの起動・送金・Pruning後IBDも今回再実行していない。
- v1の14ケースは合成データのハッシュ比較。全SIGHASHモード、実署名、VM、予算上限、storage mass照合、Mempool受理、Genesis認証の網羅的な試験ではない。
- SilverScriptの30ケースは固定コンパイラーの依存先でのローカル検査で、今回のv2.1.0へ移植・送信した結果ではない。
- 他OS・CPU、ブラウザー表示／日本語検索／モバイル、機器ウォレット、L2のブリッジ／退出、全既存外部URLのHTTP取得は未確認。
- 将来の改訂では、再現基準を更新する依頼があった時点でv1の実SDK／実ノード送信、他署名モード、資源境界を固定ベクトルで追加確認するとよい。初版の必須作業を自動的に増やすものではない。

今回、修正後にも残る具体的な初版公開の阻害要因は確認範囲では特定していない。これは「完璧」「本番運用の安全性確認済み」という判定ではない。ROADMAPの完了済み項目はそのまま維持する。

### GitHub Aboutの推奨値（設定は変更しない）

- Description：`KaspaのBlockDAG・GHOSTDAG・Rust実装を、固定版ソースと再現実験で学ぶ日本語技術書。`
- Website：`https://shiga0.github.io/mastering-kaspa/`
- Topics：`kaspa`, `blockdag`, `ghostdag`, `rust`, `mdbook`, `blockchain`, `japanese`, `distributed-systems`

最終の変更ファイル一覧、`git diff --stat`、`git status`、確認したファイルの識別値はJSONと作業終了時の報告を参照する。新規ファイルは通常の`git diff --stat`へ含まれないため、別に列挙する。
