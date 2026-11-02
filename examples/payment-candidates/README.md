# SDKの現在UTXOをRustの支払い候補へ変換する

第35章35.11.1の実行例です。公式Node.js/WASM SDK v1.0.1で取得した現在UTXOをJSONへ変換し、Rustの集計モデルへ`Candidate`として取り込みます。
SDK応答をRustへ渡す経路を示すもので、完成したPayment Trackerではありません。

## まず保存済みの実データで試す

リポジトリのルートで実行します。Rust 1.90.0 / edition 2021で確認しています。
JSON変換にserde 1.0.228とserde_json 1.0.150を使い、間接依存はCargo.lockに記録しています。初回の依存取得にはネットワーク接続が必要です。

```bash
cargo run --locked --manifest-path examples/payment-candidates/Cargo.toml -- \
  kaspadev:qpumuen7l8wthtz45p3ftn58pvrs9xlumvkuu2xet8egzkcklqtese6y7tc99 100000000 \
  < examples/payment-candidates/fixtures/devnet-snapshot.json
```

引数は、請求に割り当てたDevnetアドレスと請求額（sompi）です。
この例では採掘実習用のアドレスを、仮の請求の監視先にしています。
実測fixtureでは`candidateOutputCount: 14`、`invoiceStatus: "Observed"`、`eligibleTotalSompi: "0"`になります。
UTXOの合計が請求額を超えても、確認ポリシーを実装していないため`Paid`には進めません。

## 実ノードから取得して試す

[第28章28.6.2](../../src/part-04-development/chapter-28.md#2862-公式nodejswasm-sdk)でSDK取得・SHA-256照合・WebSocket依存導入を行います。
Docker、Node.js 20.0.0、linux/arm64の既存Compose構成を使います。SDK監視サンプルと同じ専用プロジェクト・ボリュームです。
18310番ポートを使う他の実習は先に停止してください。

```bash
export KASPA_SDK_ROOT="$PWD/experiments/devnet/.sdk"
docker compose -p mastering-kaspa-sdk-demo -f experiments/devnet/compose.yaml -f experiments/devnet/mining.yaml up -d node
```

ノードのwRPC待受を確認したら、現在UTXOを出力します。接続失敗時はログを確認し、次へ進まず再実行します。

```bash
node examples/payment-candidates/export.cjs > /tmp/mastering-kaspa-candidates.json
```

`export.cjs`はローカルDevnet v1.0.1・UTXO indexを検査し、実習アドレスだけを照会します。
取得した各エントリのアドレスも一致することを確認し、金額は精度を保つため10進文字列で出力します。
スクリプト全体の期限は15秒です。出力するのは公開UTXO情報で、秘密鍵は不要です。

```bash
cargo run --locked --manifest-path examples/payment-candidates/Cargo.toml -- \
  kaspadev:qpumuen7l8wthtz45p3ftn58pvrs9xlumvkuu2xet8egzkcklqtese6y7tc99 100000000 \
  < /tmp/mastering-kaspa-candidates.json
```

新規ボリュームで未採掘なら、候補0件・`Pending`が正常です。
実通知・採掘から試したい場合は、[SDK監視サンプル](../sdk-monitor/README.md)を先に実行して採掘・停止まで済ませ、この手順でノードだけを再起動します。
採掘済みボリュームの件数は実行タイミングで変わります。

成功・失敗にかかわらず、最後に実習を停止します。ボリュームは保持します。

```bash
docker compose -p mastering-kaspa-sdk-demo -f experiments/devnet/compose.yaml -f experiments/devnet/mining.yaml --profile mining stop -t 2 miner node
```

## 変換と判定の境界

- `export.cjs`：公式SDKの`entry.outpoint`・`entry.amount`をJSONの`outputs`へ変換。
- `src/main.rs`：ネットワーク・請求に割り当てたアドレス・TX IDの16進形式・出力番号・金額を検査して`Observation`を作成。金額は正のu64範囲、入力JSONは最大1 MiB。
- `src/lib.rs`：本文35.11・35.12と同期する集計モデル。候補だけの入力では`Observed`になり、集計対象額は0。

同じOutpoint・同じ金額が重複しても候補は増えず、同じOutpointの異なる金額は拒否します。
JSONから`state`や`revision`を受け取らず、すべて`Candidate`・revision 0で取り込みます。
毎回新しいメモリ内の請求へ取り込む初回インポートの例で、継続監視の版管理・DB・通知処理は実装していません。

請求先との対応は別途与えたアドレスとの一致に基づきます。JSON自体の真正性、ノード応答の暗号学的証明、請求専用アドレスの発行・履歴は検証しません。
採掘報酬の成熟・支出履歴・採用取消・業務上の確定を判断せず、UTXOの消失をそのまま`Retracted`にもしません。
実アプリでは確認ポリシーと履歴照合を追加してから`Eligible`へ遷移させます。

## 検査

```bash
cargo test --locked --manifest-path examples/payment-candidates/Cargo.toml
cargo fmt --manifest-path examples/payment-candidates/Cargo.toml -- --check
cargo clippy --locked --manifest-path examples/payment-candidates/Cargo.toml --all-targets -- -D warnings
python3 scripts/check-book-examples.py
```

元の集計モデル12テストと、変換・不正入力・実SDKの保存済み応答の7テストを実行します。
実ノードでの一往復はCIに含めず、確認条件は[実測記録](../../experiments/payment-candidates-review-2026-11-02.md)に残しています。
