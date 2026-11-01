# SUMMARY順レビュー 42：付録A「Rust環境・Docker環境の構築」

確認日：2026-11-01。手順・構成図・版の前提を通読し、独自Rust最小例、Compose設定、Dockerの版/ローカルimageの読取を確認した。新規導入や全OS・実Devnet/Pruningの再検証ではない。

## 版と手順の整理

- [rustupのtoolchain指定](https://rust-lang.github.io/rustup/concepts/toolchains.html)を参照し、変化するstableへの一括変更からRust 1.90.0を明示して使う教材へ整理。新規rustup導入と既存環境への版追加を分け、rustup default stableを手順から除いた。
- 初回導入スクリプトは一旦保存して内容を読んでから実行する形へ変更。rustup/Docker/OS依存パッケージのインストール自体は今回行っていない。
- 本書教材はRust 1.90.0 / edition 2021、第32章のSilverscript/Argentは要求版1.94.0と区別。+版指定はrust-toolchain.toml等とは別の版を明示的に選べるため、対象要件を確認する。
- ソース調査はv1.0.1 / fcd9c28f9b211b9872a0c90a8c94f79466ffb063へ固定。MSRV宣言1.82.0と実測版1.90.0を区別し、1.82.0を実行済みとはしない。新しいcloneは今回していない。
- --no-deps metadataは構成列挙であり、依存解決・kaspad全体のビルド成功ではない。ローカルパスを含む生JSONを公開しない。ソースSHAと配布imageの申告版/digestも別の証拠。
- 未定義だったKASPA_IMAGEに本書の固定digestを明示。ルートにComposeファイルがない本書での診断例へ-f experiments/devnet/compose.yamlを加えた。
- 実在しないexamples/wallet・実験フォルダー・rust.ymlの構成図を、実在するdag-relations/minimal-node、experiments/devnet、deploy.yml等へ変更。新規examplesを作っていない。
- CI例は独立したCargoプロジェクト用と明記し、Rust1.90・contents: read・Cargo.lock/lockedを前提にした。本書の実CIがこの付録のコードを自動検査済みとしない。workflow自体は変更/実行していない。

## Rust最小例

本書外の一時プロジェクトへ本文src/lib.rsとHello World用src/main.rsを配置した。Rust/Cargo 1.90.0（既存stable）、edition 2021、外部依存なし。libの掲載コードと実行ソースが一致することを確認。

```bash
cargo run --offline
cargo test --offline
cargo fmt --all -- --check
cargo clippy --offline --all-targets -- -D warnings
```

Hello, world!を表示、libの2テスト成功、fmt/Clippyも終了コード0。binとdocの0テストを成功件数へ加えていない。通常加算2+3とu64::MAX+1の表現範囲超えという算術の最小動作確認であり、状態遷移・合意や資金の妥当性を検査するものではない。

本文のcargo +1.90.0は読者用の版指定。今回は既存stableが1.90.0であることをrustc/cargo --versionで確認して実行し、名前付き1.90.0 toolchainを新しくインストールしたわけではない。Rustupの新規導入手順・OS package導入・Cargo newによる初期化を通した新規環境試験ではない。

## Dockerの読取とCompose

- unameはDarwin / arm64、Python 3.11.0、Compose v2.31.0-desktop.2。
- 最初のDocker Server取得はsandboxからsocketへのアクセスが拒否された。承認された読取でClient/Serverとも27.4.0と確認。接続拒否をEngine停止と断定せず、context・権限も確認する説明へ修正。
- ローカルのkaspanet/rusty-kaspad@sha256:5bc3195c2cd2d171885781ed55ead9485d93c7bd7b784d46e9e393c96757fe22をdocker image inspectで読取り、linux/arm64とRepoDigest一致を確認。pull/runはしていない。バイナリーと基準ソースが一致する証明ではない。
- [Compose config](https://docs.docker.com/reference/cli/docker/compose/config/)と[image inspect](https://docs.docker.com/reference/cli/docker/image/inspect/)の公式説明を確認。設定展開・ローカルimage情報・remote manifestと、実際の起動/対応CPUでの動作を区別。

次の5構成を--profile mining config --format jsonで展開して解析した。すべて終了コード0。

1. compose.yaml
2. compose.yaml + mining.yaml
3. two-node.yaml
4. two-node.yaml + partition.yaml
5. two-node.yaml + partition.yaml + reorg.yaml

全構成の公開portのhost_ipが127.0.0.1であることを検査。base nodeはlinux/arm64・entrypoint=kaspad・--devnet/--appdir=/data、名前付きvolumeの/dataへのmountを確認。専用bridgeのinternalはtrueでなく、外向き通信を全面禁止していない。環境変数のdefault値のまま設定が展開されることと、2ノードが実接続することは別である。

これらは静的な設定確認で、コンテナ作成/起動・再起動/停止・volume操作やhello-world実行はしていない。Docker Desktopの導入や別context、新規ホスト/amd64/Linux/Windows・WSL2の試験でもない。

## 既存実測・RPCとの対応

experiments/devnet/README.mdとcompose.yaml、requirements.txt、probe.py、src/baseline.mdを照合した。既存ARM64イメージでの起動・送金/通知等の記録を今回の再実行と混同しない。

- probe.pyはgetInfo/getCurrentNetwork/getBlockDagInfoの応答・Devnet・serverVersion1.0.1・isUtxoIndexedを検査し、isSyncedを観測値として出力する。完全同期やE2Eを証明する検査ではない。
- Python 3.11 / websockets==14.1等の前提を第28章へ結び付けた。既存venv/SDKの再導入・probeのRPC実行はしていない。
- コンテナ内0.0.0.0待受とホスト127.0.0.1公開を区別した。Devnetという識別子だけでGenesis・パラメータ・データ世代等も一致したとはしない。
- 孤立Devnetの未同期状態や、別の採掘overlayの未同期採掘例外を、通常の同期/確定要件へ一般化しない。

mdBookはローカル0.5.4のserve --helpと[公式説明](https://rust-lang.github.io/mdBook/cli/serve.html)を照合。--hostname 127.0.0.1 / --port 3000を明示した。今回HTTPサーバーの起動やGUIでの閲覧は行っていない。

## 今回行っていないこと

Rustup/Docker/mdBook/OS packageのインストール、既定toolchain変更、hello-worldコンテナ、Devnet起動/採掘/鍵生成/署名/送金・2ノード再編成/Pruning後IBD・データ削除・新規OS/CPUのE2Eは実施していない。既存CIも変更していない。

## 照合対象のSHA-256

資料の識別用であり、各スクリプトの全経路/実動作を監査した印ではない。

| ファイル | SHA-256 |
|---|---|
| experiments/devnet/README.md | `4b855ddf3a7866f191068bf5199e47e7d75a2e253a3ba5147b5cc864cfcf30c0` |
| experiments/devnet/compose.yaml | `ff3bd152d0afefff506f8d1a1d068e187639e2d6d55146abbe3c13ff437f894a` |
| experiments/devnet/mining.yaml | `625e3501b55f7be8610a9c29e4fe90a8d2b470e0ae3fb7167f3bee7d4fa999f4` |
| experiments/devnet/two-node.yaml | `727e581a370b76f3019bd8a78427ab75f2b5f862446f4f7c6355c753288c2cd5` |
| experiments/devnet/partition.yaml | `b91671fa6355078b3650faa9a2e4f0b121adfe6fdfaaf46d19419406ff3d4803` |
| experiments/devnet/reorg.yaml | `75bdfabe15ec3fad272f6703496f06e10910576c2d4b1504c4bcaffba3475720` |
| experiments/devnet/probe.py | `f625ab53f46b382e3feeb1a34fc0c322aabc1b1d5297ed8574f0777fa68c816c` |
| experiments/devnet/requirements.txt | `67b5a29d65e1d2a5988ec22405b24f88b850f9809577c99fd27f769b68374049` |
| src/baseline.md | `1078f60fde8880d1468447106eac789fb2230d3dd99043d7cd8d9a30a9e6af3a` |

## 統合・公開前確認

- mdbook 0.5.4 build成功（警告なし）。53 HTMLのローカルリンク/アンカー検査で不整合0。
- SUMMARY49件と進捗表の一致、前段1〜43番の順序を検査。42番を確認済み、43番の付録Bを次の対象とした。
- scripts/check-book-examples.py：収録済み2教材が本文と一致。
- git diff --check成功。
- 追跡対象と非ignore候補188ファイルの既知token prefix・秘密鍵・credential URL・個人home path・機密設定名等のパターン走査で検出0。gitleaks/trufflehogは未導入で、未知の機密やGit全履歴を含む完全監査ではない。
- 公開差分は付録A、進捗/ROADMAP、本記録。私有パス・生ログ/metadata・DB/鍵・バイナリー・ダウンロードarchive・新しいexamplesを追加していない。push/公開操作は実施していない。
