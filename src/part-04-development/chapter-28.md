# Chapter 28 — Devnet & Integration Testing（Devnetと統合テスト）

> **Mastering Kaspa / Part IV — Kaspa Development & Implementation**  
> 原稿バージョン：v0.1（技術レビュー前）  
> 本章の対象：Kaspa Devnetでの再現可能な統合テスト、RPC、障害注入、検証証跡

## 28.1 本章の目的

第27章では教育用の最小Kaspa型ノードを組み立てた。本章では、そのモデルから一歩進み、**実際の `rusty-kaspa` ノードを隔離されたDevnetで動かし、アプリケーションとノードの境界を検証する**。

読み終えると、次の作業を説明・実践できるようになる。

1. DevnetとMainnet・Testnetの用途とリスクを区別する。
2. ノード・クライアント・テストハーネスの責務を分離する。
3. RPC接続、同期状態、トランザクション送信、状態確認を段階的に検証する。
4. タイムアウト、再起動、重複通知、ネットワーク分断をテストする。
5. 実験条件・ログ・結果を保存し、第三者が再現できるようにする。

**注意：** KaspaのCLIフラグ、RPCメソッド、Dockerイメージの挙動はリリースによって変わる。以下のコマンドは実験の雛形であり、使用するバージョンの `--help` と公式ソースを確認してから実行する。実験結果を実測したものとして記述しない。

## 28.2 なぜ統合テストが必要なのか

単体テストが保証するのは、原則として**特定コンポーネントに与えた入力に対する振る舞い**である。実際のアプリケーションでは、複数コンポーネントの間に失敗が生まれる。

```text
Wallet / Application
        │
        │ RPC request
        ▼
    RPC Gateway ───────────────┐
        │                      │
        ▼                      ▼
    Kaspa Node              Observability
        │
        ├── Mempool
        ├── Consensus / Virtual State
        ├── Storage
        └── P2P ─────── Other Nodes
```

例えば、送金RPCがタイムアウトしたとき、送信自体が失敗したとは限らない。サーバーが受理した後に応答だけ失われた可能性がある。したがって「再試行すればよい」と単純には言えない。

| テスト種別 | 主な対象 | 例 |
|---|---|---|
| 単体テスト | 関数・ドメインモデル | UTXOの重複入力を拒否する |
| コンポーネントテスト | 1サービスと代替依存 | RPCアダプターが通信エラーを分類する |
| 統合テスト | 実ノードとの接続 | RPCでノード情報を取得する |
| E2Eテスト | ユーザー操作から状態反映まで | Devnet送金から受理状態の確認まで |
| 障害注入テスト | 障害時の回復 | 接続断後にイベント購読を再開する |
| 性能テスト | 負荷と応答時間 | 複数購読者のイベント遅延を計測する |

## 28.3 Mainnet・Testnet・Devnet・Simnet

| 環境 | 用途 | 注意点 |
|---|---|---|
| Mainnet | 実運用 | 実資産を扱う。破壊的実験をしない |
| Testnet | 公開された検証ネットワーク | 他者のノード・ネットワーク状態に依存する |
| Devnet | 開発用の独立ネットワーク | 初期化・マイニング・設定を自分で管理する |
| Simnet | シミュレーション・制御された試験 | バージョンごとの提供方法を確認する |

本章では**Devnetを基本**とする。Devnet上のKASはMainnetのKASとは別物で、経済的価値を前提にしない。

**実験環境を本番環境から隔離する**こと。ウォレットのシードや秘密鍵をMainnetと共有しない。テスト用の鍵もGitにコミットしない。

## 28.4 テスト対象のC4モデル

### システムコンテキスト

```text
┌──────────────────────────────────────────┐
│             Developer / CI               │
└────────────────────┬─────────────────────┘
                     │ test command
                     ▼
┌──────────────────────────────────────────┐
│             Test Harness                 │
│   fixtures / assertions / log capture    │
└────────────┬─────────────────────────────┘
             │ RPC / wRPC
             ▼
┌──────────────────────────────────────────┐
│              Kaspa Devnet                │
│    Node A ─── P2P ─── Node B             │
│      │                    │              │
│   Storage              Storage           │
└──────────────────────────────────────────┘
```

### 責務と境界

- **Test Harness**：初期状態の準備、入力、期待値の検証、ログ収集。
- **RPC Adapter**：通信方式、タイムアウト、リトライ、接続再確立。
- **Node**：プロトコル検証、Mempool、BlockDAG、Virtual State。
- **Storage**：永続化、再起動後の復元。
- **P2P**：他ノードへの伝播と同期。

テストコードに「RPCが成功したらトランザクションは確定」といった誤った業務ルールを埋め込まない。

## 28.5 実験の再現性を先に設計する

実験の開始前に以下を記録する。

```text
experiment-id: devnet-integration-001
node-version: <git commit or image digest>
network: devnet
node-count: 2
rpc-protocol: <gRPC / wRPC and version>
platform: <OS / architecture>
started-at: <ISO 8601>
expected-result: <assertions>
actual-result: <fill after execution>
artifacts: logs/, metrics/, test-results/
```

`latest` タグだけでは再現性が弱い。リリースタグやイメージdigest、Gitコミットを固定する。ポート番号・ネットワーク識別子・データディレクトリも記録する。

## 28.6 ローカル環境の準備

**実行検証の状況（2026年10月31日）：** Docker起動後、固定digestのkaspad 1.0.1（linux/arm64）で、Devnet起動・読み取りwRPC・採掘と再起動後のDAG/報酬UTXO保持を確認した。再現用のCompose設定、Python検査スクリプト、依存バージョン、実測JSONはリポジトリの `experiments/devnet/` に収録している。初回の採掘前は `isSynced: false`、ブロック数0を観測した。追加の採掘プロファイルで151ブロックと150件の報酬UTXOを生成し、再起動後の一致を確認した。実習はARM64専用で、本文の固定ソースコミットとバイナリの同一性は未検証。追加の `send.cjs` で成熟資金の準備、単一署名送金、virtual受理、UTXO変化を確認した。異常系・Mempool/包含の独立観測・2ノード検証は未完了。章別の進捗は[技術レビューの進捗](../review-status.md)を参照する。


必要なものはDocker、Docker Compose、Rustツールチェーン、Git、KaspaのRPCクライアントである。

```bash
rustc --version
cargo --version
docker --version
docker compose version
git --version
```

Kaspa公式実装：

```bash
git clone https://github.com/kaspanet/rusty-kaspa.git
cd rusty-kaspa
git rev-parse HEAD
```

実行オプションはチェックアウトしたコミットに合わせて確認する。

```bash
# ビルド方法やバイナリ名は対象リリースのREADMEを参照
cargo build --release

# 利用可能なフラグを実機で確認
./target/release/kaspad --help
```

> **注意**：`kaspad` の実際の配置場所やビルドに必要な機能フラグは対象コミットによって異なる。上記のビルドは実行環境での成功を保証しない。

## 28.7 Dockerで単一ノードを起動する

実行確認した最小構成は、リポジトリの `experiments/devnet/README.md` と `compose.yaml` を参照する。

以下は**概念的な起動例**である。ポート公開や追加フラグは対象バージョンの公式ドキュメントで確定させる。

```bash
docker run --rm \
  --name kaspa-devnet-a \
  -v kaspa-devnet-a:/data \
  kaspanet/rusty-kaspad:<固定したタグ> \
  --devnet \
  --utxoindex
```

`<固定したタグ>` は実際に存在するバージョンに置き換える。RPCをホストから利用する場合は、使用するRPC方式の**正しいlistenフラグとポート**を確認し、`127.0.0.1` に限定して公開する。例示的なポート番号を無検証で本番に転用しない。

確認する項目：

1. プロセスが起動し、異常終了を繰り返さない。
2. ログにDevnetのネットワーク識別が出ている。
3. RPCエンドポイントが応答する。
4. 同期状態を取得できる。
5. 永続化ディレクトリにデータが生成される。

## 28.8 2ノード構成とピア接続

実行確認した構成は、リポジトリの `experiments/devnet/two-node.yaml` と `two-node.py` に収録している。独立したデータ領域の2ノードで、接続・ブロック伝播・片側停止後の追従を確認した。再開後に27ブロックと指定した報酬先の26件のUTXOが一致したが、全UTXO集合・双方採掘時の分断・Pruning後の同期の検証ではない。実測記録は `experiments/devnet-two-node-2026-10-31.md` を参照する。

```text
Node A                     Node B
  │                          │
  │────── P2P handshake ────▶│
  │◀────── peer info ────────│
  │                          │
  │──── block / inv ────────▶│
  │                          │
  │◀──── sync messages ──────│
```

2ノードで実験すると、単一ノードでは観察できない**伝播遅延・同期差・ネットワーク分断**を検証できる。

- ノードごとにデータディレクトリを分ける。
- P2PポートとRPCポートの衝突を避ける。
- 起動順を入れ替えて接続が成立するか確認する。
- 片方を停止し、もう片方が継続して動作するか観察する。

ピア接続用の具体的なCLIフラグはバージョン固定後に決める。P2PポートとRPCポートを混同しない。

## 28.9 RPCの疎通確認

RPC検証は段階を分ける。

```text
Transport connection
       │
       ▼
Network identification
       │
       ▼
Node sync status
       │
       ▼
Read-only queries
       │
       ▼
Transaction submission
       │
       ▼
State observation
```

最初から送金を試すのではなく、**読み取り専用のRPC**で接続先と同期状態を確認する。

検証すべき不変条件：

- 接続先ネットワークが期待するDevnetである。
- 未同期状態を「同期済み」と扱わない。
- 通信失敗とプロトコル拒否を区別する。
- 送信成功とコンセンサス受理を区別する。

## 28.10 トランザクションのE2Eテスト

実行確認した単一署名の手順は、リポジトリの `experiments/devnet/README.md` の送金実習を参照する。公式SDKはv1.0.1配布物、署名鍵はメモリ内だけの一時鍵を使う。公開記録はTX ID・公開アドレス・受理結果だけに限定している。

テスト用ウォレットAからウォレットBへDevnet資金を送る例を考える。

```text
A: UTXO selection
        │
        ▼
Transaction construction
        │
        ▼
Signing with test key
        │
        ▼
RPC submit
        │
        ▼
Mempool observation
        │
        ▼
Block inclusion
        │
        ▼
Consensus acceptance
        │
        ▼
B: UTXO / balance observation
```

### 検証項目

| 段階 | 期待すること | 失敗時の調査先 |
|---|---|---|
| UTXO取得 | 入力候補が存在する | UTXO index・同期状態 |
| 署名 | 対象ネットワークに適した署名が生成される | ウォレット・署名方式 |
| RPC送信 | 応答またはエラーを記録できる | RPC・接続・タイムアウト |
| Mempool | ローカルポリシーに従い取り扱われる | Mempool・競合入力 |
| ブロック | 対象トランザクションが含まれる | マイニング・テンプレート |
| コンセンサス | 現在のVirtual Stateで受理される | DAG・コンセンサス |
| 残高 | 期待するUTXO状態が観測される | UTXO index・再編成 |

Devnetではブロック生成の準備が別途必要な場合がある。ノード起動だけで継続的にブロックが生成されると仮定しない。

## 28.11 非同期イベント購読の検証

イベント購読はポーリングより低遅延になり得るが、接続断で通知を取りこぼす可能性がある。

**推奨する設計**：

1. 通知は「再確認のきっかけ」として扱う。
2. 接続が切れたら再接続する。
3. 再接続後は必要な範囲の状態を再照会する。
4. イベント処理を冪等にする。
5. トランザクションIDだけでなく、受理状態や競合状態も確認する。

```text
Event stream ──▶ Event handler ──▶ Reconciliation
      │                │                │
      └── disconnect ──┘                ▼
                                 Durable app state
```

## 28.12 タイムアウトと再試行

本書の `experiments/devnet/send.cjs --retry-checks` は、ノードの送信成功応答をローカルプロキシで1件だけ破棄する。クライアント側の期限切れ後、既知のTX IDでMempoolを照会して復旧し、同じ署名済みトランザクションを意図的に再送すると重複として拒否されることを確認した。

この検証では、正常送金の受理後に受取UTXO情報を2回処理し、Outpoint単位の計上で金額を1回分に保つことも確認した。ただし、インメモリの教材試験であり、永続台帳・複数ワーカー・再編成による取り消しの検証ではない。実測記録は `experiments/devnet-send-retry-2026-10-31.md` を参照する。

分散システムでは、応答がないことから処理の未実行を推論できない。

```text
Client                 Node
  │── submit(tx) ──────▶│
  │                     │ accepted locally
  │      X response     │
  │◀───── timeout ──────│
  │                     │
  │── query(txid) ─────▶│
```

このケースで新しい送金トランザクションを無条件に作成すると、意図しない重複送金につながる。**同じ送金意図に対する識別子**をアプリケーションで持ち、txidやUTXO状態を再照会する。

再試行戦略は「読み取り」「同一データの再送」「新規署名・新規トランザクションの作成」で分ける。

## 28.13 Reorganizationと状態の巻き戻り

BlockDAGの観測結果は、新たなブロックやコンセンサス状態の変化により更新され得る。

アプリケーション側で保持する状態例：

```text
CREATED
   │
   ▼
SUBMITTED
   │
   ▼
OBSERVED
   │
   ▼
ACCEPTED
   │
   ├──▶ RECHECK_REQUIRED
   │           │
   │           └──▶ ACCEPTED / CONFLICTED
   ▼
APPLICATION_CONFIRMED
```

`APPLICATION_CONFIRMED` はアプリケーションの確認ポリシーであり、Kaspaプロトコルの状態名ではない。確認済み表示後の例外対応や補償処理も設計する。

## 28.14 障害注入テストの計画

| 故障 | 操作 | 確認すること |
|---|---|---|
| RPC切断 | クライアント接続を閉じる | 再接続と状態再照会 |
| ノード停止 | Node Aを停止 | エラーが明示される |
| ノード再起動 | 永続データを残して再起動 | 状態の復元 |
| ネットワーク分断 | Node AとBのP2P通信を遮断 | 伝播差・回復後の同期 |
| 重複通知 | 同じイベントを複数回投入 | 副作用が重複しない |
| RPC遅延 | 応答を遅延させる | タイムアウト処理 |
| 未同期ノード | 同期途中でRPCを呼ぶ | 不適切な受理判断を防ぐ |
| ディスク容量不足 | 隔離環境で容量を制限 | 異常検知と復旧手順 |

障害注入は必ず専用のDevnet・テスト用ストレージで実施する。

## 28.15 Rustでテストハーネスの境界を設計する

実際のKaspa RPC SDKはバージョンに依存するため、まずは外部依存を抽象化する。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Devnet,
    Testnet,
    Mainnet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeInfo {
    pub network: Network,
    pub synced: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayError {
    Unreachable,
    Timeout,
    ProtocolRejected,
}

pub trait NodeGateway {
    fn node_info(&self) -> Result<NodeInfo, GatewayError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadinessError {
    Gateway(GatewayError),
    WrongNetwork,
    NotSynced,
}

pub fn ensure_devnet_ready(
    gateway: &impl NodeGateway,
) -> Result<(), ReadinessError> {
    let info = gateway.node_info().map_err(ReadinessError::Gateway)?;
    if info.network != Network::Devnet {
        return Err(ReadinessError::WrongNetwork);
    }
    if !info.synced {
        return Err(ReadinessError::NotSynced);
    }
    Ok(())
}
```

これは実ノードへの接続を行わない**教育用のポート設計**である。実際のRPCクライアントは、この `NodeGateway` を実装するアダプターとして追加する。

## 28.16 Rust単体テスト

前節のコードに以下を続ける。

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct FakeGateway(Result<NodeInfo, GatewayError>);

    impl NodeGateway for FakeGateway {
        fn node_info(&self) -> Result<NodeInfo, GatewayError> {
            self.0.clone()
        }
    }

    #[test]
    fn accepts_synced_devnet() {
        let gateway = FakeGateway(Ok(NodeInfo {
            network: Network::Devnet,
            synced: true,
        }));
        assert_eq!(ensure_devnet_ready(&gateway), Ok(()));
    }

    #[test]
    fn rejects_wrong_network() {
        let gateway = FakeGateway(Ok(NodeInfo {
            network: Network::Mainnet,
            synced: true,
        }));
        assert_eq!(
            ensure_devnet_ready(&gateway),
            Err(ReadinessError::WrongNetwork)
        );
    }

    #[test]
    fn rejects_unsynced_node() {
        let gateway = FakeGateway(Ok(NodeInfo {
            network: Network::Devnet,
            synced: false,
        }));
        assert_eq!(
            ensure_devnet_ready(&gateway),
            Err(ReadinessError::NotSynced)
        );
    }

    #[test]
    fn propagates_transport_error() {
        let gateway = FakeGateway(Err(GatewayError::Timeout));
        assert_eq!(
            ensure_devnet_ready(&gateway),
            Err(ReadinessError::Gateway(GatewayError::Timeout))
        );
    }
}
```

この例は標準ライブラリのみを使うため、独立したCargoライブラリに配置して `cargo test` できる構成である。ただし、**本章執筆時点で実行結果は検証していない**。

## 28.17 実ノード用アダプターの設計

```text
                  Application Test
                         │
                         ▼
                    NodeGateway
                         ▲
                ┌────────┴────────┐
                │                 │
          FakeGateway      KaspaRpcGateway
                │                 │
          Unit Tests        Devnet Node
```

`KaspaRpcGateway` に閉じ込めるもの：

- RPC接続先と認証・TLS設定（利用する方式に応じて）
- リクエスト／レスポンスの型変換
- タイムアウトと再接続
- RPCエラーからドメインエラーへの変換
- ノードのネットワーク・同期状態の確認

SDKの具体的なAPIを本章で固定しないのは、バージョン差を隠して**動かないコードを実装例として掲載しないため**である。対象コミットを固定した後、実APIに合わせた検証済みアダプターを追補する。

## 28.18 テストフィクスチャとデータの独立性

良い統合テストは、実行順に依存しない。

- テストごとに独立したウォレット・UTXOを使用する。
- 共有ノードを使う場合は同時実行による競合を明示する。
- 初期状態を生成する手順を自動化する。
- 一意のテストIDをトランザクションメタデータやログに関連付ける。
- テスト後に残るデータと削除できるデータを区別する。

**危険な例**：前のテストが生成したUTXOを次のテストが暗黙に使用する。テスト単独実行で失敗し、原因追跡が困難になる。

## 28.19 待機条件は時間ではなく状態で書く

次のような固定待機は不安定になりやすい。

```text
sleep 10 seconds
assert transaction accepted
```

代わりに、期限付きで状態を観測する。

```text
start deadline
while before deadline:
    query node state
    if expected condition holds:
        success
    wait bounded interval
fail with diagnostics
```

この方法は、速い環境で無駄に待たず、遅い環境ではタイムアウト理由を説明できる。ただし無期限ポーリングにしない。

## 28.20 ログ・メトリクス・トレース

最低限、次の情報を関連付ける。

| 対象 | 記録する情報 |
|---|---|
| テスト | experiment ID、テストケース、開始・終了時刻 |
| ノード | バージョン、ネットワーク、同期状態、ピア数 |
| RPC | メソッド、所要時間、結果分類 |
| トランザクション | txid、送信時刻、観測状態 |
| P2P | 接続・切断、同期進捗 |
| ストレージ | データ容量、I/Oエラー |

秘密鍵・シード・署名に必要な機密情報をログに出さない。ログを共有する場合はRPCエンドポイントや内部IPの扱いも確認する。

## 28.21 CIへの組み込み

CIでは次の順序で実行する。

```text
Lint / Format
     │
     ▼
Unit Tests
     │
     ▼
Build Test Harness
     │
     ▼
Start Devnet Containers
     │
     ▼
Wait for Readiness
     │
     ▼
Integration Tests
     │
     ▼
Collect Logs / Results
     │
     ▼
Stop Containers / Cleanup
```

`cargo fmt --check`、`cargo clippy`、`cargo test` などの標準的な品質ゲートと、実ノードを必要とする統合テストを分ける。Devnetテストは起動時間やリソース消費が大きいため、Pull Request時と夜間実行で範囲を変えることもできる。

## 28.22 セキュリティ上の注意

1. RPCをインターネットへ無防備に公開しない。
2. DevnetとMainnetの鍵を混用しない。
3. コンテナを不要に特権モードで起動しない。
4. テスト用ネットワークを本番システムから分離する。
5. 外部から受け取ったブロック・RPC応答を信頼済み入力とみなさない。
6. 依存ライブラリとノードイメージのバージョンを固定・監査する。
7. 障害注入の対象が本番ノードでないことを実行前に検証する。

## 28.23 よくある失敗と調査方法

### ノードが起動しない

**確認順序**：バージョン → CLIフラグ → ポート競合 → データディレクトリ → ログ。

### RPCに接続できない

**確認順序**：対象RPC方式 → listenアドレス → コンテナポート公開 → ファイアウォール → クライアント互換性。

### 送信したトランザクションが確認できない

**確認順序**：ネットワーク一致 → ノード同期 → トランザクションの受理結果 → Mempool → マイニング → Virtual State。

### 2ノードで状態が違う

**確認順序**：ピア接続 → 同期状態 → 観測時刻 → DAGの進捗 → コンセンサス状態。

## 28.24 実践課題

### 課題A：単一ノードの健全性確認

1. Devnetノードを起動する。
2. バージョン・ネットワーク・同期状態を記録する。
3. 停止・再起動後に状態が復元されるか確認する。
4. 実験結果をMarkdownで記録する。

### 課題B：2ノードの伝播確認

1. Devnetノードを2台起動する。
2. ピア接続を確認する。
3. ブロック伝播前後の状態を比較する。
4. 片方を停止・再起動して再同期を観測する。

### 課題C：RPCアダプターの実装

1. 使用する `rusty-kaspa` のコミットを固定する。
2. 対応するRPC SDKのバージョンを調べる。
3. `NodeGateway` の実装を作る。
4. ネットワーク不一致・未同期・タイムアウトをテストする。

### 課題D：トランザクションのE2E検証

1. Devnet用のテスト鍵を用意する。
2. テスト用UTXOを準備する。
3. トランザクションを構築・署名・送信する。
4. Mempool観測とコンセンサス受理を分けて記録する。
5. 失敗時の再試行方針を設計する。

## 28.25 理解度チェック

1. 単体テストと統合テストは何を異なる対象としているか。
2. DevnetをMainnetから隔離すべき理由は何か。
3. RPC送信がタイムアウトした場合、トランザクションの未受理を断定できないのはなぜか。
4. イベント通知を唯一の真実として扱うと何が起こり得るか。
5. なぜ固定 `sleep` より期限付き状態待機が適しているか。
6. 2ノード構成でしか観察しにくい問題を3つ挙げよ。
7. `NodeGateway` のような抽象化がテストに役立つ理由は何か。
8. テストの再現性に必要なバージョン情報は何か。
9. 「送信」「Mempool観測」「コンセンサス受理」はどう違うか。
10. 障害注入テストの安全な実行条件を説明せよ。

## 28.26 まとめ

本章の中心は、**ノードを起動することではなく、何を観測し、何を保証できるかを明確にすること**である。

- Devnetは実験用の独立環境であり、本番環境と分離する。
- RPCの成功・トランザクションの観測・コンセンサス上の受理は別々に検証する。
- タイムアウトや再接続を通常の設計対象とする。
- テストハーネスはノード実装への依存をアダプターに閉じ込める。
- 実験条件、ログ、期待値、実測結果を保存して再現可能性を高める。

第27章で構築した教育用ノードと、本章で使用する実際のKaspaノードを比較することで、**プロトコルの概念モデルと実装上の複雑さの差**が見えてくる。

次章からは、より高度なコンセンサス研究とDAGKnightの設計を扱う。

## 28.27 参考資料

- [Kaspa公式サイト](https://kaspa.org/)
- [Kaspa Wiki](https://wiki.kaspa.org/en/home)
- [rusty-kaspa — GitHub](https://github.com/kaspanet/rusty-kaspa)
- [Kaspa Improvement Proposals (KIPs)](https://github.com/kaspanet/kips)
- [Rust公式ドキュメント](https://doc.rust-lang.org/book/)
- [Tokio — Asynchronous Rust Runtime](https://tokio.rs/)
- [Docker Compose ドキュメント](https://docs.docker.com/compose/)

> **技術レビュー時の確認事項**：対象 `rusty-kaspa` コミットでのDevnet起動フラグ、gRPC/wRPCのlistenポート、ピア接続方法、マイニング方法、ウォレットSDK API、同期状態の取得方法、実際のテスト実行結果を検証し、検証済みコマンド・ログ・期待値を追記する。
