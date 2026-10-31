# Chapter 22 — Kaspa RPC, wRPC & Application Integration（Kaspa RPC・wRPCとアプリケーション統合）

> **Part IV — Kaspa Development & Implementation**  
> 原稿バージョン：v0.1（2026年10月、技術レビュー前）  
> 対象：Kaspaノードとバックエンド・Webアプリケーションを接続したい開発者

## この章で学ぶこと

Kaspaのコンセンサスを理解するだけでは、アプリケーションは動かない。ウォレット、決済サービス、ブロックエクスプローラー、監視ツールは、ノードの状態を取得し、トランザクションを送信し、ネットワークの変化を受信する必要がある。たとえば送信要求がタイムアウトしても、ノードがトランザクションを受理している場合がある。アプリケーションは接続の成否とトランザクションの受理を分けて確認する必要がある。本章では、このようなやり取りをRPCとwRPCでどう扱うかを学ぶ。

学習後には以下を説明できる。

1. RPC・wRPC・P2Pの役割の違い
2. 接続先ノードの同期状態を確認する理由
3. トランザクション送信とコンセンサス受理の違い
4. イベント購読・再接続・重複処理の設計
5. RPCアダプターを介したアプリケーションの依存関係の整理
6. Rustで接続障害を再現するための最小モデル
7. `rusty-kaspa` の実装とAPI定義の調査方法

---

## 22.1 Kaspaアプリケーションの全体像

```text
+-------------------------+
| Web UI / Mobile / CLI   |
+------------+------------+
             |
             v
+-------------------------+
| Application Backend     |
| - business rules        |
| - transaction tracking  |
| - idempotency           |
+------------+------------+
             |
       RPC / wRPC
             |
             v
+-------------------------+
| Kaspa Node (kaspad)     |
| - RPC interface         |
| - mempool               |
| - consensus             |
| - UTXO / BlockDAG       |
+------------+------------+
             |
          P2P Network
             |
        Other Nodes
```

ここで重要なのは、**アプリケーションがコンセンサスを独自に決定しない**ことである。アプリケーションは、ノードが提供する情報を読み取り、独自のビジネスルール（入金反映条件、ユーザー認証、再試行ポリシーなど）を適用する。

RPCは「ノードに何を問い合わせるか」という境界であり、P2Pは「ノード同士がどう情報を交換するか」という境界である。両者を混同すると、接続先や認証の設計を誤る。

## 22.2 RPCとは何か

Remote Procedure Call（RPC）は、別プロセスの機能をネットワーク越しに呼び出すための仕組みである。KaspaノードのRPCでは、例えば次の種類の操作が必要になる。

| 種類 | 目的 | アプリケーション上の利用例 |
|---|---|---|
| ネットワーク情報 | 接続先の状態を把握 | 起動時のヘルスチェック |
| BlockDAG情報 | コンセンサスの進行状況を把握 | ダッシュボード |
| UTXO照会 | 特定アドレスに関連する未使用出力を取得 | ウォレット残高表示 |
| トランザクション送信 | 署名済みトランザクションをノードに提出 | 送金 |
| 通知購読 | 状態変化を継続的に受信 | 入金検知 |

**注意：** 表中の名称は機能分類であり、APIメソッド名の一覧ではない。実際のメソッド・フィールド・エラー型は使用するノードとSDKのバージョンで確認する。

## 22.3 gRPCとwRPCの違い

`rusty-kaspa` のエコシステムでは、ノードとの接続方法としてgRPCとwRPCが重要になる。

| 観点 | gRPC | wRPC |
|---|---|---|
| 主な通信基盤 | HTTP/2 + Protocol Buffers | WebSocketベースのRPC |
| 主な用途 | サーバー間通信・ネイティブクライアント | Webアプリ・SDK・継続接続 |
| データ形式 | Protobuf | 実装・設定に応じたシリアライズ形式 |
| イベント通知 | ストリーミング等 | WebSocket接続上の通知 |
| 注意点 | クライアント生成・プロトコル互換性 | 切断・再接続・購読の復旧 |

wRPCは単に「gRPCをWebSocketで包んだもの」と考えるべきではない。APIの型や通知の扱いを、対象バージョンの実装で確認する。

また、**HTTPのJSON-RPCがKaspaノードの標準インターフェースだと決めつけない**。BitcoinのRPC経験をそのまま持ち込まないことが大切である。

## 22.4 RPCとP2Pの責務境界

```text
App --(RPC request)--> Node A
                         |
                         +-- local query: DAG / UTXO / mempool
                         |
                         +-- transaction submission
                                  |
                                  v
                           P2P propagation
                                  |
                                  v
                               Node B
```

RPCの応答は通常、**接続したノードの観測結果**を表す。ネットワーク全体での最終的な結果を保証するものではない。たとえばトランザクション送信に成功しても、後からコンセンサスで受理されることは別途確認する必要がある。

## 22.5 接続先ノードの信頼性

アプリケーションがRPCの値を使う前に、少なくとも次を確認したい。

1. 想定したネットワーク（mainnet、testnet、devnetなど）か。
2. ノードが十分に同期しているか。
3. RPCの応答が古くないか。
4. 接続先が信頼できる運用主体か。
5. 必要なインデックス・機能が有効か。

特に**「RPCが応答する」ことと「正しい最新状態を返す」ことは違う**。起動直後やIBD中のノードは、通信できても業務判断に使うべきではない場合がある。

## 22.6 ネットワークと環境の分離

開発環境で使ったアドレスやトランザクションをmainnetで扱わないように、接続設定を明示的に分ける。

```text
config/
  devnet.toml
  testnet.toml
  mainnet.toml
```

設定には少なくとも、ネットワーク種別、接続先、タイムアウト、再試行上限、必要な機能を含める。秘密情報が必要な環境では、設定ファイルをGitへコミットしない。

## 22.7 RPCの代表的な利用フロー

### 22.7.1 ノードの稼働確認

```text
Start
  |
  v
Connect RPC
  |
  v
Read network / sync status
  |
  +-- mismatch --> Fail closed
  |
  +-- not synced --> Degraded / Wait
  |
  v
Ready
```

ここでの `Ready` は、TCP接続が確立しただけではなく、**アプリケーションが必要とする条件を満たした状態**を意味する。

### 22.7.2 UTXOの取得

ウォレットはアドレスに対応するUTXOを照会し、入力として利用する候補を選択する。ただし、複数の送金が同時に走る場合、**同じUTXOを重複して選ばない**ためのローカルな予約管理も必要になる。

### 22.7.3 トランザクションの送信

```text
Build transaction
       |
       v
Sign transaction
       |
       v
Submit via RPC
       |
       v
Track transaction status
       |
       v
Observe consensus acceptance
       |
       v
Apply application policy
```

送信直後のレスポンスは「送信処理に対する応答」であって、「不可逆な確定」の証明ではない。

## 22.8 wRPCによる通知購読

WebSocketによる通知は、ブロックやUTXOなどの変化をリアルタイムに扱う際に便利である。しかし、継続接続は必ず切断され得る。

```text
DISCONNECTED
     |
     v
 CONNECTING
     |
     v
 CONNECTED
     |
     v
 SUBSCRIBED
     |
     +---- connection lost ----> DISCONNECTED
     |
     +---- resubscribe --------> SUBSCRIBED
```

設計上は次の区別が必要になる。

- **イベント受信**：ノードから通知が届いた。
- **イベント処理**：アプリケーションが通知を検証した。
- **状態反映**：データベースに結果を保存した。

この3段階を単一の操作として扱うと、再接続時の重複や欠落を見逃しやすい。

## 22.9 イベントの重複と欠落

WebSocket通知は、アプリケーションの永続的なイベントログと同義ではない。切断中の通知を再生できるかどうかは、APIの仕様に依存する。

そのため、**通知は更新のきっかけとし、必要に応じてRPCで現在状態を再照会する**設計が堅牢である。

```text
Notification
    |
    v
Enqueue reconciliation job
    |
    v
Query current node state
    |
    v
Compare persisted state
    |
    v
Idempotent update
```

「少なくとも1回処理される」ことを前提に、重複しても結果が変わらない処理を目指す。

## 22.10 冪等性（Idempotency）

同じイベントを2回受信しても、入金を2回加算してはいけない。

例として、アプリケーションDBに次の一意キーを設ける。

```text
(network, transaction_id, output_index, event_kind)
```

ただし、キーの選び方はドメインに依存する。たとえば再編成によって「受理された」「受理されなくなった」が変化する場合、単にトランザクションIDだけで全イベントを同一視してはいけない。

**不変条件：同じ受理済み出力に対して、残高加算を重複実行しない。**

## 22.11 Reorganizationへの対応

BlockDAG上で観測された状態は、コンセンサスの進行に伴って変化し得る。アプリケーションは「一度見たトランザクションは永久に受理済み」という前提を置かない。

推奨される考え方は次のとおり。

1. **観測**：トランザクションやUTXOの存在を認識する。
2. **受理状態の確認**：ノードが報告するコンセンサス状態を確認する。
3. **業務上の反映**：リスクに応じた独自ポリシーで入金や配送を確定する。
4. **再評価**：再編成や同期異常が起きたら状態を見直す。

KaspaのFinality関連パラメータと、サービス独自の入金確定条件は別物である（第20章参照）。

## 22.12 RPCエラーの分類

| エラー | 例 | 推奨対応 |
|---|---|---|
| 通信障害 | タイムアウト、接続拒否 | 上限付き再試行 |
| 一時的な利用不可 | ノード起動中、未同期 | Readinessを落とす |
| 入力不正 | 不正なアドレス・トランザクション | 再試行せず修正 |
| 業務競合 | UTXOが既に使用済み | 状態再照会と再構築 |
| プロトコル不一致 | 型やメソッドの不一致 | バージョン調査 |
| 不明な送信結果 | 応答前に切断 | **再送前に状態確認** |

特に送信処理でタイムアウトした場合、ノードがトランザクションを受理したかどうかは不明である。**タイムアウト＝未送信ではない**。

## 22.13 リトライ・バックオフ・サーキットブレーカー

一時的な障害には指数バックオフを用いる。

```text
retry_delay = min(base_delay * 2^attempt, max_delay)
```

実運用ではジッターを加え、複数クライアントの再試行が同時に集中するのを避ける。入力不正など、再試行しても解決しないエラーは除外する。

RPCの再試行は**副作用のない読み取り**と**トランザクション送信**を区別して設計する。

## 22.14 RPCの公開範囲とセキュリティ

ノードのRPCをインターネットへ無条件に公開しない。

- 可能ならループバックやプライベートネットワークへバインドする。
- 必要に応じて認証・TLS・リバースプロキシを設ける。
- 接続元制限、レート制限、タイムアウトを設定する。
- リクエストサイズや同時接続数を制限する。
- ログに秘密鍵、シードフレーズ、署名用秘密情報を記録しない。
- 公開RPCサービスを利用する場合、可用性・プライバシー・利用規約を評価する。

**秘密鍵をRPCノードへ渡すことを標準設計にしない。** 署名は可能な限りウォレット側の責務として分離する。

## 22.15 C4モデルで責務を整理する

```text
[User]
   |
   v
[Frontend]
   |
   v
[Backend API] ------> [Application Database]
   |
   v
[Kaspa RPC Adapter]
   |
   v
[Kaspa Node] <------> [Kaspa P2P Network]
```

バックエンドの内部をさらに分ける。

| コンポーネント | 責務 | 依存先 |
|---|---|---|
| Wallet Service | 送金要求の検証と構築 | UTXO Reader、Signer |
| Transaction Tracker | 送信後の状態追跡 | Node Gateway、DB |
| Deposit Service | 入金の業務判定 | Tracker、DB |
| Node Gateway | RPC呼び出しの抽象化 | Kaspa RPC client |
| Event Consumer | 通知受信と再照会の起動 | wRPC、Job Queue |

この構成なら、ノードのAPI変更がアプリケーション全体へ波及しにくい。

## 22.16 DDDで考える境界

**Kaspaのコンセンサスルール**と**サービスの業務ルール**を同じモデルへ押し込まない。

例：

- Kaspa側の事実：ある出力が現在のコンセンサス状態で存在する。
- 決済サービス側の判断：その出力を入金として計上してよい。

「入金確定」はサービス固有の用語であり、Kaspaの内部状態と必ずしも一対一ではない。境界を明示することで、ポリシー変更や複数ノード対応が容易になる。

## 22.17 Rustによる最小RPCアダプター

ここでは**実ネットワーク接続を行わない教育用モデル**を作り、ノードの未同期・通信障害をアプリケーション側で扱う。実際のKaspa RPCメソッドやSDKの型とは独立している。

`Cargo.toml`：

```toml
[package]
name = "kaspa-rpc-boundary-demo"
version = "0.1.0"
edition = "2021"
```

`src/lib.rs`：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet,
    Devnet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeStatus {
    pub network: Network,
    pub synced: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RpcError {
    Unavailable,
    WrongNetwork,
    NotSynced,
}

pub trait NodeGateway {
    fn status(&self) -> Result<NodeStatus, RpcError>;
}

pub fn check_readiness(
    gateway: &impl NodeGateway,
    expected: Network,
) -> Result<(), RpcError> {
    let status = gateway.status()?;
    if status.network != expected {
        return Err(RpcError::WrongNetwork);
    }
    if !status.synced {
        return Err(RpcError::NotSynced);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubNode(Result<NodeStatus, RpcError>);

    impl NodeGateway for StubNode {
        fn status(&self) -> Result<NodeStatus, RpcError> {
            self.0.clone()
        }
    }

    #[test]
    fn ready_when_network_matches_and_synced() {
        let node = StubNode(Ok(NodeStatus {
            network: Network::Devnet,
            synced: true,
        }));
        assert_eq!(check_readiness(&node, Network::Devnet), Ok(()));
    }

    #[test]
    fn reject_wrong_network() {
        let node = StubNode(Ok(NodeStatus {
            network: Network::Mainnet,
            synced: true,
        }));
        assert_eq!(
            check_readiness(&node, Network::Devnet),
            Err(RpcError::WrongNetwork)
        );
    }

    #[test]
    fn reject_unsynced_node() {
        let node = StubNode(Ok(NodeStatus {
            network: Network::Devnet,
            synced: false,
        }));
        assert_eq!(
            check_readiness(&node, Network::Devnet),
            Err(RpcError::NotSynced)
        );
    }

    #[test]
    fn propagate_transport_failure() {
        let node = StubNode(Err(RpcError::Unavailable));
        assert_eq!(
            check_readiness(&node, Network::Devnet),
            Err(RpcError::Unavailable)
        );
    }
}
```

実行：

```bash
cargo test
```

この例の目的は、**ネットワーク接続の詳細をビジネスロジックから切り離す**ことにある。SDKが変わっても `NodeGateway` の実装だけを差し替えられる。

## 22.18 実際のKaspa RPCへ接続する際の手順

実装時は次の順序で調べる。

1. 利用する `rusty-kaspa` とSDKのバージョンを固定する。
2. 公式リポジトリのRPC実装とサンプルを確認する。
3. 使用するネットワークとRPC接続先を決める。
4. 最初はネットワーク情報・同期状態などの読み取りから始める。
5. UTXO照会を追加し、期待する状態との差を確認する。
6. Devnetで署名済みトランザクションの送信と状態追跡を試す。
7. 通知購読を追加し、切断・再接続・重複を試験する。

**注意：** ポート番号、APIメソッド名、データ型、通知名はリリースや起動オプションによって異なる可能性がある。本章では固定値を断定せず、使用するバージョンの公式実装を一次情報とする。

### 実測したAPIへの入口

[第28章](chapter-28.md)の28.6〜28.10は、基準版1.0.1で起動・読み取り・送金を再現する手順である。API名と応答の要点は[付録D](../appendices/appendix-d.md)に整理した。

生のJSON wRPCでは `getBlockDagInfo` を使う。公式Node.js/WASM SDKではJSON接続に `Encoding.SerdeJson` を指定し、UTXO購読には `subscribeUtxosChanged([address])` を使う。生RPCとSDKで名前・引数・UTXOエントリの形が同一とは限らない。

実習の `send.cjs --retry-checks` は、ノードが送信を受理した後に成功応答だけを失わせ、TX IDによる照会と同一署名済みTXの再送を検査する。`live-reconciliation.py` は実通知を再照会のヒントとして扱い、欠落補完と古い照会結果の拒否を確認する。これらは単一の観測ストアの検査であり、会計台帳の実装ではない。

## 22.19 `rusty-kaspa` のコードベースを読む

調査対象の入口：

- [rusty-kaspa（公式リポジトリ）](https://github.com/kaspanet/rusty-kaspa)
- [rusty-kaspa のRPC関連コード検索](https://github.com/search?q=repo%3Akaspanet%2Frusty-kaspa+rpc&type=code)
- [Kaspa公式ドキュメント](https://docs.kaspa.org/)

ローカルでは次のように調べる。

```bash
git clone https://github.com/kaspanet/rusty-kaspa.git
cd rusty-kaspa
rg 'wRPC|wrpc|RpcApi|GetBlockDAGInfo|SubmitTransaction' .
```

`rg` の検索語は調査の出発点であり、すべての名称が現在のリビジョンに存在する保証はない。見つかった実装から、公開API、通知経路、エラー処理、コンセンサスへの依存関係をたどる。

ソースを読むときは、以下の表を自分で埋めると理解が深まる。

| 問い | 調査結果 |
|---|---|
| RPCの入口はどこか | 実際のファイルパスを記録 |
| メソッドの型はどこで定義されるか | 型定義を記録 |
| ノード同期状態はどこから取得するか | 呼び出し先を記録 |
| 通知購読はどこで管理するか | ライフサイクルを記録 |
| コンセンサス層への依存はどこか | 境界を記録 |

## 22.20 障害調査と可観測性

本番で問題になるのは、単なる「RPCがつながらない」だけではない。

| 症状 | 可能性 | 最初の確認 |
|---|---|---|
| 接続はできるが残高が古い | 未同期・インデックスの遅延 | ノード同期状態 |
| 送信タイムアウト | 応答喪失・一時障害 | トランザクションIDと現在状態 |
| 通知が来なくなった | WebSocket切断・購読消失 | 接続状態・再購読ログ |
| 同じ入金が2回反映 | 重複通知・冪等性不足 | 一意キー・処理履歴 |
| 開発環境だけ失敗 | ネットワーク設定不一致 | mainnet/testnet/devnet |
| ノード切替後に結果が違う | 同期差・観測時点の差 | 各ノードの状態 |

監視する代表的な指標：RPC成功率、p95/p99応答時間、接続数、再接続回数、同期状態、通知からDB反映までの遅延、重複イベント件数、未解決の送信結果件数。

## 22.21 実践課題：Devnetに接続する

以下は**手順の設計課題**である。起動・送金・通知補完の実測手順は第28章と `experiments/devnet/README.md` を参照し、実習で扱わないケースは追加課題として区別する。

1. Devnetノードを起動し、RPCのリッスン設定を確認する。
2. SDKからノードのネットワーク種別を取得する。
3. 同期状態を表示するヘルスチェックを実装する。
4. 署名済みトランザクションを送信し、結果を記録する。
5. トランザクションの状態をポーリングで追跡する。
6. wRPC通知へ切り替え、同じ状態を追跡する。
7. 接続を意図的に切り、復旧後に再照会する。
8. 同じ通知を2回与え、DB更新が冪等であることを確認する。

実験記録には、GitコミットID、ネットワーク種別、ノード起動オプション、SDKバージョン、観測ログを残す。

## 22.22 設計上の不変条件

本章で扱うアプリケーションの不変条件をまとめる。

1. **ネットワーク不一致の接続先から業務状態を更新しない。**
2. **同期不足のノードの応答だけで入金を確定しない。**
3. **同一の入金を重複計上しない。**
4. **送信タイムアウトを未送信と断定しない。**
5. **イベント通知をコンセンサス確定の証拠とみなさない。**
6. **秘密鍵管理とノードRPCを必要以上に結合しない。**
7. **再接続後は購読だけでなく状態の再照会も検討する。**

## 22.23 よくある誤解

**Q. RPCが成功したら送金は完了？**  
A. いいえ。送信処理の成功とコンセンサスでの受理、サービス独自の確定判定は別である。

**Q. wRPCなら通知の取りこぼしは起きない？**  
A. いいえ。切断やプロセス停止があるため、再照会・再処理を設計する必要がある。

**Q. RPCはP2Pと同じ通信？**  
A. いいえ。RPCはアプリケーションとノードの境界、P2Pはノード同士の境界である。

**Q. ブラウザから公開RPCに直接接続すればバックエンドは不要？**  
A. 読み取り中心のアプリでは可能な場合がある。ただし認証、業務ルール、入金管理、レート制限、可用性などを考慮して決める。

**Q. 一つのノードの返答は常に最新？**  
A. いいえ。同期状態や接続状況、観測時点に依存する。

## 22.24 理解度チェック

1. RPCとP2Pの責務の違いを説明せよ。
2. gRPCとwRPCの利用場面を比較せよ。
3. RPCの応答が正常でも入金処理を止めるべき状況を2つ挙げよ。
4. トランザクション送信がタイムアウトしたとき、即時再送が危険な理由を述べよ。
5. 通知購読が切断された後、どのように状態を回復するか設計せよ。
6. 同じ入金通知を2回処理しても残高が二重加算されない仕組みを考えよ。
7. `NodeGateway` のようなインターフェースを置くメリットを説明せよ。
8. Kaspaのコンセンサス上の事実と決済サービスの業務判断を分けて説明せよ。

## 22.25 まとめ

Kaspaアプリケーションの設計では、**ノードが提供する事実**と**アプリケーションが下す判断**の境界が重要になる。

- RPCはノードの機能を外部へ公開する境界である。
- wRPCは継続的な通知に適しているが、切断・重複・欠落への備えが必要である。
- RPCの成功はコンセンサス受理やファイナリティを意味しない。
- ノードの同期状態とネットワーク種別を必ず確認する。
- RPCアダプターを分離すると、SDK変更や障害テストに対応しやすい。
- 通知をきっかけに状態を再照会し、冪等に反映する設計が堅牢である。

次章では、KaspaのRust SDKとWASMを使った実装へ進む。

## 22.26 参考資料

- [Kaspa Documentation](https://docs.kaspa.org/)
- [kaspanet/rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)
- [Kaspa Wiki](https://wiki.kaspa.org/en/home)
- [gRPC Introduction](https://grpc.io/docs/what-is-grpc/introduction/)
- [RFC 6455 — The WebSocket Protocol](https://www.rfc-editor.org/rfc/rfc6455)
- [RFC 9110 — HTTP Semantics](https://www.rfc-editor.org/rfc/rfc9110)

> **技術レビュー時の確認事項：** RPCメソッドの正確な名称、gRPC/wRPCの利用可能なエンコーディング、購読可能な通知、デフォルトポート、同期状態の意味、取引送信のエラー仕様は、対象とする `rusty-kaspa` のコミット・SDKバージョンで照合すること。ここでのRustコードは教育用の抽象化であり、実際のKaspa RPCクライアントではない。
