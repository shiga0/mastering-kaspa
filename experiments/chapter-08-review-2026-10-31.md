# SUMMARY順レビュー 12：第8章「ブロックチェーンからKaspaへ」

確認日：2026-10-31。対象は `src/part-01-foundations/chapter-08.md`。第1部の総まとめとして、前章までの説明・基準版の処理経路・後続章への導線を確認した。

## 修正内容

- Bitcoinから別の構造への変更が必須であるような導入を、Kaspaが何を改善しようとする設計なのかという問いへ変更した。
- BitcoinはGenesis以外で一つの親を持ち、候補全体に分岐があると説明。分岐図の親子関係と参照方向を整理した。
- 追加発見と別枝の発生を区別し、確率表は仮の条件であると明記した。
- Past/Futureの自己除外、Futureは時計の未来ではないこと、Anticoneと反鎖の違いを第7章に揃えた。
- selected parentのblue work・hash、blue scoreとの区別、Virtualの先端選択と仮想的な性質を整理した。
- PHANTOMの最適化問題の計算負荷と、GHOSTDAGの貪欲法という関係を論文要旨へ対応づけた。「現行ノード」ではなく本書の基準版と記載した。
- 「基本検証→DAG取り込み→GHOSTDAG」という処理図を、ヘッダー（GHOSTDAGを含む）→本体→Virtualの案内へ修正した。保存と全検証の完了を同一視せず、並列処理・全経路を網羅した図とも扱わない。
- inclusionとacceptanceの区別を維持し、Virtualの全tips無条件採用・永久不変という誤読を避けた。
- 主要コンポーネント表と次章のリンクを具体化し、Mempoolや支出権限の表記を前章に揃えた。
- 第6章と重複し、古い計算方法・競合機会という表示を持つRustコードの再掲をなくした。既存の確認済み教材を参照し、設計判断へつなぐ考察に集約した。

## 根拠と再利用した検証

| 根拠 | 確認範囲 |
|---|---|
| [PHANTOM and GHOSTDAG](https://eprint.iacr.org/2018/104) | 要旨のPHANTOM/GHOSTDAGの関係。PDFの全証明を確認した記録ではない |
| [Bitcoin開発者ガイド](https://developer.bitcoin.org/devguide/block_chain.html) | 親参照・分岐・有効チェーンの選択。現行ネットワークの数値の測定ではない |
| `chapter-06-review-2026-10-31.md` | 確率計算教材の既存の実行・5テスト・表示と高精度計算の照合。本章でコードを新規実行したものとはしない |
| `chapter-07-review-2026-10-31.md` | 集合の定義、自己包含の規約の区別、教材の確認範囲 |
| `ghostdag-baseline-review-2026-10-31.md` | 親選択・score/work等の既存の基準版照合 |
| `core-validation-review-2026-10-31.md` | ヘッダー/GHOSTDAG/本体/UTXOの検査段階と状態区分 |
| `consensus-state-review-2026-10-31.md` | 順序に沿ったUTXO検証、Virtualの親選択と状態の主要経路 |

HTTP資料は上記確認日に参照。今回改めて読んだ基準版v1.0.1の入口は以下の通り。完全SHAは `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。

| ファイル | SHA-256 |
|---|---|
| [consensus/src/pipeline/header_processor/processor.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/header_processor/processor.rs) | `27d5e5964e41ad4fd47df07a7c02fe5506386a5f91b13d3288578638cfba443c` |
| [consensus/src/pipeline/virtual_processor/utxo_validation.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/virtual_processor/utxo_validation.rs) | `06eaec070d3829452b2584aabd58289f5c7c6dac6b7711c91fd8c272d08ea246` |

通常のvalidate_headerでGHOSTDAGがヘッダー処理に含まれること、UTXO検証側でselected parent等の受理・差分を扱う入口を確認した。UTXO検証ファイルのハッシュは既存記録とも一致した。

## 今回の確認と限界

確率表の600/10/1秒の3行は、Python Decimalの精度60桁で計算し、百分率の小数第2位まで0.17/9.52/63.21と一致した。新たなRustコードやノード実験を追加せず、既存の教材・検証結果を再利用した。Mainnetの現在の性能・有効化状態、論文の全安全性証明、全Kaspa処理の正確性を確認したものではない。

mdBookビルド、生成HTML 53ページの内部リンク・アンカー、SUMMARY全49ページと進捗表の順・ラベル、既存2教材の本文一致、差分の空白検査は成功。第6章教材への見出しリンクも生成HTMLで存在を確認した。

これで第1部の目次順の通読・修正を終えた。第1部に関係する全技術・全規則の検証が完了したという意味ではない。次はSUMMARYの13ページ目、第2部・第9章 `src/part-02-consensus/chapter-09.md`。

公開候補158ファイルの秘密情報・個人ホームパスのパターン検査で検出なし。新たなノード接続・採掘・送金・外部公開は行っていない。
