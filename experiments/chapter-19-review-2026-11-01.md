# SUMMARY順レビュー 23：第19章「マイニング経済とネットワークセキュリティ」

確認日：2026-11-01。基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。本文全体・式・例・教材・参考資料の導線を確認した。市場実測・採算の推奨・攻撃成功確率の検証ではない。

## 実装の照合と本文の整理

- expected_coinbase_transactionの主要分岐を確認。blueかつnon-DAAでない報酬は元のScriptへ、対象redの報酬は取り込む側のminer dataのScriptへ集約する。redかつnon-DAAではCrescendo前は除外、有効化後は手数料分を加算する。正の報酬に出力を作り、現在ブロックのsubsidyをpayloadにも記す。
- calc_block_subsidyは初期フェーズと月次テーブルを分ける。subsidy_monthはDAA scoreをBPSで換算し、Crescendoをまたぐ区間を分ける。1か月は2629800秒相当で、BPSに応じたテーブルは切り上げ除算。カレンダー月や今日の日付だけで決める説明とは区別した。
- 平均有効報酬Rはプール手数料控除前と明記。すでに控除済みの入金を元に再度手数料を掛けないよう整理した。ハッシュシェアへの比例と一律のプール手数料はモデル上の仮定である。
- 営業寄与と会計上の利益・実際の入出金を区別。設備投資・減価償却・税額・売却時点は関数の入力に含まない。
- Bを固定した感度分析はDAA追従後の定常近似とし、移行過程や確率的な収益を予測するものではないと説明。
- HHIの百分率版と0〜1版、母集団・集計主体、未知の主体のまとめ方を補足。HHIを攻撃確率やセキュリティの閾値と同一視しない。
- 手数料収入の相対比率が増す説明には条件を付け、将来の手数料額や円建て収入を保証しないようにした。
- 固定版取得・探索、bin/edition、リポジトリ外の作業場所、次章・研究のリンクを整理した。

## 一次資料の確認

HHIの定義は [米国司法省の解説](https://www.justice.gov/atr/herfindahl-hirschman-index) と照合した。合併審査の閾値をマイニングの安全判定へ転用せず、法的助言も行っていない。

Selfish miningの書誌とBitcoinを対象とする研究であることは [Eyal/Sirerの原論文概要](https://arxiv.org/abs/1311.0243) を確認した。Kaspaの攻撃閾値・証明として扱わない。研究の証明全体や他の全参考文献を今回読み直したものではない。

固定ソースの識別情報：

| ファイル | SHA-256 |
|---|---|
| [consensus/src/processes/coinbase.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/coinbase.rs) | `4d9d9a975f4b7e688af535f4764ac9954177cfc543059da07ba6c31bb78a13ea` |
| [consensus/core/src/config/params.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/params.rs) | `9b9a88195e530888de6543004a241c136c142a0bf606e27de4c527901b17ffea` |
| [consensus/core/src/coinbase.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/coinbase.rs) | `e889e20e05851bfcfdeb4958f77ea76fa19dc06c28489190622841456c7ec063` |

coinbase.rsのexpected_coinbase_transaction、calc_block_subsidy、subsidy_monthと初期化、params.rsの関連パラメータ、coreの報酬データ型を確認した。ファイル全体・全分岐・全テーブルの検証完了を表さない。

## 教材の修正

以前は入力が有限でも、積や合計がオーバーフローして無限大・NaNを返せた。中間のgross_reward_kasと、期待KAS・売上・電力費・営業寄与が有限かを確認してから返すよう修正した。

極端な入力では代数的な式変形で最終結果が有限になり得ても、現在の演算順で中間値が範囲外なら拒否する。f64の丸めや極小値のアンダーフローは残り、正確な送金・会計計算ではない。マイナー比率・出力値等に現在の市場データを使ったものではない。

## 実行した検査

本文のRustブロックを一時Cargo binプロジェクトへ抽出し、rustfmtの結果を本文へ反映。Rust/Cargo 1.90.0、macOS ARM64、edition 2021、外部依存なし。cargo test --offline、cargo fmt --check、cargo clippy --offline --all-targets -- -D warnings、cargo run --offlineを実行した。

最初の実行は7成功・1失敗。追加した境界テストで電力費を72.0との完全一致で比較し、72.00000000000001という丸め差で失敗した。既存の数値例の検査と同じ絶対誤差1e-10未満へ修正後、8テスト成功。途中の失敗を成功件数に含めない。

8テストの範囲：

1. 例の電力費72円に許容誤差内で一致する。
2. 価格を2倍にすると売上は2倍、電力費は同じ。
3. ネットワークハッシュレート0を拒否する。
4. マイナーハッシュレート0なら売上0。
5. 例の期待KAS・売上・営業寄与が本文の数値に許容誤差内で一致する。
6. NaN・正負の無限大、負の費用・手数料、手数料率1超、ネットワーク全体を超えるマイナーハッシュレートを拒否する。
7. 有限の入力から総報酬・売上・電力費・営業寄与が範囲外になる5ケースを拒否。手数料率1でも中間のオーバーフローは拒否する。
8. 手数料率1、稼働ゼロ、手数料率0・シェア1の境界を確認する。

5〜8が追加テスト。書式・Clippy・mainも成功。mainの出力：

```text
DailyEstimate {
    expected_kas: 0.42768,
    revenue_jpy: 4.2768,
    electricity_jpy: 72.00000000000001,
    operating_margin_jpy: -67.72320000000002,
}
```

これは仮定例の実行であり、実際の収入や採算の予測ではない。cargo newから新規環境で手入力した試験とも区別する。

## 限界と次の対象

報酬の全適格性・成熟・全発行テーブル・フォーク境界の公式差分、攻撃成功確率、現行Mainnet、市場価格・ASIC・電力契約・プールの実測は未検証。今回は鍵生成・採掘・ノード起動・署名・送金を行っていない。

新しいexamplesフォルダー・生成物は公開物へ追加せず、CI同期対象も変更していない。

次はSUMMARYの24ページ目、第20章 `src/part-03-transactions/chapter-20.md`。

## 統合・公開物の確認

既存の本文とRustソースの同期対象2件が一致。mdbook build成功、生成HTML53ページの内部ファイル・アンカーのリンク切れ0件。SUMMARYとreview-statusの49項目は名称・順序一致。git diff --checkも成功した。

追跡済み・追加候補169ファイルへの既存の秘密情報パターン検査は検出0件。gitleaks/trufflehogは未導入であり、秘密情報が一切ないとの保証ではない。新しいexamplesフォルダー、生成物、秘密鍵、個人パスは追加していない。
