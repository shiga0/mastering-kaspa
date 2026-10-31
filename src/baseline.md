# 本書の対象バージョン

基準の決定日・確認日：2026年10月31日。本書の実装照合と実習では、以下のソース・配布物・環境を基準にします。これは実験を再現するための条件であり、最新版や現在のMainnetの仕様を示すものではありません。

## 基準ソースとネットワーク

| 項目 | 固定した基準 | 確認範囲 |
|---|---|---|
| ノードの参照ソース | `kaspanet/rusty-kaspa` v1.0.1 | コミット `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` を参照 |
| 実ノードの実習 | Devnet | Mainnetの実資産・秘密鍵は使いません |
| 公式workspaceの宣言版 | 1.0.1、edition 2021 | 各クレートの版をworkspaceから継承する構成を確認 |
| 公式workspaceのRust要件 | `rust-version = 1.82.0` | 宣言されている最低版。Rust 1.82.0自体でのビルドは未確認 |

根拠は[固定コミットのCargo.toml](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/Cargo.toml)です。公式ライブラリの関数比較には、このソースをRust 1.90.0でビルドして使いました。`kaspad`実行ファイル全体をソースからビルドしたという意味ではありません。

コンセンサス関数の比較には、小さな合成DAGや明示した窓・パラメータも使います。これらは関数単位の試験条件であり、Devnetの既定設定をそのまま実行した結果とは分けて扱います。

## SDKとRPCクライアント

| 対象 | 採用する版・経路 | 確認範囲 |
|---|---|---|
| Rust RPC/ウォレットSDK | 基準コミット内の `kaspa-wrpc-client`、`kaspa-rpc-core`、`kaspa-wallet-core` 等 | workspaceの宣言版は1.0.1。本書のRust製RPCアダプターによる送金実習は未実装・未実行 |
| Node.js/WASM SDK | 公式v1.0.1リリースのNode.js用配布物。パッケージ名 `kaspa-wasm`、宣言版1.0.1 | Devnetで単一署名送金・virtual受理・UTXO通知等を実測。ソースから再ビルドした配布物との一致は未確認 |
| PythonのwRPC検査 | Python 3.11、`websockets==14.1` | 手書きのJSONクライアントとして実行確認。公式Rust SDKを使う経路ではありません |

Node.js用SDKは[公式v1.0.1リリース](https://github.com/kaspanet/rusty-kaspa/releases/tag/v1.0.1)の `kaspa-wasm32-sdk-v1.0.1.zip` を使います。実行環境はNode.js 20.0.0、WebSocket依存は `websocket@1.0.34` で確認しました。npmの推移依存まで固定する作業は未完了です。HDウォレットのバックアップ復旧も、単一署名送金とは別の未検証事項です。

SDK導入の手順は[第28章](part-04-development/chapter-28.md)、実測条件はリポジトリの `experiments/devnet-send-review-2026-10-31.md` と各後続記録を参照してください。

## 固定アーカイブと依存の扱い

配布物は版名に加えて、次のSHA-256で識別します。

| 配布物 | SHA-256 |
|---|---|
| [基準コミットのソースアーカイブ](https://codeload.github.com/kaspanet/rusty-kaspa/tar.gz/fcd9c28f9b211b9872a0c90a8c94f79466ffb063) | `4585123204609640af4f7482ea8851ced912c2bfae5f36955c808c14210fbe2a` |
| `kaspa-wasm32-sdk-v1.0.1.zip` | `70679fe150689eedd588a475ae93f1135cfb99bd7b205dc2661d104d5db564d0` |

公式ソースでのGHOSTDAG・DAA比較は、固定アーカイブを検査して展開し、元のCargo.lockを変更せず `--locked` で実行します。このCargo.lockのSHA-256は次のとおりです。

```text
14108c8c41a84a1a9c8cd8648238047a0620ca390058f9d8a176fefc78beeabf
```

再実行手順と入力は `experiments/ghostdag/`、`experiments/daa/`、`experiments/daa-window/`、`experiments/daa-transition/` に収録しています。実験用のソース・ビルド成果物・SDK配布物を、この公開リポジトリへ追加する必要はありません。

RustのGit依存を追加する場合は、同じ完全なrevisionを指定してCargo.lockも保存します。公式workspaceの宣言版が1.0.1であることだけから、レジストリで同じ名前・版のクレートを取得できるとは判断しません。

## 教材・ツールと動作確認環境

| 項目 | 記録した版・条件 | 確認範囲 |
|---|---|---|
| Rust教材 | Rust 1.90.0、edition 2021 | `examples/dag-relations/` と `examples/minimal-node/` でテスト・Clippy・書式を確認 |
| mdBook | 0.5.4 | ローカルでHTMLビルドを確認 |
| DevnetのDocker環境 | macOS Apple Silicon、Docker Desktop 4.37.2 / Engine 27.4.0 | ノード・マイナーを `linux/arm64` 指定で実行。Linuxホスト・amd64は未確認 |
| Python実習・比較スクリプト | Python 3.11 | Devnet検査と公式関数比較の実行に使用。Devnet用の `websockets==14.1` は `experiments/devnet/requirements.txt` で固定 |
| 公式ライブラリ比較のネイティブビルド環境 | Apple clang 17.0.0 | C/C++依存を含む比較実習の確認環境。各比較ディレクトリのREADMEに前提を記載 |

CIのRust検査ジョブにはRust 1.90.0、書籍ビルドにはmdBook 0.5.4を指定しています。一方、書籍ビルドジョブでmdBookをインストールするRustと、Pythonの実行版はrunnerの環境を使い、同じ版へ固定していません。CI設定があることと、GitHub上の実ジョブの成功を確認したことは別です。GitHub上の実行・Pages公開は未確認です。

このページの版は実測時の記録です。読者の環境での再現性は、対象OS・CPU・依存関係を合わせて確認してください。

## 実測したDockerイメージ

| 用途 | 固定参照 |
|---|---|
| ノード | `kaspanet/rusty-kaspad@sha256:5bc3195c2cd2d171885781ed55ead9485d93c7bd7b784d46e9e393c96757fe22` |
| CPUマイナー | `kaspanet/cpuminer@sha256:60f78ab2828ab24b249c99210eee5a2825303a5226154260dd021ff26d46748b` |
| P2P分断試験のTCP中継 | `nginx@sha256:b2e814d28359e77bd0aa5fed1939620075e4ffa0eb20423cc557b375bd5c14ad` |

ノードは `1.0.1` と応答し、イメージのビルド変数もv1.0.1を宣言しています。ただし、イメージのsourceラベルはDocker包装用リポジトリを指します。包装用revisionは、ノード本体のソースのコミットではありません。基準ソースとこのバイナリの一致を、再ビルドやビルド証明で確認したものではありません。

したがって、**固定ソースをビルドして行う関数比較**と、**公開Dockerイメージで観測したノードの動作**は別の証拠として記録します。バイナリの申告版が同じでも、一方の成功を他方の検証へ広げません。

イメージの由来と当時の宣言値は `experiments/devnet/image-provenance.json`、構成と再実行手順は `experiments/devnet/README.md` を参照してください。由来のJSONにある第12章の照合コミットは、初期照合時点の記録です。後続の再照合結果は次の節に記載します。

## 既存原稿との違い

第12・26章の主要GHOSTDAG経路は、最初に別コミットを照合していました。

```text
初期の照合：4edcbbccb449f84832afb1ca157904b35f37d006
本書の基準：fcd9c28f9b211b9872a0c90a8c94f79466ffb063（v1.0.1）
```

2026年10月31日に基準版へ再照合し、親選択・候補順・blue判定・score/workの主要規則を確認しました。通常のmanagerで使うkが、固定値からselected parentのDAA scoreに依存する `ForkedParam` へ変わった点も本文へ反映しています。記録は `experiments/ghostdag-baseline-review-2026-10-31.md`、基準版の分類関数と集合モデルの実行比較は `experiments/ghostdag-comparison-2026-10-31.md` を参照してください。両版を実行した動作比較と章全体のレビューは未完了です。

付録Eなどに残る `master` 参照の数値や、発展的な技術の説明を、基準版の機能と自動的に同一視しないでください。基準版以外の話題は、対象コミット・ネットワーク・研究や実装の段階を各ページで示します。確認済みの範囲と残作業は、次の[技術レビューの進捗](review-status.md)で確認できます。

## 基準を変更するとき

このページに加え、実習の依存・アーカイブ・イメージ・各章の照合範囲を更新します。旧版の実測記録は履歴として残し、別の版で得た成功結果へ書き換えません。新しい版で必要な検査を行い、実行結果がある項目だけ検証状態を更新します。
