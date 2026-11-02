# Mastering Kaspa — BlockDAGの原理と実装

Kaspaの基礎から、BlockDAG、GHOSTDAG、コンセンサス、Rustによる実装、アプリケーション開発までを体系的に学ぶための技術書です。

**[オンラインで読む](https://shiga0.github.io/mastering-kaspa/)**

> **執筆・技術検証中（Draft）**
>
> 本書には未検証の説明や実行確認前のコードが含まれています。重要な内容は必ず一次資料と照合してください。

## この本の目的

- Kaspaの仕組みを「概念 → 原理 → 設計 → 実装 → 検証」の順に理解する。
- 論文・KIP・`rusty-kaspa`のソースコードを関連づけて学ぶ。
- Rustによる最小実装と再現可能な実験を通じて理解を深める。
- 学習・調査・検証の過程を記録し、コミュニティに共有する。

## 書籍構成

| Part | テーマ | 章 |
| --- | --- | --- |
| I | Foundations | 1〜8 |
| II | Consensus | 9〜14 |
| III | Transactions & Mining | 15〜20 |
| IV | Development & Implementation | 21〜28 |
| V | Advanced Kaspa Topics（SilverScript・Argentを含む） | 29〜34 ＋ 独立ページ「Argent」 |
| VI | Building & Contributing | 35〜36 |
| Appendices | 環境構築・用語集・参考文献など | A〜H |

Part Vでは、Covenants、SilverScript、**Argent（Actorベースのアプリケーション言語）**、Layer 2、vProgsなどを扱います。Argentは独立ページとして掲載し、既存のChapter 01〜36の番号は変更しません。

詳細な目次は `src/SUMMARY.md` を参照してください。

## ローカルで読む

本書はmdBookを使用しています。

mdBookのインストール：

```bash
cargo install mdbook --version 0.5.4 --locked
```

ローカルでプレビュー：

```bash
mdbook serve --open
```

HTMLの生成：

```bash
mdbook build
```

基準ソースは `rusty-kaspa` v1.0.1、実習はDevnetです。[対象バージョンと確認範囲](src/baseline.md)に、SDK・Rust・Dockerイメージの対応と既存原稿の版差をまとめています。

## 実行できるサンプル

[第27章の最小ノード](examples/minimal-node/README.md)を掲載しています。標準ライブラリのみで実行でき、16件の単体テストを含みます。

[第12章のDAG解析器](examples/dag-relations/README.md)も収録し、Past・Anticone・k-cluster条件と追加順の違いを検証しています。

完成したウォレット・報酬アプリは本書の収録対象外です。`examples/wallet/`、`examples/reward-app/` に実行可能なアプリは収録していません。HD復旧・送金などの限定した検証コードと記録は `experiments/` に保存しています。

章・付録別の確認範囲と既知の制限は[技術レビューの進捗](src/review-status.md)を参照してください。

Devnetの起動・読み取りwRPC・再起動の検証手順は[Devnet実習](experiments/devnet/README.md)を参照してください。公式WASM SDKによる単一署名送金とvirtual受理まで確認済みです。所定の送金異常系、2ノードの伝播・停止復帰・分断合流も確認済みです。Pruning後IBD、競合送金の再編成と会計処理は未検証です。

全49ページのレビューと所定の個別検証は一巡しました。CI・Pagesを含む公開確認は完了済みです。ライセンス・貢献方針と編集の区切りも決定・反映済みで、今回合意した仕上げ作業は完了です。[初版の仕上げ](ROADMAP.md)に完了条件をまとめ、追加の技術検証は既知の制限・将来課題として分けています。

## 原稿の状態

各章の状態を次のように区別します。

- **Draft**：公開前の原稿。レビュー済みの範囲と未検証部分は進捗表に記録。

技術的な記述の検証状態は、付録Hの基準に合わせて記録します。

- **Verified**：個別の主張を固定資料・実装と必要な試験で確認。対象条件と限界を明記。
- **Source-checked**：対象範囲の固定ソースと使用箇所を照合。実行の有無・結果は別に記録。
- **Literature-checked**：引用する論文・KIPの要旨/節を確認。実装・有効化の確認とは別。
- **Metadata-checked**：題名・著者・版・Status等のみを確認。内容や実装の検証とは別。
- **Illustrative**：教育用の擬似コード・模式図。
- **Unverified**：根拠・検証が不足。
- **Deprecated**：古い仕様・非推奨。

章の執筆状態（Draft）と、個々の説明・コードの検証状態は分けて扱います。

初稿があることは、技術的な正確性やコードの動作を保証するものではありません。特に開発中の技術は、仕様・実装・メインネットでの利用可否を区別して記載します。

## 一次資料

- [Kaspa公式サイト](https://kaspa.org/)
- [Kaspa Wiki](https://wiki.kaspa.org/)
- [Kaspa Improvement Proposals（KIPs）](https://github.com/kaspanet/kips)
- [rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)
- [Argent](https://github.com/argent-lang/argent)

## ライセンス・コントリビューション

本文・図・検証記録は **CC BY 4.0**、自作コードと本文中のコード例・設定例は **MIT** で提供します。外部由来の資料・コードには元の条件を維持します。適用範囲と条文は [LICENSE](LICENSE) を参照してください。

誤字・説明の改善・技術的な訂正をIssue／PRで受け付けます。大きな変更は事前にIssueで相談してください。投稿手順とライセンスは [CONTRIBUTING.md](CONTRIBUTING.md) にまとめています。

---

**Status:** Draft / 全49ページのレビュー一巡済み・検証範囲に制限あり

**Language:** 日本語（将来的に英語版も検討）

## 本文とサンプルの同期検査

```bash
python3 scripts/check-book-examples.py
```

第12章・第27章の `example:…:start/end` コメントで囲んだ範囲からRustコードを取り出し、対応するサンプルと比較します。サンプルを修正したら本文も更新してください。同期検査はCIでも実行します。
