# SUMMARY順レビュー 05：第1章「Kaspa入門」

確認日：2026-10-31。対象は `src/part-01-foundations/chapter-01.md`。入門概説・図・説明の範囲・後続章との対応・コード閲覧課題を確認した。

## 修正内容

- 「なぜ必要か」という見出しを、具体的に取り組む課題の説明へ変更した。
- Bitcoinの選択チェーンと、分岐を含むブロック候補全体を区別。Blockchainもグラフの一種であり、比較対象を限定した。
- 両図で親が上、子が下であることを説明。参照は子から親へ向かい、線自体は方向を持つ矢印ではないことを明記した。
- 並行ブロックも検証規則を満たす必要があり、DAGへ無条件に取り込まれる説明にしないよう修正した。
- GHOSTDAGの順序付けと、UTXO・署名等の検証によるTX受理を区別。blue/redは評価対象の過去に対する分類であり、有効性や永久不変の属性ではないことを補った。
- blue scoreとblue workを区別し、selected parentはblue work・同値時hashで選ぶと説明した。Virtualは採掘・配信される実ブロックとして扱わない。
- PoWがID数ではなく仕事量を用いる役割と、PoWの解の確認以外にも検証があることを補った。
- Mempoolを未確定TX全体と同一視しない説明へ変更した。
- ソース閲覧をv1.0.1の取得へ固定。リポジトリ外で実行し、完全SHAとworkspaceの3項目を確認する課題にした。新規cloneやビルドの成功は今回主張しない。
- 初稿の一律な未確認表示を、今回確認した範囲と残る範囲の表示へ変更した。

## 根拠と照合範囲

| 根拠 | 確認範囲 |
|---|---|
| [Bitcoin論文](https://bitcoin.org/bitcoin.pdf) 第4・5節 | PoWと参加者数の関係、並行ブロック・チェーン選択の概説。全証明のレビューではない |
| [Bitcoin開発者ガイド](https://developer.bitcoin.org/devguide/block_chain.html) | 親参照、UTXO、難易度調整、分岐と累積仕事量による有効チェーン選択。平均10分は目標値である |
| [PHANTOM and GHOSTDAG](https://eprint.iacr.org/2018/104) | ePrintレコード・要旨。BlockDAGと順序付けの研究上の位置づけ。今回PDF全文の証明は確認していない |
| 基準版GHOSTDAGのprotocol.rs / ordering.rs | selected parentのblue work/hash比較。既存ソースのSHA-256を以前の記録と照合した |
| 基準版Cargo.toml | workspaceのconsensus、rpc/core、wallet/coreが存在することを解析して確認した |
| 既存のGHOSTDAG・第14/15章照合記録 | 分類の文脈、blue score/work、VirtualとUTXO受理の範囲。以前の実測を今回の新規実行へ読み替えていない |

基準版はv1.0.1、完全SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。固定ソースのパスとSHA-256は `ghostdag-baseline-review-2026-10-31.md`、状態処理は `consensus-state-review-2026-10-31.md` を参照。今回のHTTP資料の確認日は上記確認日であり、後から更新され得る。

## 残る範囲

論文の全安全性証明、現在のMainnetの性能・有効化、全ネットワークの規則、新規環境でのclone・ビルドは今回の対象外。学習用の構成図は実モジュールの依存図ではない。数値として最新のBPSや即時の不可逆性を追加していない。

次の対象はSUMMARYの6ページ目、第2章 `src/part-01-foundations/chapter-02.md`。

## 修正後の確認

mdBookビルド、生成HTML 53ページの内部リンク・アンカー、SUMMARY全49ページと進捗表の順・ラベル、既存2教材の本文一致、差分の空白検査は成功。固定GHOSTDAGソース2件のハッシュ一致と、Cargo.tomlの閲覧課題3項目も確認した。公開候補150ファイルの秘密情報・個人ホームパスのパターン検査で検出なし。今回Rust・Dockerの実行試験や新規cloneは行っていない。
