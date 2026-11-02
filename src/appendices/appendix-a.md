# Appendix A — Rust環境・Docker環境の構築

> **対象**：『Mastering Kaspa』のRustサンプル、教育用BlockDAGノード、Kaspa Devnet実験を再現する読者。  
> **目次順の本文レビュー**：2026-11-01。独自Rust教材・Compose設定・環境の読取検査と既存実測記録を照合。全OSの新規導入/Devnet再実行ではない。
> **原則**：コマンドは原則macOS（Apple Silicon）とLinuxを想定する。Windows/WSL2を含む他環境は未検証で、ここでの動作保証には含めない。CLIのフラグやポートはバージョンで変わるため、実行時の`--help`を優先する。

対象版の一覧は[本書の対象バージョン](../baseline.md)、この回の結果は `experiments/appendix-a-review-2026-11-01.md` を参照。既存のDevnet実測と、今回新しく確認した事項を分けて記録する。

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

Rust公式の`rustup`を使う。未導入の場合だけインストールする。すでにrustupがあれば、再インストールせず後の版追加へ進む。スクリプトは公式の配布元から取得し、内容を確認してから実行する。

- Rust公式：<https://www.rust-lang.org/tools/install>
- rustup：<https://rust-lang.github.io/rustup/>

Unix系OSでの一般的なインストール方法：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/mastering-kaspa-rustup-init.sh
# 保存したスクリプトを読んでから、未導入の環境で実行する
sh /tmp/mastering-kaspa-rustup-init.sh --profile minimal
```

インストール後、ターミナルを開き直すか、環境設定を読み込む。

```bash
. "$HOME/.cargo/env"
rustup --version
```

本書のRust教材は1.90.0 / edition 2021で検証した。変化するstableへ一括変更せず、固定版を追加して明示的に呼び分ける。

Playgroundで試す場合もEditionを2021に設定する。本文で分割しているコードは、指定された順に結合する。`src/lib.rs`向けのコードには`main`関数がないため、通常の「Run」ではなく「Test」でコンパイルと付属テストを確認する。テスト関数がない断片は0件の実行になる。外部依存がある例は、各章の`Cargo.toml`と依存バージョンに従ってCargoで確認する。

```bash
rustup toolchain install 1.90.0 --profile minimal --component rustfmt --component clippy
rustc +1.90.0 --version
cargo +1.90.0 --version
```

以下ではcargo +1.90.0を使い、既存プロジェクトのdefault/overrideを変更しない。rust-toolchain.tomlがある場合も、明示的な+版指定は別の版を選ぶため、対象リポジトリの要件と照合する。

第32章のSilverscriptとArgent補足はRust 1.94.0以上を要求する別の実習である。必要な場合だけrustup toolchain install 1.94.0 --profile minimalで追加し、cargo +1.94.0で使う。1.94を入れたことが1.90の教材や基準ノードを再検証した証拠にはならない（[rustupの版指定](https://rust-lang.github.io/rustup/concepts/toolchains.html)）。

### A.5.1 Rustの最小動作確認

```bash
# 本書とは別の調査用ディレクトリで作る
cargo +1.90.0 new --edition 2021 kaspa-rust-lab
cd kaspa-rust-lab
cargo +1.90.0 run --offline
cargo +1.90.0 test --offline
cargo +1.90.0 fmt --all -- --check
cargo +1.90.0 clippy --offline --all-targets -- -D warnings
```

Hello, world!の表示と書式/Lintの成功は基本の動作確認であり、cargo new直後はテスト0件でも成功する。次節で実際に2件が走ることを確認する。これはC/C++依存や公式ノード・WASMのビルド試験ではない。

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
cargo +1.90.0 test --offline
```

2件のテストで通常の加算とu64の表現範囲超えを確認する。引数自体は有効なu64であり、状態遷移やKaspaの金額/合意規則の検証ではない。

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

`docker version`でクライアントのみ表示されサーバーへ接続できない場合、Docker Desktop/Engineの起動に加え、接続先context・ソケット/OS側の権限も確認する。アクセス拒否をEngine停止と同一視しない。ソケットを誰でも書込可能にする対応は避ける。

### A.7.1 Dockerの基本操作

```bash
docker ps
docker ps -a
docker images
docker logs --tail 100 <container-name>
docker stop <container-name>
docker rm <container-name>
```

`<container-name>`は説明用のプレースホルダーで、そのままシェルへ貼らず実名へ置き換える。上の行は基本操作の一覧であり、一括実行する手順ではない。データを消去する操作（`docker volume rm`や`docker compose down -v`）は、対象を確認してから実行する。

## A.8 Kaspaのソースコードを取得する

公式リポジトリ：<https://github.com/kaspanet/rusty-kaspa>

第1章と同じ固定版を、本書とは別のソース調査用ディレクトリへ取得する。既存の作業コピーへresetして合わせない。

```bash
git clone --branch v1.0.1 --depth 1 https://github.com/kaspanet/rusty-kaspa.git rusty-kaspa-v1.0.1
cd rusty-kaspa-v1.0.1
git rev-parse HEAD
git status --short
# HEADが fcd9c28f9b211b9872a0c90a8c94f79466ffb063 と一致することを確認
cargo +1.90.0 metadata --offline --locked --no-deps --format-version 1 > /tmp/rusty-kaspa-review-metadata.json
```

固定版の宣言MSRVは1.82.0で、教材の実測Rust 1.90.0とは別の値。1.82.0自体のビルドを検証したという意味ではない。公式workspace全体のビルドにはREADMEのProtobuf・clang/RocksDB・用途別のWASM要件等も確認する。

metadata --no-depsはパッケージ一覧の確認で、依存解決・ノードのビルド成功ではない。生JSONはローカルパスを含むため、そのまま公開記録へ入れない。ソースのSHAと配布Dockerバイナリーの申告版/digestも、再ビルドやビルド証明なしに同一性の証拠として扱わない。

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

Composeのconfig成功は構文・展開後の設定確認で、起動・RPC・P2Pの成功ではない。初回はprobeに必要なPython 3.11とwebsockets==14.1の環境を第28章に従って作る。probeはDevnet・版1.0.1・UTXO index等を検査する読取スモークテストで、同期完了/E2Eの証明ではない。

### A.10.1 保存先と終了

設定は公開イメージのdigestを固定し、`--appdir=/data` と名前付きボリュームの保存先を合わせている。JSON wRPCはホストの127.0.0.1:18310だけへ公開する。コンテナ内の0.0.0.0待受とホスト側公開範囲を混同しない。ピア探索・UPnP・自動外部接続は抑制しているが、専用bridge自体は外向き通信を全面禁止する設定ではない。

```bash
docker compose -f experiments/devnet/compose.yaml stop node
```

停止ではデータを削除しない。既存データがあれば、再開時のブロック数やUTXOは初回の値と異なる。送金・通知・2ノードの試験は追加設定と専用スクリプトを使うため、第28章とリポジトリの `experiments/devnet/README.md` を参照する。

この構成はmacOS Apple Siliconで検証済みであり、amd64やLinuxホストで実行確認したものではない。Pruning後の同期も未検証である。これらは既存の実験記録の範囲で、今回起動/採掘/送金を再実行したわけではない。

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
5. タイムアウト・再接続・購読再登録と欠落補完をテストする。

参照：<https://github.com/kaspanet/rusty-kaspa>

孤立したDevnetでは未採掘時等にisSynced=falseを観測し得る。既存の採掘実験が使う未同期採掘の例外設定を、通常の同期要件やアプリの確定判定へ一般化しない。2ノードで同じ文字列Devnetと出ても、Genesis/パラメータ・接続先・データ世代が一致することは別に確認する。

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
KASPA_IMAGE="kaspanet/rusty-kaspad@sha256:5bc3195c2cd2d171885781ed55ead9485d93c7bd7b784d46e9e393c96757fe22"
docker image inspect "$KASPA_IMAGE" --format '{{json .RepoDigests}}'
```

RepoDigestsが空の場合もあり、ローカルimage IDとリポジトリdigestは同じ識別子ではない。OS/Architectureもdocker image inspectで照合する。今回の固定参照はローカルでlinux/arm64と一致したが、これは他のCPU・Linuxホストの動作確認や、ノードのソース/バイナリー一致の証明ではない。タグのみでは後日の完全な再現性を保証できない。

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

本書のルートからの診断用コマンド（版は実際に使ったtoolchainを指定）：

```bash
rustc --version
cargo --version
docker version
docker compose -f experiments/devnet/compose.yaml ps
docker compose -f experiments/devnet/compose.yaml logs --tail=200 node
```

`docker system prune`などの一括削除コマンドは、他のプロジェクトのデータにも影響し得るため、安易に実行しない。

## A.16 CIでRustコードを検証する

独立した単一Cargoプロジェクト向けの参考CI例。本書に.github/workflows/rust.ymlを追加したものではない。本書の実設定は.github/workflows/deploy.ymlである。

```yaml
name: Rust checks
permissions:
  contents: read
on:
  push:
  pull_request:
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.90.0
        with:
          components: rustfmt, clippy
      - run: cargo fmt --all -- --check
      - run: cargo clippy --locked --all-targets -- -D warnings
      - run: cargo test --locked --all-targets
```

この例は生成したCargo.lockを版管理する前提。本書の既存CIが検査するRust教材は第12・27章の2件で、本付録の手動確認を自動実行済みとは扱わない。ワークスペースやサブディレクトリ配置の場合は`working-directory`等を調整する。第三者アクションは運用時にコミットSHAへ固定することも検討する。

## A.17 mdBookのインストールと表示

本書自体をローカルで閲覧する場合：

```bash
cargo install mdbook --version 0.5.4 --locked
cd mastering-kaspa
mdbook build
mdbook serve --hostname 127.0.0.1 --port 3000
```

ブラウザで通常`http://localhost:3000`を開く。ポートや設定は実行時の表示に従う。

`mdbook build`が生成する`book/`は通常、Markdown原稿とは別の**ビルド成果物**として扱う。

## A.18 本書の関連ファイル

```text
mastering-kaspa/
├── book.toml
├── src/
│   ├── SUMMARY.md
│   └── appendices/appendix-a.md
├── examples/
│   ├── dag-relations/
│   └── minimal-node/
├── experiments/
│   └── devnet/
│       ├── README.md
│       ├── compose.yaml
│       ├── requirements.txt
│       └── probe.py
├── scripts/check-book-examples.py
└── .github/workflows/deploy.yml
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

**確認範囲：** Rust最小例・Compose設定・Dockerの版/ローカルイメージ読取と既存記録の照合。rustup/Docker等の新規導入、hello-worldコンテナ、Devnet起動/採掘/送金・他OS/CPU・Pruningは今回実行していない。記録は `experiments/appendix-a-review-2026-11-01.md`。

次は[付録B — Kaspa用語集](appendix-b.md)。
