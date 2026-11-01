# Appendix H — 参考文献・変更履歴

> **Mastering Kaspa** / 付録H  
> 目次順の本文レビュー：2026-11-01。主要書誌・参照版・Git履歴/検証索引を確認。\
> 本付録は書籍全体の出典管理、実装バージョンの固定、変更履歴、レビュー記録を扱う。

基準版は[本書の対象バージョン](../baseline.md)、ページごとの範囲は[技術レビューの進捗](../review-status.md)、今回の記録は `experiments/appendix-h-review-2026-11-01.md` を参照。目次順の通読と、全主張・全実装の検証完了は区別する。

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

## H.3 公式サイト・ソースと調査の入口

| ID | 資料 | URL | 主な用途 |
|---|---|---|---|
| K-01 | Kaspa公式サイト | https://kaspa.org/ | 概要・公式発表 |
| K-02 | Kaspa Wiki | https://wiki.kaspa.org/ | 用語の入口。仕様は原論文・固定実装へ戻る |
| K-03 | rusty-kaspa | https://github.com/kaspanet/rusty-kaspa | ノード・コンセンサス・SDK |
| K-04 | rusty-kaspa Releases | https://github.com/kaspanet/rusty-kaspa/releases | リリースと変更点 |
| K-05 | Kaspa Improvement Proposals | https://github.com/kaspanet/kips | 改善提案と仕様 |
| K-06 | KIP一覧 | https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/README.md | KIP番号・状態 |
| K-07 | Kaspa Research Forum | https://research.kas.pa/ | 研究議論（一次論文とは区別） |

### H.3.1 版の固定

GitHubのmaster/mainやタグ名だけでなくSHAを記録する。本書ではノードv1.0.1 / `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`、KIP文書 `e4ae2332117b5cb68bd6188e065ef885b6d17939` を別の参照軸として使う。同じ時点の資料でも、Mainnetへの有効化証拠でもない。

本書外の調査用ディレクトリで取得する読者用手順：

```bash
git clone --branch v1.0.1 --depth 1 https://github.com/kaspanet/rusty-kaspa.git rusty-kaspa-v1.0.1
cd rusty-kaspa-v1.0.1
git rev-parse HEAD
# fcd9c28f9b211b9872a0c90a8c94f79466ffb063 と一致することを確認
git status --short
```

今回は新規cloneをしていない。既存の固定ソースarchiveを読むことと、Git checkoutのHEAD取得や全ノードの再ビルドは別。depth 1は全履歴を含まず、過去の設計変更を追うには別の完全な調査コピーが必要。

注記の形式例：

> rusty-kaspa v1.0.1 / `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` の主要処理を照合。配布Dockerバイナリーとのソース同一性や現行Mainnet仕様を保証しない。環境・配布物hashと確認範囲はbaseline・個別実験記録へ記載。

## H.4 基礎論文と学術資料

### H.4.1 BlockDAGとコンセンサス

| ID | 文献 | 主な対象 | 検証状況 |
|---|---|---|---|
| P-01 | Yonatan Sompolinsky, Yoad Lewenberg, Aviv Zohar, [*SPECTRE: A Fast and Scalable Cryptocurrency Protocol*](https://eprint.iacr.org/2016/1159), ePrint 2016/1159 | BlockDAG上の投票・順序付け | 公開書誌/要旨を確認。ページの最終改訂2018-01-15。証明の全面照合ではない |
| P-02 | Yonatan Sompolinsky, Shai Wyborski, Aviv Zohar, [*PHANTOM and GHOSTDAG: A Scalable Generalization of Nakamoto Consensus*](https://eprint.iacr.org/2018/104), ePrint 2018/104 | k-cluster、BlockDAG | 第29章の書誌確認記録に合わせた。今回の公開ページ取得は失敗し、PDF版/hashは未固定 |
| P-03 | Yonatan Sompolinsky, Michael Sutton, [*The DAG KNIGHT Protocol: A Parameterless Generalization of Nakamoto Consensus*](https://eprint.iacr.org/2022/1494), ePrint 2022/1494 | DAGKnightの研究モデル | 公開書誌/要旨を確認。最終改訂2023-02-24。証明の全面照合ではない |
| P-04 | Satoshi Nakamoto, [*Bitcoin: A Peer-to-Peer Electronic Cash System*](https://bitcoin.org/bitcoin.pdf), 2008 | PoW・電子現金の歴史的背景 | PDF表紙の題名・著者と[2008-10-31の原告知](https://www.metzdowd.com/pipermail/cryptography/2008-October/014810.html)を確認。全モデルや現行Bitcoin実装との照合ではない |

確認日は2026-11-01。ePrint識別子の年、受理日、改訂日、学会での出版年を同じ値と扱わない。上記は公開ページや表紙の確認であり、厳密な引用には実際に読んだPDFの版/hashと節・定理番号を追加する。未確認のDOIや改訂日を推測で埋めない。書誌と公開要旨の確認経緯は[第29章](../part-05-advanced/chapter-29.md)と各レビュー記録を参照。

### H.4.2 暗号技術とZK

- [NIST FIPS 180-4, Secure Hash Standard](https://csrc.nist.gov/pubs/fips/180-4/upd1/final)：2015-08の公開情報を確認。掲載ページに改訂予定の注記がある。
- [NIST FIPS 186-5, Digital Signature Standard](https://csrc.nist.gov/pubs/fips/186-5/final)：2023-02-03公開。掲載ページに将来修正/errataの案内がある。
- Justin Thaler, [*Proofs, Arguments, and Zero-Knowledge*](https://people.cs.georgetown.edu/jthaler/ProofsArgsAndZK.pdf)、2023-07-18版。付録Cの概念整理で参照した範囲を同付録の記録に明記。
- ZK-SNARK/STARKについては、採用する証明方式・検証器・ライブラリの論文および固定版を**個別に追加**する。

NISTのページmetadata/注意書きの照合であり、規格本文やerrata全体を実装監査したものではない。

**区別：** 一般的な暗号学の定義と、Kaspaで採用される具体的なアルゴリズムや署名方式は同じとは限らない。

## H.5 主要KIP対応表

第30章・付録Fと同じ固定KIP `e4ae2332117b5cb68bd6188e065ef885b6d17939` の一覧に合わせる。表題は要旨であり本文のTitleと異なる場合がある。主要表の番号/Statusを照合し、最新の状態やネットワークでの稼働を示す表にはしない。

| KIP | 公開一覧上の主題 | 一覧の状態（確認時点） | 本書の関連章 |
|---|---|---|---|
| [KIP-0001](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0001.md) | Rustフルノードへの書き換え | Implemented | 9, 25, 36 |
| [KIP-0002](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0002.md) | DAGKnightへの移行 | Proposed | 29, 30 |
| [KIP-0004](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0004.md) | Sparse Difficulty Windows | Active | 13, 30 |
| [KIP-0009](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0009.md) | Extended Mass Formula | Active | 15, 30 |
| [KIP-0010](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0010.md) | Scriptオペコード拡張 | Active | 16, 31 |
| [KIP-0014](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0014.md) | Crescendo Hardfork | Active | 6, 9, 30 |
| [KIP-0015](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0015.md) | Canonical Transaction Ordering | Active | 14, 34 |
| [KIP-0016](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0016.md) | ZK Precompile Opcode | Active | 33, 34 |
| [KIP-0017](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0017.md) | Covenants | Active | 31, 32 |
| [KIP-0020](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0020.md) | Covenant IDs | Active | 31, 32 |
| [KIP-0021](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0021.md) | Partitioned Sequencing Commitment | Active | 33, 34 |

一次資料：https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/README.md

KIPの番号・タイトル・状態は公開一覧に基づく。実装箇所や仕様の適用範囲は**各KIP本文と対象リリース**を確認すること。

## H.6 実装とリリース履歴の扱い

本書の実装照合/実習対象は[公式v1.0.1リリース](https://github.com/kaspanet/rusty-kaspa/releases/tag/v1.0.1)と、baselineに記録したSHA/配布物である。今回リリースページを参照し、タグv1.0.1と短縮SHA fcd9c28の表示を確認した。ソース/SDK/コンテナの同一性をページ表示だけで保証しない。

この付録は最新版や別リリースの提供機能を認定する一覧ではない。別リリースを扱う場合は、そのタグ/SHA・変更点・ネットワーク有効化・互換性を別途確認して記録する。

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
| Verified | 個別の主張について固定資料/実装と必要な試験を確認 | 対象条件・限界付きで採用。章全体/安全性の証明へ拡張しない |
| Source-checked | 対象範囲の固定ソース/使用箇所を照合 | 実行の有無・部分的な試験結果は別に記録 |
| Literature-checked | 引用する論文/KIPの要旨・節の内容を確認 | 読んだ範囲を明記。実装/有効化や他節の証明は別 |
| Metadata-checked | 題名・著者・版・Status等のみを確認 | 内容の正しさや実装の根拠にしない |
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

## H.11 本書の変更履歴（確認した区切り）

Git logの実在コミットと、今回の未コミット作業を分けて記載する。新しい書籍版のタグや公開リリースを作成した意味ではない。

| 日付 | 参照 | 対象 | 内容 |
|---|---|---|---|
| 2026-10-31 | `ed762c1` | 導入4ページ・第1〜8章 | 目次順レビューの最初の区切り |
| 2026-11-01 | `d80cc68` | 第9〜14章 | 第2部の目次順レビュー |
| 2026-11-01 | `f61b100` | 第15〜20章 | 第3部の目次順レビュー |
| 2026-11-01 | `3fd455b` | 第21〜28章 | 第4部の目次順レビュー |
| 2026-11-01 | `3a002f8` | 第29〜34章 | 第5部の目次順レビュー |
| 2026-11-01 | `58e80d2` | 第35・36章、Argent補足 | 本文/補足と検証記録の区切り |
| 2026-11-01 | 作業記録（本レビュー時点で未コミット） | 付録A〜H、進捗/ROADMAP | 各付録の本文・参照/教材/記録を確認。詳細は個別レビュー記録 |

[進捗表](../review-status.md)はSUMMARYの49ページを掲載する。今回で目次順の本文レビューは一巡したが、未照合の主張/書誌、暗号・安全性の証明、Finality境界・HD復旧・Pruning後IBD・複数ワーカー/会計・新規環境再現等は残る。個別結果は付録Gと実験記録、完了条件はリポジトリのROADMAP.mdを参照する。

## H.12 Gitによる変更履歴の追跡

```bash
# 書籍リポジトリで実行
git status --short
git log --oneline --decorate -n 15

# 特定章の履歴
git log --follow -- src/part-05-advanced/chapter-34.md

# 差分確認
git diff -- src/appendices/appendix-h.md

# 以下はレビュー後に必要な場合のコミット例（今回自動実行していない）
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
| CLM-003 | ZK検証機能の存在 | K-05/K-04 | 未記入 | 未記入 | Unverified |

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

今回の本文/リンク確認と、全技術の検証・ライセンス全面監査・公開作業は別。チェックは明示した範囲だけを示す。

- [x] 導入4ページ・全36章・Argent補足・付録A〜Hの49ページがSUMMARY順に対応する。
- [x] ローカルmdBook 0.5.4でHTMLビルドが成功する。
- [x] ビルドした53 HTMLのローカルリンク/アンカー検査で不整合0（外部URLの全件確認ではない）。
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
mdbook serve --hostname 127.0.0.1 --port 3000
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

**目次順レビュー：一巡済み。** 本付録は主要書誌/版と追跡管理を確認した範囲。PDF版/hash・全引用節/出典IDの最終照合、全機能/全OSの実行、ライセンス全面監査、ネットワーク有効化等は完了を意味しない。未完了事項は進捗表・ROADMAPと個別記録に残す。
