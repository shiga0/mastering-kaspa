# Mastering Kaspa

Kaspaの基礎から、BlockDAG、GHOSTDAG、コンセンサス、Rustによる実装、アプリケーション開発までを体系的に学ぶための技術書です。

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

## 実行できるサンプル

[第27章の最小ノード](examples/minimal-node/README.md)を掲載しています。標準ライブラリのみで実行でき、6件の単体テストを含みます。

[第12章のDAG解析器](examples/dag-relations/README.md)も収録し、Past・Anticone・k-cluster条件と追加順の違いを検証しています。

`examples/wallet/`、`examples/reward-app/` は今後の実装用で、現時点では実行できるコードを収録していません。`experiments/` には検証記録を保存しています。

章・付録別の確認範囲と残作業は[技術レビューの進捗](src/review-status.md)を参照してください。

Devnetの起動・読み取りwRPC・再起動の検証手順は[Devnet実習](experiments/devnet/README.md)を参照してください。送金のE2E検証は未完了です。

## 原稿の状態

各章の状態を次のように区別します。

- **Draft**：初稿。技術レビュー前。

技術的な記述の検証状態は、付録Hの基準に合わせて記録します。

- **Verified**：一次資料と固定実装を確認し、必要なテストを実施。
- **Source-checked**：一次資料と固定実装を確認、実行未検証。
- **Literature-checked**：論文・KIPを確認、実装未確認。
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

ライセンスおよび外部からのコントリビューションの受け入れ方針は検討中です。

---

**Status:** Draft / Technical review pending

**Language:** 日本語（将来的に英語版も検討）

## 本文とサンプルの同期検査

```bash
python3 scripts/check-book-examples.py
```

第12章・第27章の `example:…:start/end` コメントで囲んだ範囲からRustコードを取り出し、対応するサンプルと比較します。サンプルを修正したら本文も更新してください。同期検査はCIでも実行します。
