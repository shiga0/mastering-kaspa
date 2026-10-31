# SUMMARY順レビュー 04：「Kaspa 技術・エコシステムマップ」

確認日：2026-10-31。対象は `src/SUMMARY.md` の4ページ目 `src/ecosystem-map.md`。分類・文章・読む順を確認し、プロジェクト自身の説明と位置づけを照合した。掲載プロジェクトの動作検証ではない。

## 修正した点

- アプリ・ツール・外部システムが同じ線でL1に接続する図を、分野の分類図へ変更。分類と通信・依存・有効化を区別した。
- 読む順から抜けていた第1〜4・17〜21・29〜30章を含め、全6部への案内に変更。Covenants→言語→L2→vProgsという一列の矢印をなくした。
- SilverScriptの生成先、Argentとの関係、公開リリースと各プロジェクトの開発段階を出典に対応づけた。
- vProgsの公開プロトタイプと、公式資料のFull vProgsの将来方向を区別。Based AppsとInline ZKも別の実行モデルとして案内し、ZK関連技術を外部L2へ一括分類しないようにした。
- 基準版v1.0.1と、後発の資料・開発環境の版を区別した。確認したSilverScript READMEはKaspa依存にrusty-kaspa 2.1.0クレートを使うと記載しており、基準版との互換性の証拠にはしない。
- Igra/Kasplexは提供者資料に基づく位置づけに限定。稼働・性能・セキュリティや、両者に共通のブリッジ構成を断定していない。

## 固定して確認したREADME

GitHub APIで参照先の完全なコミットSHAを取得し、そのSHAのREADME本文を取得・確認した。以下のSHA-256はREADMEファイルのバイト列に対する値。リポジトリ全体のコードレビューや依存グラフの照合を意味しない。

| 対象 | READMEの固定参照 | README SHA-256 |
|---|---|---|
| silverscript | [15b7bc27680bdf0d83b294fbbfda55ecb59958ca](https://github.com/kaspanet/silverscript/blob/15b7bc27680bdf0d83b294fbbfda55ecb59958ca/README.md) | `294d336218f6b1524f7af3cd63a96ea564a9fa202e634b3465e9a412dfbb1313` |
| argent | [2796809f167144902f04a4b3333a4c144f91fdaf](https://github.com/argent-lang/argent/blob/2796809f167144902f04a4b3333a4c144f91fdaf/README.md) | `bf8b4218368bfbecf8b80757adc689de7d99b27fdf933f53d7dcfec8f0aa396e` |
| vprogs | [f9b84a863a7c7c20586a9cf947550475e894f72e](https://github.com/kaspanet/vprogs/blob/f9b84a863a7c7c20586a9cf947550475e894f72e/README.md) | `0f171c036bb2c62dcf100eda76867b42fc1bdba203045e8cde6238be0fb8c7d9` |

SilverScriptはKaspa Scriptを対象とする言語・コンパイラ。ArgentはSilverScriptを基盤とし、READMEのProject statusは開発中・リリース準備未完了とする。vprogs READMEは初期開発・プロトタイプで、API・構成が変わり得るとする。これらは参照したREADMEの説明であり、本書による安全性評価ではない。

## その他の一次資料

いずれも上記確認日のWeb資料。版の固定されていないページは後から内容が変わり得る。

| 資料 | 確認した範囲 |
|---|---|
| [SilverScript v1.0.0](https://github.com/kaspanet/silverscript/releases/tag/v1.0.0) | リリース公開とKaspa Scriptへのコンパイルという説明。自分でリリースバイナリを実行したものではない |
| [Kaspa programmability overview](https://docs.kaspa.org/programmability) | Covenants、Based Apps、Inline ZKの区別と、Future DirectionにあるFull vProgsの位置づけ |
| [Igra Network Documentation](https://igra-labs.gitbook.io/igralabs-docs) | トップページのEVM互換という自己説明。全アーキテクチャ・ブリッジや稼働ネットワークは未照合 |
| [What is Kasplex L2](https://docs-kasplex.gitbook.io/l2-network) | L2資料の表題・公開ページと検索索引の自己説明。本文の取得は限定的で、詳細方式・性能・信頼条件の検証には使っていない |
| [DAG KNIGHT論文](https://eprint.iacr.org/2022/1494) | ePrintレコードの題名・要旨・著者・改訂情報。本文の全証明や実装・有効化は未検証 |

WebツールではGitHub APIを取得できなかったため、公開APIと固定コミットのREADMEはHTTPクライアントで取得した。KasplexのMarkdown版はWebツールで取得エラーとなったため、本文全体を読めたものとして扱っていない。動的なプロジェクトサイトの表示だけで稼働状況を断定することも避けた。

## 確認範囲と残作業

関連章のレビューを完了としたものではない。Covenants・各モデルの実装と有効化、言語のビルド・生成物、ネットワーク稼働、ブリッジの信頼条件、基準版との互換性は、それぞれの章と検証で確認する。ノードや外部ネットワークへの接続、採掘・送金・ブリッジ操作は今回行っていない。

次の対象はSUMMARYの5ページ目、第1章 `src/part-01-foundations/chapter-01.md`。

## 修正後の確認

mdBookビルド、生成HTML 53ページの内部リンク・アンカー、全49ページのSUMMARYと進捗表の順・ラベル、既存2教材の本文一致、差分の空白検査は成功。公開候補149ファイルの秘密情報・個人ホームパスのパターン検査で検出なし。これはパターン検査の結果であり、秘密情報の不存在を保証するものではない。GitHub上の実ジョブは今回確認していない。
