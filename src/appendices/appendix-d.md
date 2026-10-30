# Appendix D — RPC・CLIリファレンス

> **Mastering Kaspa — 付録D**  
> 原稿ステータス：**v0.1（技術レビュー前）**  
> 対象：Kaspaノードの操作、RPC・wRPC、CLI、Devnetでの観測と検証  
> **重要：** KaspaのRPCインターフェース、メソッド名、引数、ポート、CLIオプションはリリースやネットワーク設定によって異なる。本付録のコマンドは、特に断りがない限り**学習用の例**であり、実環境での動作は未検証である。実行前に使用バージョンのヘルプと一次資料を確認する。

---

## D.1 この付録の目的

第10章でノード、第22章でRPC・wRPC、第28章でDevnet、第35章でアプリケーション統合を学んだ。本付録は、それらを横断して参照できる**操作・調査・障害対応の手引き**である。

この付録を終えると、次の問いに自分で答えられるようになる。

1. ノードがどのネットワークに接続しているか。
2. 同期が完了しているか。
3. ブロックやトランザクションをどう調べるか。
4. アドレスのUTXOをどう観測するか。
5. 通信断や重複通知にアプリケーション側でどう対処するか。
6. 実験結果を他人が再現できるようにどう記録するか。

**基本原則：読み取り・観測を先に、送信・変更を後に。**

## D.2 全体構成：CLI・RPC・wRPC

```text
                  開発者
                    │
       ┌────────────┼────────────┐
       ▼            ▼            ▼
   kaspad CLI    RPC Client   wRPC Client
   (起動設定)    (要求/応答)   (要求/応答・通知)
       │            │            │
       └────────────┴────────────┘
                    │
                 kaspad
                    │
       ┌────────────┼────────────┐
       ▼            ▼            ▼
     P2P同期    Consensus     UTXO Index
```

| 要素 | 役割 | 注意点 |
|---|---|---|
| `kaspad` | Kaspaフルノード | 起動引数はバージョン依存 |
| `kaspa-cli` 等 | ノードの情報取得・操作 | 配布物と対応RPCを確認 |
| RPC | ノードに要求を送って結果を取得 | gRPCなど複数の実装形態がある |
| wRPC | WebSocket経由のRPC・イベント購読 | エンドポイントとSDKの組み合わせを確認 |
| P2P | ノード間のブロック・取引伝播 | アプリ向けRPCとは別 |
| UTXO Index | アドレスに関連するUTXOの照会を補助 | 起動時の有効化が必要な構成がある |

**RPCは「ノードが何を知っているか」を問い合わせる手段であり、ウォレットの秘密鍵管理やアプリケーションの業務ルールを代行しない。**

## D.3 作業前の安全確認

- Mainnetではなく、まず**TestnetまたはDevnet**を使用する。
- 秘密鍵・シードフレーズ・認証情報をコマンド履歴、ログ、Gitに残さない。
- RPCポートを無条件にインターネットへ公開しない。
- 破壊的操作や送金の前に、ネットワーク識別子・宛先・金額・手数料を確認する。
- DevnetのデータディレクトリはMainnetと分離する。
- 公開APIはレート制限、可用性、プライバシーの制約を持つ。学習用の本番依存先として固定しない。

## D.4 実験環境の記録

動作確認の前に、環境を記録する。

```bash
uname -a
rustc --version
cargo --version
docker --version
docker compose version
git --version
```

ノードのソースを取得した場合：

```bash
git -C rusty-kaspa rev-parse HEAD
git -C rusty-kaspa status --short
```

`rusty-kaspa`というディレクトリ名はローカルでの例。実際の配置先に読み替える。

| 項目 | 記録例 |
|---|---|
| OS | macOS / Linux とバージョン |
| ノード | リリース番号、Dockerイメージのdigest、またはGit commit |
| ネットワーク | Mainnet / Testnet / Devnet と識別子 |
| RPC | gRPC / wRPC、ホスト、ポート、認証設定 |
| UTXO Index | 有効 / 無効 |
| 同期状態 | IBD中 / 同期済み / 不明 |
| 観測時刻 | UTCの日時 |

## D.5 一次資料からコマンドを確定する

**公式リポジトリと、実行中のバイナリのヘルプを優先する。**

```bash
# リポジトリのREADMEやリリースノートを確認する
git clone https://github.com/kaspanet/rusty-kaspa.git
cd rusty-kaspa

# 利用可能な実行ターゲットを調べる
cargo metadata --no-deps --format-version 1

# 実際にビルドしたバイナリのヘルプを確認する（例）
# ./target/release/kaspad --help
```

上記の `cargo metadata` はCargoの情報を読むコマンドであり、ノードを起動しない。バイナリ名や配置場所はリリース構成による。

**確認する順序**

1. リリース番号とGit commitを特定する。
2. そのバージョンのREADME、CLIヘルプ、RPCスキーマを読む。
3. 接続するネットワークとRPCエンドポイントを確認する。
4. 読み取り専用メソッドから試す。
5. 結果とログを保存する。

## D.6 ノードの起動・停止・ログ

以下は**起動方法を考えるための概念例**である。`--devnet`、`--utxoindex`、`--rpclisten` などの引数の有無や表記は、対象バージョンで `--help` を確認すること。

```bash
# 疑似的な起動イメージ。コピー実行する前に引数を検証する。
kaspad --devnet --utxoindex --rpclisten 127.0.0.1:<RPC_PORT>
```

`<RPC_PORT>` はプレースホルダーであり、シェルへそのまま入力しない。

Dockerを利用する場合の**確認用コマンド**：

```bash
docker ps
docker logs --tail 100 <CONTAINER_NAME>
docker logs -f <CONTAINER_NAME>
docker inspect <CONTAINER_NAME>
docker stop <CONTAINER_NAME>
```

`<CONTAINER_NAME>` は実在するコンテナ名に置換する。停止操作は実験用コンテナだけを対象にする。

## D.7 RPC接続の基本

RPCクライアントは通常、次の情報を必要とする。

```text
ConnectionConfig {
  network: "devnet",
  transport: "grpc | wrpc",
  endpoint: "127.0.0.1:<PORT>",
  timeout: "数秒〜数十秒（用途に応じて）"
}
```

これは**説明用のデータ構造**であり、特定SDKの実際の型名ではない。

接続時のチェック項目：

- DNSまたはIPアドレスが正しいか。
- ポートが待ち受けているか。
- HTTP、WebSocket、gRPCの方式が一致しているか。
- TLSの有無が一致しているか。
- ノードとクライアントのRPCスキーマが互換か。
- ネットワークが一致しているか。

## D.8 主要な読み取り系RPC

以下はKaspaのRPCでよく参照される**代表的なメソッド名**である。利用可能性、要求フィールド、応答形式は必ず使用するバージョンのRPC定義を確認する。

| メソッド名 | 主な用途 | 読むべきポイント |
|---|---|---|
| `getBlockDAGInfo` | DAGの全体情報 | ネットワーク、スコア、同期の目安 |
| `getInfo` | ノード情報 | バージョンや動作状態 |
| `getPeerAddresses` | 接続先候補の調査 | アドレスの取り扱いに注意 |
| `getConnectedPeerInfo` | 接続ピアの情報 | P2Pの接続状況 |
| `getBlock` | ブロック取得 | ブロックハッシュ、詳細指定 |
| `getBlocks` | 複数ブロック取得 | 範囲と取得量の制約 |
| `getVirtualSelectedParentBlueScore` | 仮想選択親のBlue Score | ブロック高とは異なる |
| `getVirtualSelectedParentChainFromBlock` | 選択親チェーンの変化 | 再編成の追跡 |
| `getUtxosByAddresses` | アドレスのUTXO照会 | UTXO Indexの要件 |
| `getBalanceByAddress` | アドレス残高の照会 | UTXO Indexの要件 |
| `getBalancesByAddresses` | 複数アドレスの残高照会 | 大量リクエストに注意 |
| `getMempoolEntry` | Mempool内の取引確認 | 未確定取引であること |
| `getMempoolEntries` | Mempool一覧の取得 | 取得量・メモリ使用量 |
| `getMempoolEntriesByAddresses` | アドレスに関連する未確定取引 | 対応状況を確認 |
| `getFeeEstimate` | 手数料の推定 | 推定値は保証ではない |
| `getCurrentNetwork` | 接続ネットワークの確認 | 送信前に必須 |
| `getCoinSupply` | 供給量情報 | 単位と意味を確認 |

**名称の存在と、特定クライアントから呼べることは別。** 特にgRPC、wRPC、SDKラッパー間で表記や戻り値が変換される場合がある。

## D.9 ブロック情報の読み方

```text
blockHash
   │
   ├── header
   │    ├── parents
   │    ├── timestamp
   │    ├── blueScore / blueWork
   │    └── その他のコンセンサス関連フィールド
   │
   └── transactions（取得オプション次第）
```

**確認したい問い**

1. このブロックはどの親を参照しているか。
2. DAG上の位置関係はどうなっているか。
3. Blue ScoreとBlue Workを混同していないか。
4. 取引データが省略されていないか。
5. 取得した時点で、仮想状態から見た位置づけはどうか。

ブロックの「取得に成功した」ことだけでは、アプリケーションに必要な確定条件を満たしたことにはならない。

## D.10 UTXO照会の読み方

```text
Address
  └── UTXO 1
       ├── Outpoint (transactionId, index)
       ├── Amount
       ├── ScriptPublicKey
       └── Block / acceptance context
  └── UTXO 2
       └── ...
```

UTXOの識別には、一般に**トランザクションIDと出力インデックス**の組が使われる。残高は、観測時点で該当するUTXOの金額を合計した値として捉えられる。

```text
balance(address) = Σ amount(unspent outputs associated with address)
```

ただし、実際のAPIが返す残高の意味（成熟前の出力、利用可能性、同期時点など）は確認が必要。

**重要：** `getUtxosByAddresses` や残高系メソッドを利用する場合、ノードのUTXOインデックスが必要になる構成がある。単にノードが同期済みでも、必要な照会機能が有効とは限らない。

## D.11 Mempoolと未確定トランザクション

```text
Wallet broadcasts transaction
            │
            ▼
       Node Mempool
            │
       ┌────┴────┐
       ▼         ▼
    Included   Evicted / Rejected
       │
       ▼
 Consensus processing
```

Mempoolに存在する取引は、**ネットワーク全体で受理・確定したことを意味しない**。

| 観測結果 | いえること | いえないこと |
|---|---|---|
| Mempoolにある | そのノードが保持している | 全ノードが保持している |
| Mempoolから消えた | 現時点の一覧にない | 必ず確定した |
| ブロックに含まれる | ブロック内容に存在する | 業務上の確定条件を満たす |
| 通知を受けた | 何らかのイベントを観測した | 一度しか通知されない |

## D.12 通知系RPCと購読

通知・購読は、アプリケーションが状態変化を追跡するために利用する。具体的なメソッド名・通知型はSDKのバージョンに依存する。

代表的な関心対象：

- Virtual Chain Changed
- Block Added
- UTXOs Changed
- Selected Tip Changed（利用可能な通知型は要確認）
- Finality Conflict / Resolution（利用可能な通知型は要確認）

```text
        wRPC Client
             │
             ├── connect
             ├── subscribe
             │       │
             │       ▼
             │    notifications
             │       │
             ├── disconnect / timeout
             │       │
             ├── reconnect
             └── resync from durable cursor or state
```

**通知は真実の唯一の保存場所ではない。** 通信断後に取りこぼしたイベントを、RPCで再照会できる設計が必要。

## D.13 購読の疑似コード

次は**SDK非依存の擬似コード**。実際のRust APIではない。

```text
client = connect(endpoint)
assert client.current_network() == EXPECTED_NETWORK

last_checkpoint = database.load_checkpoint()

loop:
    try:
        client.subscribe_utxo_changes(watched_addresses)
        for event in client.events():
            database.transaction:
                if not database.has_seen(event.stable_id):
                    database.store_observation(event)
                database.save_checkpoint(event.position)
    except ConnectionError:
        wait_with_exponential_backoff()
        client = connect(endpoint)
        reconcile_chain_state(last_checkpoint)
```

**注意：** イベントに安定したIDや再開カーソルが存在するかはAPI依存。存在しなければ、取引ID・出力インデックス・観測位置などから**アプリ側の冪等キーと再照会手順**を設計する。

## D.14 送信系RPCと注意事項

送信に関係する代表例として `submitTransaction` がある。ただし、要求オブジェクト、署名形式、手数料、ネットワーク要件はバージョンに依存する。

```text
Transaction creation
       │
       ▼
UTXO selection
       │
       ▼
Fee estimation
       │
       ▼
Signing (wallet / secure signer)
       │
       ▼
Validation before submission
       │
       ▼
submitTransaction
       │
       ▼
Mempool / DAG observation
```

**送信前チェックリスト**

- 接続先は意図したネットワークか。
- 宛先アドレスと金額は正しいか。
- 入力UTXOは現在も利用可能か。
- 手数料の単位と金額は妥当か。
- 署名済みデータに秘密情報が含まれていないか。
- 再送が二重支払いを発生させない設計か。
- エラー時に送信済みか未送信かを再照会できるか。

## D.15 CLI操作の分類

`kaspa-cli` の具体的なコマンド構文はバージョンによって異なるため、ここでは**操作カテゴリ**を示す。

| 操作 | 目的 | 典型的な注意 |
|---|---|---|
| ノード情報取得 | バージョン・同期状態確認 | 接続先が正しいか |
| ネットワーク確認 | Mainnet/Testnet/Devnetの識別 | 送金前に確認 |
| DAG情報取得 | Blue Scoreなどの観測 | 高さとの混同を避ける |
| ブロック取得 | ヘッダー・取引の調査 | 大きなレスポンス |
| UTXO照会 | 入出金の観測 | インデックス要件 |
| Mempool照会 | 未確定取引の確認 | 最終性の保証ではない |
| トランザクション送信 | 署名済み取引を配信 | 資金移動・誤送信に注意 |

CLIの実際のヘルプを確認してから、各カテゴリをローカルの「実行確認済みコマンド表」に転記する。

## D.16 Dockerでのネットワーク確認

```bash
# コンテナの状態
docker ps --format 'table {{.Names}}\t{{.Image}}\t{{.Ports}}'

# ポート公開状況
docker port <CONTAINER_NAME>

# 最近のログ
docker logs --since 10m <CONTAINER_NAME>

# Docker Composeを使っている場合
docker compose ps
docker compose logs --tail=100
```

Dockerの公開ポートがあることと、RPCサーバーが正常に応答することは別。さらに、Dockerコンテナ内の `127.0.0.1` とホストの `127.0.0.1` は異なるネットワーク名前空間を指す。

## D.17 ローカル接続を段階的に切り分ける

```text
[1] process running?
       │ yes
       ▼
[2] port listening?
       │ yes
       ▼
[3] transport correct?
       │ yes
       ▼
[4] RPC schema compatible?
       │ yes
       ▼
[5] network correct?
       │ yes
       ▼
[6] method prerequisites satisfied?
```

**切り分けのポイント**

- `Connection refused`：待ち受け先・ポート・プロセスを確認。
- `Timeout`：同期負荷・ファイアウォール・ネットワーク・応答遅延を確認。
- `Method not found`：RPC実装・メソッド名・バージョン差異を確認。
- `Invalid argument`：フィールド・型・ネットワーク・エンコードを確認。
- `Not found`：データ未存在、pruning、同期状況、照会範囲を確認。
- UTXO関連エラー：インデックス有効化と同期状態を確認。

エラー文字列は一般例であり、実際のメッセージはクライアントに依存する。

## D.18 同期状態（IBD）の判断

IBD（Initial Block Download）中は、ノードがネットワークの過去の情報を取得・検証している。

```text
Node started
     │
     ▼
Peer discovery
     │
     ▼
IBD / synchronization
     │
     ▼
Near network tip
     │
     ▼
Steady-state observation
```

**同期済みの判定は、単一の数値だけで行わない。** ノードが返す同期フラグ、接続ピア、DAG情報、最新の観測時刻を組み合わせる。公開エクスプローラーの値との単純比較も、観測時刻の差を考慮する。

## D.19 Pruningと履歴照会

Pruningノードは、すべての過去データを永久保存することを目的としない。したがって、アプリケーションで**任意の過去トランザクションを後から取得できる**と仮定しない。

履歴が必要なら、次の選択肢を検討する。

1. アプリケーションが必要な観測結果を永続化する。
2. 用途に合うインデクサーを利用する。
3. ノードが保持するデータの範囲を確認する。
4. 再同期・復旧に必要なチェックポイントを管理する。

## D.20 アプリケーションの状態管理

第35章の支払い監視アプリを例にする。

```text
InvoiceCreated
      │
      ▼
AwaitingPayment
      │
      ├── UTXO observed ──> PaymentCandidate
      │                           │
      │                           ▼
      │                     Policy evaluation
      │                           │
      │              ┌────────────┴────────────┐
      │              ▼                         ▼
      │         PaymentAccepted          CandidateInvalidated
      │
      └── Expired
```

**アプリケーション不変条件**

- 同一UTXOを同じ請求に二重計上しない。
- 異なる請求に同一UTXOを誤って割り当てない。
- 受信イベントの回数で金額を増やさない。
- 「通知を受信した」だけで確定済みにしない。
- 再編成や状態変化により、過去の判断を再評価できる。

## D.21 冪等性とイベントの重複

```text
Event A received
     │
     ▼
Idempotency key derived
     │
     ├── already exists ──> ignore duplicate
     │
     └── new ─────────────> persist observation
```

冪等キーの例：

```text
(network_id, transaction_id, output_index, observation_kind)
```

ただし、**観測イベントの同一性**と**現在のUTXOの有効性**は別。後者はチェーン状態の再照会で更新する。

## D.22 再接続・再同期の設計

WebSocketの切断を前提として、次を実装する。

1. 接続状態を監視する。
2. 指数バックオフで再接続する。
3. 再購読する。
4. 保存した最終観測位置から差分を調べる。
5. 差分取得が不可能なら対象アドレスの状態を再照会する。
6. データベース上の観測結果と突き合わせる。
7. 重複を除去し、状態を再評価する。

```text
wRPC notification = latency optimization
RPC reconciliation = correctness mechanism
```

これは設計上の指針であり、実際の正しさは再照会可能なデータ範囲と業務要件に依存する。

## D.23 RPCのエラーハンドリング

| エラー分類 | 例 | アプリの基本方針 |
|---|---|---|
| 一時的 | 接続断・タイムアウト | バックオフ付き再試行 |
| 恒久的 | 不正な要求フィールド | リクエストを修正 |
| 前提不足 | UTXO Index無効 | 設定を見直す |
| 状態依存 | 対象UTXO消費済み | 状態を再取得 |
| 互換性 | RPCスキーマ不一致 | SDKとノードの版を合わせる |
| 不確定 | 送信後に応答喪失 | 同一取引IDで再照会 |

**送信のタイムアウトを「送信失敗」と決めつけない。** ノードに届いた後で応答だけが失われた可能性がある。

## D.24 セキュリティ：RPC公開範囲

```text
[Internet]
    │
    ▼
[Authenticated API Gateway]
    │
    ▼
[Application Backend]
    │
    ▼
[Private RPC Network]
    │
    ▼
[Kaspa Node]
```

- ノードRPCは原則として信頼できるネットワークからのみ接続する。
- 公開する場合は認証、レート制限、監査ログ、ネットワーク分離を検討する。
- 秘密鍵はノードのRPCアクセス権とは分離して管理する。
- アプリケーションAPIにRPCの全機能を無制限に中継しない。
- ウォレットや管理操作に関わるログはマスキングする。

## D.25 監視項目

| 指標 | 目的 |
|---|---|
| ノード稼働状態 | 障害の検知 |
| RPC応答時間 | 遅延の把握 |
| RPCエラー率 | 互換性・障害の検知 |
| 接続ピア数 | P2Pの健全性確認 |
| 同期状態 | 古いデータでの判断防止 |
| 通知接続状態 | イベント遅延の検知 |
| 最終再照会時刻 | 取りこぼし対策の確認 |
| アプリの未処理イベント数 | バックログ検知 |
| DBとチェーンの不一致件数 | 整合性の検証 |

監視値に対するしきい値は、ネットワーク条件・アプリのSLO・実測値から決める。

## D.26 Rustで学ぶRPCレスポンスの検証

次のコードは**Kaspa SDKに接続しない、標準ライブラリのみの教育用例**である。RPCから受け取った値を業務ロジックに渡す前に検証する考え方を示す。

```rust
#[derive(Debug, PartialEq, Eq)]
struct UtxoObservation {
    transaction_id: String,
    output_index: u32,
    amount_sompi: u64,
}

#[derive(Debug, PartialEq, Eq)]
enum ObservationError {
    EmptyTransactionId,
    ZeroAmount,
}

fn validate_observation(
    observation: &UtxoObservation,
) -> Result<(), ObservationError> {
    if observation.transaction_id.is_empty() {
        return Err(ObservationError::EmptyTransactionId);
    }
    if observation.amount_sompi == 0 {
        return Err(ObservationError::ZeroAmount);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_nonzero_observation() {
        let item = UtxoObservation {
            transaction_id: "example-tx-id".into(),
            output_index: 0,
            amount_sompi: 100,
        };
        assert_eq!(validate_observation(&item), Ok(()));
    }

    #[test]
    fn rejects_zero_amount() {
        let item = UtxoObservation {
            transaction_id: "example-tx-id".into(),
            output_index: 0,
            amount_sompi: 0,
        };
        assert_eq!(
            validate_observation(&item),
            Err(ObservationError::ZeroAmount)
        );
    }
}
```

ここでの `transaction_id` は単なる文字列であり、実際のKaspaのトランザクションID形式を検証していない。また、ゼロ金額の扱いはこの教材の**業務上の仮定**であり、Kaspaコンセンサスの一般ルールを表したものではない。

## D.27 Rustの単体テストを実行する

上記コードを単独で `src/lib.rs` に保存した場合：

```bash
cargo new --lib rpc-observation-lab
cd rpc-observation-lab
# src/lib.rs を上記のコードに置き換える
cargo test
```

**実行結果は未検証。** 読者はテストの実行ログとRustバージョンを記録する。

## D.28 APIのバージョン差異を吸収する

```text
         Application Domain
                 ▲
                 │
          KaspaGateway Trait
                 ▲
       ┌─────────┴─────────┐
       │                   │
       ▼                   ▼
  gRPC Adapter         wRPC Adapter
       │                   │
       └─────────┬─────────┘
                 ▼
             Kaspa Node
```

業務ロジックにSDK固有のレスポンス型を直接広げない。境界でアプリケーションのドメイン型に変換する。

**抽象化の利点**

- RPCクライアントの差し替えが容易。
- モックで単体テストができる。
- SDKアップデートの影響をアダプターに閉じ込められる。

**抽象化のコスト**

- Kaspa固有の意味を失う危険がある。
- メソッド追加時に境界の変更が必要になる。

## D.29 C4モデル：Kaspaアプリケーション

```text
C1: Context
  User ──> Payment Application ──> Kaspa Network

C2: Containers
  Web UI ──> Backend API ──> Database
                  │
                  └──> Chain Observer ──> Kaspa Node

C3: Components
  Chain Observer
    ├── RPC Adapter
    ├── Subscription Manager
    ├── Reconciliation Worker
    ├── Idempotency Store
    └── Payment Policy Evaluator
```

外部RPCの応答をそのまま業務上の真実にせず、**観測 → 検証 → 保存 → 状態遷移**の境界を設ける。

## D.30 Devnet実験：最低限の手順

**目的：** ノード起動から、DAG・UTXO・通知の観測までを記録する。

1. 使用する `rusty-kaspa` のバージョンを固定する。
2. `--help` を確認し、Devnetの起動引数を確定する。
3. ノードを起動する。
4. 接続ピアと同期状態を確認する。
5. `getCurrentNetwork` 相当の手段でDevnetであることを確認する。
6. `getBlockDAGInfo` 相当の読み取りを行う。
7. UTXO Indexを有効にしている場合、実験用アドレスのUTXOを照会する。
8. 通知購読を開始する。
9. Devnetでトランザクションを発生させ、通知と再照会の差を確認する。
10. ノードまたはクライアントを再起動し、取りこぼしの復旧手順を検証する。

**注意：** Devnetのブロック生成、資金準備、ウォレット接続、マイニング操作はバージョンや実験構成に依存するため、この付録では固定コマンドを提示しない。

## D.31 実験記録テンプレート

```markdown
# RPC Observation Experiment

- Date (UTC):
- OS:
- rusty-kaspa version / commit:
- Client / SDK version:
- Network:
- Transport: gRPC / wRPC
- Endpoint (secrets removed):
- UTXO index: enabled / disabled

## Objective

## Steps
1.
2.
3.

## Observed results

## Expected results

## Differences / errors

## Reproduction instructions

## Next action
```

## D.32 トラブルシューティング早見表

| 症状 | 最初に確認すること | 次の一手 |
|---|---|---|
| RPCに接続できない | プロセス・ポート・トランスポート | ログと待ち受け設定を照合 |
| wRPCが切れる | クライアント・サーバーのバージョン | 再接続と再照会を実装 |
| アドレス照会に失敗 | UTXO Index | 設定・同期状況を確認 |
| データが古い | IBD・ピア・観測時刻 | 同期状態を再確認 |
| 取引が見つからない | MempoolかDAGか | 別の観測手段と照合 |
| 通知が二重に届く | 冪等キー | 重複排除・再計算 |
| 送信応答が消えた | 送信済みか不明 | 取引IDで再照会 |
| Docker内では動くが外から接続不可 | bind address・port mapping | ネットワーク設定を修正 |
| SDK更新後に失敗 | RPCスキーマ互換性 | 版を固定して差分確認 |

## D.33 よくある誤解

**誤解1：RPCに接続できたら同期済み。**  
接続成功と同期完了は別。

**誤解2：通知が来たら入金確定。**  
通知は観測であり、確定ポリシーの評価は別。

**誤解3：Mempoolから消えたらブロックに入った。**  
削除やその他の理由があり得る。

**誤解4：Dockerでポート公開すれば安全。**  
公開範囲と認証・アクセス制御を別途検討する。

**誤解5：RPCのメソッド名が同じなら引数も同じ。**  
SDK・バージョン・トランスポート間の差異がある。

**誤解6：残高照会ができれば全履歴を取得できる。**  
残高・UTXO・過去の履歴は異なるデータ。

## D.34 理解度チェック

1. P2P通信とアプリケーション向けRPCは何が違うか。
2. gRPCとwRPCを選ぶ際に何を確認するか。
3. `getBlockDAGInfo` から何を読み取るか。
4. UTXO Indexが必要になるのはどのような場合か。
5. Mempoolに存在する取引を「確定」と扱えないのはなぜか。
6. 通知購読が切断された場合、何を再照会するか。
7. トランザクション送信後のタイムアウトをどう扱うか。
8. アプリケーションで冪等性を確保する方法は何か。
9. RPCのバージョン差異をどう隔離するか。
10. 実験の再現性を確保するために最低限何を記録するか。

## D.35 まとめ

KaspaのRPC・CLIを使いこなすために必要なのは、コマンドを暗記することではない。

1. **何を観測したいかを明確にする。**
2. **使用中のノード・SDK・ネットワークを特定する。**
3. **そのバージョンの一次資料からメソッドと引数を確認する。**
4. **観測結果と業務判断を分離する。**
5. **通信断・再編成・重複通知を前提に設計する。**
6. **再現可能なログと手順を残す。**

本付録の最終目標は、CLI操作を一通り実行することではなく、**ノードの状態を検証し、アプリケーションの正しさを説明できるようになること**である。

## D.36 一次資料・参考資料

以下の資料を、使用するリリース・コミットに合わせて確認する。

- [Kaspa公式リポジトリ — rusty-kaspa](https://github.com/kaspanet/rusty-kaspa) — ノード実装、README、ビルド、CLI・RPC関連コード。
- [Kaspa公式リポジトリ — kaspa-cli](https://github.com/kaspanet/kaspa-cli) — CLIのコマンド体系・対応状況。**リポジトリの存続と最新互換性は要確認。**
- [Kaspa公式ドキュメント — Wiki](https://wiki.kaspa.org/) — ノード運用・ネットワーク・開発者向け情報。
- [Kaspa Improvement Proposals](https://github.com/kaspanet/kips) — プロトコル変更の背景と提案状況。
- [Docker公式ドキュメント](https://docs.docker.com/) — コンテナ・ネットワーク・ログ・Compose。
- [The Rust Programming Language](https://doc.rust-lang.org/book/) — Rust開発の基礎。
- [Cargo Book](https://doc.rust-lang.org/cargo/) — ビルド・テスト・依存管理。

### 技術レビュー時に必ず確認する項目

- [ ] 使用する `rusty-kaspa` のバージョンとコミットを固定した。
- [ ] 本文の各RPCメソッドが対象バージョンに存在することを確認した。
- [ ] gRPC / wRPCのメソッド表記と引数・戻り値を確認した。
- [ ] ノード起動オプションとネットワークごとのポートを実機で確認した。
- [ ] CLIの対応バージョンとコマンド体系を確認した。
- [ ] UTXO Indexの前提条件を確認した。
- [ ] Devnet実験を実行し、ログを保存した。
- [ ] Rustの単体テストを実行した。
- [ ] 誤解を招く固定値・未検証コマンドがないか再確認した。

---

**次の付録：Appendix E — プロトコルパラメータ一覧**
