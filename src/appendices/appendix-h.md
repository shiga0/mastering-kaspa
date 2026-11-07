# Appendix H — 参考文献・変更履歴

> **Mastering Kaspa** / 付録H  
> 本付録は書籍全体の出典管理、実装バージョンの固定、変更履歴、レビュー記録を扱う。

対象版は[本書の対象バージョン](../baseline.md)を参照してください。

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

## H.5 KIP文献の参照先

KIPの番号・表題・索引上の状態・本書の対応章は、[付録F「KIPの対応表」](appendix-f.md#f6-kipの対応表公開索引の例)にまとめている。第30章・付録Fと同じ固定コミット `e4ae2332117b5cb68bd6188e065ef885b6d17939` の[KIP索引](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/README.md)を参照軸とし、この付録では状態一覧を重複して管理しない。

引用する際は、索引の要旨だけでなく各KIP本文の表題・該当節・参照コミットを記録する。実装箇所や仕様の適用範囲は対象リリースで確認し、索引上の状態とネットワークでの有効化を分ける。

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
| 第31〜32章・[Argent補足](../part-05-advanced/argent.md) | KIP-0010/0017/0020、Silverscript・Argentの固定版ソース | Covenant機能、言語・コンパイラー・実行環境の対応範囲 |
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

## H.10 本書の変更履歴（確認した区切り）

目次順レビューの区切りとなった実在コミットを示す。書籍版のタグとは区別する。

| 日付 | 参照 | 対象 | 内容 |
|---|---|---|---|
| 2026-10-31 | `ed762c1` | 導入4ページ・第1〜8章 | 目次順レビューの最初の区切り |
| 2026-11-01 | `d80cc68` | 第9〜14章 | 第2部の目次順レビュー |
| 2026-11-01 | `f61b100` | 第15〜20章 | 第3部の目次順レビュー |
| 2026-11-01 | `3fd455b` | 第21〜28章 | 第4部の目次順レビュー |
| 2026-11-01 | `3a002f8` | 第29〜34章 | 第5部の目次順レビュー |
| 2026-11-01 | `58e80d2` | 第35・36章、Argent補足 | 本文/補足と検証記録の区切り |
| 2026-11-01 | `c86e88e` | 付録A〜H、進捗/ROADMAP | 各付録の本文・参照/教材/記録を確認。詳細は個別レビュー記録 |

目次順の49ページの本文レビューは一巡した。その後のFinality・HD復旧等の個別結果と確認範囲は[進捗表](../review-status.md)と[付録G](appendix-g.md)を参照する。未検証部分は既知の制限として保持する。

執筆者向けの管理表・レビューIssue・リリースノートの雛形、執筆/ビルド手順は、リポジトリの[執筆・技術レビューの管理資料](https://github.com/shiga0/mastering-kaspa/blob/main/docs/editorial-guide.md)へ移した。

## H.11 まとめ

<details class="summary-details">
<summary>まとめを見る</summary>

本書の価値は、技術を説明することだけではなく、**説明を検証可能な知識に変えること**にある。

- 論文は「なぜその設計か」を示す。
- KIPは「何を変えるか」を示す。
- コードは「その版で何が実装されたか」を示す。
- 実験は「何を確認したか」を示す。
- 変更履歴は「いつ、なぜ記述を更新したか」を示す。

『Mastering Kaspa』を読み続けられる技術書にするために、参考文献と変更履歴を本文と同じくらい重要な成果物として扱う。

</details>

<details class="verification-details">
<summary>検証状況を見る</summary>

[対象バージョン](../baseline.md)・[このページのレビュー記録](../review-status.md#review-appendix-h)

**目次順レビュー：一巡済み。** 本付録は主要書誌/版と追跡管理を確認した範囲。PDF版/hash・全引用節/出典IDの最終照合、全機能/全OSの実行、ライセンス全面監査、ネットワーク有効化等は完了を意味しない。未完了事項は進捗表・ROADMAPと個別記録に残す。


</details>
