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

| Part       | テーマ                         | 章     |
| ---------- | ------------------------------ | ------ |
| I          | Foundations                    | 1〜8   |
| II         | Consensus                      | 9〜14  |
| III        | Transactions & Mining          | 15〜20 |
| IV         | Development & Implementation   | 21〜28 |
| V          | Advanced Kaspa Topics          | 29〜34 |
| VI         | Building & Contributing        | 35〜36 |
| Appendices | 環境構築・用語集・参考文献など | A〜H   |

詳細な目次は `src/SUMMARY.md` を参照してください。

## ローカルで読む

本書はmdBookを使用しています。

mdBookのインストール：

```bash
cargo install mdbook
```

ローカルでプレビュー：

```bash
mdbook serve --open
```

HTMLの生成：

```bash
mdbook build
```

## 原稿の状態

各章の状態を次のように区別します。

- **Draft**：初稿。技術レビュー前。
- **Reviewed**：一次資料との照合済み。
- **Verified**：掲載コードや実験の検証済み。

初稿があることは、技術的な正確性やコードの動作を保証するものではありません。

## 一次資料

- Kaspa公式サイト
- Kaspa Wiki
- Kaspa Improvement Proposals（KIPs）
- rusty-kaspa

## ライセンス・コントリビューション

ライセンスおよび外部からのコントリビューションの受け入れ方針は検討中です。

---

**Status:** Draft / Technical review pending

**Language:** 日本語（将来的に英語版も検討）
