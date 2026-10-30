# Chapter 23 — Rust SDK & WASM（Rust SDKとWASM）

> **Mastering Kaspa — Part IV: Kaspa Development & Implementation**  
> 原稿ステータス: v0.1（技術レビュー前）

## この章で学ぶこと

本章では、KaspaのRust実装とWebAssembly（WASM）の接点を整理し、ノードを直接操作するコードと、ウォレット・ブラウザアプリケーションを構築するコードの責務を区別する。具体的には、SDKの構造、型境界、非同期処理、接続管理、署名、テスト、セキュリティを扱う。

学習目標は次の5点である。

1. `rusty-kaspa`、RPCクライアント、WASMバインディングの関係を説明できる。
2. ブラウザとサーバーで秘密鍵・接続情報をどこに置くべきか判断できる。
3. RPC接続の失敗・再接続・ネットワーク不一致を設計できる。
4. Rustの純粋なドメインロジックをWASM対応の境界から切り離せる。
5. 実ノードとの統合テストを、単体テストと分離して計画できる。

> **バージョン上の注意:** Kaspa SDKの公開API、パッケージ名、ビルド手順は更新される。本章のコードは明示した教育用サンプルを除き、特定のSDKバージョンに依存する呼び出しを断定しない。実際の接続コードは使用するリリースの公式サンプルと照合すること。

---

## 23.1 なぜSDKが必要なのか

Kaspaノードの内部処理には、P2P、BlockDAG、コンセンサス、ストレージ、RPCなどがある。アプリケーションがこれらをすべて直接実装する必要はない。SDKは、ノードやウォレットの機能を扱いやすいインターフェースにまとめるための境界である。

```text
┌───────────────────────────────────────────────┐
│ Application                                   │
│  Web UI / CLI / Backend / Wallet              │
└──────────────────────┬────────────────────────┘
                       │ SDK API
┌──────────────────────▼────────────────────────┐
│ Client / Wallet / WASM bindings               │
│  types, serialization, subscriptions, signing │
└──────────────────────┬────────────────────────┘
                       │ RPC / network protocol
┌──────────────────────▼────────────────────────┐
│ Kaspa Node                                    │
│  RPC ─ Consensus ─ BlockDAG ─ UTXO ─ Storage  │
└──────────────────────┬────────────────────────┘
                       │ P2P
                 Kaspa Network
```

この図は**概念的な依存関係**であり、すべてのSDKが署名・ウォレット・RPCを同一パッケージに提供するという意味ではない。

### SDKを使っても不要にならない責務

- 接続先ネットワークの検証
- 未同期ノードの扱い
- 取引送信後の状態追跡
- 再接続時のイベント取りこぼし対策
- 秘密鍵の安全な保管
- 手数料・残高・送金先の業務ルール

SDKは便利な抽象化であって、アプリケーションの正しさを自動保証するものではない。

## 23.2 `rusty-kaspa`とSDKの位置付け

Kaspaの主要なRust実装は [`kaspanet/rusty-kaspa`](https://github.com/kaspanet/rusty-kaspa) に公開されている。リポジトリには、ノード本体だけでなく、RPC、ウォレット、暗号・データ型、WASMに関連するコンポーネントが含まれる。**正確なcrate名とモジュール境界は対象コミットで確認する。**

| 層 | 主な責務 | アプリケーション側の注意 |
|---|---|---|
| Consensus | DAG・順序・UTXO状態 | アプリから直接変更しない |
| RPC | ノード情報・取引送信・通知 | ネットワークと同期状態を確認 |
| Client | 接続・再接続・型変換 | タイムアウトとエラー分類 |
| Wallet | 鍵・UTXO選択・送金構築 | 秘密鍵と金額を慎重に扱う |
| WASM bindings | Rust機能をJSなどに公開 | 型変換・メモリ・非同期境界 |

> **重要:** 「Rust SDK」は単一の固定されたライブラリ名ではなく、Rust製の関連クレートやクライアント機能を指す文脈がある。本書では、個別の公開API名を示す際に必ずバージョンを併記する方針とする。

## 23.3 Rustを使う理由

Rustの所有権・型・エラー処理は、金額やトランザクション状態のような誤用を避けたいデータを扱うのに適している。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Sompi(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NetworkId(&'static str);
```

`Sompi` とネットワーク識別子を異なる型にすれば、単なる整数・文字列の取り違えを減らせる。ただし、**型だけでネットワークや送金先の正当性が保証されるわけではない**。

### `Result`を使った失敗の明示

```rust
#[derive(Debug, PartialEq, Eq)]
enum TransferError {
    ZeroAmount,
    InsufficientFunds,
    Overflow,
}

fn calculate_change(
    inputs: u64,
    amount: u64,
    fee: u64,
) -> Result<u64, TransferError> {
    if amount == 0 {
        return Err(TransferError::ZeroAmount);
    }
    let required = amount.checked_add(fee)
        .ok_or(TransferError::Overflow)?;
    inputs.checked_sub(required)
        .ok_or(TransferError::InsufficientFunds)
}
```

この関数は送金額の算術チェックを示すだけで、実際のトランザクション構築や署名は行わない。

## 23.4 WebAssembly（WASM）の基本

WASMは、ブラウザなどの実行環境で利用できるバイナリ形式である。Rustの一部のコードをWASMにコンパイルすることで、同じドメインロジックをブラウザ側でも利用できる。

```text
Rust source
   │
   ├── native target ──► CLI / backend
   │
   └── wasm32 target ──► WASM module
                               │
                               ▼
                       JavaScript / Browser
```

ただし、**ネイティブRustのコードが無変更でWASMに移植できるとは限らない**。OS固有のファイルI/O、ソケット、スレッド、暗号乱数、時間APIなどは実行環境によって異なる。

### WASMが向く処理

- アドレスや入力データの構文検査
- 決定的な金額計算
- 公開情報のエンコード・デコード
- ブラウザ内での署名処理（鍵保護設計が前提）

### WASMが自動的には解決しないこと

- 秘密鍵の永続保管
- 悪意あるJavaScriptや拡張機能からの防御
- ネットワーク切断の回復
- ブラウザ環境のリソース制約

## 23.5 RustとJavaScriptの境界

RustとJavaScriptでは、値・エラー・非同期処理の表現が異なる。境界は薄く保ち、複雑な業務ルールをできるだけRustの純粋関数に置く。

| Rust側 | JavaScript側 | 注意点 |
|---|---|---|
| `u64` | `number` / `bigint` | `number`の安全整数範囲に注意 |
| `Result<T, E>` | 返り値 / 例外など | エラーの意味を失わない |
| `Vec<u8>` | `Uint8Array` | コピーと所有権を確認 |
| `async fn` | `Promise` | キャンセル・タイムアウト |
| `String` | `string` | アドレスや識別子の検証 |

JavaScriptの`number`は、整数を常に正確に表現できるわけではない。**通貨の最小単位を浮動小数点数で安易に扱わない**こと。

## 23.6 最小のWASMモジュールを作る

ここではKaspa SDKに依存しない**教育用のRust/WASM実験**を作る。

### ディレクトリ

```text
kaspa-wasm-lab/
├── Cargo.toml
└── src/
    └── lib.rs
```

### `Cargo.toml`

```toml
[package]
name = "kaspa-wasm-lab"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
```

### `src/lib.rs`

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn can_cover_payment(
    available: u32,
    amount: u32,
    fee: u32,
) -> bool {
    if amount == 0 {
        return false;
    }
    match amount.checked_add(fee) {
        Some(required) => available >= required,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enough_balance() {
        assert!(can_cover_payment(1_000, 900, 100));
    }

    #[test]
    fn insufficient_balance() {
        assert!(!can_cover_payment(1_000, 901, 100));
    }

    #[test]
    fn rejects_zero_amount() {
        assert!(!can_cover_payment(1_000, 0, 10));
    }

    #[test]
    fn rejects_overflow() {
        assert!(!can_cover_payment(u32::MAX, u32::MAX, 1));
    }
}
```

`u32`はブラウザ境界の例を簡単にするために選んだ。**実際のKaspa金額型を`u32`に制限してよいという意味ではない**。

### 実行手順

```bash
cargo test
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build --target web
```

生成された`pkg/`を、ローカルWebサーバーで配信するHTML/JavaScriptから読み込む。`file://`で直接開く場合はモジュール読み込み制約に注意する。

> このサンプルはコード例として提示している。ここでは実際のビルド・テスト実行結果を保証しない。

## 23.7 アプリケーションの依存関係を整理する

Kaspaアプリケーションでは、UIから直接RPCを呼ぶ設計より、ノード接続をアダプターに閉じ込める設計がテストしやすい。

```text
┌──────────────────────┐
│ UI / API Controller  │
└──────────┬───────────┘
           │
┌──────────▼───────────┐
│ Application Service  │
│ Send / Track Payment │
└──────┬─────────┬─────┘
       │         │
┌──────▼─────┐ ┌─▼────────────┐
│ Domain     │ │ NodeGateway  │  ← interface
│ Rules      │ └──────┬───────┘
└────────────┘        │
                ┌─────▼─────────┐
                │ Kaspa RPC SDK │
                └───────────────┘
```

**変更されやすいもの（SDK・RPC仕様・接続先）を、変更されにくい業務ルールから切り離す**のが設計の基本である。

## 23.8 `NodeGateway`による境界設計

SDKの実際のAPI名に依存しない、アプリケーション側のポートを定義する。

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
struct NodeStatus {
    network: String,
    synced: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum GatewayError {
    Unavailable,
    InvalidResponse,
}

trait NodeGateway {
    fn status(&self) -> Result<NodeStatus, GatewayError>;
}

#[derive(Debug, PartialEq, Eq)]
enum AppError {
    Node(GatewayError),
    WrongNetwork,
    NotSynced,
}

fn ensure_ready(
    gateway: &impl NodeGateway,
    expected_network: &str,
) -> Result<(), AppError> {
    let status = gateway.status().map_err(AppError::Node)?;
    if status.network != expected_network {
        return Err(AppError::WrongNetwork);
    }
    if !status.synced {
        return Err(AppError::NotSynced);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeGateway(NodeStatus);

    impl NodeGateway for FakeGateway {
        fn status(&self) -> Result<NodeStatus, GatewayError> {
            Ok(self.0.clone())
        }
    }

    #[test]
    fn ready_on_expected_network() {
        let gateway = FakeGateway(NodeStatus {
            network: "testnet".into(),
            synced: true,
        });
        assert_eq!(ensure_ready(&gateway, "testnet"), Ok(()));
    }

    #[test]
    fn rejects_wrong_network() {
        let gateway = FakeGateway(NodeStatus {
            network: "mainnet".into(),
            synced: true,
        });
        assert_eq!(
            ensure_ready(&gateway, "testnet"),
            Err(AppError::WrongNetwork)
        );
    }

    #[test]
    fn rejects_unsynced_node() {
        let gateway = FakeGateway(NodeStatus {
            network: "testnet".into(),
            synced: false,
        });
        assert_eq!(
            ensure_ready(&gateway, "testnet"),
            Err(AppError::NotSynced)
        );
    }
}
```

実アプリケーションでは、ネットワークIDを単純な文字列ではなく、SDKが提供する識別子型などで扱うことが望ましい。

## 23.9 非同期処理と接続管理

RPC通信には待ち時間と失敗がある。Rustの`async`/`await`は非同期処理を記述する仕組みだが、**タイムアウトや再試行のポリシーはアプリケーション側で設計する**。

### 接続ステートマシン

```text
Disconnected
     │ connect
     ▼
 Connecting ── failure ──► Backoff
     │                       │
     │ success               │ retry
     ▼                       └──► Connecting
  Connected
     │
     ├── subscription active ─► Streaming
     │                              │
     └── disconnect ◄───────────────┘
```

必要な不変条件:

- 接続先ネットワークを検証するまで送金しない。
- 再接続後は購読状態を復元する。
- 再試行してよい操作と、重複送信が危険な操作を区別する。
- イベントを受信できなかった期間は、RPCで現在状態を再照会する。

### 冪等性

送信処理のタイムアウトは「送信されなかった」ことを意味しない。再送前にトランザクションIDやノード状態を確認し、業務上の二重処理を避ける。

## 23.10 イベント購読と状態の整合性

取引の追跡は、単発のRPC応答だけで完結しない。

```text
Submit Transaction
       │
       ▼
Receive RPC Response
       │
       ▼
Track Transaction ID
       │
       ├──► Subscribe to relevant events
       │
       └──► Reconcile with current node state
                         │
                         ▼
                 Update app status
```

アプリケーション側の状態例:

| 状態 | 意味 | 次の処理 |
|---|---|---|
| `created` | ローカルで構築 | 署名・検査 |
| `submitted` | ノードに送信要求 | 追跡開始 |
| `observed` | ネットワーク上で観測 | 状態再照会 |
| `accepted` | ノードが受理と報告 | 確定ポリシー評価 |
| `settled` | アプリの確定条件を満たす | 業務処理完了 |
| `rejected` | 無効・競合など | 原因記録 |

この表は**アプリケーション独自の状態モデル**であり、Kaspa RPCが同名のステータスを提供するという意味ではない。

## 23.11 ウォレットと秘密鍵の境界

ブラウザウォレットとサーバーウォレットでは、脅威モデルが大きく異なる。

| 構成 | 利点 | 主なリスク |
|---|---|---|
| ブラウザ内署名 | 秘密鍵をサーバーへ渡さない設計が可能 | XSS、拡張機能、端末侵害 |
| サーバー側署名 | 自動決済・運用に適する | サーバー侵害で鍵が流出し得る |
| 外部署名器 | 鍵をアプリ本体から分離 | 署名要求の検証・UX |

**WASMで署名すれば秘密鍵が安全になるわけではない。** 同じブラウザ実行環境のJavaScriptや周辺コードが侵害される可能性を考慮する必要がある。

最低限の原則:

1. シード・秘密鍵をログに出さない。
2. 送金先・金額・手数料・ネットワークを署名前に確認する。
3. MainnetとTestnetを明確に分離する。
4. 鍵のバックアップと復旧手順を別途設計する。
5. サーバー側では権限分離と秘密情報管理を徹底する。

## 23.12 RPCとWASMの実行環境差

ブラウザではWebSocketなど利用できる通信手段に制約がある。ネイティブアプリケーションではOSのネットワーク機能を利用できるが、ファイアウォールやTLS設定が必要になる。

| 観点 | ネイティブRust | ブラウザ/WASM |
|---|---|---|
| ソケット | OS APIにアクセス可能 | ブラウザAPIに制約 |
| ファイル | 権限に応じて利用可能 | ブラウザの保存API |
| 秘密情報 | OS保護機構などを選択可能 | XSS等への注意が重要 |
| スレッド | ランタイム・OS依存 | ブラウザ制約あり |
| 配布 | 実行ファイル | JS/WASMアセット |

SDKの通信バックエンドが対象環境に対応しているかを、実装前に確認する。

## 23.13 エラーを分類する

すべてのエラーを「接続失敗」にまとめると、運用時に原因を特定できない。

| エラー分類 | 例 | 対応 |
|---|---|---|
| Transport | 切断・タイムアウト | バックオフ再接続 |
| Protocol | 形式不正・互換性不一致 | バージョン確認 |
| Network | Mainnet/Testnet不一致 | 処理停止 |
| Node state | IBD中・未同期 | 同期待ち |
| Transaction | 競合・入力不正 | 再試行前に原因分析 |
| Authorization | 鍵・署名・権限不備 | 操作中止 |
| Application | 金額・送金先ルール違反 | ユーザーに修正を促す |

再試行は**安全な読み取り**を基本とし、書き込み系操作では冪等性と状態照会を組み合わせる。

## 23.14 バージョンと互換性を管理する

SDKを導入するときは、最低限次の組み合わせを記録する。

```text
Node version / commit
SDK crate or package version
WASM toolchain version
Rust toolchain version
Network (mainnet / testnet / devnet)
RPC transport and endpoint
```

依存関係を固定する例:

```bash
rustc --version
cargo --version
cargo tree
cargo metadata --no-deps
```

`Cargo.lock`はアプリケーションで原則コミットし、再現性を確保する。ライブラリの配布方針は用途に応じて検討する。

## 23.15 `rusty-kaspa`の読み方

最初から全コードを読むのではなく、**アプリケーションが呼びたい操作から逆向きに依存関係を追う**。

1. 対象リリース・コミットを固定する。
2. READMEやexamplesから、クライアントの初期化方法を確認する。
3. 接続処理の型・戻り値・エラーを調べる。
4. RPCメソッドがどのノード機能へ到達するか追跡する。
5. WASMバインディングの公開境界を確認する。
6. 統合テスト・既存サンプルを実行する。

```bash
git clone https://github.com/kaspanet/rusty-kaspa.git
cd rusty-kaspa
rg 'wasm_bindgen|RpcClient|connect|subscribe' .
```

`rg`はripgrepのコマンド。該当する型やディレクトリはバージョンによって変わり得る。

### 調査メモのテンプレート

```markdown
## 調査対象
- commit:
- API / symbol:
- source path:
- caller:
- dependency:
- errors:
- tests:
- open questions:
```

## 23.16 Devnetでの統合実験

単体テストが通っても、実ノードと通信できる保証はない。Devnetなどの検証環境で、次の実験を行う。

1. ノードを起動し、ネットワーク識別子を確認する。
2. 同期状態を取得する。
3. RPCクライアントからノード情報を取得する。
4. イベント購読を開始する。
5. ノードを再起動し、再接続・購読復元を確認する。
6. 誤ったネットワークへの接続を拒否できるか確認する。
7. テスト用の取引を送信し、結果を追跡する。

> RPCのポート、フラグ、メソッド名は利用するノードのバージョンに合わせて公式ドキュメントで確認する。Mainnetの資金を使って実験しない。

### テストの階層

```text
Unit tests
  └─ amount rules, validation, state transitions
Contract tests
  └─ SDK adapter and mock responses
Integration tests
  └─ real node / RPC / reconnection
End-to-end tests
  └─ wallet → submit → observe → settle
```

## 23.17 可観測性（Observability）

ノード接続の障害は、ユーザーからは「送金が進まない」に見える。最低限、以下を監視する。

- 接続状態と最後の接続成功時刻
- ノードの同期状態
- RPC応答時間・タイムアウト率
- 再接続回数とバックオフ時間
- 購読イベントの最終受信時刻
- 取引追跡の滞留件数
- SDK・ノードのバージョン

秘密鍵・シード・認証情報・個人情報をログに含めない。

## 23.18 C4とDDDで責務を整理する

### Context

利用者はウォレットまたはアプリケーションを操作し、アプリケーションはKaspaノードと通信する。

### Container

- Frontend: 画面表示、ユーザー操作
- Backend: 業務ルール、状態追跡、永続化
- Wallet/Signer: 鍵管理、署名
- Node Gateway: RPC通信、エラー変換
- Kaspa Node: コンセンサス状態の提供

### Component

- `PaymentService`: 支払いユースケース
- `TransactionBuilder`: トランザクション構築
- `Signer`: 署名
- `NodeGateway`: ノード通信
- `TransactionTracker`: 状態追跡
- `SettlementPolicy`: アプリ側の確定判定

境界を越える際には、**「誰が何を保証するか」**を明確にする。

| 不変条件 | 責務の例 |
|---|---|
| 金額は正の値 | Domain |
| 必要資金が不足していない | Wallet/Domain |
| 接続先ネットワークが正しい | Gateway |
| ノードが同期済み | Gateway / Application |
| 署名が支出条件を満たす | Signer + Node validation |
| 入金を業務上確定してよい | SettlementPolicy |

## 23.19 よくある誤解

**Q. WASMならRustのすべてのcrateをブラウザで動かせる？**  
A. いいえ。OS固有の機能や未対応の依存関係がある。

**Q. SDKの送信APIが成功すれば入金確定？**  
A. いいえ。RPC成功、ネットワーク観測、コンセンサス受理、アプリの確定判断は異なる。

**Q. SDKを使えば秘密鍵管理を考えなくてよい？**  
A. いいえ。鍵の生成・保存・復旧・署名承認は別途設計する。

**Q. Rustの型があれば金額のバグはなくなる？**  
A. いいえ。単位、丸め、オーバーフロー、業務上の妥当性を検証する必要がある。

**Q. ブラウザから直接ノードRPCへ接続すべき？**  
A. 用途次第。公開RPCの負荷・権限・可用性・プライバシーを考慮する。

## 23.20 実践課題

### 課題A — 型安全な金額

`Sompi(u64)`を作り、加算・減算を`checked_*`で実装する。ゼロ金額・オーバーフロー・不足をテストする。

### 課題B — RPCアダプター

`NodeGateway`を`async`に拡張し、接続タイムアウト、未同期、ネットワーク不一致のテストを追加する。

### 課題C — 再接続

接続ステートマシンを実装し、切断→再接続→購読復元の遷移をテストする。

### 課題D — WASM

本章の`can_cover_payment`をブラウザから呼び出し、Rust単体テストとブラウザ側テストの結果を比較する。

### 課題E — 実ノード

対象の`rusty-kaspa`コミットを固定し、ノード情報を取得する最小クライアントを作成する。APIのシグネチャ、依存関係、エラー型を調査メモに残す。

## 23.21 理解度チェック

1. SDKとノード本体の責務はどう異なるか。
2. WASMが動作する環境で、ネイティブRustと異なる制約は何か。
3. JavaScriptの`number`で大きな整数を扱う際の問題は何か。
4. RPC送信がタイムアウトした場合、なぜ即座の再送が危険になり得るか。
5. `NodeGateway`を抽象化する利点は何か。
6. 秘密鍵をWASM内で扱うだけでは安全性が保証されない理由は何か。
7. 再接続後にイベント購読だけを再開しても不十分な場合があるのはなぜか。
8. 単体テストと統合テストは何をそれぞれ保証するか。
9. SDK更新時に固定・記録すべきバージョン情報は何か。
10. コンセンサスの受理とアプリケーションの入金確定を分ける理由は何か。

## 23.22 まとめ

Rust SDKとWASMは、Kaspaのコンセンサス機能をアプリケーションから利用するための重要な接点である。ただし、SDKはネットワーク・鍵管理・業務上の正しさをすべて引き受けるわけではない。

本章で押さえるべき原則は以下のとおり。

- **Rustのドメインロジックと、RPC/WASMの実行環境依存を分離する。**
- **RPC応答とコンセンサス状態を混同しない。**
- **再接続・冪等性・イベント欠落を通常の設計対象とする。**
- **秘密鍵の管理は、SDK選定とは別のセキュリティ設計である。**
- **実装を調べるときは、必ずバージョンとソースコードを固定する。**

次章「Wallet Architecture & Key Management」では、鍵生成、HDウォレット、アドレス、署名、バックアップと復旧をさらに掘り下げる。

## 23.23 参考資料

1. [Kaspa — Official Website](https://kaspa.org/)
2. [Kaspa Wiki](https://wiki.kaspa.org/en/home)
3. [kaspanet/rusty-kaspa — GitHub](https://github.com/kaspanet/rusty-kaspa)
4. [Kaspa Improvement Proposals (KIPs)](https://github.com/kaspanet/kips)
5. [The Rust Programming Language](https://doc.rust-lang.org/book/)
6. [The Rust and WebAssembly Book](https://rustwasm.github.io/docs/book/)
7. [wasm-bindgen Guide](https://rustwasm.github.io/docs/wasm-bindgen/)
8. [wasm-pack Documentation](https://rustwasm.github.io/docs/wasm-pack/)
9. [MDN — WebAssembly](https://developer.mozilla.org/en-US/docs/WebAssembly)

---

**技術レビューTODO:** 対象とする`rusty-kaspa`のリリース/コミットを決め、実際のRPCクライアント初期化、WASMパッケージ名、接続・購読API、各サンプルのビルド結果を確認して追記する。
