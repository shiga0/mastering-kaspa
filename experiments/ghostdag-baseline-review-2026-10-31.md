# GHOSTDAG：第12・26章の基準版再照合

確認日：2026-10-31。

**後続の実行検証：** 同日の `ghostdag-comparison-2026-10-31.md` に24ケースの差分実行結果を記録した。本記録の「今回は未実施」はソース再照合の段階を指す。

既存照合版 `4edcbbccb449f84832afb1ca157904b35f37d006` と本書の基準版v1.0.1 `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` の固定ソースを比較した。ソース読み取りの記録であり、公式実装の実行時差分テストではない。

## 取得元と同一性

既存4ファイルのSHA-256が先の `ghostdag-review-2026-10-31.md` の記録と一致することを確認してから比較した。基準版のハッシュとファイル単位の一致判定は `ghostdag-baseline-sources-2026-10-31.json` に保存する。

- [consensus/src/processes/ghostdag/ordering.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/ghostdag/ordering.rs) — SHA-256 `8b6716afbaddd2becd1910e8d80108747daaa97970669cb50c22ff0fca0dc47f`
- [consensus/src/processes/ghostdag/protocol.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/ghostdag/protocol.rs) — SHA-256 `97fe5e1db8afae9d6a3074cf6b6e56ed72e453db9edcbf9b539a4469cc2918ee`
- [consensus/src/model/stores/ghostdag.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/model/stores/ghostdag.rs) — SHA-256 `c23fa656f30227291ee75e17cb0634c71a7a02dcea17fdba435ceef61cea1e18`
- [consensus/src/processes/ghostdag/mergeset.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/ghostdag/mergeset.rs) — SHA-256 `8a86df5cf864a17c963be9cd1cf08961c040ea28e4684b238506ec0a51671122`
- [consensus/core/src/config/params.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/params.rs) — SHA-256 `9b9a88195e530888de6543004a241c136c142a0bf606e27de4c527901b17ffea`
- [consensus/core/src/config/constants.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/constants.rs) — SHA-256 `c24c359579264110b4ff89a11fb0dbe8002a22478c68b22f2c0f2c6687d5c653`

## 維持された主要規則

| 項目 | 基準版で読み取った処理 |
|---|---|
| 親選択 | `SortableBlock` のblue work、同値時hashで最大の親を選ぶ |
| merge set | selected parentを除く候補を集め、blue work、hashの昇順で処理する |
| 初期blue | `new_with_selected_parent` がselected parentをmergeset_bluesの先頭に追加 |
| blue判定 | 候補のblue anticoneだけでなく、影響を受ける既存blueの上限も確認する |
| blue score | selected parentのscoreにmergeset_bluesの要素数を加える |
| blue work | selected parentのworkに各merge-set blueの `max(calc_work(bits), level_work)` を加える |
| 境界 | Genesis/ORIGINの初期化と通常ブロックの計算は別経路 |

ordering.rsとmergeset.rsはバイト単位で同一。ストアの差はimportの並べ替えで、照合した初期化・加算処理は同じだった。protocol.rsの二段のifへの変更は、候補とchain blockの到達可能性を調べる条件を保っている。全コードの振る舞いの同値性を証明したものではない。

## 重要な差：kはフォーク条件に依存する

旧版の通常 `GhostdagManager` は固定の `KType` を持つ。基準版では `ForkedParam<KType>` を持ち、通常ブロックの分類前に次を評価する。

```rust
let k = self.k.get(self.headers_store.get_daa_score(selected_parent).unwrap());
```

参照するのは**selected parentのDAA score**であり、新ブロック自身の値やblue scoreではない。`ForkedParam::get` はactivation以上でpost、それより前ではpreを返す。`with_level` で構築する上位レベル用のmanagerは `new_const(k)` で定数にしているため、通常managerと混同しない。

基準版 `DEVNET_PARAMS` はCrescendo activationがneverで、priorの `LEGACY_DEFAULT_GHOSTDAG_K = 18` を使用する。ここで扱う値は無変更の基準ソースのDevnet既定値であり、CLIによる上書きや別ネットワーク、現在のMainnetを確認した記録ではない。

protocol.rsの注記はkが増加することを前提に既存blueのanticoneサイズを検査している。任意のkの減少までそのまま安全に扱えるという説明にはしない。

## 次の実行時差分検証の条件

第12章の `examples/dag-relations` はPast・Anticone・k-cluster判定だけを実装し、親選択・blue/red分類・blue score/workは実装していない。この教材だけを公式GHOSTDAGと比較しても、分類の差分テストにはならない。

別のハーネスで公式の固定コミットをビルドし、同じ親・ハッシュ・bits・DAA score・k条件を入力したうえで、少なくとも以下を比較する必要がある。

1. 直鎖と並行した親：selected parent、merge-set blue/red、score/work。
2. 同一blue work：ハッシュによる親選択と候補順、親配列順を変えた結果。
3. k境界：候補自身のanticone超過と、既存blueの上限到達を別ケースで確認。
4. 異なるbits：単なるブロック数とworkの区別、selected parentを含む加算対象。
5. activation前後：selected parentのDAA scoreで切り替わること。Devnet既定値とは別の明示的な試験設定として扱う。
6. Genesis・ORIGIN・上位レベル：通常ブロックの式を無条件に当てはめない。

今回はこれらの実行時差分テストを行っていない。原論文の証明、最終的な取引処理順、永続化の原子性、並行処理、現行Mainnetとの一致もこの照合範囲外。C12/C26は部分進捗として残す。

## 今回の確認結果

- `cargo test --manifest-path examples/dag-relations/Cargo.toml --locked --offline`：教材5テスト成功。公式GHOSTDAGのテストではない。
- `python3 scripts/check-book-examples.py`：2教材の本文とRustソースが一致。
- `mdbook build`：成功。生成HTML 53ページの内部リンク・アンカーに不備なし。
- `git diff --check`：成功。公開候補の秘密情報・ローカルホームパスのパターン検査で検出なし。取得した公式ソース本体やローカル作業ディレクトリは追加していない。
