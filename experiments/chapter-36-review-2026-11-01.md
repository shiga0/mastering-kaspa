# SUMMARY順レビュー 41：第36章「Kaspaへのコントリビュート」

確認日：2026-11-01。基準版の主要開発用入口・公開の貢献手順と、独自の不変条件教材を確認した。これで第1〜36章とArgent補足の本文確認は一区切り。公式実装・CI・研究/暗号方式の検証をすべて完了した意味ではなく、次は付録Aから順に確認する。

## 固定版と公開手順

既存のv1.0.1 / fcd9c28f9b211b9872a0c90a8c94f79466ffb063のソースアーカイブを再利用した。今回Git clone/HEADを確認したという意味ではない。実PRのbaseは、歴史的な再現用tagと別にその時点の指定を確認する。

- 基準版READMEは開発の既定をmaster、安定版の入口をstableと説明する。Protobuf、clang/RocksDB、用途別のWASM等の要件を参照し、一般的なRust導入だけで十分としない。
- rootと.githubにCONTRIBUTING.md、rootにSECURITY.md/rust-toolchain.tomlはなかった。貢献ルールやtoolchain固定ファイルが存在するという前提にしない。組織/現在の全ガイドを網羅的に監査した意味ではない。
- checkはcargo fmt --allで書式を書き換え、workspaceと複数WASM featureのClippyを呼ぶ。testはnextestのrelease/workspace/integration向けコマンドとwasm/build-nodeを呼ぶ。read-onlyの狭い検査とは区別した。
- .github/workflows/ci.yamlはworkspace check/release build・nextest・docテスト・追加feature等を含む。runnerのディスク掃除もあるので手元へそのまま持ち込まない。
- check/testはbash -nで構文確認が成功しただけで、スクリプト自体は実行していない。公式CIも実行/成功判定をしていない。
- metadataを--offline --locked --no-depsで取得し、59パッケージ・kaspa-mathの存在を確認。resolveはnull。依存解決/ビルド/テスト成功ではない。ローカルパスを含む生JSONは本書へ追加していない。

[Cargo testの公式説明](https://doc.rust-lang.org/cargo/commands/cargo-test.html)と、[GitHubのfork/PR手順](https://docs.github.com/en/get-started/exploring-projects-on-github/contributing-to-a-project)を確認した。本文のkaspa-math --listは存在するpackageの一覧コマンド例で、この回に実行していない。フィルター/--exactの完全名と実際の実行件数を照合し、0件を成功件数へ数えないよう補足した。

[Security policy](https://github.com/kaspanet/rusty-kaspa/security/policy)の今回参照した表示はNo security policy detected / SECURITY.md未設定だった。報告先やメールを創作せず、その時点で公式に案内される信頼できる経路を調べるよう記載した。報告フォームの可用性を検証したり、脆弱性報告/Issue/PRを投稿したりはしていない。

## 説明・手順の整理

- 再現用固定版と、貢献先のbase/fork/headを区別。作業中の変更を破棄するreset等を手順に加えていない。
- Issueの環境には完全なNetworkId、feature/起動条件等も記録する。testnetだけで番号を落とさない。公開証拠は最小化し、秘密情報や個人ホームパス・不要なrawログ/DBを含めない。
- 同じ入力の決定性には初期状態・DAG・設定/activation文脈も含む。一時的な観測差を固定入力の判定不一致と混同しない。
- Result::Errで一切の永続化がないという普遍的な保証は置かず、無効状態の記録・ヘッダー/Virtualの別batch等の契約を確認する。Finalityのプロトコル上の意味とアプリの確定ポリシーも区別。
- PR説明テンプレートを問題/変更後の動作・根拠・実際の検証・互換性へ整理し、変更規模へ合わせる。投稿やpushを実際に行ったわけではない。
- AI利用/開示方針はプロジェクトごとに異なる。検証責任に加え、その方針を確認することも明記した。
- 実験ディレクトリ図は提案であり、本章で新規examples/実験プロジェクトやCI対象を追加していない。

## 独自Rust教材

元のtransferは自己送金の場合、残高より大きいamountでもOkを返した。先にchecked_subで不足を検査してから自己送金のno-opへ進むよう修正した。0額/空IDを拒否する教材上の方針も明記。どちらかのアカウントが存在しない場合は新規作成しない。

公開型・関数のlibとして本書外の一時Cargoプロジェクトへ抽出した。Rust/Cargo 1.90.0、edition 2021、外部依存なし。rustfmtを本文へ反映し、実行したソースと本文の完全一致を確認した。

```bash
cargo test --offline
cargo fmt --all -- --check
cargo clippy --offline --all-targets -- -D warnings
```

9テスト成功、fmt/Clippyは終了コード0。

1. 通常移転の各残高。
2. 残高不足の拒否と全状態保持。
3. 受取人overflowの拒否と全状態保持。
4. 資金が足りる自己送金のno-op。
5. 0額/空ラベルの拒否と状態保持。
6. 送信元/宛先/両方が未知のときの拒否と状態保持。
7. 残高を超える自己送金の拒否と状態保持。
8. 両残高/金額を各0・1・10・u64::MAXとする64ケースで、成功時のu128総量保存またはエラー時の全状態保持。
9. 同じ要求の2回適用で2回移転でき、Replay防止が存在しないこと。

これは架空IDを持つアカウント残高の局所モデルで、KaspaのUTXO/署名/認証、nonce、手数料、永続化/並行制御は実装しない。通常のResultエラーで検証前更新を避けることを確認したが、OOM/panic・プロセスクラッシュ・DB/分散更新の原子性の証明ではない。有限グリッドを全入力の証明やproptest実行と扱っていない。

## 今回行っていないこと

公式crate/全workspaceのビルド・テスト、check/test/上流CIの実行、実PR/Issue/脆弱性報告やpush、ノード接続・採掘・鍵生成・署名・送金は実施していない。既定Rust、Docker、既存CIも変更していない。

## 主要参照ファイルのSHA-256

ファイルの識別用であり、全コード/全手順を監査した印ではない。

| ファイル | SHA-256 |
|---|---|
| README.md | `8fe16a2428a0cd33524bf5b7842fcb3670e6227d367e1cc1778bc5f5d5d76d31` |
| check | `74c82caa53df44bb91e177bc99d90d66f1cbb8fca309fbcba26dc32c6ebb8b22` |
| test | `f456b6c122e1be5e15442d41d6bd49ff5f75c3566cba6c24a2495ae718a9a9ea` |
| .github/workflows/ci.yaml | `a029f209a230c55a85e61c64622f40b9bf30a3e191de5c330d0faa0c1e086e06` |
| .rustfmt.toml | `b6ccf9fa54da9252a2b17e904726f850b155d95568a22a48d80f78cc8b0437c2` |
| Cargo.toml | `a4ac0081533750ca27a929b80575463ee59439c412cae748212e24b3bc07cada` |
| Cargo.lock | `14108c8c41a84a1a9c8cd8648238047a0620ca390058f9d8a176fefc78beeabf` |

## 書籍・公開対象の検査

- mdBook 0.5.4でビルド成功。生成HTML 53ページの内部ファイル・アンカー切れ0件。
- scripts/check-book-examples.pyで既存教材2組が一致。本章教材は本書外での手動確認で、examples/CI対象を増やしていない。
- SUMMARYとreview-statusの本文表49項目、進捗表1〜42の番号/順序が一致。次は付録A。git diff --check成功。
- 追跡対象・未追跡候補187ファイルの秘密鍵ブロック・既知トークン・認証付きURL・個人ホームパス等のパターン検出0件。gitleaks/trufflehogは未導入で、秘密情報の不存在を保証する監査ではない。
- 公開候補は本文・進捗・レビュー記録のMarkdown。鍵・DB・生ログ・metadata生JSON・アーカイブ・ソースclone・ビルド成果物は追加していない。コミット・pushもしていない。
