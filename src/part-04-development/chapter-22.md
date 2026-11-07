# Chapter 22 — Kaspa RPC, wRPC & Application Integration（Kaspa RPC・wRPCとアプリケーション統合）

<p class="chapter-part">Part IV — Kaspa Development &amp; Implementation（Kaspaの開発と実装）</p>

> **対象読者：** Kaspaノードとバックエンド・Webアプリケーションを接続したい開発者

公式SDKのDevnet実測と、通信をモックに置き換えた教育用モデルの検査は区別する。

## この章で学ぶこと

Kaspaのコンセンサスを理解するだけでは、アプリケーションは動かない。ウォレット、決済サービス、ブロックエクスプローラー、監視ツールは、ノードの状態を取得し、トランザクションを送信し、ネットワークの変化を受信する必要がある。たとえば送信要求がタイムアウトしても、ノードがトランザクションを受理している場合がある。アプリケーションは接続の成否とトランザクションの受理を分けて確認する必要がある。本章では、このようなやり取りをRPCとwRPCでどう扱うかを学ぶ。

学習後には以下を説明できる。

- RPC・wRPC・P2Pの役割の違い
- 接続先ノードの同期状態を確認する理由
- トランザクション送信とコンセンサス受理の違い
- イベント購読・再接続・重複処理の設計
- RPCアダプターを介したアプリケーションの依存関係の整理
- Rustで接続障害を再現するための最小モデル
- `rusty-kaspa` の実装とAPI定義の調査方法

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

> **注意：** 表中の名称は機能分類であり、APIメソッド名の一覧ではない。実際のメソッド・フィールド・エラー型は使用するノードとSDKのバージョンで確認する。

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

基準版のwRPCはBorshとSerdeJsonのエンコーディングを区別する（[server/connection.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/rpc/wrpc/server/src/connection.rs)）。JSON形式であっても、HTTPのJSON-RPCと同じ通信・要求形式だとは限らない。

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

### 22.5.1 基準版の確認に使うAPI

[`GetServerInfoResponse`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/rpc/core/src/model/message.rs) はRPC API version/revision、server version、network_id、has_utxo_index、is_synced、virtual_daa_scoreを持つ。SDKの型や生のJSONの表記とは区別する。

[`get_server_info_call`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/rpc/service/src/service.rs) のis_syncedはMiningRuleEngineによるsinkの新しさ・接続に関する判定であり、ネットワーク全体の正しさや全インデックスの完了を証明する値ではない。`get_utxos_by_addresses_call` はUTXO indexが無効ならNoUtxoIndexを返す。返却順が要求アドレス順とは限らないため、配列の位置だけでアドレスと対応付けない。

基準版の [`NetworkId`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/network.rs) はnetwork typeとsuffixを組み合わせる。Testnetという種別だけでなく、testnet-10等の番号も照合する。応答の鮮度やAPI互換性は、アダプター・呼び出し元で別途検査する。

## 22.6 ネットワークと環境の分離

開発環境で使ったアドレスやトランザクションをmainnetで扱わないように、接続設定を明示的に分ける。

```text
config/
  devnet.toml
  testnet.toml
  mainnet.toml
```

設定には少なくとも、完全なNetworkId、接続先、タイムアウト、再試行上限、必要な機能を含める。秘密情報が必要な環境では、設定ファイルをGitへコミットしない。

## 22.7 RPCの代表的な利用フロー

### 22.7.1 ノードの稼働確認

```text
Start
  |
  v
Connect RPC
  |
  v
Read network / sync / capabilities
  |
  +-- wrong NetworkId --> Fail closed
  |
  +-- stale response --> Not ready
  |
  +-- not synced --> Degraded / Wait
  |
  +-- missing required index --> Not ready
  |
  v
Ready
```

ここでの `Ready` は、TCP接続が確立しただけではなく、**アプリケーションが必要とする条件を満たした状態**を意味する。判断時点の観測なので、切断やノード状態の変化で見直す。Readyを通過した後の各RPC呼び出しでもエラー処理を続ける。

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

WebSocket通知は、アプリケーションの永続的なイベントログと同義ではない。切断中の通知を再生できるかどうかは、APIの仕様に依存する。再接続・再購読が成功しても、切断中のすべての通知が再生されたとの保証にはならない。

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

通知は重複も欠落も起こり得る。少なくとも1回届くという保証を置かず、再照会・補完で状態を照合し、重複しても結果が変わらない反映を設計する。RPCの現在状態から分かる範囲と、切断中の全履歴を復元することも別である。照会の間に作成され、消費された出力は、現在UTXOの完全照会だけでは復元できない。売上・会計には取引履歴や受理状態の別記録が必要になる。

## 22.10 冪等性（Idempotency）

同じイベントを2回受信しても、入金を2回加算してはいけない。

例として、アプリケーションDBに次の一意キーを設ける。

```text
(network_id, transaction_id, output_index)
```

これは現在の出力を識別するキーであり、すべての通知を永久に重複排除するイベントIDではない。追加→削除→再追加が起こり得るため、event_kindを足した同じキーを一度処理済みにしただけでは、正当な再追加まで捨ててしまう。現在状態のupsert/無効化、評価世代、変更履歴を分け、DB更新をトランザクションでまとめる。

ただし、キーの選び方はドメインに依存する。たとえば再編成によって「受理された」「受理されなくなった」が変化する場合、単にトランザクションIDだけで全イベントを同一視してはいけない。

**不変条件：同じ受理済み出力に対して、残高加算を重複実行しない。**

## 22.11 Reorganizationへの対応

BlockDAG上で観測された状態は、コンセンサスの進行に伴って変化し得る。アプリケーションは「一度見たトランザクションは永久に受理済み」という前提を置かない。

推奨される考え方は次のとおり。

1. **観測**：トランザクションやUTXOの存在を認識する。
2. **受理状態の確認**：ノードが報告するコンセンサス状態を確認する。
3. **業務上の反映**：リスクに応じた独自ポリシーで入金や配送を確定する。
4. **再評価**：再編成や同期異常が起きたら状態を見直す。

KaspaのFinality関連パラメータと、サービス独自の入金確定条件は別物である（[第20章](../part-03-transactions/chapter-20.md)参照）。

## 22.12 RPCエラーの分類

| エラー | 例 | 推奨対応 |
|---|---|---|
| 通信障害 | タイムアウト、接続拒否 | 読み取りは上限付き再試行。送信は成否を確認 |
| 一時的な利用不可 | ノード起動中、未同期 | Readinessを落とす |
| 入力不正 | 不正なアドレス・トランザクション | 再試行せず修正 |
| 業務競合 | UTXOが既に使用済み | 状態・以前の送信成否を確認してから再選択・再構築 |
| プロトコル不一致 | 型やメソッドの不一致 | バージョン調査 |
| 不明な送信結果 | 応答前に切断 | **再送前に状態確認** |

特に送信処理でタイムアウトした場合、ノードがトランザクションを受理したかどうかは不明である。**タイムアウト＝未送信ではない**。

## 22.13 リトライ・バックオフ・サーキットブレーカー

一時的な障害には指数バックオフを用いる。

```text
retry_delay = min(base_delay * 2^attempt, max_delay)
```

実運用ではジッターを加え、複数クライアントの再試行が同時に集中するのを避ける。入力不正など、再試行しても解決しないエラーは除外する。

RPCの再試行は**副作用のない読み取り**と**トランザクション送信**を区別して設計する。送信成否が不明なまま新しい取引を作ると、別の支払いとして二重に送る危険がある。同じ署名済み取引の再送、Mempool・UTXO・受理IDの照会、新規取引の構築を分ける。

バックオフは [RFC 6455 §7.2.3](https://www.rfc-editor.org/rfc/rfc6455#section-7.2.3) の異常切断からの復旧も参照する。指数の計算自体がオーバーフローしないよう、実装ではattempt・遅延・総待機時間を制限する。

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

本書とは別の作業用ディレクトリで `cargo new --lib --edition 2021 kaspa-rpc-boundary-demo` を実行し、以下へ置き換える。Rust 1.90.0で確認した。

`Cargo.toml`：

```toml
[package]
name = "kaspa-rpc-boundary-demo"
version = "0.1.0"
edition = "2021"
```

`src/lib.rs`：

<details class="code-details">
<summary>RPCアダプターモデルの実装とテストを見る</summary>

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet(u32),
    Simnet,
    Devnet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeStatus {
    pub network: Network,
    pub synced: bool,
    pub fresh: bool,
    pub has_utxo_index: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RpcError {
    Unavailable,
    WrongNetwork,
    NotSynced,
    StaleStatus,
    MissingUtxoIndex,
}

pub trait NodeGateway {
    fn status(&self) -> Result<NodeStatus, RpcError>;
}

pub fn check_readiness(
    gateway: &impl NodeGateway,
    expected: Network,
    require_utxo_index: bool,
) -> Result<(), RpcError> {
    let status = gateway.status()?;
    if status.network != expected {
        return Err(RpcError::WrongNetwork);
    }
    if !status.fresh {
        return Err(RpcError::StaleStatus);
    }
    if !status.synced {
        return Err(RpcError::NotSynced);
    }
    if require_utxo_index && !status.has_utxo_index {
        return Err(RpcError::MissingUtxoIndex);
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
            fresh: true,
            has_utxo_index: true,
        }));
        assert_eq!(check_readiness(&node, Network::Devnet, true), Ok(()));
    }

    #[test]
    fn reject_wrong_network() {
        let node = StubNode(Ok(NodeStatus {
            network: Network::Mainnet,
            synced: true,
            fresh: true,
            has_utxo_index: true,
        }));
        assert_eq!(
            check_readiness(&node, Network::Devnet, true),
            Err(RpcError::WrongNetwork)
        );
    }

    #[test]
    fn reject_unsynced_node() {
        let node = StubNode(Ok(NodeStatus {
            network: Network::Devnet,
            synced: false,
            fresh: true,
            has_utxo_index: true,
        }));
        assert_eq!(
            check_readiness(&node, Network::Devnet, true),
            Err(RpcError::NotSynced)
        );
    }

    fn healthy(network: Network) -> NodeStatus {
        NodeStatus {
            network,
            synced: true,
            fresh: true,
            has_utxo_index: true,
        }
    }

    #[test]
    fn testnet_suffix_must_match() {
        let node = StubNode(Ok(healthy(Network::Testnet(11))));
        assert_eq!(
            check_readiness(&node, Network::Testnet(10), true),
            Err(RpcError::WrongNetwork)
        );
        assert_eq!(check_readiness(&node, Network::Testnet(11), true), Ok(()));
    }

    #[test]
    fn utxo_index_is_required_only_when_requested() {
        let node = StubNode(Ok(NodeStatus {
            has_utxo_index: false,
            ..healthy(Network::Devnet)
        }));
        assert_eq!(
            check_readiness(&node, Network::Devnet, true),
            Err(RpcError::MissingUtxoIndex)
        );
        assert_eq!(check_readiness(&node, Network::Devnet, false), Ok(()));
    }

    #[test]
    fn stale_status_is_not_ready() {
        let node = StubNode(Ok(NodeStatus {
            fresh: false,
            ..healthy(Network::Devnet)
        }));
        assert_eq!(
            check_readiness(&node, Network::Devnet, true),
            Err(RpcError::StaleStatus)
        );
    }

    #[test]
    fn readiness_is_rechecked_after_changes() {
        use std::{cell::RefCell, collections::VecDeque};
        struct ChangingNode(RefCell<VecDeque<Result<NodeStatus, RpcError>>>);
        impl NodeGateway for ChangingNode {
            fn status(&self) -> Result<NodeStatus, RpcError> {
                self.0
                    .borrow_mut()
                    .pop_front()
                    .unwrap_or(Err(RpcError::Unavailable))
            }
        }
        let node = ChangingNode(RefCell::new(VecDeque::from([
            Ok(healthy(Network::Devnet)),
            Ok(NodeStatus {
                synced: false,
                ..healthy(Network::Devnet)
            }),
            Err(RpcError::Unavailable),
            Ok(healthy(Network::Devnet)),
        ])));
        for expected in [
            Ok(()),
            Err(RpcError::NotSynced),
            Err(RpcError::Unavailable),
            Ok(()),
        ] {
            assert_eq!(check_readiness(&node, Network::Devnet, true), expected);
        }
        assert!(node.0.borrow().is_empty());
    }

    #[test]
    fn propagate_transport_failure() {
        let node = StubNode(Err(RpcError::Unavailable));
        assert_eq!(
            check_readiness(&node, Network::Devnet, true),
            Err(RpcError::Unavailable)
        );
    }
}
```

</details>

**実行手順：**

```bash
cargo test
```

本文から抽出した教材は8テスト成功（`experiments/chapter-22-review-2026-11-01.md`）。Testnetの番号、必要な機能、古い応答、Readyの後の未同期・切断・回復も検査した。

型は独自モデルであり、freshはアダプターが判断した鮮度を表す。実際のGetServerInfoResponseにfreshフィールドがあるという意味ではない。freshを算出する時計・TTLの実装は教材に含めない。この関数は取得時刻・TLS・ノードの正当性・インデックスの追従完了を証明しない。

この例の目的は、**ネットワーク接続の詳細をビジネスロジックから切り離す**ことにある。SDKへの依存を `NodeGateway` の実装へ集約できるが、APIの意味や保証が変わった場合はアプリケーション側の要件も見直す。

## 22.18 実際のKaspa RPCへ接続する際の手順

実装時は次の順序で調べる。

1. 利用する `rusty-kaspa` とSDKのバージョンを固定する。
2. 公式リポジトリのRPC実装とサンプルを確認する。
3. 使用するネットワークとRPC接続先を決める。
4. 最初はネットワーク情報・同期状態などの読み取りから始める。
5. UTXO照会を追加し、期待する状態との差を確認する。
6. Devnetで署名済みトランザクションの送信と状態追跡を試す。
7. 通知購読を追加し、切断・再接続・重複を試験する。

> **注意：** ポート番号、APIメソッド名、データ型、通知名はリリースや起動オプションによって異なる可能性がある。本章の照合対象は冒頭の固定版である。別の版へ適用する前に、使用する版の公式実装・SDKとの互換性を確認する。

### 実測したAPIへの入口

まず手を動かす場合は、[第28章28.6.4の公式SDK監視サンプル](chapter-28.md#2864-公式sdkを使った最小監視サンプル)へ進む。収録したNode.jsプログラムで、Devnetへの接続・情報取得・UTXO通知受信・再接続と再照会を実行できる。本章のRust教材で分けた責務を、実際のSDK呼び出しへ対応付けて説明している。

[第28章](chapter-28.md)の28.6〜28.10は、基準版1.0.1で起動・読み取り・送金を再現する手順である。API名と応答の要点は[付録D](../appendices/appendix-d.md)に整理した。

生のJSON wRPCでは `getBlockDagInfo` を使う。公式Node.js/WASM SDKではJSON接続に `Encoding.SerdeJson` を指定し、UTXO購読には `subscribeUtxosChanged([address])` を使う。生RPCとSDKで名前・引数・UTXOエントリの形が同一とは限らない。

実習の `send.cjs --retry-checks` は、ノードが送信を受理した後に成功応答だけを失わせ、TX IDによる照会と同一署名済みTXの再送を検査する。`live-reconciliation.py` は実通知を再照会のヒントとして扱い、欠落補完と古い照会結果の拒否を確認する。これらは単一の観測ストアの検査であり、会計台帳の実装ではない。

## 22.19 `rusty-kaspa` のコードベースを読む

調査対象の入口：

- [rusty-kaspa（公式リポジトリ）](https://github.com/kaspanet/rusty-kaspa)
- [固定版のRPC型](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/rpc/core/src/model/message.rs)
- [Kaspa公式ドキュメント](https://docs.kaspa.org/)

ローカルでは次のように調べる。

```bash
# 第1章で取得した固定版の親ディレクトリから実行する
cd rusty-kaspa-v1.0.1
git rev-parse HEAD
grep -n -E 'GetServerInfoResponse|GetSyncStatusResponse' rpc/core/src/model/message.rs
grep -n -E 'get_server_info_call|get_utxos_by_addresses_call|submit_transaction_call' rpc/service/src/service.rs
grep -n -E 'Borsh|SerdeJson' rpc/wrpc/server/src/connection.rs
```

未取得なら[第1章](../part-01-foundations/chapter-01.md)の手順を使い、HEADを本章冒頭の完全SHAと照合する。検索語は固定版の探索の入口である。見つかった実装から、公開API、通知経路、エラー処理、コンセンサスへの依存関係をたどる。

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

<details class="exercise-details">
<summary>実践課題を見る</summary>

以下は**手順の設計課題**である。起動・送金・通知補完の実測手順は第28章と `experiments/devnet/README.md` を参照し、実習で扱わないケースは追加課題として区別する。

1. Devnetノードを起動し、RPCのリッスン設定を確認する。
2. SDKからノードのネットワーク種別を取得する。
3. 同期状態を表示するヘルスチェックを実装する。
4. 署名済みトランザクションを送信し、結果を記録する。
5. トランザクションの状態をポーリングで追跡する。
6. wRPC通知を再照会のきっかけとして追加する。取引の受理状態は対応するRPCで確認し、通知の受信だけで受理・確定と判断しない。通知が欠落しても状態を補完できるよう、定期的な照会も組み合わせる。
7. 接続を意図的に切り、復旧後に再照会する。
8. 同じ通知を2回与え、DB更新が冪等であることを確認する。

実験記録にはGitコミットID、完全なNetworkId、公開可能な設定・SDK版・集計を残す。生ログ・接続認証情報・秘密鍵・シード・DB・内部パスは公開しない。

</details>

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

<details class="misconception-details">
<summary>よくある誤解を見る</summary>

**1. RPCが成功したら送金は完了？**

いいえ。送信処理の成功とコンセンサスでの受理、サービス独自の確定判定は別である。

**2. wRPCなら通知の取りこぼしは起きない？**

いいえ。切断やプロセス停止があるため、再照会・再処理を設計する必要がある。

**3. RPCはP2Pと同じ通信？**

いいえ。RPCはアプリケーションとノードの境界、P2Pはノード同士の境界である。

**4. ブラウザから公開RPCに直接接続すればバックエンドは不要？**

読み取り中心のアプリでは可能な場合がある。ただし認証、業務ルール、入金管理、レート制限、可用性などを考慮して決める。

**5. 一つのノードの返答は常に最新？**

いいえ。同期状態や接続状況、観測時点に依存する。

</details>

## 22.24 理解度チェック

<details class="quiz-answers">
<summary>理解度チェックを見る</summary>

1. RPCとP2Pの責務の違いを説明せよ。
2. gRPCとwRPCの利用場面を比較せよ。
3. RPCの応答が正常でも入金処理を止めるべき状況を2つ挙げよ。
4. トランザクション送信がタイムアウトしたとき、成否不明のまま別の取引を再構築して送る危険と、同一署名済み取引の再送との違いを述べよ。
5. 通知購読が切断された後、どのように状態を回復するか設計せよ。
6. 同じ入金通知を2回処理しても残高が二重加算されない仕組みを考えよ。
7. `NodeGateway` のようなインターフェースを置くメリットを説明せよ。
8. Kaspaのコンセンサス上の事実と決済サービスの業務判断を分けて説明せよ。

**解答例**

1. RPCは外部アプリがノードへ照会・提出・購読する境界、P2Pはノード同士のデータ伝播・同期を担う。
2. gRPCは対応するクライアント・型定義を使うサービス間連携等、wRPCはWebSocketを使うブラウザ・WASM等の接続に適する。実際に使うノード版・SDK・公開エンドポイントの対応を確認する。
3. 一例は接続先NetworkIdが違う場合と、ノード未同期または必要なindexが未追従の場合。HTTPやRPCの成功だけでは入金判定の前提を満たさない。
4. 最初の取引が既に受理されている可能性がある。別取引を作ると二重の業務処理や競合を招き得る。同一署名済み取引の再送は同じTX IDを扱い、照会・重複応答と合わせて成否を整理できるが、成功を保証するものではない。
5. 再接続時に前提を確認し、再購読を確立して現在状態を再照会する。通知との競合・重複を扱い、必要なら保存したチェックポイント・履歴を使って再照合する。
6. NetworkIdとOutpoint等に一意制約を置き、観測の更新と業務上の計上を分離する。計上済み識別子を保存し、関連更新をDBトランザクションでまとめる。
7. SDKの詳細と業務ロジックを分離し、モックや実アダプターを差し替えて、失敗・再接続等の条件をテストしやすくする。
8. 合意上の事実はノードの規則で得られる取引受理・UTXO状態等。業務判断は金額・経過・リスク等に基づく入金反映条件であり、RPC成功や単なる観測とは分ける。

</details>

## 22.25 まとめ

<details class="summary-details">
<summary>まとめを見る</summary>

Kaspaアプリケーションの設計では、**ノードが提供する事実**と**アプリケーションが下す判断**の境界が重要になる。

- RPCはノードの機能を外部へ公開する境界である。
- wRPCは継続的な通知に適しているが、切断・重複・欠落への備えが必要である。
- RPCの成功はコンセンサス受理やファイナリティを意味しない。
- ノードの同期状態とネットワーク種別を必ず確認する。
- RPCアダプターを分離すると、SDK変更や障害テストに対応しやすい。
- 通知をきっかけに状態を再照会し、冪等に反映する設計が堅牢である。

</details>

[次章](chapter-23.md)では、KaspaのRust SDKとWASMを使った実装へ進む。

## 22.26 参考資料

<details class="reference-details">
<summary>参考資料を見る</summary>

- [Kaspa Documentation](https://docs.kaspa.org/)
- [kaspanet/rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)
- [Kaspa Wiki](https://wiki.kaspa.org/en/home)
- [gRPC Introduction](https://grpc.io/docs/what-is-grpc/introduction/)
- [RFC 6455 — The WebSocket Protocol](https://www.rfc-editor.org/rfc/rfc6455)
- [RFC 9110 — HTTP Semantics](https://www.rfc-editor.org/rfc/rfc9110)

主要な状態照会・UTXO照会・wRPCエンコーディングと、独自教材の確認記録は `experiments/chapter-22-review-2026-11-01.md`。全RPC/SDK・自動再接続・欠落履歴の完全復元・TLS/認証・複数ワーカーの会計台帳は今回検証していない。モック教材は実際のRPCクライアントではなく、今回はノード起動・署名・送金・通知購読を再実行していない。

</details>

<nav class="chapter-next" aria-label="次のページ">

[次章：Chapter 23 — Rust SDK & WASM（Rust SDKとWASM）](chapter-23.md)

</nav>

<details class="verification-details">
<summary>検証状況を見る</summary>

[対象バージョン](../baseline.md)・[このページのレビュー記録](../review-status.md#review-chapter-22)

</details>
