# SUMMARY順レビュー 02：「本書の対象バージョン」

確認日：2026-10-31。対象は `src/SUMMARY.md` の2ページ目 `src/baseline.md`。前ページの読者案内に続き、版・配布物・依存・実測環境と証拠の対応を確認した。

## 修正した説明

- 基準ソース・実習ネットワーク、SDK/RPC経路、アーカイブ、ツール、Dockerイメージを分け、何を固定した値なのかを整理した。
- 公式workspaceの `rust-version = 1.82.0` は宣言された最低版で、Rust 1.82.0による実ビルド確認ではないことを明記した。Rust 1.90.0での公式ライブラリ検証と、kaspad実行ファイル全体のビルドを区別した。
- Rust SDKの将来の実装予定という表現を、本書のRust RPCアダプターは未実装・未実行という現状へ変更。Node.js/WASM SDK・PythonのJSONクライアントとは別の経路であることを示した。
- 公式ライブラリの比較記録にあるApple clang 17.0.0と、Python実行スクリプトの役割も環境表へ反映した。
- Node.js 20.0.0、`websocket@1.0.34`、SDK配布パッケージ名と版を補った。推移依存の固定とHD復旧は未完了のまま明記した。
- 公式ソースアーカイブ・SDKアーカイブ・公式Cargo.lockのSHA-256を中心ページへ集約。Git revision固定と、元のworkspace/Cargo.lockでビルドする実習の両方を説明した。
- 分断試験に使用するnginx中継イメージのdigestを追加した。
- CIのRust検査ジョブと書籍ビルドジョブを区別した。Rust教材用1.90.0・mdBook 0.5.4の指定があっても、mdBookをインストールするRustとPythonの版はrunner依存であることを確認した。
- CIの設定とGitHub上の実行確認、ソース関数の実行比較とDockerノードの実測、合成パラメータとDevnet既定値を区別した。
- `image-provenance.json` の旧GHOSTDAG照合コミットは当時の記録として保持し、後の再照合結果との対応を案内した。
- 個別の実験件数をこのページで重複管理する部分を減らし、最新の確認範囲はレビュー進捗と各検証記録へつないだ。

## 照合した根拠

| 項目 | 参照したファイル・確認 |
|---|---|
| 基準コミットとworkspace | [固定Cargo.toml](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/Cargo.toml)と、取得済み固定アーカイブ中の同ファイル。version 1.0.1・edition 2021・rust-version 1.82.0 |
| Rust SDKの版継承 | 基準ソース内 `rpc/wrpc/client/Cargo.toml`、`rpc/core/Cargo.toml`、`wallet/core/Cargo.toml` とworkspaceの依存宣言 |
| ソース・lockの同一性 | `experiments/ghostdag/observed-2026-10-31.json` とDAA各集計、および実際に使用したアーカイブ・Cargo.lockのSHA-256再計算 |
| Node.js SDK | [公式v1.0.1リリース](https://github.com/kaspanet/rusty-kaspa/releases/tag/v1.0.1)から取得済みのSDKアーカイブのSHA-256再計算、展開済み `nodejs/kaspa/package.json`、`devnet-send-review-2026-10-31.md` |
| Dockerの版・CPU/OS | `devnet-environment-2026-10-31.md` と既存の送金・分断等の実測記録。今回Dockerを再起動した記録ではない |
| イメージと包装用ソース | `devnet/image-provenance.json` と `compose.yaml`、`mining.yaml`、`two-node.yaml`、`partition.yaml`、`reorg.yaml` |
| 教材とCI | 両教材のCargo.toml、`.github/workflows/deploy.yml`、`devnet/requirements.txt` |
| ソースの実行検証 | GHOSTDAG・DAA各記録と `core-validation-review-2026-10-31.md`。ライブラリを実行した証拠であり、Dockerバイナリの由来の証明ではない |

## 再計算した識別値

- ソースアーカイブ：`4585123204609640af4f7482ea8851ced912c2bfae5f36955c808c14210fbe2a`
- SDKアーカイブ：`70679fe150689eedd588a475ae93f1135cfb99bd7b205dc2661d104d5db564d0`
- 公式Cargo.lock：`14108c8c41a84a1a9c8cd8648238047a0620ca390058f9d8a176fefc78beeabf`

宣言版、取得物の識別値、ローカル実行結果、公開イメージの由来は、それぞれ違う証拠として扱った。Rust SDKのレジストリ配布状況、現在のMainnet、最低Rust版でのビルド、ソースと配布バイナリの完全一致は今回確認していない。既存の実行記録を読み直したことで、実習を新たに実行したことにはしない。

## 次のページ

目次順の次は `src/review-status.md`。今回は同ページの順番レビューの現在地と、基準版ページの確認済み範囲を記録しただけで、進捗ページ全体のレビュー完了とはしない。

## 修正後の確認

アーカイブとCargo.lockのSHA-256、全Composeのimage参照、workspaceと教材のCargo設定、SDKパッケージ情報、Python依存、CIの指定を本文と照合して一致した。mdBookビルド、生成HTML 53ページの内部リンク・アンカー、既存2教材の本文一致、差分の空白検査は成功。公開候補の秘密情報・ローカルホームパスのパターン検査で検出なし。今回の変更は文書だけで、Rust・Dockerの実習は再実行していない。
