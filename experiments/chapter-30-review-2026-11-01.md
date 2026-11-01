# SUMMARY順レビュー 34：第30章「Kaspa改善提案・KIP」

確認日：2026-11-01。本文・図・教材を通読し、固定KIPの主要説明とTitle/Status、PRの区別、基準版の規則選択を確認した。全KIPの仕様・暗号方式・証明・実装・本番有効化の全面監査ではない。

## 参照版と一次資料

[KIPリポジトリ](https://github.com/kaspanet/kips)のmasterを取得し、GitのHEADが `e4ae2332117b5cb68bd6188e065ef885b6d17939` であることを確認した。コミット日2026-07-15、件名Mark kips 16,17,20,21 as active (#45)。実際の取得はリポジトリ外へのshallow clone。本文の読者用手順は、新規調査コピーのcloneと固定SHAへのdetachを示す。作業中のcheckoutをresetする指示にはしていない。

最初のsandbox内のls-remoteは名前解決に失敗し、承認されたネットワークアクセスで再取得した。Web APIだけではHEADを取得できず、実cloneのrev-parseで固定した。Rust側の既存ソースアーカイブはv1.0.1 / fcd9c28f9b211b9872a0c90a8c94f79466ffb063を使った。こちらを今回Git clone/HEAD確認したという意味ではない。両者は同じ時点の資料ではなく、新しいActive文書が基準版SDK/Devnetでの利用可能性を保証することはない。

- KIP-1の本文/一覧はImplemented、KIP-2はProposed。確認対象のKIP-4/9/10/13/14/15/16/17/20/21はActiveだった。これは固定文書の状態であり、観測時のMainnet対応表ではない。
- KIP-15は一覧と本文の題名が異なる。本文はCanonical Transaction Ordering and SelectedParent Accepted Transactions Commitment。KIP-16も本文のNew Transaction Opcodes for Verifiable Computationと一覧のZK Precompile Opcodeを区別した。
- KIP-21本文はkip-0021.md。同名ディレクトリは補足資料で、旧リンクを本文へ修正した。要旨にあるレーンごとのコミットメントとaccepted_id_merkle_rootのactivation後の意味を説明し、既存名から同じhash構造を仮定しないようにした。全SMT・hash layout・bootstrapの照合はしていない。
- KIP-2のdeliverablesは、ウォレット側確認APIに加えdifficulty・minting・Pruning等に必要なグローバル遅延上限への適応を挙げる。parameterlessをあらゆる用途の遅延条件不要へ拡張しない。
- KIP-13の一時的なstorageは転送中のRAMだけでなくPruning期間のディスク保持を含む。KIP-9の永続UTXO負担・計算massと区別した。全数式・定数・資源上限の再導出ではない。
- [PR #37](https://github.com/kaspanet/kips/pull/37)の題名はKIP22 - P2MR ScriptPublicKey for quantum resistance、[PR #48](https://github.com/kaspanet/kips/pull/48)はKIP 22: Pay-to-Public-Key-Hash via P2SH。確認日にどちらもOpenと表示され、固定masterにはKIP-22がなかった。番号だけで提案を同定しない。PRのtitle/base/head表示・状態の確認であり、PRの全差分・暗号方式・安全性は検証していない。PRページの状態は後日変化し得る。

## 主要説明と教材

- KIP文書と実装・リリース・ネットワーク有効化を分離。分類/提出テンプレート/遷移図は本書の概念整理で、公式の必須手続きとして扱わない。
- P2Pの外部接続をStorage経由に見せていた図を修正。各処理の呼び出し順や一つの原子的更新と同一視しない。
- params.rsのForkActivation::is_activeとForkedParam::getの比較・規則選択を確認。呼び出し側が渡すscoreを読む必要がある。GHOSTDAGのkはselected parentのDAA scoreを文脈にする。
- 単純なat_height教材を、独自のActivation::{Never, Always, AtScore}とrules_at(context_score)へ変更。文脈ごとに再評価する。公式の番兵値や有効化設定を再現したAPIではない。
- 境界付近のブロックやRPCイベントを一律に悪意ある入力と呼ばず、正常・境界・敵対的ケースに分けた。同じ完全な入力/設定に達した場合の決定性と、一時的な観測の一致も区別した。
- 演習の保存先は提案例。新規examples/kip-reviews等は作成していない。

本文のRustコードをリポジトリ外の一時Cargo libへ配置。Rust/Cargo 1.90.0、edition 2021、外部依存なし。rustfmtを本文へ反映し、本文と実行ソースの一致を確認した。

```bash
cargo test --offline
cargo fmt --all -- --check
cargo clippy --offline --all-targets -- -D warnings
```

4テスト成功、fmt/Clippyも終了コード0。

1. 閾値100の直前・一致・直後。
2. Never/Alwaysを0・100・u64::MAXで区別。
3. 閾値0とu64::MAXの定義域端。
4. 新しい文脈を判定後も過去の文脈で旧規則を選べること。

これらは独自教材の境界確認であり、実ノードでのfork/reorg・移行・DB互換性・全ネットワークのactivation試験ではない。公式ノードの再ビルド/実行、採掘・鍵生成・署名・送金も行っていない。

## 固定KIPファイルのSHA-256

主要説明/metadataの照合対象を識別するhashであり、各ファイルの全仕様・証明を精査したという意味ではない。

| ファイル | SHA-256 |
|---|---|
| README.md | `16d52aee9a0d07437e71b2532c16eabd6dc27087dacc8f06dce99d4924ab88fb` |
| kip-0001.md | `e1769e21676adf68932d2010a34eb3e9fa8e313a71fdbb9813a8d1977e928258` |
| kip-0002.md | `b40c50ef2a47ecf7853267d7f795c2792baf9122f8978041ae3e68c287efe09f` |
| kip-0004.md | `dd7b247129a471b7978dd6e482fcc7caaf3ee37f5a22d3f1fe3436ef0d4ed376` |
| kip-0009.md | `153a0e674e98763468fc24270676883b5d86db8efca2c1177fb6a1090a05309c` |
| kip-0010.md | `da9774f32605c54a61dfc7204be92198fab3e6b68d3c59c149d74e6897c9b98e` |
| kip-0013.md | `48efe56486898df4dd1950d1aace0ff74c09a1990f7041b9e1e3edf77484e33e` |
| kip-0014.md | `7b7855c59abbab2c0d29991f82d32ac011727183af24609584947969c05cae1c` |
| kip-0015.md | `d5a6aef73c3d594a30703e8af44b2176f7a2ca85984e4b68f3f0582d7304d401` |
| kip-0016.md | `c70ef07766732677a72ff3e6c5c9c639104c6b2df6fa726ffc6752671ef5e1d4` |
| kip-0017.md | `345a0a02f0590f4fe12704e82d25ee2fe7e703001b7c089ed6e3a639eea5fb90` |
| kip-0020.md | `94610881c7ef905fe5346c7514b920f21a0e1a10e462776ba45eda73ac179cc8` |
| kip-0021.md | `ce1606e74e20a9652a0c67ddd349d84b8b2ab8265df5a1d10b27d35f7a95cf26` |

Rustの既存アーカイブで読んだ主要部分のファイルも識別する。全ファイル監査の印ではない。

- consensus/core/src/config/params.rs: `9b9a88195e530888de6543004a241c136c142a0bf606e27de4c527901b17ffea`
- consensus/core/src/mass/mod.rs: `e630fa4f222229e67797df0b8d59ddee8627070a90d8effa6e2b1d10fa35e60e`
- consensus/src/processes/ghostdag/protocol.rs: `97fe5e1db8afae9d6a3074cf6b6e56ed72e453db9edcbf9b539a4469cc2918ee`

## 書籍・公開対象の検査

- mdBook 0.5.4でビルド成功。生成HTML 53ページの内部ファイル・アンカー切れ0件。
- scripts/check-book-examples.pyで既存教材2組が一致。今回の独自教材は手動検査で、examples/CIの対象を増やしていない。
- SUMMARYとreview-statusの本文表49項目が同じ順序。次は第31章。git diff --check成功。
- 追跡対象・未追跡候補180ファイルの秘密鍵・既知トークン・認証付きURL・個人ホームパス等のパターン検出0件。gitleaks/trufflehogは未導入で、秘密情報の不存在を保証する監査ではない。
- 新規公開候補はレビュー記録のMarkdown。鍵・DB・生ログ・KIPのclone・アーカイブ・ビルド成果物を追加していない。コミット・pushも行っていない。
