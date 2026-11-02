# 現行原稿の横断レビューと再実行検証

実施日：2026-11-03（Asia/Tokyo）。対象リポジトリ：`shiga0/mastering-kaspa`。

## 開始状態と対象

- 開始コミット：`4f222ccb0087beb4d66d9eeae10edfa4ea47f109`。
- 開始時の `git status --porcelain=v1` は空。保護すべき未コミット変更はなかった。
- `git ls-remote origin refs/heads/main` の結果も上記SHAと一致。fetch・pull・checkoutは行っていない。
- 開始時の追跡ファイルは257件。そのうち要求された範囲の247件をインベントリ化した。`src/` のMarkdown59件（SUMMARYを含む）、README・ROADMAP・book.tomlの3件、workflow 1件、既存scripts 3件、examples 17件、experiments 164件。
- 247件すべてを人手で精読した件数ではない。現行本文を入力にした構造・表記・参照・版の横断検査と、主要仕様・実習・記録の重点照合を行った。全36章・付録の全行再通読、全主張の独立検証を完了したという判定にはしない。
- 開始時各ファイルのSHA-256、環境、再実行の結果と限界は[機械可読記録](repository-review-2026-11-03.json)に保存した。

全36章・Argent・付録A〜Hを横断検査の対象とした。技術的な重点照合は、第3・5〜8章の暗号/Bitcoin/DAG、第11〜21・26章の検証/GHOSTDAG/DAA/UTXO/署名/報酬/Finality/Pruning、第22〜24・28・35章のSDK/復旧/アプリ境界、第29〜34章・Argentの研究/提案/実装段階とした。教育用Rustコードは後述のスクリプトで現行本文から抽出して実行した。

Chapter 01〜36の章番号・パス・ディレクトリ、独立Argentページ、ecosystem-mapと目次登録を維持した。基準ノードのv1.0.1を新しいリリースへ変更していない。コミット・Push・公開操作は行っていない。

## 候補A〜Cの再確認

| 候補 | 現状と対応 |
|---|---|
| A：HD復旧 | baselineの「バックアップ復旧が未検証」は、暗号化DB/バックアップ全体を指す限り誤りではない。ROADMAP・第24章・実験コード/記録は、gen1の既知パスの再構成、復旧受取/変更鍵の送金2件、固定RPCによる探索境界を別々に扱っていた。baselineにもこの区別と実験リンクを追加し、ウォレット全体の復旧を検証済みへ変更しなかった。11月3日はオフライン復旧/固定RPC探索を再実行し、実送金は再実行していない |
| B：Argent | READMEは独立ページと説明し、SUMMARYにも独立項目があったため変更不要。ecosystem-mapの「第32章の補足」が残っていたため独立解説への案内に修正。Argent本文冒頭も、現行目次の順番と混同しやすい「目次順37番目」を独立ページの説明へ変更 |
| C：日本語・目次 | `language = "ja"`、ecosystem-map・Argentの登録、全36章のパスがすでに正しかったため変更なし |

HDの根拠は[オフライン復旧記録](hd-recovery-review-2026-11-02.md)、[復旧鍵の送金記録](hd-send-review-2026-11-02.md)、[探索境界](hd-discovery-review-2026-11-02.md)と[現在の実習コード/手順](hd-recovery/README.md)。送金は公式v1.0.1 SDK・Devnet・gen1単一署名・既知パス・ASCII追加パスフレーズの限定構成。一般的なウォレットの完全復旧へ広げない。

## 確認できた問題と修正

行番号は開始コミットの行を基準とする。修正後に行が移動しても、開始SHAと該当文字列で追跡できる。

| ID | 重要度 | 対象・開始時行 | 問題・理由 | 根拠と修正 |
|---|---|---|---|---|
| R1 | Medium | `src/baseline.md:53` | 現行教材の検証件数が59 Rustブロック・271テストと記載され、今回の現行本文からの抽出結果54ブロック・233テストと一致しない | `scripts/check-rust-snippets.py` の今回の成功結果を日付付きで反映。収録3教材のfmt/Clippy/計43テスト・同期3件も別の検査として記載し、重複を含むテスト数を合算しない |
| R2 | Low | `src/review-status.md:176`、`ROADMAP.md:25` | CIと本文同期の説明が第12/27章の2教材のまま。実設定は第35章のpayment-candidatesを含む3教材 | `.github/workflows/deploy.yml` のmatrixと `scripts/check-book-examples.py` のPAIRSを読み、3教材へ修正。第35章の同期対象は集計モデル/traitであり、JSON変換全体ではないことも明記 |
| R3 | Low | `src/ecosystem-map.md:38`、`src/part-05-advanced/argent.md:7` | 独立ページという構成に対し、マップの「第32章の補足」と本文の旧レビュー順番号が残る | 現行SUMMARYとREADMEの構成に合わせ、独立解説ページの表現に限定して修正。言語の機能や開発段階は変更しない |
| R4 | Low | `src/baseline.md:91`、`:93` | 「章全体のレビューは未完了」「付録Eなどに残るmaster参照の数値」が、現行のレビュー進捗・固定版へ整理済みの付録Eとずれる | 通読レビューと全規則の技術検証を分ける文へ修正。付録Eの主要値は整理済みと明記し、別版の発展技術に関する注意は維持 |
| R5 | Low | `.github/workflows/deploy.yml:32` | ビルド成功だけでは内部リンク・アンカー・画像等の欠落を検出できず、実施した検査をCIで再現するスクリプトもなかった | 標準ライブラリのみの `scripts/check-book-links.py` を追加し、HTMLビルド直後・artifactアップロード前に実行。現行リンク切れは見つかっておらず、検査の再現性を補う変更 |

この範囲でCritical/Highに分類できる確定した問題は見つからなかった。これは全コードの安全性認定・全主張の正しさの証明ではない。

## 一次資料との照合

- [固定v1.0.1 README](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/README.md)を取得して読み、現在の公開READMEとも区別した。開発用masterとstable、用途別ビルド依存、wRPC/SDK、公式テストの入口を確認。最新版のREADMEを基準版のAPI仕様へ置き換えていない。
- SHA-256がbaselineと一致する固定ソースアーカイブを使い、再実行runnerで展開済みソースも照合した。本文から基準コミットへ向かう170参照・80個の異なるパスはすべてアーカイブ内に存在した。これはGitHubの全HTTP応答の検査とは別。
- 固定ソースの `params.rs`・`bps.rs`・`constants.rs`・GHOSTDAG・DAA・`block_depth.rs`、署名種別等を確認し、関数比較/公式単体テストも再実行した。BitcoinのSIGHASH SINGLEとKaspaの`0x04`、受理とMempool、blue scoreとDAA score、Finalityと業務上の支払い確定の区別を維持。
- [固定KIP索引](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/README.md)の15番号・Statusは付録Fと一致。DAGKnightの索引上Proposedや他のActiveを、ネットワーク有効化の証拠へ変更していない。[DAG KNIGHT論文](https://eprint.iacr.org/2022/1494)の公開要旨・書誌も参照したが、証明の全面照合ではない。
- [固定Argent README](https://github.com/argent-lang/argent/blob/2796809f167144902f04a4b3333a4c144f91fdaf/README.md)は開発中・一般利用向けリリース準備未了、[固定vProgs README](https://github.com/kaspanet/vprogs/blob/f9b84a863a7c7c20586a9cf947550475e894f72e/README.md)は初期開発/プロトタイプと明記していた。本文の区分を維持。
- [公式programmability案内](https://docs.kaspa.org/programmability)、[Based Apps](https://docs.kaspa.org/programmability/based-apps)、[Full vProgs](https://docs.kaspa.org/programmability/full-vprogs)は、モデルを分け、Based Appsを構築中・Full vProgsを将来方向としていた。第34章の区別と矛盾しない。
- [Toccataガイド](https://docs.kaspa.org/toccata)の2026-06-30・DAA score `474_165_565`という記載は第34章の引用と一致した。公式文書の記載確認であり、今回Mainnetノードを観測した結果ではない。
- Silverscriptの2-of-2教材は、固定コンパイラー/依存を使ったローカルVMで再確認した。本書基準ノードv1.0.1への送信・受理と同一視しない。

取得できた固定README/公式ガイドの応答SHA-256はJSONへ保存した。外部サイトの全リンク、全論文PDF・証明・全KIPの本文を今回全面再照合したものではない。

## 今回実行した検査

環境：macOS ARM64、Rust/Cargo 1.90.0、Python 3.11.0、Node.js v20.0.0、mdBook 0.5.4。VM実習は別途インストール済みRust 1.94.0。依存キャッシュを使い、Rust依存を更新しなかった。

| 検査 | 結果 |
|---|---|
| `mdbook build` | 成功 |
| `python3 scripts/check-book-links.py` | 生成62 HTMLの内部参照・アンカー・画像等を検査し欠落0。最終件数はJSON参照。画像参照3件は生成ページ中の出現数 |
| 新リンク検査スクリプトの一時fixture | 正常な日本語アンカー/画像/外部URL除外、欠落ファイル、欠落アンカー、欠落画像、出力外参照、空ディレクトリの6ケース成功 |
| 現行Markdownのローカルリンクと自リポジトリURL | README/ROADMAP/CONTRIBUTING、src全件、編集ガイドを照合。インラインコード内の記号列はリンク扱いしない。欠落0 |
| `python3 scripts/check-book-examples.py` | 本文と収録Rustソースの同期3件が一致 |
| `python3 scripts/check-rust-snippets.py` | 現行54 Rustブロックのコンパイル、233テスト、mainのある例の実行が成功。ブラウザーやネットワーク接続を検査するものではない |
| `python3 scripts/check-utxo-example.py` | 第5章の成功/拒否・状態保持2テスト成功 |
| `python3 -m unittest discover -s experiments/devnet -p 'test_*.py' -v` | SQLite観測ストア7テスト成功 |
| `dag-relations` | fmt・Clippy・8テスト成功 |
| `minimal-node` | fmt・Clippy・16テスト成功 |
| `payment-candidates` | fmt・Clippy・19テスト成功。CLIに保存済みfixtureを与え、候補14件・Observed・集計対象額0を確認 |
| Python/Node/JSON構文 | Python25ファイル、CJS9ファイル、開始時追跡JSON33ファイルに構文エラーなし |
| GHOSTDAG比較 | 24ケース、親配列順ごと712ブロックで一致 |
| DAA算術 | 27ケース・窓への挿入2順で一致 |
| Full窓 | 120ケース・11,700回の公式ブロック比較で一致 |
| Sampled/Dual切り替え | 288ケース、Dual/直接計算器それぞれ28,080評価で一致 |
| Finality point | 25ケース・112照会でモデルと一致 |
| 公式ブロック本体検証 | 下記2件を個別に `--exact` で実行し、各1テスト成功。公式コードに由来する6 warningsは出たが、テスト失敗ではない。公式全体のClippy成功とは扱わない |
| HDオフライン復旧 | 独立16 workerプロセス、160鍵/アドレス再構成・メッセージ署名が成功。UnicodeのNFKD差も再現 |
| HD探索 | 公式APIの固定RPC14ケース、gap教材の固定8例・12,288比較、範囲外4例の拒否が成功 |
| Silverscript実VM | VM評価5件・ABI拒否1件が期待どおり。正常署名だけでなく、改変・署名重複・入れ替え等も検査 |
| `git diff --check` | 成功 |

Rust3教材では、それぞれのディレクトリで次を実行した。`cargo`の実版は上記環境とJSONへ記録した。

```bash
cargo fmt --check
cargo clippy --locked --offline --all-targets -- -D warnings
cargo test --locked --offline
```

公式比較の再実行は、既存の検査済みアーカイブとリポジトリ外のビルド領域を利用した。各runnerの `--offline` は依存取得済み環境が前提である。

```bash
python3 experiments/ghostdag/run.py --archive /tmp/kaspa-finality-v101.tar.gz --work-dir /tmp/kaspa-finality-build --offline
python3 experiments/daa/run.py --archive /tmp/kaspa-finality-v101.tar.gz --work-dir /tmp/kaspa-finality-build --offline
python3 experiments/daa-window/run.py --archive /tmp/kaspa-finality-v101.tar.gz --work-dir /tmp/kaspa-finality-build --offline
python3 experiments/daa-transition/run.py --archive /tmp/kaspa-finality-v101.tar.gz --work-dir /tmp/kaspa-finality-build --offline
python3 experiments/finality/run.py --archive /tmp/kaspa-finality-v101.tar.gz --work-dir /tmp/kaspa-finality-build --offline
```

公式本体検証は固定ソースのworkspaceで次の2件を実行した。

```bash
cargo test --locked --offline --jobs 2 -p kaspa-consensus --lib pipeline::body_processor::body_validation_in_isolation::tests::validate_body_in_isolation_test -- --exact
cargo test --locked --offline --jobs 2 -p kaspa-consensus --lib pipeline::body_processor::body_validation_in_isolation::tests::merkle_root_missing_parents_known_invalid_test -- --exact
```

SDK/VMの再実行はリポジトリのルートから、既存のSDK/固定コンパイラーを使用した。

```bash
KASPA_SDK_ROOT="$PWD/experiments/devnet/.sdk" node experiments/hd-recovery/run.cjs
KASPA_SDK_ROOT="$PWD/experiments/devnet/.sdk" node experiments/hd-recovery/discovery.cjs
node -e 'console.log(JSON.stringify(require("./experiments/hd-recovery/discovery-model.cjs").verifyModel()))'
python3 experiments/silverscript-vm/run.py --source /tmp/mastering-kaspa-silverscript-v1.0.0 --work-dir /tmp/mastering-kaspa-silverscript-vm-build --offline
```

`discovery-model.cjs`はモジュールなので、ファイルを `node` に渡しただけの終了コード0を試験成功と数えず、`verifyModel()`の実行結果を確認した。

ネットワークアクセスは最初サンドボックスのDNS制限で失敗し、許可された読み取りでremote SHA/資料を取得した。localhost固定RPCの探索試験も最初はsetup段階で失敗し、許可された再実行で14ケースを確認した。外部Kaspaネットワークへ接続した試験ではない。再実行の成功で、先行する失敗を「成功していた」と書き換えていない。

## CI・デプロイ条件

workflowは `main` push、pull_request、手動起動が入口。buildは本文同期・Pythonテスト・mdBook・内部参照検査、rustは3教材をRust 1.90.0でfmt/Clippy/testする。追加したリンク検査は標準ライブラリのみで、pip依存を増やさない。

artifactアップロードとdeployは `github.ref == 'refs/heads/main' && github.event_name != 'pull_request'` で制限される。deployは `needs: [build, rust]` を維持し、両方が成功しない限り進まない。write権限はdeployのPages/id-tokenに限定される。このローカル検査でGitHub上の実ジョブ・Pages環境の成功を確認したとはしない。

## 未検証・未解決

- 全36章・関連ページの全行再通読と全主張の独立した一次資料照合。今回の横断検査/重点照合や教材成功を、その代わりの証明にはしない。
- 現在のMainnet全機能・DAGKnight等の有効化を実ノードで観測すること。公式ガイドの記載と実ネットワーク観測は別。
- Devnet起動・採掘・監視サンプルの実通知・送金・2ノード分断/復帰は今回再実行していない。11月2日以前の記録を今回の実測へ変更していない。
- Pruning後IBD、VirtualProcessorの全Finality拒否経路、未知index/支出済み履歴と暗号化DBを含む完全復旧、複数ワーカーの会計・耐障害性。
- 公式workspace全体のfmt/Clippy/test、全Argent/ICC/Genesis・vProgs実行、全暗号方式の監査、全KIP仕様・論文の証明。
- 外部URL全件のHTTP到達性。固定ソースパスの存在と、公開サービスの応答は分けて扱う。
- Linux/amd64/Windows等の実行、ブラウザーDOM/WASM・日本語検索・モバイル表示の再確認、GitHub上のCI/Pages実行。今回の確認環境/実行範囲の外。

未検証事項を推測で修正したり、Verifiedへ格上げしたりしていない。

## 変更ファイル

既存6ファイル：`.github/workflows/deploy.yml`、`ROADMAP.md`、`src/baseline.md`、`src/ecosystem-map.md`、`src/part-05-advanced/argent.md`、`src/review-status.md`。

新規3ファイル：`scripts/check-book-links.py`、本記録、`repository-review-2026-11-03.json`。

`git diff --stat`は未追跡の新規3ファイルを含まないため、`git status --short`と併せて確認する。既存ファイルの修正は版・確認範囲・案内の整合とCI検査追加に限定し、章本文の大規模な書き換えや教育用アルゴリズムの差し替えは行っていない。
