# SUMMARY順レビュー 17：第13章「難易度調整アルゴリズム」

確認日：2026-11-01。基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。本文・式・図・教材・実習案内・既存の検証記録を通読した。

## 本文の修正

- PoW値とブロック識別子のハッシュを区別し、理解度チェックの表現も合わせた。
- 確率式を追加プラグインに依存しないtext表記にし、一様モデルのターゲット範囲とポアソン近似の条件を明記した。book.tomlに数式表示の設定は追加していない。
- Bitcoin Mainnetの通常調整の比較であること、時刻差は対象ブロック列の先頭と末尾で測ることを補った。[Bitcoin Developer Guide](https://developer.bitcoin.org/devguide/block_chain.html#proof-of-work)の説明と照合。全ネットワークの例外・現行Bitcoin Coreの全挙動を検証したものではない。
- Fullの通常計算に最大targetでの制限とcompact変換を明示。最小・最大は(timestamp, blue_work, hash)で決めると整理した。平均targetの途中の整数除算は維持した。
- Full/Sampled算術27ケース、Full窓120ケース、Sampled/Dual288ケースにそれぞれ対応する記録を示した。今回の新規実行ではないと明記。
- 期待bitsの算出にローカル時計を使わないことと、未来時刻の許容検査でローカル時計を使うことを区別した。
- 通常の検証順を、申告bitsに対するPoW検査→親関係/GHOSTDAG→期待bits照合として図に反映。Rejectedと永続的なStatusInvalid、HeaderOnlyと検証完了を区別した。
- 責務図・DAA計算モデルは呼び出し順と区別し、次章と固定ソースへの導線を整理。新たにcloneせず第1章の固定版を再利用する。
- 静的な間隔列の演習だけでは振動や回復性を実測したことにならず、難易度変更後の発見を生成するモデルが必要と補った。

## 簡易DAA教材の修正

教材はf64の相対difficultyを更新する独自モデルであり、公式の整数target計算を再現しない。

以前は、最小の正のf64値に更新倍率0.25を掛けるとゼロに丸められ、Ok(0.0)が返り得た。修正後は次の値が有限かつ正であるかを確認し、ゼロもエラーとする。現在の難易度・各間隔の0/負値/NaN/正負の無限大も確認するようテストを拡充した。

極小の正の観測間隔に対する比が無限大となる場合は、clampで上限4倍へ抑える教材の方針を維持した。合計・次の難易度のオーバーフロー、次の難易度のゼロへのアンダーフローは拒否する。公式DAAにもこの倍率制限や浮動小数点の処理があるという意味ではない。

本文の唯一のRustブロックを一時Cargo binプロジェクトへ抽出し、rustfmtの結果を本文へ反映した。Rust/Cargo 1.90.0、macOS ARM64、edition 2021、外部依存なし。

- cargo test --offline：7テスト成功。
- cargo fmt --check：成功。
- cargo clippy --offline --all-targets -- -D warnings：成功。
- cargo run --offline：標準出力が次の2行と完全一致。

```text
fast: Ok(200.0)
slow: Ok(50.0)
```

既存5テストは速い/遅い/目標間隔・倍率上限下限・不正入力。追加2テストは表現範囲外の合計/結果と、極小間隔でも上限4倍を維持することを確認した。本文のcargo newを使って新規環境で手入力した試験とは区別する。

実行したのは本文の教育用コードであり、公式ノード・実際のPoW・採掘・ネットワーク分断・攻撃シナリオではない。一時プロジェクトと生成物を公開リポジトリへ追加せず、CI同期対象も変更していない。

## 再確認した固定ソース

| ソース | SHA-256 |
|---|---|
| [consensus/src/processes/difficulty.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/difficulty.rs) | `749d4ec53a38ee2cd2e9f25a395ddd69ab019d6f813f6cc3ffb3b988395566ff` |
| [consensus/src/pipeline/header_processor/pre_ghostdag_validation.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/header_processor/pre_ghostdag_validation.rs) | `946a6762302a9b8bdc7b8c782d0fc1470e97ad102d9394ee21663addc14fb58f` |
| [consensus/src/pipeline/header_processor/pre_pow_validation.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/header_processor/pre_pow_validation.rs) | `09c3d0201f0a384883b2725c432b441e7e0fd52e3c2b97cf2ef2e72181a2b00b` |

difficulty.rsではFull/Sampledの通常計算、最小時刻の要素除外、最小標本数未満の分岐、最大target制限、DifficultyBlockの比較キーを確認した。pre_ghostdag_validation.rsの単独PoW/未来時刻の検査と、pre_pow_validation.rsの期待bits照合も確認した。

主要経路と旧教材5テストは `core-validation-review-2026-10-31.md`。既存の公式関数比較は `daa-comparison-2026-10-31.md`、`daa-window-comparison-2026-10-31.md`、`daa-transition-comparison-2026-10-31.md` を参照した。今回は再実行・再ビルドしていない。

## 限界と次の対象

ページの通読・教材の実行と、プロトコル全体の検証は別である。全sample rate・全ネットワークの実有効設定・中央値時刻窓・Pruning後IBD・攻撃耐性・KIP/原論文の証明は今回確認していない。過去の合成入力の比較を、Mainnetとの一致や実ネットワークの切り替え成功に一般化しない。

次はSUMMARYの18ページ目、第14章 `src/part-02-consensus/chapter-14.md`。

## 統合・公開物の確認

既存の本文とRustソースの同期対象2件が一致。mdbook build成功、生成HTML53ページの内部ファイル・アンカーのリンク切れ0件。SUMMARYとreview-statusの49項目は名称・順序一致。git diff --checkも成功した。

追跡済み・追加候補163ファイルへの既存の秘密情報パターン検査は検出0件。gitleaks/trufflehogは未導入であり、秘密情報が一切ないとの保証ではない。新しいexamplesフォルダー、生成物、秘密鍵、個人パスは追加していない。
