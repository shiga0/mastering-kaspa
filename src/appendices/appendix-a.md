# Appendix A — Rust環境・Docker環境の構築

> **対象**：『Mastering Kaspa』のRustサンプル、教育用BlockDAGノード、Kaspa Devnet実験を再現する読者。  
> **原稿ステータス**：v0.1（技術レビュー前）  
> **原則**：コマンドは原則macOS（Apple Silicon）とLinuxを想定する。WindowsはWSL2を推奨する。CLIのフラグやポートはバージョンで変わるため、実行時の`--help`を優先する。

## A.1 この付録の目的

この付録では、次の環境を整える。

1. Rustのビルド・テスト環境
2. Dockerによる隔離された実験環境
3. `rusty-kaspa`のソースコード調査環境
4. Kaspaの**Devnet**を利用した安全な実験環境
5. 実験の再現性とトラブルシューティングの基盤

本書の教育用Rustコードと実際のKaspaコンセンサス実装は別物である。教育用コードをKaspaメインネットのノードとして扱ってはいけない。

## A.2 構成と責務

```text
開発用Mac / Linux
├── Rust toolchain (rustup / cargo / rustc)
│   ├── 教育用実装
│   └── 単体テスト
├── Git
│   └── rusty-kaspa ソースコード
└── Docker Engine / Docker Desktop
    ├── Devnet node A
    ├── Devnet node B（必要に応じて）
    └── データボリューム

アプリケーション → RPC → Devnetノード → P2P → 他ノード
```

**分離の目的**：Rustで書いたアルゴリズムの単体テストと、実ノードの統合テストを混同しないこと。

## A.3 事前確認

macOSではターミナルを開く。次のコマンドでCPUとOSを確認する。

```bash
uname -s
uname -m
```

Apple Siliconの場合、通常は`Darwin`と`arm64`が表示される。Linuxでは`Linux`、x86-64では`x86_64`となる。

必要なもの：

| ツール | 用途 | 必須となる場面 |
|---|---|---|
| Git | ソース取得・差分管理 | ほぼすべて |
| Rust (`rustup`, `cargo`) | サンプル実装・テスト | Rust演習 |
| Docker | Devnetノードを隔離して起動 | Devnet演習 |
| mdBook | 本書のローカル閲覧 | 執筆・読書 |
| エディタ | コード・Markdown編集 | 推奨 |

## A.4 Gitの準備

```bash
git --version
```

macOSで未導入の場合、Command Line Toolsを導入する。

```bash
xcode-select --install
```

Linuxの場合はディストリビューションのパッケージマネージャーを使用する。例（Ubuntu/Debian）：

```bash
sudo apt update
sudo apt install -y git curl build-essential pkg-config libssl-dev
```

上記は一般的な開発用依存パッケージの例であり、`rusty-kaspa`の全ビルド要件を保証するものではない。

## A.5 Rustのインストール

Rust公式の`rustup`を使う。インストールスクリプトは実行前に公式サイトで内容を確認すること。

- Rust公式：<https://www.rust-lang.org/tools/install>
- rustup：<https://rust-lang.github.io/rustup/>

Unix系OSでの一般的なインストール方法：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

インストール後、ターミナルを開き直すか、環境設定を読み込む。

```bash
. "$HOME/.cargo/env"
rustc --version
cargo --version
rustup --version
```

安定版ツールチェーンを選択する。

```bash
rustup toolchain install stable
rustup default stable
rustup component add rustfmt clippy
```

既存プロジェクトが特定のRustバージョンを要求する場合は、リポジトリの`rust-toolchain.toml`などを優先する。

### A.5.1 Rustの最小動作確認

```bash
cargo new kaspa-rust-lab
cd kaspa-rust-lab
cargo run
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

`Hello, world!`が表示され、テスト・フォーマット・Lintが成功すれば基礎環境は動いている。

### A.5.2 テスト付きの最小実験

`src/lib.rs`を作成する。

```rust
/// 学習用の単純な関数。Kaspaのコンセンサス処理ではない。
pub fn add_checked(a: u64, b: u64) -> Option<u64> {
    a.checked_add(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_two_values() {
        assert_eq!(add_checked(2, 3), Some(5));
    }

    #[test]
    fn rejects_overflow() {
        assert_eq!(add_checked(u64::MAX, 1), None);
    }
}
```

```bash
cargo test
```

ここでは「正常な状態遷移」と「不正な入力の拒否」をテストで表現する基本形を確認する。

## A.6 Rustのディレクトリと主要コマンド

```text
kaspa-rust-lab/
├── Cargo.toml       # パッケージ定義と依存関係
├── Cargo.lock       # 解決された依存バージョン
├── src/
│   ├── main.rs      # 実行ファイル
│   └── lib.rs       # ライブラリとテスト
└── target/          # ビルド生成物（通常Gitに含めない）
```

| コマンド | 意味 |
|---|---|
| `cargo check` | バイナリ生成を省略した型・コンパイル検査 |
| `cargo build` | ビルド |
| `cargo test` | テスト |
| `cargo fmt --check` | 書式検査 |
| `cargo clippy` | Lint |
| `cargo doc --open` | ドキュメント生成と閲覧 |
| `cargo tree` | 依存関係の確認 |

再現性のため、実験時には`rustc --version`、GitコミットID、使用した依存関係の情報を記録する。

## A.7 Dockerの準備

- Docker公式：<https://docs.docker.com/get-docker/>
- Docker Desktop for Mac：<https://docs.docker.com/desktop/setup/install/mac-install/>

macOSではDocker Desktopをインストールして起動する。LinuxではDocker Engineを公式の手順で導入する。Dockerの実行権限はホストOSのセキュリティに影響するため、無闇にユーザー権限を拡大しない。

確認：

```bash
docker version
docker compose version
docker run --rm hello-world
```

`docker version`でクライアントのみ表示されサーバーへ接続できない場合、Docker DesktopまたはDocker Engineが起動しているか確認する。

### A.7.1 Dockerの基本操作

```bash
docker ps
docker ps -a
docker images
docker logs --tail 100 <container-name>
docker stop <container-name>
docker rm <container-name>
```

`<container-name>`は実際のコンテナ名に置き換える。データを消去する操作（`docker volume rm`や`docker compose down -v`）は、対象を確認してから実行する。

## A.8 Kaspaのソースコードを取得する

公式リポジトリ：<https://github.com/kaspanet/rusty-kaspa>

```bash
mkdir -p ~/dev/kaspa
cd ~/dev/kaspa
git clone https://github.com/kaspanet/rusty-kaspa.git
cd rusty-kaspa
git rev-parse HEAD
git status --short
```

**注意**：`main`や`master`の最新状態は時間とともに変化する。実験記録にはコミットハッシュを残し、可能なら特定のタグまたはコミットに固定する。

```bash
git tag --list | tail -n 20
cargo metadata --no-deps --format-version 1 > /tmp/rusty-kaspa-metadata.json
```

`cargo metadata`はワークスペースの構成調査に使える。ただし、依存関係や開発環境が不足すると失敗することがある。

### A.8.1 最初に読む場所

- ルートの`README`と開発手順
- `Cargo.toml`と`rust-toolchain.toml`（存在する場合）
- コンセンサス、P2P、RPC、ウォレット関連のクレート
- テスト・ベンチマーク・CI設定

ディレクトリ名や関数名は変更される可能性があるため、書籍に掲載する場合は**対象コミットで実在を確認**する。

## A.9 Devnetとは

Kaspaの**Devnet**は、実験・開発向けのネットワークである。Mainnet、Testnet、Devnetは目的も状態も異なる。Devnetで得たコインを実資産として扱ってはいけない。

```text
Mainnet: 実際の価値を持つネットワーク
Testnet: 公開テスト向けネットワーク
Devnet : 開発・実験向けネットワーク
```

実験時には、意図したネットワークを明示し、メインネットのウォレット・秘密鍵・データ領域と分離する。

## A.10 DockerでKaspa Devnetを起動する

実行確認したARM64向け構成を使う。[第28章](../part-04-development/chapter-28.md)の28.6でPython環境と、必要な場合は公式SDKを準備してから実行する。以下はリポジトリのルートでの操作である。

```bash
docker compose -f experiments/devnet/compose.yaml config
docker compose -f experiments/devnet/compose.yaml up -d node
docker compose -f experiments/devnet/compose.yaml logs --tail=30 node
experiments/devnet/.venv/bin/python experiments/devnet/probe.py
```

### A.10.1 保存先と終了

設定は公開イメージのdigestを固定し、`--appdir=/data` と名前付きボリュームの保存先を合わせている。JSON wRPCはホストの127.0.0.1:18310だけへ公開する。

```bash
docker compose -f experiments/devnet/compose.yaml stop node
```

停止ではデータを削除しない。既存データがあれば、再開時のブロック数やUTXOは初回の値と異なる。送金・通知・2ノードの試験は追加設定と専用スクリプトを使うため、第28章とリポジトリの `experiments/devnet/README.md` を参照する。

この構成はmacOS Apple Siliconで検証済みであり、amd64やLinuxホストで実行確認したものではない。Pruning後の同期も未検証である。

## A.11 2ノード統合テストの考え方

2ノード構成では、各ノードに**独立したデータ領域**と適切なネットワーク設定を割り当てる。

```text
Devnet Node A  ←── P2P ──→  Devnet Node B
      │                         │
    RPC A                     RPC B
      │                         │
      └──────── Test Harness ───┘
```

確認したい性質：

1. ブロックの伝播
2. 受信順が異なる場合の最終状態の整合
3. ノード停止・再起動からの復帰
4. ネットワーク切断後の再同期
5. RPCのタイムアウトと再接続

同じDevnetでも設定や初期状態が異なると、意図せず別のネットワークとして動作する可能性がある。ログやネットワーク識別情報を確認する。

## A.12 RPC接続の確認

RPCの方式・ポート・コマンドはノードのバージョンによって異なる。TCPポートが開いているだけでは、正しいネットワークへ接続できたとは限らない。

確認手順：

1. ノードのログで起動完了を確認する。
2. 利用するバージョンのRPC仕様を確認する。
3. ネットワーク識別子を取得してDevnetであることを検証する。
4. 同期状態とブロック情報を取得する。
5. タイムアウト・再接続をテストする。

参照：<https://github.com/kaspanet/rusty-kaspa>

## A.13 セキュリティ上の注意

- Mainnetの秘密鍵・シードフレーズをDevnet実験に使わない。
- RPCを`0.0.0.0`で公開する場合は、ネットワーク境界・認証・ファイアウォールを検討する。
- Dockerソケットへのアクセスは高い権限を持つため、信頼できないコンテナに渡さない。
- 不明なスクリプトを内容確認なしに実行しない。
- ウォレットやノードのログに秘密情報を出力しない。
- データボリュームの削除前に必要なデータをバックアップする。

## A.14 Gitと実験の再現性

実験ごとに次の情報を残す。

```text
experiment-id: devnet-sync-001
os: macOS / Linux
arch: arm64 / x86_64
rustc: <rustc --version>
cargo: <cargo --version>
docker: <docker version>
rusty-kaspa-commit: <git rev-parse HEAD>
container-image: <image@sha256:...>
network: devnet
config: <設定ファイルへの参照>
expected: <期待結果>
observed: <実測結果>
logs: <保存場所>
```

### A.14.1 Dockerイメージのdigestを記録

```bash
docker image inspect "$KASPA_IMAGE" --format '{{json .RepoDigests}}'
```

取得できるdigestはローカルのイメージ状態によって異なる。タグのみでは後日の完全な再現性を保証できない。

## A.15 トラブルシューティング

| 症状 | 主な確認点 | 対処の方向性 |
|---|---|---|
| `cargo`が見つからない | PATH、シェル設定 | `. "$HOME/.cargo/env"`、ターミナル再起動 |
| Rustビルドが失敗する | Rust版、OS依存ライブラリ | `rust-toolchain.toml`、README、エラーログ確認 |
| Dockerに接続できない | Docker Engine起動状態 | Docker Desktop/サービスを起動 |
| コンテナがすぐ終了する | CLIフラグ、設定、マウント | `docker compose logs`で原因確認 |
| RPCに接続できない | バインド先、ポート、方式 | ノードの実際のRPC設定を照合 |
| 同期しない | ネットワーク設定、ピア、時計 | ログ・ネットワーク識別子を確認 |
| ディスクが不足する | ボリューム・ログ・ビルド生成物 | 使用量確認後、不要データのみ整理 |
| Apple Siliconで動かない | イメージの対応アーキテクチャ | `docker manifest inspect`などで確認 |

診断用コマンド：

```bash
rustc --version
cargo --version
docker version
docker compose ps
docker compose logs --tail=200
```

`docker system prune`などの一括削除コマンドは、他のプロジェクトのデータにも影響し得るため、安易に実行しない。

## A.16 CIでRustコードを検証する

GitHub Actionsで教育用Rustプロジェクトを検証する例。`.github/workflows/rust.yml`：

```yaml
name: Rust checks
on:
  push:
  pull_request:
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test --all-targets
```

これは単一Cargoプロジェクト向けの基本例。ワークスペースやサブディレクトリ配置の場合は`working-directory`等を調整する。第三者アクションは運用時にコミットSHAへ固定することも検討する。

## A.17 mdBookのインストールと表示

本書自体をローカルで閲覧する場合：

```bash
cargo install mdbook --version 0.5.4 --locked
cd mastering-kaspa
mdbook build
mdbook serve
```

ブラウザで通常`http://localhost:3000`を開く。ポートや設定は実行時の表示に従う。

`mdbook build`が生成する`book/`は通常、Markdown原稿とは別の**ビルド成果物**として扱う。

## A.18 推奨リポジトリ構成

```text
mastering-kaspa/
├── book.toml
├── src/
│   ├── SUMMARY.md
│   └── appendices/
│       └── appendix-a.md
├── examples/
│   ├── minimal-node/
│   └── wallet/
├── experiments/
│   └── devnet-sync-001/
│       ├── README.md
│       ├── compose.yaml
│       └── results.md
└── .github/
    └── workflows/
        └── rust.yml
```

生成物、秘密鍵、ノードの巨大なデータベース、実験用の機密設定はGitに含めない。`.gitignore`を適切に設定する。

## A.19 最初の実践課題

**課題1：Rust環境の検証**

- `cargo new`でプロジェクトを作る。
- `cargo test`で正常系と異常系を1件ずつ実行する。
- `cargo fmt --check`と`cargo clippy`を通す。

**課題2：Docker環境の検証**

- `hello-world`を実行する。
- Dockerのバージョンを記録する。
- 使用するKaspaイメージのタグとdigestを記録する。

**課題3：`rusty-kaspa`の調査**

- リポジトリをcloneする。
- コミットハッシュを記録する。
- `Cargo.toml`から主要なクレートを分類する。

**課題4：Devnetの統合テスト**

- 固定したバージョンでDevnetノードを起動する。
- ログからネットワークと起動状態を確認する。
- RPCの応答と同期状態を記録する。
- 停止・再起動でデータが維持されるか確認する。

## A.20 完了チェックリスト

- [ ] `rustc --version`と`cargo --version`を確認した
- [ ] `cargo test`が成功した
- [ ] `cargo fmt --check`と`cargo clippy`が成功した
- [ ] `docker run --rm hello-world`が成功した
- [ ] `rusty-kaspa`のコミットIDを記録した
- [ ] DevnetとMainnetのデータ・鍵を分離した
- [ ] 使用したDockerイメージと起動フラグを記録した
- [ ] RPC接続とネットワーク識別子を確認した
- [ ] 実験ログと結果を保存した

## A.21 まとめ

開発環境の構築は、単にコマンドが動けば完了ではない。**何を、どのバージョンで、どの設定で実行し、どの結果を観測したか**を記録して初めて、第三者が再現できる実験になる。

Rustの単体テストで確認するのは、**自分が定義したモデルの性質**。Devnetで確認するのは、**実際のKaspaノードとの接続やネットワーク動作**。両者を分離して記録することが、本書の実装・検証パートの基本となる。

## A.22 参考資料

- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust Install](https://www.rust-lang.org/tools/install)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [rustup Book](https://rust-lang.github.io/rustup/)
- [Docker Docs](https://docs.docker.com/)
- [Docker Compose Docs](https://docs.docker.com/compose/)
- [rusty-kaspa（公式リポジトリ）](https://github.com/kaspanet/rusty-kaspa)
- [Kaspa Wiki](https://wiki.kaspa.org/)
- [mdBook Documentation](https://rust-lang.github.io/mdBook/)

---

**編集メモ**：出版前に、採用する`rusty-kaspa`のタグ・Dockerイメージ・Devnetフラグ・RPC仕様を固定し、実機で検証したコマンドとログを追記すること。
