# SUMMARY順レビュー 46：付録E「プロトコルパラメータ一覧」

確認日：2026-11-01。全文を通読し、基準版v1.0.1 / fcd9c28f9b211b9872a0c90a8c94f79466ffb063の主要設定・利用経路と独自教材を確認。現在稼働するMainnetの設定/有効化状態や全合意規則を検証したものではない。

## 旧稿と基準版の差

旧稿はコミットを固定しないmasterの転記だったため、本書の基準版にはないBlockrateParams/blockrate、block_mass_limits、bps_historyや定数名が含まれていた。別版の構造/値を基準版へ混ぜず、固定ソースへの参照と表へ変更した。

- 基準版はCrescendoParams / CRESCENDOとBps<10>、prior_*およびForkedParamの構造。前後値はgetへ渡すDAA scoreで選ぶ。呼出箇所のスコアを確認する必要がある。
- Mainnet前後の目標1000/100ms、k18/124、親数10/16、merge set180/248、finality86400/432000、pruning185798/1080000、maturity100/1000等を転記。
- prior入力/出力数やScript長は1,000,000,000、CRESCENDOは各1000/10,000 byte。他のmass/Script engine/通信制限を無視して巨大取引を受理できるとはしない。
- Mainnet activation DAAは110165000。Testnet-10は88657000、Simnetはalways、Devnetはneverという設定を読み取った。本書DevnetにCRESCENDOの値があるだけで10 BPSが適用されるわけではない。
- 定数の正しい名前はNEW_DIFFICULTY_WINDOW_DURATION、NEW_FINALITY_DURATION、NEW_PRUNING_DURATION、LEGACY_COINBASE_MATURITY等。maturityではDAA scoreに対する検証も確認し、単なる秒としない。
- Params::max_block_mass=500000。Crescendo後のbody側チェックはcompute/transient/storageを別々に集計し、それぞれ500000と比較する。旧稿のtransient 1000000を修正。前はcompute集計。storage commitmentと文脈検証は別段階であり、mass全規則の実行試験ではない。
- bps_history()ではなくCoinbaseManagerの前後BPS/activationを用いる補助金実装を案内。現在報酬の金額を掲げない。
- 基準版にdocs/override-params.mdはないのでリンクをBps定義/引数定義へ置換。kaspad helpの--override-dag-params-fileはDevnet用と明記される。上書きの全項目/実行やSimnetでの可否を確認したものではない。
- 固定タグ/SHAを調査用ディレクトリで確認する手順へ修正。既存コピーのresetやノード操作はしていない。

## 導出算術

固定ソースの式をPython整数算術で計算し、以下を確認。公式Bps関数/Params全体をコンパイルして実行比較した結果ではない。

| 10 BPS側 | 値 |
|---|---:|
| k / max parents / merge set | 124 / 16 / 248 |
| Median sample rate/window | 100 / 27 |
| Difficulty sample rate/window | 40 / 661 |
| DAA窓期待時間 | 100×40×661=2644000 ms |
| merge / finality depth | 36000 / 432000 |
| pruning安全性下限 | 432000+2×36000+4×248×124+2×124+2=627258 |
| pruning採用値 | max(627258,1080000)=1080000 |
| maturity | 10×100=1000 |

DIFFICULTY_SAMPLED_WINDOW_SIZEはceil(2641/4)=661なので、設計期間2641秒と期待窓2644秒を同一視しない。これらの期待時間は実ブロックの待ち時間保証ではない。

定数10件は手元constants.rsの宣言を抽出して比較：NETWORK_DELAY_BOUND=5、TIMESTAMP_DEVIATION_TOLERANCE=132、PAST_MEDIAN_TIME_SAMPLE_INTERVAL=10、NEW_DIFFICULTY_WINDOW_DURATION=2641、DIFFICULTY_WINDOW_SAMPLE_INTERVAL=4、NEW_FINALITY_DURATION=43200、NEW_PRUNING_DURATION=108000、MERGE_DEPTH_DURATION=3600、LEGACY_COINBASE_MATURITY=100、PRUNING_PROOF_M=1000。

## 資料と限界

以前取得した固定版の手元ソースarchiveを使用。新規clone/HEADの取得はしていない。固定GitHub params.rsのWeb取得はcache missだったため、今回オンラインで再取得できた資料とは扱わない。本文の固定ソースURLのパスが手元に存在することを検査した。

| 基準版の参照資料 | SHA-256 |
|---|---|
| consensus/core/src/config/params.rs | `9b9a88195e530888de6543004a241c136c142a0bf606e27de4c527901b17ffea` |
| consensus/core/src/config/constants.rs | `c24c359579264110b4ff89a11fb0dbe8002a22478c68b22f2c0f2c6687d5c653` |
| consensus/core/src/config/bps.rs | `1f94cc759a8427160494c50178d5b36848008b339a5f480badf1001588a14fab` |
| consensus/core/src/mass/mod.rs | `e630fa4f222229e67797df0b8d59ddee8627070a90d8effa6e2b1d10fa35e60e` |
| consensus/src/pipeline/body_processor/body_validation_in_isolation.rs | `a292d8eaa6a5748dceba7f93a20b0a93cbe0469d915b4e6e8df689ca0755110a` |
| consensus/src/processes/coinbase.rs | `4d9d9a975f4b7e688af535f4764ac9954177cfc543059da07ba6c31bb78a13ea` |
| consensus/src/processes/transaction_validator/tx_validation_in_utxo_context.rs | `43c445db82fbd046cd2cbcdb90646ea84d5df732985163483ff7f8a004b25ce7` |
| kaspad/src/args.rs | `b094990866f3c4ab0e242d861251e993eaf96314b3e2e638746ed98400aa3b5e` |

Mass算出全経路、fork境界・KIP/全履歴、全ネットワークの全値、実際の上書き/設定不一致の挙動は未検証。現在のMainnet情報・RPC値を取得したり、Devnet起動/採掘/送金を再実行したりしていない。

## Rust教材

本文の単位換算例を一時Cargo libへ抽出し、既存Rust/Cargo 1.90.0 / edition 2021、外部依存なしで確認。struct/field/methodを公開APIとして使える形へ整え、dead_code警告を抑制属性で隠さず解消した。

```bash
cargo test --offline
cargo fmt --all -- --check
cargo clippy --offline --all-targets -- -D warnings
```

4テストが成功（10 BPS/1 BPS、0/割切れない値の拒否、u64掛算overflow）。doc-testは0件。整形後ソースと掲載コードが一致。新規toolchain導入はしていない。

E.21の定数2つは避けたい設計を示す断片で、実行可能な独立プログラムとして検証したものではない。新しいexamplesやCI検査を追加していない。

## 統合・公開確認

- mdBook 0.5.4のビルド成功（警告なし）。53 HTMLのローカルリンク/アンカー不整合0。
- SUMMARY49件と主表、前段1〜47番の順序が一致。46番の付録Eを今回確認、47番の付録Fを次の対象とした。
- scripts/check-book-examples.pyで既存2教材の本文/実装一致。
- git diff --check成功。
- 追跡対象/非ignore候補192ファイルの既知token prefix・秘密鍵・credential URL・私有home path等のパターン検出0。gitleaks/trufflehogは未導入で、未知の機密やGit全履歴の完全監査ではない。
- 既存付録A〜Dの未コミット差分を保持。追加は本記録で、秘密鍵・生ログ・DB・バイナリー・私有パス・新しいexamplesは追加していない。commit/push/実ノード操作はしていない。
