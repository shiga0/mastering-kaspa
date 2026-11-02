# Covenant・SilverScriptを中心とする技術改訂（2026-11-03）

## 作業開始時点と確認範囲

- 開始コミット：`6481d0c7eb88f25ac06db331c327b64974a2d796`。
- 開始時の`git status --porcelain=v1`は空。既存の未コミット変更はなかった。
- 開始時の追跡対象は260ファイル。本文・目次等は59 Markdown、全36章、独立Argentページ、付録A〜H、扉・基準版・進捗・マップを含む。`examples/`は17、`experiments/`は166、`scripts/`は4、workflowは1ファイル。
- README、ROADMAP、book.toml、SUMMARY、基準版・進捗、各章と付録の版・実装状態・教材・関連記述、examplesと実験の構成、CIを現行ファイルで横断確認した。重点は第15・24・29〜32・34・35章とArgent。すべての文章の逐語的な再通読や、全引用・全仕様の新規全面照合を完了したという報告ではない。
- 追加前に既存の記述を確認し、Argentの`co_spent()`・Genesis認証や、第33章のDA・退出など、既にある説明は参照でつないだ。過去のレビュー結果を今回の実行結果に置き換えていない。
- 36章の番号・パス・配置、Argentの独立ページ、ecosystem-mapの目次登録を維持。`language = "ja"`、既存のHD復旧の検証範囲は修正不要だった。ROADMAPの初版完了条件・基準ノード・CI構成は変更しない。
- コミット・push・外部への送信やデプロイは行わない。

開始時の章ハッシュ、固定ソース、取得物のSHA-256、PR状態、追加リンクのHTTP確認は[機械可読記録](programmability-review-2026-11-03.json)に保存する。ファイル数は棚卸しの数であり、260ファイルを同じ深さで精査・検証したという意味ではない。

## 現状レビューと重要度

Criticalの確定した指摘は今回の確認範囲ではない。以下は主に説明・検証の不足で、既存の本番コントラクトに攻撃が成立したと報告するものではない。

| 重要度 | 対象・節 | 問題と理由 | 対応・根拠 |
|---|---|---|---|
| High | 第31章31.10.1 | 状態遷移の概説はあるが、ID・プログラム・状態の出所、複製と偽造を分けた攻撃例が不足。形式が読める／同時消費されたことを認証と誤認し得る | 問題→攻撃→原因→対策→検証を追加。KCC-1の3.7/3.8・7節、KCC-2のセキュリティ節、既存ArgentのICC・Genesis説明に基づく |
| High | 第32章32.10.1、32.15.2、VM実習 | 署名以外の時間・状態・系列の失敗を試せる橋渡しが不足 | 固定v1.0.0で返金・Counter・BoundCounterの24ケースを追加。状態不正、金額流出、binding欠落、誤ったID等を試験。別IDの正当な系列は許可することも明示 |
| Medium | 第15章15.9.1〜2 | KRC-20のインデクサー残高、KCC-20状態、UTXOのKAS額を区別する説明と検索APIの版差が不足 | 専用ノード／残高API、KCC-20、後続masterのV2 RPC、検索PRを別々に説明。Storage Massを返金される預金と誤認しない説明を追加 |
| Medium | 第24章24.9.1、第30章30.14.1 | KIP-22案、KCC-12案、PSKBとHD復旧の関係が説明されていない | 公開鍵秘匿の限界、Provider探索とHD探索、署名追加とfinalize/送信、ALLとスポンサー入力の制約を追加。Draft/Openを明示 |
| Medium | 第29章29.13.1 | DAGKnightの研究と本番を分ける記述は既存だが、Testnet-13とactivation設定の具体例がない | 開発ブランチ固定ソースのalways/neverとMainnet実行ガードを確認。設定の存在と公開ネットワーク稼働・リリースを分けた |
| Medium | 第34章34.16.1 | prover停止等の脅威表はあるが、正しい証明と業務の正しさ・退出可能性の関係が抽象的 | 不正な承認を見落とすプログラム／Operator停止の例を追加。DA・証明コスト・退出条件は設計上の確認事項として記述 |
| Low | 第35章35.24課題E | 発展課題が抽象的で、条件付き支払い・スポンサー・API課金に必要な設計判断へつながりにくい | 5つの設計例と失敗ケースの表、x402の役割を追加。未実装・Kaspa互換未検証と明記 |
| Low | マップ、README、基準版・進捗 | マップで読むSilverScriptのREADMEと実習の固定版が異なる。実習拡張の入口と確認範囲を揃える必要 | マップに版の参照先を追加し、README・基準版・進捗・実習READMEを30ケースに対応。過去の6ケース記録は維持 |

本文の新設節はアンカー付きで参照できる。改訂時点のファイル・行番号はJSONの`findings`に記録し、変更後にも探せるよう節番号を併記した。

## 各章の評価

依頼の8観点を、①正確性／②古さ、③仕様と実装、④重複／⑤読者への導線、⑥例、⑦一次資料、⑧安全性の順に整理する。「維持」は今回の横断確認で変更を要する具体的な問題を特定しなかった範囲であり、全主張のVerifiedではない。一次資料を今回重点照合したものは下の固定資料表に示す。それ以外の全引用の再取得・再照合は未実施。

⑥の「R」は今回のRust抽出コンパイル・テスト対象、「VM」は公式コンパイラー／VMの具体例、「概念」は実行可能な完成実装として扱わない例を示す。

| 章 | ①正確性／②古さ | ③仕様・実装状態 | ④重複／⑤導線 | ⑥例 | ⑦根拠の扱い | ⑧安全性・判定 |
|---|---|---|---|---|---|---|
| 01 | 概説・基準版を維持 | 頻度と不可逆性を区別 | 入門・後続章への案内を維持 | 閲覧手順 | GHOSTDAG等の引用を維持 | blue/redと有効性を区別 |
| 02 | 台帳概説を維持 | 疑似コードとDB実装を区別 | 導入として維持 | 概念 | DB分離レベル等の既存参照 | 認可・再送・競合の注意あり。DB実測は未実施 |
| 03 | ハッシュ教材を維持 | 教材とKaspa暗号を区別 | 暗号の入口として維持 | R | SHA等の既存参照 | 秘密鍵・アドレス・署名範囲の注意を維持 |
| 04 | 伝播の近似を維持 | シミュレーションと実通信を区別 | P2Pの入口として維持 | R | 通信の全仕様は再照合せず | 重複・切断・資源上限の説明あり |
| 05 | Bitcoinを比較対象に限定 | 教材は互換実装でない | UTXOの導入として維持 | R | Bitcoin固定参照を維持 | 二重消費・拒否時状態保持を試験 |
| 06 | 近似確率を性能値にしない | 安全性の予測と区別 | 制約の理解として維持 | R | 既存のモデル前提を維持 | 資源・集中化等の前提あり |
| 07 | Past/Anticoneを維持 | DAG解析と合意を区別 | 第12章と目的が異なる | R | 集合と実装の区別を維持 | 不足親・不正挿入の試験あり |
| 08 | 移行概説を維持 | 提案／コード／稼働を区別 | 部の橋渡しとして維持 | 概念 | 情報源の区分を維持 | 高BPSを確定性としない |
| 09 | ノード構造を基準版に限定 | 後発同期仕様を混ぜない | 役割と読解順を維持 | 閲覧手順 | 固定版の参照を維持 | Pruning後IBD未検証を維持 |
| 10 | P2Pと伝播を維持 | モデルと実プロトコルを区別 | ノード章との分担を維持 | R | 全wire仕様は再照合せず | 再接続が履歴補完でないことを試験 |
| 11 | 検証段階を維持 | 保存／受理／不足親を区別 | パイプライン学習として維持 | R | 全経路監査とはしない | 不正時状態保持・親到着後の再試行を試験 |
| 12 | kとscore/workの区別を維持 | 関数比較と解析器を区別 | 第7章から段階を上げる | R | 固定版比較の既存記録は履歴 | 任意DAGや全Reachability検証ではない |
| 13 | DAAと平均近似を区別 | 前後フォークの参照軸を維持 | 第6章の確率と目的が異なる | R | 公式比較の既存記録を維持 | 浮動小数・境界・無効入力を試験 |
| 14 | 順序と受理を区別 | 教材とVirtualの実装を区別 | 第15章は実務側へ分担 | R | 固定版主要経路の記録を維持 | 競合順序・依存・状態保持を試験 |
| 15 | KAS／トークン／Massを補強 | KCC・KAS20・RPCの版差を追加 | 詳細は30/31章へ参照 | R | KCC・Kasplex・RPCを追加照合 | 系列・KAS負担・再照会を補強 |
| 16 | 基準版署名説明を維持 | モックと実署名を区別 | 発展署名の注意は24/31へ集約 | R | 固定sighashの参照軸を維持 | モックは出力改変も通ると試験で明示 |
| 17 | Mempoolの役割を維持 | ポリシーと合意を区別 | 予約・競合の責務を維持 | R | 全受付経路は再実行せず | 入力予約の解放・状態保持を試験 |
| 18 | 採掘教材を維持 | 公式の選択方針と異なる | テンプレートの入口として維持 | R | 基準版参照を維持 | 実PoW・全報酬を実装したとしない |
| 19 | 算術と仮定を維持 | 観測価格・現行性能と区別 | Massの説明は15章から参照 | R | 固定Mass等の参照を維持 | 非有限値・ゼロ活動等を試験 |
| 20 | 確認とfinalityを区別 | アプリ方針は合意定数でない | DAGKnight具体例は29章へ | R | 既存finality比較を今回の実行としない | 安全な承認秒数の保証をしない |
| 21 | Pruningと状態検証を維持 | 実削減・IBD成功と区別 | 学習用snapshotの限界あり | R | 既存固定版の範囲を維持 | snapshot認証だけではDAG選択を証明しない |
| 22 | RPCの責務を維持 | ネットワークsuffixを含め照合 | V2検索の追加は15章へ集約 | R | 基準版API参照を維持 | 送信不明時に別取引を作る危険を説明 |
| 23 | SDKとWASMの境界を維持 | 教材と公式アダプターを区別 | 実API入口は28章へ | R | SDK配布物の参照軸を維持 | u64精度・ネットワーク・鍵の注意あり |
| 24 | ウォレット提案と量子限界を追加 | Draft/Openと基準版を区別 | Provider探索とHD探索を分離 | R、提案は未実装 | KIP-22/KCC-12を重点照合 | 部分署名・ALL・スポンサーの注意追加 |
| 25 | コード調査の方法を維持 | metadata成功とビルドを区別 | 閲覧の導線を維持 | R | 調査手順を実装仕様としない | エラー時の保存・秘密情報の注意あり |
| 26 | 合意の責務を維持 | DAG教材を公式合意としない | 12/14/27章へ参照 | R | 基準版主要経路の参照を維持 | 不足親・無効・保存単位を区別 |
| 27 | 最小ノードの目的を維持 | 実Kaspaとの非互換を明記 | 構成を接続する学習として維持 | R＋収録crate | 本文との同期成功 | 到着順依存の限界、拒否時状態保持を試験 |
| 28 | Devnetの版を維持 | 実測履歴と実験計画を区別 | 最初の実習ルートを維持 | R、SDKは今回は構文のみ | 新規の実ノード送金はしない | 鍵・DB分離、送信不明時の扱いを維持 |
| 29 | DKの設定・指標比較を追加 | TN13／Mainnet／研究を分離 | 数値ランキングを増やさない | 概念、ノード未起動 | 固定dagknightを重点照合 | always/neverとMainnetガードを確認 |
| 30 | KCCの位置付けを追加 | Final≠実装・有効化 | 定義とStatusの集約先にする | R | KCC索引・PRを重点照合 | 同番号KIP案・参照軸を区別 |
| 31 | ID・出所・承認を重点補強 | 概念／規約／VMを分離 | Argentの既存説明へ参照 | R、攻撃試験は32へ | KCC-1/2を重点照合 | 複製、出所、co-spent、Safety/Livenessを追加 |
| 32 | 型・単位・状態規則を追加 | CLI／VM／ネットワークを分離 | 参照モデルからVMへつなぐ | R＋VM30件 | 固定v1.0.0を重点照合 | 正常・異常・許される複製を実測 |
| 33 | L2・DA・ZKの区分を維持 | 仮想設計と稼働を分離 | 退出説明は34から参照 | R | 各プロジェクトの現行全監査なし | replay保護なしのモデルの限界を試験 |
| 34 | 証明・業務・運用の境界を補強 | prototype／Full／本番を分離 | 33章のDA説明と接続 | 概念、証明は未生成 | vprogs HEADが固定版と一致 | Operator停止・退出・証明費用を設計課題化 |
| 35 | 既存の部分実装を維持 | 追加5例は未実装の設計 | 本文を拡張アプリ実装にしない | R＋収録crate | x402固定READMEを追加照合 | 資産ID、返金競合、スポンサー、リプレイ |
| 36 | 貢献手順を維持 | 個人の変更と合意導入を区別 | 実装・提案への出口を維持 | 手順・概念 | 全上流CIは実行しない | 合意変更・再編成等の回帰範囲を明示 |

Argentは既存のICC・Genesis説明を維持し、31.10.1への参照を追加した。付録A〜Hは、基準版・用語・確率・RPC・パラメータ・証拠区分・実験索引・文献という役割を維持する。Rustブロックは本文とともに再検査し、リンク・目次との整合性も確認する。付録D/Fへ後続APIやKCC表を重複追加せず、基準版の索引のままにした。

長い完全一致の説明段落（コード・見出し・表・箇条書きを除く140文字以上）も比較した。第14/15章の確認範囲の定型文1組で、削る必要のある技術説明の完全重複は検出しなかった。意味の重複を網羅的に排除できたという判定ではない。

## 固定した一次資料と実装状態

確認日はいずれも2026-11-03。PR状態はGitHub APIの`state`と`merged`、head SHAを取得し、文書Statusと別に記録した。

| 対象 | 固定点と確認した内容 | 状態・検証の境界 |
|---|---|---|
| [KCC一覧](https://github.com/kaspanet/kccs/blob/1ed87aef59faff8354061aee30f734a3b8b2c3d2/README.md) | `1ed87ae…`、KCC-1/2/20/23本文 | 1/2/20 Final、23 Draft。ネットワーク有効化の根拠ではない |
| [KCC-12 PR #24](https://github.com/kaspanet/kccs/pull/24) | head `8e1053e…`、Providerイベント、取引/PSKB署名の責務 | Draft、Open、merged=false。実機ウォレット間互換は未検証 |
| [KIP-22 PR #48](https://github.com/kaspanet/kips/pull/48) | head `b031344…`、P2SH公開鍵秘匿案 | Draft、Open、merged=false。既存Scriptを使うウォレット規約の案で、量子耐性署名の実装ではない |
| [Covenant Index PR #1152](https://github.com/kaspanet/rusty-kaspa/pull/1152) | head `9df9e9c…`、CID別検索とページ化の提案 | Open、merged=false。PR説明は検索案の存在・状態に限定して使用。IDの継承規則はKIP/KCC本文に戻る |
| [KAS20 PR #4](https://github.com/argent-lang/kcc20-reference/pull/4) | head `2f7cc78…`、KAS裏付け・burn・合成資金のREADME | Open、merged=false。例を本書で実行していない。正規デプロイの指定なし |
| [node master](https://github.com/kaspanet/rusty-kaspa/blob/b85380d16514e65136001530e7c717eef304e654/rpc/core/src/model/message.rs) | `b85380d…`、V2要求・応答とserviceのcursor検査 | ソース確認のみ。baseline SDKのAPIへ置き換えない |
| [dagknight設定](https://github.com/kaspanet/rusty-kaspa/blob/ad45e241e6688a14901fd24dd8dc33c5c9a33f40/consensus/core/src/config/params.rs) | `ad45e24…`、TN13 always/Mainnet never、daemonのMainnetガード | 開発ソースを確認。公開ネットワーク稼働・移行・安全性証明は未検証 |
| [SilverScript v1.0.0](https://github.com/kaspanet/silverscript/tree/3ed973335b59269293564805cc2c58a14595ec03) | `3ed9733…`、TUTORIAL、CONSTRUCTOR_ARGS、公式tests | Releasedタグを固定。今回の4教材をローカルでコンパイル／評価 |
| [vprogs](https://github.com/kaspanet/vprogs/tree/f9b84a863a7c7c20586a9cf947550475e894f72e) | HEAD `f9b84a8…`が既存参照と一致 | prototypeの参照軸を維持。今回のビルド・証明生成なし |
| [Kasplex](https://github.com/kasplex/go-krc20d/tree/6d21c8668bd77db6d89f28560cc37aaa205f8517) | `6d21c86…`、README、storage/state.go、api/v1address.go | 専用状態・残高APIをソース確認。インデクサーは未起動 |
| [x402](https://github.com/x402-foundation/x402/blob/5220ceba2298ec41b0adabc75f1308da655ecbab/README.md) | `5220ceb…`、HTTP402と役割・scheme/networkの区別 | 設計の入口。Kaspa向けscheme/facilitator互換を検証したものではない |

固定版Argentの[ICC semantics](https://github.com/argent-lang/argent/blob/2796809f167144902f04a4b3333a4c144f91fdaf/docs/icc-semantics.md)とGenesis認証の既存記述は再利用し、Argentのコンパイラーやruntimeの再実行結果を今回追加したとは扱わない。KIP文書、基準ノード、各開発リポジトリは異なる参照軸のままである。

## SilverScriptの追加検証

ソースは`experiments/silverscript-vm/`の`two-of-two.sil`、`timeout-refund.sil`、`counter.sil`、`bound-counter.sil`。追加3例は本文から直接includeし、転記の食い違いを避ける。`security.rs`が実際の署名・状態・出力を変更して期待と照合する。上流由来のVM組立てのISC条文を保持した。

- コンパイラー：v1.0.0 / `3ed973335b59269293564805cc2c58a14595ec03`、clean checkout。
- 依存Kaspa：`a41a333b08848f41bf737b72592e463a6011b8ac`。ノード基準版とは別。
- macOS/ARM64、Rust 1.94.0、edition 2024、Python 3.11、取得済み依存で`--offline --locked`。
- 初回ハーネスビルドではハッシュの配列に参照を渡していない型エラーを修正した。その後、30件すべての期待と一致した。上流を修正して成功したわけではない。
- 結果は[observed JSON](silverscript-vm/observed-2026-11-03.json)。ソース／ハーネス／runner／manifestテンプレート／lockfileのSHA-256を含む。一時鍵を毎回生成するため、固定Scriptのバイト完全一致を立証する記録ではない。

| ケース群 | 件数 | 期待・観測 |
|---|---|---|
| 2-of-2 | 6 | 正常署名を許可、4種の不正をVM拒否、空のsigをABI拒否 |
| 期限付き返金 | 8 | 正常受取・期限到達返金・期限後の受取を許可。期限前・誤単位・不正署名・署名後改変を拒否 |
| Counter | 7 | 正常後継を許可。状態・プログラム・額・個数の改変を拒否。bindingなし・別系列を許可 |
| BoundCounter | 9 | 正常後継・別IDで整合した系列を許可。状態・プログラム・額・個数・binding欠落を拒否。不整合IDと範囲外の入力をcontextで拒否 |

内訳はVM評価27、ABI拒否1、covenant context拒否2で、失敗した層を混同しない。合成ID付きUTXOから始めるためGenesis取引・Genesis Proofは検証しない。Counterは無認証で更新可能、単一入力試験は手数料0、返金例は期限後にも通常支出が可能である。ノードへの送信、Mass、成熟、現在時刻、全コンセンサス規則、資産受入の認証、Mainnet有効化の試験ではない。

再実行：

```bash
python3 experiments/silverscript-vm/run.py \
  --source /tmp/mastering-kaspa-silverscript-v1.0.0 \
  --work-dir /tmp/mastering-kaspa-silverscript-vm-security-build \
  --offline
cargo +1.94.0 fmt --manifest-path /tmp/mastering-kaspa-silverscript-vm-security-build/Cargo.toml --check
cargo +1.94.0 clippy --manifest-path /tmp/mastering-kaspa-silverscript-vm-security-build/Cargo.toml --offline --locked --all-targets -- -D warnings
```

前提・取得手順は[実習README](silverscript-vm/README.md)を参照する。初回はC/C++環境・依存取得・数分以上のビルド時間が必要。

## ビルドと回帰確認

| コマンド・対象 | 結果 |
|---|---|
| `mdbook build -d /tmp/mastering-kaspa-programmability-book` | 成功。SUMMARYと新includeの解決を含む |
| `python3 scripts/check-book-links.py --book-dir /tmp/mastering-kaspa-programmability-book` | 成功。62 HTML、6,329ローカル参照、画像参照3件。内部リンク・アンカー・画像参照を検査 |
| ソースMarkdownのローカル参照と自リポジトリへのGitHubリンク | README等のソース実体・見出しと自リポジトリ参照76件を追加確認。GitHubの未公開新ファイルはローカル実体として確認。エラー0件 |
| 今回追加した外部一次資料リンク | 固定ファイル・PRの22 URLをHTTP取得できた。取得成功は主張の正しさやサイトの将来の可用性の証明ではない |
| `python3 scripts/check-book-examples.py` | 第12/27/35章の3件が収録Rustと一致 |
| `python3 scripts/check-rust-snippets.py` | 54 Rustブロックのコンパイル、233テストと`main`実行が成功。edition 2021、Rust 1.90.0 |
| `python3 -m unittest discover -s experiments/devnet -p 'test_*.py'` | 7テスト成功。SQLite観測ストア、実ノード接続なし |
| `python3 scripts/check-utxo-example.py` | 2テスト成功 |
| 収録3crateの`cargo +1.90.0 fmt --check` | 成功 |
| 同`cargo +1.90.0 clippy --offline --locked --all-targets -- -D warnings` | 成功 |
| 同`cargo +1.90.0 test --offline --locked` | 8＋16＋19＝43テスト成功 |
| SilverScriptハーネスの`fmt --check`・Clippy | Rust 1.94.0で成功。コンパイル／VM試験は上記30ケース |
| `git diff --check` | 成功 |

Rust 1.94.0のrustfmt/Clippyと、明示したRust 1.90.0ツールチェーンは必要に応じて取得した。既存stableも1.90.0だったが、ツールチェーンの既定は変更していない。fmtの初回失敗・sandboxによるrustup一時ファイル作成失敗は、必要コンポーネントの取得後に解消した。

CIは現状の1 workflowを確認した。書籍ビルドとRust matrixが通ってから、mainのpush／手動実行でPagesへ進み、PRではデプロイしない条件を維持する。Python観測テスト・教材同期・リンク確認の既存ジョブを変更していない。SilverScriptは1.94.0・Git依存・ネイティブビルドを要する手動実習のままで、CIがこの30ケースを自動実行するとは記載しない。GitHub Actions・Pagesの実ジョブは今回起動していない。

## 変更ファイル

- 読者向け：README、`src/baseline.md`、`src/review-status.md`、`src/ecosystem-map.md`。
- 本文：第15・24・29・30・31・32・34・35章、`src/part-05-advanced/argent.md`。
- 実習：`experiments/silverscript-vm/README.md`、`main.rs`、`run.py`。新規`timeout-refund.sil`、`counter.sil`、`bound-counter.sil`、`security.rs`、`observed-2026-11-03.json`。
- 記録：本ファイルと`programmability-review-2026-11-03.json`。

`git diff --stat`は追跡済み16ファイル、244行追加・12行削除。新規7ファイルはこの値に含まれず、合計23ファイルを変更・追加した。全パスと追跡済みのstat出力はJSONの`changedFiles`・`trackedDiffStat`に記録した。削除・移動したファイルはない。

## 未検証事項・今後の改訂候補

1. 全章全主張・全一次引用の全面再照合、論文の安全性証明、全プログラムの監査は今回の限定レビューからは完了判定できない。
2. Testnet-13の公開ネットワーク稼働、DAGKnightの全テスト・リリース・Mainnet移行、現行ネットワークのactivationを実ノードで照合していない。
3. KCC準拠トークンの実デプロイ、Genesis Proofの生成・検証、co-spent承認スキーム、複数入力／系列の全保存則は未実行。検証する場合は別の固定ベクトルが必要。
4. KCC-12のウォレット間PSKB互換、KIP-22案の復旧、ページ付きRPC/Covenant Indexの実機利用、スポンサーの複数署名は未実装・未検証。
5. KAS20参照例はREADMEの確認にとどめ、実行結果を転載していない。KRC-20インデクサーも未起動。
6. Full vProgsの証明生成、Tic-Tac-Toeの証明試験、Escape Hatchの実装、Igra等の現行L2のブリッジ／DA／退出の監査は未実施。
7. x402のKaspa向けscheme／facilitator互換、完成Payment Tracker、スポンサー決済・エスクロー等は設計例に限定する。
8. 他OS・CPU、ブラウザー表示・ウォレット機器、全既存外部URLのHTTP再取得は未確認。KasplexのGitBookは今回の取得手段ではリダイレクト先を読めなかったため、公式GitHub実装を根拠にした。

これらは初版の必須作業を増やす一覧ではない。実験を追加する場合は、対象版・ネットワーク・期待する失敗を先に固定する。
