# Appendix H — 参考文献・変更履歴

> **Mastering Kaspa** / 付録H  
> 原稿：v0.1（技術レビュー前） / 参照確認日：2026-10-30  
> 本付録は書籍全体の出典管理、実装バージョンの固定、変更履歴、レビュー記録を扱う。

## H.1 この付録の目的

本書の技術的な記述について、読者が「どこに書かれているか」「いつの実装か」「実験で確かめたか」を追跡できるようにする。研究論文、KIP、ソースコード、リリースノート、コミュニティの解説は、**証明できる内容が異なる**。

- **論文**：モデル、仮定、アルゴリズム、証明の根拠。
- **KIP**：変更の動機、仕様、議論、提案状態の根拠。
- **ソースコード**：指定コミットにおける実装の根拠。
- **リリースノート**：リリースで提供された変更の根拠。
- **実験結果**：記録した環境で観測した振る舞いの根拠。
- **解説記事**：理解を助ける二次資料。仕様の最終根拠にはしない。

## H.2 出典の優先順位

| 確認したいこと | 優先する資料 | 追加で必要な確認 |
|---|---|---|
| GHOSTDAGの定義 | 原論文 | 用語・仮定・数式 |
| DAGKnightの提案内容 | 論文とKIP-0002 | 実装・有効化とは分離 |
| コンセンサス規則 | 対象リリースのコード | テスト・有効化条件 |
| パラメータ値 | 固定コミットの設定 | ネットワーク別・アップグレード別 |
| RPCの振る舞い | 対象バージョンの実装・API | 接続方式・エラー・通知 |
| Covenants/ZKの利用可否 | KIP・リリース・実装 | メインネット有効化と機能範囲 |
| アプリケーションの安全性 | 設計・監査・実験 | 脅威モデルと残余リスク |

**重要：** KIPの `Active` は、個々のクライアント・ネットワークでの実行結果を直接証明するものではない。逆に、実装コードが存在しても、有効化されていない機能は利用できない場合がある。

## H.3 Kaspa公式・一次資料

| ID | 資料 | URL | 主な用途 |
|---|---|---|---|
| K-01 | Kaspa公式サイト | https://kaspa.org/ | 概要・公式発表 |
| K-02 | Kaspa Wiki | https://wiki.kaspa.org/ | 用語・開発者向け資料 |
| K-03 | rusty-kaspa | https://github.com/kaspanet/rusty-kaspa | ノード・コンセンサス・SDK |
| K-04 | rusty-kaspa Releases | https://github.com/kaspanet/rusty-kaspa/releases | リリースと変更点 |
| K-05 | Kaspa Improvement Proposals | https://github.com/kaspanet/kips | 改善提案と仕様 |
| K-06 | KIP一覧 | https://github.com/kaspanet/kips/blob/master/README.md | KIP番号・状態 |
| K-07 | Kaspa Research Forum | https://research.kas.pa/ | 研究議論（一次論文とは区別） |

### H.3.1 版の固定

GitHubの `master` や `main` は変化するため、出版時には以下のように固定する。

```bash
# 調査時点の実装を取得
 git clone https://github.com/kaspanet/rusty-kaspa.git
 cd rusty-kaspa
 git rev-parse HEAD
 git describe --tags --always
 git status --short
```

書籍の注記例：

> `rusty-kaspa` v2.1.0（Gitタグ・コミットIDを別途確認）を対象に調査。検証環境と実験IDは付録Gを参照。

**注：** 上記の例は記録形式の説明であり、この付録でソースの全行を実行・監査したことを意味しない。

## H.4 基礎論文と学術資料

### H.4.1 BlockDAGとコンセンサス

| ID | 文献 | 主な対象 | 検証状況 |
|---|---|---|---|
| P-01 | *SPECTRE: Serialization of Proof-of-work Events: Confirming Transactions via Recursive Elections* | BlockDAG上の投票・順序付け | 書誌情報・版の固定が必要 |
| P-02 | *PHANTOM: A Scalable BlockDAG Protocol* | k-cluster、BlockDAGの安全性 | 書誌情報・版の固定が必要 |
| P-03 | *DAGKnight: A Parameterless Generalization of GHOSTDAG* | DAGKnightの研究モデル | 書誌情報・版の固定が必要 |
| P-04 | Bitcoin: *A Peer-to-Peer Electronic Cash System* | PoW・UTXOの歴史的背景 | 原文確認推奨 |

論文タイトル・著者・発表年・識別子（DOI/arXiv等）は、最終校正時に**原本の表紙・公開ページ**と照合する。本文の説明は、論文で証明された条件を省略して一般化しない。

参考入口：

- Bitcoin whitepaper: https://bitcoin.org/bitcoin.pdf
- Kaspa research forum: https://research.kas.pa/
- Google Scholar: https://scholar.google.com/

### H.4.2 暗号技術とZK

- NIST FIPS 180-4（Secure Hash Standard）：https://csrc.nist.gov/pubs/fips/180-4/upd1/final
- NIST FIPS 186-5（Digital Signature Standard）：https://csrc.nist.gov/pubs/fips/186-5/final
- ZK-SNARK/STARKについては、採用する証明方式・検証器・ライブラリの論文および固定版を**個別に追加**する。

**区別：** 一般的な暗号学の定義と、Kaspaで採用される具体的なアルゴリズムや署名方式は同じとは限らない。

## H.5 主要KIP対応表

2026-10-30に公開KIP一覧を参照した。状態は将来変更されるため、出版時に再確認する。

| KIP | 公開一覧上の主題 | 一覧の状態（確認時点） | 本書の関連章 |
|---|---|---|---|
| KIP-0001 | Rustフルノードへの書き換え | Implemented | 9, 25, 36 |
| KIP-0002 | DAGKnightへの移行 | Proposed | 29, 30 |
| KIP-0004 | Sparse Difficulty Windows | Active | 13, 30 |
| KIP-0009 | Extended Mass Formula | Active | 15, 30 |
| KIP-0010 | Scriptオペコード拡張 | Active | 16, 31 |
| KIP-0014 | Crescendo Hardfork | Active | 6, 9, 30 |
| KIP-0015 | Canonical Transaction Ordering | Active | 14, 34 |
| KIP-0016 | ZK Precompile Opcode | Active | 33, 34 |
| KIP-0017 | Covenants | Active | 31, 32 |
| KIP-0020 | Covenant IDs | Active | 31, 32 |
| KIP-0021 | Partitioned Sequencing Commitment | Active | 33, 34 |

一次資料：https://github.com/kaspanet/kips/blob/master/README.md

KIPの番号・タイトル・状態は公開一覧に基づく。実装箇所や仕様の適用範囲は**各KIP本文と対象リリース**を確認すること。

## H.6 実装とリリース履歴の扱い

`rusty-kaspa`の公開リリースページでは、2026年9月22日付の **v2.1.0** が最新リリースとして掲載されている（2026-10-30確認）。リリースノートはP2P、IBD、ZK SDK、算術安全性、Toccata後の整理などを説明している。

また、**v2.0.0** のリリースノートはToccataハードフォークを扱い、KIP-0016/0017/0020/0021との関連を明記している。

参照：https://github.com/kaspanet/rusty-kaspa/releases

### H.6.1 「いつ利用可能になったか」を調べる手順

1. KIPの仕様と状態を確認する。
2. リリースノートで導入・有効化条件を確認する。
3. 固定コミットのコードで分岐・検証処理を探す。
4. 対象ネットワークのアップグレード条件を確認する。
5. テストネットまたはDevnetで期待した振る舞いを検証する。
6. 結果を付録Gの実験IDに紐付ける。

## H.7 書籍内の引用・参照ルール

### H.7.1 出典表記

本文の技術的な主張には、可能な限り次を残す。

```text
[出典ID] 文献またはリポジトリ名
対象：タグ / コミット / 論文版 / KIP更新日
該当箇所：節番号 / ファイルパス / 関数名 / 行範囲
確認日：YYYY-MM-DD
検証：未実施 / ソース確認 / テスト実施
```

GitHubの行番号だけではリファクタリングでずれるため、**コミットSHAとシンボル名**を併記する。

### H.7.2 原文の引用

- 必要な短い範囲に限って引用し、出典とライセンスを確認する。
- 図やコードを転載する場合、リポジトリのLICENSEと個別ファイルの条件を確認する。
- 日本語訳は「著者による訳」と明記し、原文への参照を残す。
- 説明用に改変したコードは、原実装と誤認されないよう「教育用モデル」と記載する。

## H.8 章別の出典管理表

| 章 | 優先確認対象 | レビュー観点 |
|---|---|---|
| 1–8 | Kaspa/Bitcoin基礎文献 | 歴史・前提・用語 |
| 9–14 | rusty-kaspa、GHOSTDAG、DAA | 順序付け・状態・パラメータ |
| 15–20 | Transaction/Script/Mempool/Mining実装 | 署名・UTXO・PoW・安全性 |
| 21–28 | Node/SDK/RPC/Devnet実装 | バージョン・再現手順 |
| 29–30 | DAGKnight論文、KIP一覧 | 研究・提案・導入の区別 |
| 31–32 | KIP-0010/0017/0020、言語実装 | Covenant機能とツール対応 |
| 33–34 | KIP-0015/0016/0021、ZK研究 | 証明とDA、L1/L2の責務 |
| 35–36 | RPC/SDK/CONTRIBUTING/CI | 実装・運用・貢献手順 |
| 付録A–G | ツールチェーン・設定・実験ログ | 実行可能性・追跡可能性 |

## H.9 技術レビューの判定基準

| 判定 | 定義 | 出版上の扱い |
|---|---|---|
| Verified | 一次資料と固定実装を確認し、必要なテストを実施 | 原則採用可 |
| Source-checked | 一次資料と固定実装を確認、実行未検証 | 検証条件を注記 |
| Literature-checked | 論文・KIPを確認、実装未確認 | 理論・提案として説明 |
| Illustrative | 教育用の擬似コード・模式図 | 実装との非互換性を明記 |
| Unverified | 根拠・検証が不足 | 断定を避け、公開前に解消 |
| Deprecated | 古い仕様・非推奨 | 歴史的説明に限定 |

**出版ルール：** コンセンサス・暗号・資産管理に関する強い主張を、`Unverified` のまま断定しない。

## H.10 変更履歴の設計

書籍の変更を「文章修正」と「技術的な意味の変更」に分ける。

| 変更種別 | 例 | 必要な確認 |
|---|---|---|
| Editorial | 誤字、見出し、表現 | 編集レビュー |
| Clarification | 用語・図解の改善 | 内容の整合性 |
| Technical correction | 誤った数式・コード修正 | 一次資料とテスト |
| Version update | Rust/Kaspaリリース更新 | 差分と互換性 |
| Protocol update | KIPの有効化・コンセンサス変更 | ネットワーク別確認 |
| Experiment update | 実測値・再現手順更新 | ログ・環境・証拠 |

## H.11 本書の変更履歴（初期記録）

| 日付 | 版 | 対象 | 内容 | 状態 |
|---|---|---|---|---|
| 2026-10-30 | v0.1 | Appendix H | 参考文献・KIP対応・レビュー・変更履歴の雛形を作成 | 技術レビュー前 |
| 未記入 | v0.1 | Chapters 1–36 | 各章初稿の統合と整合性確認 | 要記録 |
| 未記入 | v0.1 | Appendices A–G | 付録の統合と出典確認 | 要記録 |

他章の実際の更新日・コミット・検証状況は、この表から推測せず、**リポジトリのGit履歴を基に追記する**。

## H.12 Gitによる変更履歴の追跡

```bash
# 書籍リポジトリで実行
git status --short
git log --oneline --decorate -n 15

# 特定章の履歴
git log --follow -- src/part-05-advanced/chapter-34.md

# 差分確認
git diff -- src/appendices/appendix-h.md

# 出典修正のコミット例
git add src/appendices/appendix-h.md
git commit -m "docs: add bibliography and review policy"
```

公開後はGitタグで書籍版を固定すると、読者が同じ内容を再参照しやすくなる。

## H.13 リリースノートの雛形

```markdown
# Mastering Kaspa v0.x.y

公開日: YYYY-MM-DD
対象 Kaspa ノード: <version + commit>
対象 Rust: <rustc --version>

## Added
- ...

## Changed
- ...

## Fixed
- ...

## Known limitations
- 実行していないコード例
- 未確認のKIP/ネットワーク有効化条件

## Verification
- cargo test: <結果>
- mdbook build: <結果>
- Devnet: <実験ID>
```

## H.14 ソース・実験・章の追跡可能性

```text
       論文 / KIP / 公式仕様
                 │
                 ▼
         固定コミットの実装
                 │
                 ▼
            テスト・実験
                 │
                 ▼
         章・節・図・コード
                 │
                 ▼
           レビュー記録
                 │
                 ▼
           公開リリース
```

最低限の対応表：

| Claim ID | 主張 | Source ID | Code commit | Experiment ID | Review |
|---|---|---|---|---|---|
| CLM-001 | GHOSTDAGの特定の性質 | P-02 | 未記入 | G-005 | Unverified |
| CLM-002 | UTXO二重消費の拒否 | K-03 | 未記入 | G-003 | Unverified |
| CLM-003 | ZK検証機能の存在 | K-05/K-04 | 未記入 | 未記入 | Literature-checked |

この表は**管理形式の例**であり、検証済みの主張一覧ではない。

## H.15 技術レビューIssueのテンプレート

```markdown
## 対象
- Chapter / Section:
- Claim ID:

## 問題
- 現在の記述:
- 疑義・再現条件:

## 一次資料
- 論文/KIP:
- Commit/Tag:
- 関連テスト:

## 対応
- [ ] 記述修正
- [ ] コード修正
- [ ] 実験追加
- [ ] 参照リンク修正

## 判定
- Unverified / Literature-checked / Source-checked / Verified
```

## H.16 執筆と技術レビューの分離

**執筆**では、概念を理解できるようにする。**技術レビュー**では、その説明が資料と実装に照らして正しいかを確認する。二つの作業を混ぜると、読みやすいが誤った説明が残りやすい。

推奨する流れ：

1. 章の目的と読者の前提知識を定義。
2. 一次資料を収集して版を固定。
3. 概念図と本文を作成。
4. 数式・コード・数値に出典IDを付ける。
5. コードを実行し、実験IDを付ける。
6. 別の視点でレビュー。
7. 修正履歴を残して公開。

## H.17 出版前の確認リスト

- [ ] 全36章と付録A–Hが `SUMMARY.md` に登録されている。
- [ ] `mdbook build` が成功する。
- [ ] 章間リンクと画像リンクが切れていない。
- [ ] 主要な数式・定数・KIP状態に一次資料がある。
- [ ] ソースコード参照はタグ・コミットで固定されている。
- [ ] Rustコードはコンパイル・テスト結果を記録している。
- [ ] Devnetの実験結果に環境情報がある。
- [ ] 研究段階の構想をメインネット実装済みと混同していない。
- [ ] サードパーティのコード・図のライセンスを確認した。
- [ ] 変更履歴と既知の制限事項を公開している。

## H.18 mdBookへの組み込み

ファイルの配置：

```text
mastering-kaspa/
└── src/
    ├── SUMMARY.md
    └── appendices/
        └── appendix-h.md
```

`src/SUMMARY.md` の例：

```markdown
- [Appendix H — 参考文献・変更履歴](appendices/appendix-h.md)
```

ビルド：

```bash
mdbook build
mdbook serve --open
```

## H.19 継続的なメンテナンス

Kaspaの実装・KIP・SDKは更新される。公開後も、少なくとも**新しい主要リリース時**に関連章を見直す。更新対象は「値の変更」だけではなく、用語、互換性、セキュリティ前提、RPC応答、テスト手順なども含む。

自動化できる部分：リンクチェック、Markdown lint、Rustテスト、mdBookビルド、参照タグの存在確認。自動化しにくい部分：論文の仮定、コンセンサスの意味、脅威モデル、実際のネットワーク有効化条件。

## H.20 まとめ

本書の価値は、技術を説明することだけではなく、**説明を検証可能な知識に変えること**にある。

- 論文は「なぜその設計か」を示す。
- KIPは「何を変えるか」を示す。
- コードは「その版で何が実装されたか」を示す。
- 実験は「何を確認したか」を示す。
- 変更履歴は「いつ、なぜ記述を更新したか」を示す。

『Mastering Kaspa』を読み続けられる技術書にするために、参考文献と変更履歴を本文と同じくらい重要な成果物として扱う。

---

**原稿ステータス：v0.1（技術レビュー前）**  
**未完了：** 論文の正確な書誌情報・固定URL、全章のコミット参照、Rustコードの実行結果、各章の出典IDの最終照合。
