# 公式SDKの現在UTXOからRustの入金候補への変換

確認日：2026-11-02。

## 実装と環境

- `examples/payment-candidates/export.cjs`：公式v1.0.1 Node.js/WASM SDKでローカルDevnetから現在UTXOを取得し、JSONへ変換。
- `examples/payment-candidates/src/main.rs`：JSONの入力検査と、集計モデルへの初回Candidate取り込み。
- `examples/payment-candidates/src/lib.rs`：第35章35.11・35.12と同期するドメインモデルとtrait。
- Node.js 20.0.0、`websocket@1.0.34`、Docker Engine 27.4.0、macOS Apple Silicon / linux/arm64。
- Rust 1.90.0 / edition 2021、serde 1.0.228、serde_json 1.0.150。間接依存は収録Cargo.lockに固定。
- ノードは既存SDK監視実習の専用Composeプロジェクト`mastering-kaspa-sdk-demo`を使用。イメージ・引数は既存`compose.yaml`・`mining.yaml`を維持。

## 実ノード検査

1. 採掘済みボリュームの専用ノードを起動。採掘は新たに開始せず、接続可能になるまで待機。
2. `export.cjs`でDevnet v1.0.1・UTXO indexを検査し、実習アドレスの現在UTXOを取得。
3. JSONをRust CLIの標準入力へ渡す。請求に割り当てたアドレスを別引数で指定し、請求額は100000000 sompiとした。
4. 14件の候補・Observed・集計対象額0・終了コード0を確認。
5. 終了処理で専用node・minerを停止。ボリュームは保持。

実出力：

```json
{"candidateOutputCount":14,"eligibleTotalSompi":"0","invoiceStatus":"Observed","network":"devnet"}
```

取得した公開データを`examples/payment-candidates/fixtures/devnet-snapshot.json`へ保存した。fixtureのJSONはSDK生オブジェクトの全フィールドではなく、export.cjsによる変換後の形式である。秘密鍵・シード・認証情報は使用していない。

## ローカルの検査

- `node --check examples/payment-candidates/export.cjs`成功。
- `cargo test --offline --locked`：既存ドメイン12件、変換CLI7件が成功。
- `cargo fmt -- --check`、`cargo clippy --offline --locked --all-targets -- -D warnings`成功。
- 実測fixtureの取り込み、同一出力の重複、異なる金額の重複拒否、空集合、network/請求先不一致、TX ID・出力番号・金額の不正、入力からのstate/revision注入拒否を検査。
- 本文との同期対象に第35章を追加。CLIのテスト・書式・Clippyを既存CIのRust matrixにも追加したが、GitHub上のこの新設定の実行は今回確認していない。

## 範囲と限界

これはSDKとRustモデルを接続する読み取り専用の初回インポートで、完成Payment Tracker・RustネイティブRPCクライアントではない。
請求は毎回メモリ内で新しく作り、revisionは0、stateはCandidateに限定する。DB・継続監視・通知の競合・履歴照合・請求専用アドレスの発行は実装していない。

現在UTXOに存在することを業務上のEligibleへ昇格させない。成熟条件・入金履歴・採用取消・確認ポリシーは別途必要。後のUTXO消失をRetractedへ変換する処理もない。
JSONの設定値・形式を検査するもので、JSONやノード応答の真正性の暗号学的検証ではない。fixtureだけの成功を最新のノード状態の確認とは扱わない。
