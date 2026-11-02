# 本文の2-of-2コントラクトを公式コンパイラーとScript VMで検査する

第32章32.15.1の実習です。本文の`TwoOfTwoVault`を公式コンパイラーでコンパイルし、
生成したScriptに対して実際のSchnorr署名を作り、Kaspaの`TxScriptEngine`で評価します。
ノードやDevnetへの送信は行いません。

## 固定版と準備

- SilverScript v1.0.0：`3ed973335b59269293564805cc2c58a14595ec03`
- コンパイラーが参照するKaspa：`a41a333b08848f41bf737b72592e463a6011b8ac`
- Rust 1.94.0 / edition 2024、Python 3.11、C/C++ビルド環境。

本書のRust 1.90.0・ノードv1.0.1とは別の検証環境です。
Rustのdefault/overrideを変更せず、`+1.94.0`で指定します。

```bash
rustup toolchain install 1.94.0 --profile minimal
git clone --branch v1.0.0 --depth 1 https://github.com/kaspanet/silverscript.git /tmp/mastering-kaspa-silverscript-v1.0.0
git -C /tmp/mastering-kaspa-silverscript-v1.0.0 rev-parse HEAD
```

HEADが上の固定コミットに一致することを確認します。取得先が既にある場合は再cloneせず、既存の版と作業ツリーを確認します。
依存の初回取得にはネットワーク接続、初回ビルドには数分以上と十分なディスク空き容量が必要です。

## 実行

本書リポジトリのルートで実行します。

```bash
python3 experiments/silverscript-vm/run.py \
  --source /tmp/mastering-kaspa-silverscript-v1.0.0 \
  --work-dir /tmp/mastering-kaspa-silverscript-vm-build
```

依存取得済みなら`--offline`を追加できます。
runnerはコミットと作業ツリーに変更がないことを確認し、本書・公式checkoutの外に小さなCargoプロジェクトを作ります。
コンパイラー・ABIは固定checkoutのpath依存、Kaspaは上記のGit revisionを使います。
公式checkoutのソース・Cargo.lockは変更しません。

このハーネス用Cargo.lockを収録し、`cargo +1.94.0 run --locked --jobs 2`で実行します。
上流依存の版とchecksumは公式Cargo.lockと同じものを使っています。
ビルドと実行の期限は30分です。再実行時は指定work-dirのハーネス用ファイルを更新するため、専用の作業ディレクトリを指定してください。

## ケースと結果の意味

| ケース | 期待する結果 |
|---|---|
| `both-valid` | 許可 |
| `missing-second`：2つ目の署名引数が空 | ABIエンコードで拒否。VMへは到達しない |
| `same-signer-twice`：Aの署名を両方へ渡す | 拒否 |
| `swapped-signatures`：A/Bを逆に渡す | 拒否 |
| `changed-output-after-signing`：署名後に出力額を変更 | 拒否 |
| `corrupted-signature`：署名の1ビットを変更 | 拒否 |

署名には`SIG_HASH_ALL`を使います。すべての結果が期待と一致すると、`allChecksPassed: true`と6ケースの結果をJSONで出力します。
鍵は実行ごとにメモリ内で生成し、秘密鍵・署名・生成Script自体は出力しません。

`missing-second`はABI引数を省略する試験ではなく、2つ目へ空byte列を渡す試験です。固定版ABIはsigの65-byte長を検査して拒否します。結果の`stage`で`abi`と`vm`を区別し、VM未到達の結果をVMの署名検証成功とは扱いません。
`same-signer-twice`も、文字列IDの重複検査ではなく、Bの公開鍵に対してAの署名では条件を満たせないことを検査します。

## 検査の境界

入力取引・参照UTXOはメモリ内の合成データです。VMでは`covenants_enabled: true`を明示しています。
VMがScriptを許可しても、その入力が実チェーンに存在すること、成熟・Mass・全コンセンサス規則、ネットワークでの有効化は確認できません。
Mainnetや本書のv1.0.1 Devnetへ、この生成物を送信する手順ではありません。

文字列の承認者を数えるRustの`VaultPolicy`とは、入力・評価する規則が異なります。
参照モデルとVMの全面的な同値性やコンパイラーの正しさの証明ではなく、本文の具体的なコントラクトの6ケースを検査するものです。

## 出典とライセンス

VMと署名の組立ては、固定版の[examples_tests.rs](https://github.com/kaspanet/silverscript/blob/3ed973335b59269293564805cc2c58a14595ec03/silverscript-lang/tests/examples_tests.rs)と[common.rs](https://github.com/kaspanet/silverscript/blob/3ed973335b59269293564805cc2c58a14595ec03/silverscript-lang/tests/common.rs)を基に、本書のソースと6ケースに合わせています。
`main.rs`はISC Licenseで、原著作権・条文をファイル冒頭に保持しています。その他の自作コード・説明文には本リポジトリの区分を適用します。
