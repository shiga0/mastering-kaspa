# SUMMARY順レビュー 44：付録C「数学・確率・暗号技術の補足」

確認日：2026-11-01。全文を通読し、数式・集合の定義、数値例、2つの独自Rust教材を確認。公式コンセンサスの全規則や暗号方式の安全性を再証明したものではない。

## 修正内容

- C.3の図にはE→Bとも読める線があったが、辺集合と演習はE→Cのみ。辺の経路を列挙する図へ合わせた。有限DAGでの子→親の矢印と親を先に処理する逆順を区別。
- Past/Futureで自身を除き、Anticoneは直接だけでなく間接の到達関係も含めて判定。部分順序は直接の親辺だけではなく到達可能性と自己関係から定義する。
- kのblue anticoneに対する意味を補足し、親数・全anticone・毎秒ブロック数の上限と混同しない。
- 二項分布のn/r/pの範囲と組合せ、幾何分布の一定成功確率、均質Poisson過程の条件とλ=0の退化例、PoWの整数ターゲット範囲を明示。
- 誕生日近似に独立一様な出力という前提と衝突なしの厳密な積を補足。
- Merkleは葉の並び/内容に対するcommitmentとし、rootの信頼や全データ可用性を包含証明だけで保証しない。ZKの公開情報とw、健全性と知識健全性を区別。
- 支払いの不変条件は通貨/単位・請求額の正値・受理状態/再編成等の前提を定義し、式だけで入金確定を保証しない。
- 確率教材の1-exp(-x)を-exp_m1(-x)へ修正。xが極小のときの桁落ちを避け、有限非負のrate/secondsを要求。assertは教材用で、本番APIのエラー処理ではない。f64の丸め/underflowや極小pの逆数overflowは残る。
- DAG教材はRustfmtで整形。未知親を葉として扱い、閉路/自己参照の検証をしない前提、HashSet表示順が未規定であることを明示。新しいexamplesフォルダーは作っていない。

## Rust教材の実行

既存のRust/Cargo 1.90.0、edition 2021、外部依存なしの一時プロジェクト2つで実行。本文から抽出し、整形後のソースを本文へ戻して再度一致を確認した。通常のcargoを使った結果であり、本文のcargo +1.90.0向けに名前付きtoolchainを今回追加したわけではない。

各プロジェクトで次を実行：

```bash
cargo fmt --all
cargo test --offline
cargo run --offline
cargo clippy --offline --all-targets -- -D warnings
```

- 確率例：7テスト成功。基準値、時間0、rate0、極小値1e-20の正値保持、有限入力の大きな積の飽和、負数/NaN/無限大等の入力拒否、幾何分布の期待値を確認。表示は18.13%と1000。
- DAG例：cargo testは0件であり、テスト件数へ加えない。cargo run中のassertでPast(D)={A,B,C}を確認。両例ともClippy成功。
- 整形後のcargo fmt --all -- --checkも確認。これは教材の限られたモデルで、公式ノードやPoW/GHOSTDAGを再現・試験した結果ではない。

## 数値と演習の確認

Python標準Decimal（精度65桁）で二項分布n=20,p=0.1の確率和1、期待値2、分散1.8を1e-60以内で確認。幾何分布の残余確率と指数関数の数値を計算：

| 対象 | 値（丸め） |
|---|---|
| 0.999^1000 | 0.3676954247709640 |
| exp(-2) | 0.1353352832366127 |
| 1-exp(-0.2) | 0.1812692469220181 |
| 8bit/target15の期待試行数 | 16 |
| 8bit/target7の期待試行数 | 32 |

演習DAGを別のPython集合モデルでも計算し、Past(D)={A,B,C}、Past(E)={A,C}、Anticone(D)={E}を確認。丸めた百分率36.8/13.5/18.1/18.13と整合。一般のDAGや全確率分布に対する証明ではない。

## 一次資料・基準版との対応

- [Rust 1.90のexp_m1](https://doc.rust-lang.org/1.90.0/std/primitive.f64.html#method.exp_m1)を確認。小値向けの関数を使い、実行した1.90でも利用可能であることを確認した。
- [Thaler, Proofs, Arguments, and Zero-Knowledge](https://people.cs.georgetown.edu/jthaler/ProofsArgsAndZK.pdf)、表紙2023-07-18版。知識健全性の説明（§7.4等）と零知識のシミュレーション説明を参照。全329ページや個別証明方式を監査したものではない。
- 基準版v1.0.1 / fcd9c28f9b211b9872a0c90a8c94f79466ffb063の手元のソース資料でState::new/calculate_pow/check_powを確認。pow≤targetであり、headerを単純に一度ハッシュする式と実際の構成は区別した。Webの固定raw URL取得はcache missだったため、オンラインでソースを再取得したという記録にはしない。
- GHOSTDAG/DAA/署名等の入口は既存第12/13/16/26章へ接続。既存の公式関数比較やDevnet実測は再実行していない。

| 基準版の資料 | SHA-256 |
|---|---|
| consensus/pow/src/lib.rs | `eddc2cbf4118d63c57009345444ebdaaa8422f9d141e920bdd1f75cbe96324dc` |

## 統合・公開確認

- mdBook 0.5.4のビルド成功（警告なし）。53 HTMLのローカルリンク/アンカー検査で不整合0。
- SUMMARY49件と主表が一致、前段1〜45番の順序も一致。44番の付録Cを今回確認し、45番の付録Dを次の対象とした。
- scripts/check-book-examples.pyで既存2教材の本文と実装が一致。付録の一時教材は同スクリプトや既存CIの自動検査対象へ追加していない。
- git diff --check成功。
- 追跡対象/非ignore候補190ファイルの既知token prefix・秘密鍵・credential URL・個人home path等のパターン走査で検出0。gitleaks/trufflehogは未導入で、未知の機密やGit全履歴を含む完全監査ではない。
- 付録Cと進捗/ROADMAP、本記録を更新。既存の付録A/Bの差分を保持。私有パス・生ログ・鍵/DB・バイナリー・ダウンロードarchiveやexamplesフォルダーは追加していない。commit/push/新規インストール・実ノード操作はしていない。
