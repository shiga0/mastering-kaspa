# 第32章：本文の2-of-2ソースを公式コンパイラーとScript VMで検査

確認日：2026-11-02。

## 固定版と環境

- コンパイラー：Silverscript v1.0.0、`3ed973335b59269293564805cc2c58a14595ec03`。公式タグを取得し、HEADと変更のない作業ツリーを確認。
- Kaspa依存：`a41a333b08848f41bf737b72592e463a6011b8ac`。本書のノード基準v1.0.1とは別。
- macOS Apple Silicon、Rust 1.94.0 / edition 2024、既存C/C++ビルド環境。既定のRust・overrideは変更していない。
- ハーネス：`experiments/silverscript-vm/main.rs`、ソース：`two-of-two.sil`。本文32.10は同じソースをincludeする。
- 公式の`tests/examples_tests.rs`と`tests/common.rs`のAPIの組立てを基にした。ハーネスはISCの著作権・条文を保持。

## 手順

1. 本書・公式ソースの外へ独立したCargoプロジェクトを作成。コンパイラー/ABIを固定checkoutのpath依存、Kaspaを固定Git revisionで参照。
2. ハーネス用Cargo.lockを作成し、全上流依存の版・source・checksumが公式Cargo.lockにあるものと一致することを確認。公式checkout自体は変更しない。
3. 本文のTwoOfTwoVaultへ一時鍵2本のx-only公開鍵を渡し、公式コンパイラーでコンパイル。
4. 合成取引・参照UTXOに対しSIG_HASH_ALLのSchnorr署名を作成。ABIで引数を組み立て、ローカルTxScriptEngineで評価。
5. 収録runnerで`--offline`・`--locked`の再実行を確認。結果に公開鍵・秘密鍵・署名・取引本体は含めない。

初回の依存ビルドは約1分20秒。収録runnerの再実行ではビルド済み成果物を再利用した。これはこの環境での所要時間であり、初回取得・他環境での時間を保証しない。

## 結果

| ケース | 結果 | 層 |
|---|---|---|
| 両方の署名が正常 | 許可 | VM |
| Bの署名引数が空 | 拒否 | ABIの65-byte長検査 |
| Aの署名を両方へ渡す | 拒否 | VM |
| A/B署名を交換 | 拒否 | VM |
| 署名後に出力額を変更 | 拒否 | VM |
| Aの署名を1ビット変更 | 拒否 | VM |

6ケースが期待と一致。VM評価は5件で、ABI拒否1件をVMの試験へ数えない。生成Scriptは168 bytes。機械可読結果とソースhashは[silverscript-vm/observed-2026-11-02.json](silverscript-vm/observed-2026-11-02.json)に保存。

## 途中で修正した点

- 最初の空署名試行はVMでの拒否を期待したが、ABIエンコードで`InvalidLength`となった。該当引数名・期待長65・実長0を検査し、ABIでの拒否として記録するよう修正した。
- ロックファイルの新規解決を試みた際、上流が固定するyank済み依存の再解決が失敗した。公式Cargo.lockを基に既存固定版を保持してハーネスを構成し、その専用lockを収録した。以後は`--locked`で再解決しない。
- 1.94.0にはrustfmtを追加していない。自作ハーネスの書式は既存rustfmt 1.90.0のedition 2024指定で検査した。

## 限界

ローカルVMの`covenants_enabled`は明示的にtrueとした。入力OutpointとUTXOは合成データで、実ノード上の存在・成熟・Mass・全検証規則・ネットワークの有効化は確認しない。Devnet/Mainnetへの送信も行っていない。
文字列IDを数えるRust参照モデルとの全面的な差分テスト、コンパイラーの正しさ・全署名方式・全Scriptの証明ではない。
