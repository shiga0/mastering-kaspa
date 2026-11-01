# SUMMARY順レビュー 16：第12章「GHOSTDAG Internals」

確認日：2026-11-01。本文全体を通読し、説明・図・教材・根拠・実習導線を確認した。基準版はv1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。

## 修正内容

- 前章との接続を修正。GHOSTDAGはヘッダー処理内で計算し、本体の検証と保存を終えてから開始するものではない。
- Past/Futureが自分自身を含まない定義と、同一ブロックでもtrueとなる公式Reachability APIを区別した。
- Anticone内の要素同士に祖先・子孫関係があり得る反例を追加。k-clusterの説明を、各対象ブロックと並行な要素の数として明確にした。
- 図の親子の配置・参照方向を補い、トポロジカル順序の例を直前の3ブロックに合わせた。
- 手計算例と既存の24ケースの実行比較を区別し、Virtualの親が全tipsとは限らないことを補った。
- IACRの原論文ページに合わせ、PHANTOM/GHOSTDAGの著者をYonatan Sompolinsky, Shai Wyborski, Aviv Zohar、題名をPHANTOM and GHOSTDAG: A Scalable Generalization of Nakamoto Consensusへ訂正。最大k-clusterの計算量の説明も補った。
- lib/editionを明示し、収録教材を実行する場合とリポジトリ外で入力する場合の導線を整理。ソース探索は第1章の固定版を再利用する。

原論文の書誌とNP困難な最適化に対する貪欲アルゴリズムの説明は [IACR ePrint 2018/104](https://eprint.iacr.org/2018/104) の現行概要と照合した。証明全体や各改訂版の差は今回照合していない。

## 教材の修正と実行

既存の `examples/dag-relations/src/lib.rs` と本文のマーカー内コードを同時更新した。新しいexamplesディレクトリは作成していない。

以前は、selected内の既知ブロックでkの上限を超えるとfalseで早期終了し、後ろの未知ブロックが検査されなかった。例えばサンプルDAGの集合{C,D,Z}は、k=0ではfalse、k=1では未知Zのエラーになった。修正後は全要素の存在を先に確認し、どちらもエラーとする。

insertは空ID・重複親も拒否する。重複親がBTreeSetへの変換で暗黙に消えることを避け、すべての拒否を保存前に行う。重複IDも引き続きエラーであり、第11章のAlreadyKnownとは異なる教材上のAPI方針である。

Rust/Cargo 1.90.0、macOS ARM64、edition 2021、外部依存なし。次を実行した（生成先は一時ディレクトリ）：

```bash
cargo test --locked --offline --manifest-path examples/dag-relations/Cargo.toml
cargo fmt --check --manifest-path examples/dag-relations/Cargo.toml
cargo clippy --locked --offline --manifest-path examples/dag-relations/Cargo.toml --all-targets -- -D warnings
```

8テスト成功、書式・Clippyも成功。

- 既存5件：推移的Past、並行ブロックのAnticone、k=0/1境界、未知親の拒否、並行ブロックと親配列の順序を変えても関係が一致すること。
- 追加3件：Anticoneの要素同士が関連する反例、全選択要素の検査（空集合・単一要素も含む）、空ID/重複ID/自己参照/重複親/未知親の拒否で既存集合全体が不変であること。

この解析器は集合の条件を判定する。最大集合やGHOSTDAGのblue集合を選択せず、PoW・ハッシュ・Genesisの一意性・親数や入力サイズ上限・孤児キュー・並行処理・Pruningを実装していない。cargo newからの新規環境での手入力試験とは区別する。

## 固定ソースと既存の証拠

今回再読した主な箇所：

| ファイル | SHA-256 |
|---|---|
| [consensus/src/processes/ghostdag/protocol.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/ghostdag/protocol.rs) | `97fe5e1db8afae9d6a3074cf6b6e56ed72e453db9edcbf9b539a4469cc2918ee` |
| [consensus/src/processes/ghostdag/ordering.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/ghostdag/ordering.rs) | `8b6716afbaddd2becd1910e8d80108747daaa97970669cb50c22ff0fca0dc47f` |
| [consensus/src/model/services/reachability.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/model/services/reachability.rs) | `737aeed155f7d35692fbc8a08eda9c10889bb7279ec4a48ce7ebd1f4d02414c2` |

protocol.rsの親選択、selected parentのDAA scoreによるk選択、候補判定、blue score/workの算出と上位レベルの仕事量下限を確認した。ordering.rsは比較順、reachability.rsは自己を含む祖先判定の境界を確認した。

既存の `ghostdag-baseline-review-2026-10-31.md` と `ghostdag-comparison-2026-10-31.md` を参照した。24ケース・1親配列順あたり712ブロック、通常順/反転順で合計1424比較の一致は過去の実行記録で、今回再実行したものではない。共通DFSを使うため公式Reachabilityの実行検証ではなく、全DAG・全プロトコル・論文の証明・現在のMainnetとの一致へ一般化しない。

次はSUMMARYの17ページ目、第13章 `src/part-02-consensus/chapter-13.md`。

## 統合・公開物の確認

本文とRustソースの同期対象2件が一致。mdbook build成功、生成HTML53ページの内部ファイル・アンカーのリンク切れ0件。SUMMARYとreview-statusの49項目は名称・順序一致し、git diff --checkも成功した。

追跡済み・追加候補162ファイルへの既存の秘密情報パターン検査は検出0件。gitleaks/trufflehogは未導入であり、秘密情報が一切ないとの保証ではない。新しい教材フォルダー、生成物、秘密鍵、個人パスは追加していない。
