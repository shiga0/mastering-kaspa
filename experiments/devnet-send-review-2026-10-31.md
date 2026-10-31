# Devnet送金の実測記録

実施日：2026-10-31。Node.js 20.0.0、macOS ARM64、既存kaspad 1.0.1 Devnet。
公式SDK：[v1.0.1リリース](https://github.com/kaspanet/rusty-kaspa/releases/tag/v1.0.1)の `kaspa-wasm32-sdk-v1.0.1.zip`。
アーカイブSHA-256：`70679fe150689eedd588a475ae93f1135cfb99bd7b205dc2661d104d5db564d0`。
Node.jsパッケージの宣言版は1.0.1。WebSocket依存はwebsocket 1.0.34。

`experiments/devnet/send.cjs` を実行し、終了コード0で成功した。送金元・宛先の鍵はランダム生成してプロセスメモリ内だけで使用した。秘密鍵・シードは保存も出力もしていない。

## 観測した結果

- Coinbase成熟期間100 DAAをSDKから読み、成熟した入力を1件選択。
- 資金選択時のvirtual DAA scoreは371。
- 1 KAS（100000000 sompi）を送金。
- TX ID：`cb03f0071807ad4c509411c1342cbb21f9accbd2043fcecb9b80e6f2ff1f3dee`。
- 送信応答に加えて、宛先UTXO、入力の消費、virtual chainの受理IDへの掲載を確認。
- 採掘は実験専用1スレッド・最大1CPUで実行し、終了時に停止した。
- 公開可能な結果のみ `experiments/devnet/send-observed-2026-10-31.json` に保存。

最初の試行はEncoding.Jsonという未定義値により接続が失敗し、鍵生成前に停止した。基準版のEncoding.SerdeJsonへ修正して解消。成功後、版の検査とシグナル時停止処理を追加し、最終スクリプトを再実行して成功した。強制終了シグナルの動作自体は未実測。

## 検証の限界

既存イメージのノードソースとの同一性は未証明。送金の成功は現在のvirtualビューで確認したもので、不可逆性・2ノード収束・再編成耐性の検証ではない。Mempool掲載とブロック包含の独立観測、異常系、手数料・お釣りの独立検算、HD導出・バックアップ復旧は未確認。教材とSDKの照合範囲を章全体のVerifiedへ拡大しない。

最終スクリプトの再実行も終了コード0で成功。公開結果は `experiments/devnet/send-confirmed-2026-10-31.json`。TX IDは `9f1fa707932949b7da967eeb3e8239f6f8eb7bd7fd0330caf9acff5d0f32eb51`。
