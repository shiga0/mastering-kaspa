# Chapter 23 — Rust SDK & WASM（Rust SDKとWASM）

> **Mastering Kaspa — Part IV: Kaspa Development & Implementation**  
> 目次順の本文レビュー：2026-11-01。基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。主要バインディングの照合と教材の確認であり、SDK全体の検証ではない。

確認記録は `experiments/chapter-23-review-2026-11-01.md`。公式配布SDKを使った過去のDevnet実測と、本章の独自教材の実行を区別する。

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
- トランザクション送信後の状態追跡
- 再接続時のイベント取りこぼし対策
- 秘密鍵の安全な保管
- 手数料・残高・送金先の業務ルール

SDKは便利な抽象化であって、アプリケーションの正しさを自動保証するものではない。

## 23.2 `rusty-kaspa`とSDKの位置付け

Kaspaの主要なRust実装は [`kaspanet/rusty-kaspa`](https://github.com/kaspanet/rusty-kaspa) に公開されている。リポジトリには、ノード本体だけでなく、RPC、ウォレット、暗号・データ型、WASMに関連するコンポーネントが含まれる。**crate名・公開型・featureは対象コミットで確認する。**

| 層 | 主な責務 | アプリケーション側の注意 |
|---|---|---|
| Consensus | DAG・順序・UTXO状態 | アプリから直接変更しない |
| RPC | ノード情報・トランザクション送信・通知 | ネットワークと同期状態を確認 |
| Client | 接続・再接続・型変換 | タイムアウトとエラー分類 |
| Wallet | 鍵・UTXO選択・送金構築 | 秘密鍵と金額を慎重に扱う |
| WASM bindings | Rust機能をJSなどに公開 | 型変換・メモリ・非同期境界 |

> **重要:** 「Rust SDK」は単一の固定されたライブラリ名ではなく、Rust製の関連クレートやクライアント機能を指す文脈がある。本書では、個別の公開API名を示す際に必ずバージョンを併記する方針とする。

### 23.2.1 基準版の具体的な入口

| crate / ソース | 確認した境界 |
|---|---|
| [kaspa-wrpc-client](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/rpc/wrpc/client/src/client.rs) | Rust側のKaspaRpcClient。newはencoding、url、resolver、network_id、subscription_contextを受け取る |
| [kaspa-wrpc-wasm](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/rpc/wrpc/wasm/src/client.rs) | JS向けRpcClient。connectとsubscribeUtxosChanged等のバインディング |
| [kaspa-wasm](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/wasm/Cargo.toml) | cdylibの集約。wasm32-sdk / wasm32-rpc等で公開範囲を選ぶ |

Rustのsnake_caseとJSのcamelCase、Rust crate名と配布アセット名を混同しない。kaspa-wasmを依存に追加しただけで、すべてのAPI・環境・featureが使えるとは限らない。基準版の[集約と読み込み例](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/wasm/src/lib.rs)も確認する。

以下の独自教材はKaspa SDKに依存しない。基準版SDKの依存指定や、実際のRust RPCアダプターの実装例ではない。

## 23.3 Rustを使う理由

Rustの所有権・型・エラー処理は、金額やトランザクション状態のような誤用を避けたいデータを扱うのに適している。

本書の外で `cargo new --lib --edition 2021 kaspa-sdk-boundary-demo` を作成し、23.3の2つのRustコードと23.8のRustコードを、この順にsrc/lib.rsへ連結できる。外部依存は不要で、23.6のWASM教材とは別のプロジェクトである。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sompi(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkId(pub &'static str);
```

`Sompi` とネットワーク識別子を異なる型にすれば、単なる整数・文字列の取り違えを減らせる。ただし、**型だけでネットワークや送金先の正当性が保証されるわけではない**。

### `Result`を使った失敗の明示

```rust
#[derive(Debug, PartialEq, Eq)]
pub enum TransferError {
    ZeroAmount,
    InsufficientFunds,
    Overflow,
}

pub fn calculate_change(inputs: Sompi, amount: Sompi, fee: Sompi) -> Result<Sompi, TransferError> {
    if amount.0 == 0 {
        return Err(TransferError::ZeroAmount);
    }
    let required = amount.0.checked_add(fee.0).ok_or(TransferError::Overflow)?;
    inputs
        .0
        .checked_sub(required)
        .map(Sompi)
        .ok_or(TransferError::InsufficientFunds)
}

#[cfg(test)]
mod amount_tests {
    use super::*;

    #[test]
    fn preserves_change_and_accepts_exact_balance() {
        assert_eq!(
            calculate_change(Sompi(1_001), Sompi(900), Sompi(100)),
            Ok(Sompi(1))
        );
        assert_eq!(
            calculate_change(Sompi(1_000), Sompi(900), Sompi(100)),
            Ok(Sompi(0))
        );
    }
    #[test]
    fn rejects_zero_payment() {
        assert_eq!(
            calculate_change(Sompi(1_000), Sompi(0), Sompi(10)),
            Err(TransferError::ZeroAmount)
        );
    }
    #[test]
    fn rejects_insufficient_funds() {
        assert_eq!(
            calculate_change(Sompi(1_000), Sompi(901), Sompi(100)),
            Err(TransferError::InsufficientFunds)
        );
    }
    #[test]
    fn rejects_overflow_and_preserves_u64_precision() {
        assert_eq!(
            calculate_change(Sompi(u64::MAX), Sompi(u64::MAX), Sompi(1)),
            Err(TransferError::Overflow)
        );
        assert_eq!(
            calculate_change(Sompi(u64::MAX), Sompi(u64::MAX - 1), Sompi(1)),
            Ok(Sompi(0))
        );
    }
}
```

この関数はSompi単位の算術チェックだけを行い、UTXO選択・mass・手数料率・成熟・署名・送信を扱わない。上の型は独自モデルで、NetworkIdの文字列の構文検査も実装していない。23.3と23.8のRust断片を同じlib.rsに連結すると、後述のモックを含めてテストできる。

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
| wasm-bindgenの`u64` | `bigint` | 範囲外のBigIntは循環するため、変換前に範囲検査 |
| wasm-bindgenの`u32` | `number` | 小数・負数・NaN・範囲外を変換前に検査 |
| `Result<T, E>` | 返り値 / 例外など | エラーの意味を失わない |
| `Vec<u8>` | `Uint8Array` | コピーと所有権を確認 |
| `async fn` | `Promise` | キャンセル・タイムアウト |
| `String` | `string` | アドレスや識別子の検証 |

JavaScriptの`number`は、整数を常に正確に表現できるわけではない。**通貨の最小単位を浮動小数点数で安易に扱わない**こと。上表はwasm-bindgenの直接バインディングの説明であり、SDK独自の変換は対象版の型宣言を確認する。

[wasm-bindgenの数値境界](https://wasm-bindgen.github.io/wasm-bindgen/reference/types/numbers.html)では、整数型への変換で範囲外の値が循環したり、小数が切り捨てられたりする。Rustの引数を整数型にしても、JS入力の検証を自動で済ませたことにはならない。u64なら`typeof value === "bigint"`と`0n <= value && value <= 0xffff_ffff_ffff_ffffn`などを呼び出し前に検査する。

## 23.6 最小のWASMモジュールを作る

ここではKaspa SDKに依存しない**教育用のRust/WASM実験**を作る。

本書のリポジトリとは別の作業用ディレクトリで次を実行する。以下はこのプロジェクト内のファイルである。

```bash
cargo new --lib --edition 2021 kaspa-wasm-lab
cd kaspa-wasm-lab
```

### ディレクトリ

```text
kaspa-wasm-lab/
├── Cargo.toml
├── src/
│   └── lib.rs
├── payment.js
└── index.html
```

### `Cargo.toml`

以下へ置き換える。

```toml
[package]
name = "kaspa-wasm-lab"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "=0.2.100"
```

### `src/lib.rs`

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn can_cover_payment(available: u32, amount: u32, fee: u32) -> bool {
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
    #[test]
    fn supports_maximum_and_zero_fee() {
        assert!(can_cover_payment(u32::MAX, u32::MAX, 0));
        assert!(can_cover_payment(1, 1, 0));
    }
    #[test]
    fn empty_balance_cannot_pay() {
        assert!(!can_cover_payment(0, 1, 0));
    }
}
```

`u32`はブラウザ境界の例を簡単にするために選んだ。**実際のKaspa金額型を`u32`に制限してよいという意味ではない**。

Rust側のWASM教材6テストと、23.3/23.8を連結した金額・Gateway教材9テストが成功した。実際のノードとの接続試験ではない。

### JavaScript側の検査と読み込み

`payment.js`では、Rustへ渡す前に値域を検査する。Rust関数のchecked_addだけでは、変換前の不正値は検出できない。

```javascript
import { can_cover_payment } from "./pkg/kaspa_wasm_lab.js";

export function canCoverPayment(available, amount, fee) {
    for (const value of [available, amount, fee]) {
        if (!Number.isSafeInteger(value) || value < 0 || value > 0xffff_ffff) {
            throw new RangeError("Expected an integer in the u32 range");
        }
    }
    return can_cover_payment(available, amount, fee);
}
```

`index.html`はES moduleを読み込み、WASMの初期化完了を待ってから呼ぶ。

```html
<!doctype html>
<html lang="ja">
<meta charset="utf-8">
<title>WASM payment lab</title>
<pre id="result">Loading...</pre>
<script type="module">
import init from "./pkg/kaspa_wasm_lab.js";
import { canCoverPayment } from "./payment.js";
await init();
document.getElementById("result").textContent = String(canCoverPayment(1000, 900, 100));
</script>
</html>
```

### 実行手順

作業プロジェクト内で実行する。Rust/Cargo 1.90.0、wasm-pack 0.12.1、wasm-bindgen 0.2.100を今回の教材の基準とする。Kaspa SDK自体のツールチェーン指定ではない。

```bash
cargo test
rustup target add wasm32-unknown-unknown
# wasm-packが未導入の場合だけ実行する
cargo install wasm-pack --version 0.12.1 --locked
wasm-pack build --target web --dev
python3 -m http.server 8080 --bind 127.0.0.1
```

初回の依存・target・ビルドツールの取得にはネットワークが必要な場合がある。今回のCargo検査は依存キャッシュを使い`--offline`で実行した。wasm-packはcrateに対応したwasm-bindgen CLIも必要とする。crateとCLIの版を揃える。

`http://127.0.0.1:8080/`を開くとtrueを表示する構成である。`file://`で直接開かず、生成されたJSとWASMを一緒にHTTP配信する（[公式の読み込み例](https://wasm-bindgen.github.io/wasm-bindgen/examples/without-a-bundler.html)）。ブラウザーのDOM表示の実行確認は、ビルドやNode.jsでのWASM実行とは分けて記録する。

今回、web向けJS/WASM生成とNode.js 20.0.0からの呼び出しを確認した。正常・算術境界8件、各引数の不正値39件、生のバインディングの変換挙動4件の計51項目を検査した。NodeではWASMのバイト列を明示して初期化しており、ブラウザーのHTTP読み込み・DOM表示は未実行である。再実行用の検査コードと制限は `experiments/chapter-23-review-2026-11-01.md` に示す。

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
pub struct NodeStatus {
    pub network: NetworkId,
    pub synced: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayError {
    Unavailable,
    InvalidResponse,
}

pub trait NodeGateway {
    fn status(&self) -> Result<NodeStatus, GatewayError>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum AppError {
    Node(GatewayError),
    WrongNetwork,
    NotSynced,
}

pub fn ensure_ready(
    gateway: &impl NodeGateway,
    expected_network: NetworkId,
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
mod gateway_tests {
    use super::*;

    struct FakeGateway(Result<NodeStatus, GatewayError>);

    impl NodeGateway for FakeGateway {
        fn status(&self) -> Result<NodeStatus, GatewayError> {
            self.0.clone()
        }
    }

    #[test]
    fn ready_on_expected_network() {
        let gateway = FakeGateway(Ok(NodeStatus {
            network: NetworkId("testnet-10"),
            synced: true,
        }));
        assert_eq!(ensure_ready(&gateway, NetworkId("testnet-10")), Ok(()));
    }

    #[test]
    fn rejects_wrong_network() {
        let gateway = FakeGateway(Ok(NodeStatus {
            network: NetworkId("mainnet"),
            synced: true,
        }));
        assert_eq!(
            ensure_ready(&gateway, NetworkId("testnet-10")),
            Err(AppError::WrongNetwork)
        );
    }

    #[test]
    fn rejects_unsynced_node() {
        let gateway = FakeGateway(Ok(NodeStatus {
            network: NetworkId("testnet-10"),
            synced: false,
        }));
        assert_eq!(
            ensure_ready(&gateway, NetworkId("testnet-10")),
            Err(AppError::NotSynced)
        );
    }
    #[test]
    fn preserves_transport_and_decode_errors() {
        for error in [GatewayError::Unavailable, GatewayError::InvalidResponse] {
            let gateway = FakeGateway(Err(error.clone()));
            assert_eq!(
                ensure_ready(&gateway, NetworkId("testnet-10")),
                Err(AppError::Node(error))
            );
        }
    }

    #[test]
    fn testnet_number_must_match() {
        let gateway = FakeGateway(Ok(NodeStatus {
            network: NetworkId("testnet-11"),
            synced: true,
        }));
        assert_eq!(
            ensure_ready(&gateway, NetworkId("testnet-10")),
            Err(AppError::WrongNetwork)
        );
    }
}
```

23.3のNetworkIdを使い、Testnetの番号まで照合する。実アプリではSDKの識別子型や検証済みの値へ変換する。これはエラー変換に絞ったモック教材であり、[第22章](chapter-22.md)の応答の鮮度・必要なUTXO index等の検査は省略している。実装時にはその要件も引き継ぐ。

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

基準版のWASM RpcClient.connectは通知タスクと内部クライアントを起動する。一方、subscribe_utxos_changedにはlistenerがないとログを出してOkを返す分岐がある。購読メソッドの正常終了だけで購読の生存を判断せず、対象SDKの接続イベントと購読順序、実通知、再照会による補完を確認する。これは固定ソースの照合であり、今回切断中の購読を実測した結果ではない。

### 冪等性

送信処理のタイムアウトは「送信されなかった」ことを意味しない。送信前からTX ID・署名済み取引・業務上の支払い意図との対応を保持し、成否を照会する。同一署名済み取引の再送と、別の取引の再構築を区別する。Promiseの期限切れも元の処理の中止を保証しない。

## 23.10 イベント購読と状態の整合性

トランザクションの追跡は、単発のRPC応答だけで完結しない。

```text
Store intent / signed TX / TX ID
       │
       ▼
Submit Transaction
       │
       ▼
RPC Response / Unknown outcome
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
| `submit_unknown` | 送信の成否不明 | 既知のTX IDで照会 |
| `observed` | 接続ノードで存在を観測 | 状態再照会 |
| `accepted` | ノードのVirtual評価で受理を確認 | 確定ポリシー評価 |
| `settled` | アプリの確定条件を満たす | 業務処理完了 |
| `rejected` | 無効・競合など | 原因記録 |
| `reverted` | 以前の受理状態が変わった | 状態と業務記録を再評価 |

この表は**アプリケーション独自の状態モデル**であり、Kaspa RPCが同名のステータスを提供するという意味ではない。Mempool受理とVirtualでの受理を区別し、単調な一方向の遷移とも考えない。現在UTXOの再照会は、切断中の全取引履歴や会計台帳の復元とは別である。

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
Full NetworkId (including testnet suffix)
RPC transport and endpoint
```

依存関係・環境を確認する例:

```bash
rustc --version
cargo --version
cargo tree --locked
cargo metadata --locked --no-deps
```

単一crateの直接依存を固定するだけでは推移的依存まで固定できない。`Cargo.lock`はアプリケーションで原則コミットし、再現性を確保する。ライブラリの配布方針は用途に応じて検討する。

## 23.15 `rusty-kaspa`の読み方

最初から全コードを読むのではなく、**アプリケーションが呼びたい操作から逆向きに依存関係を追う**。

1. 対象リリース・コミットを固定する。
2. READMEやexamplesから、クライアントの初期化方法を確認する。
3. 接続処理の型・戻り値・エラーを調べる。
4. RPCメソッドがどのノード機能へ到達するか追跡する。
5. WASMバインディングの公開境界を確認する。
6. 統合テスト・既存サンプルを実行する。

```bash
# 第1章で取得した固定版の親ディレクトリから実行する
cd rusty-kaspa-v1.0.1
git rev-parse HEAD
grep -n -E 'pub fn new|pub async fn connect' rpc/wrpc/client/src/client.rs
grep -n -E 'subscribeUtxosChanged|subscribe_utxos_changed' rpc/wrpc/wasm/src/client.rs
grep -n -E 'wasm32-sdk|wasm32-rpc|crate-type' wasm/Cargo.toml
```

未取得なら[第1章](../part-01-foundations/chapter-01.md)の手順を使う。HEADが本章冒頭の完全SHAと一致することを確認し、別の版へAPIを無条件に読み替えない。

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
7. テスト用のトランザクションを送信し、結果を追跡する。

> RPCのポート、フラグ、メソッド名は利用するノードのバージョンに合わせて公式ドキュメントで確認する。Mainnetの資金を使って実験しない。

### 公式配布SDKで確認した範囲

[第28章](chapter-28.md)の28.6で、v1.0.1公式Node.js/WASM SDKの取得・SHA-256照合・WebSocket依存の導入を行う。Node.js 20.0.0で単一署名送金とUTXO通知を実測した。JSON接続のenumは `Encoding.SerdeJson`。金額はsompiの整数として扱い、SDKから返る `bigint` をJavaScriptの `number` へ無条件に変換しない。

この過去の実測は、本章の独自Rust/WASMモジュールのビルドやブラウザー実行とは別である。公式SDK配布物の利用検証と、独自WASMモジュールの開発・ビルド検証を分ける。HDウォレット復旧と全Scriptへの対応も未検証である。

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
- トランザクション追跡の滞留件数
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
4. RPC送信がタイムアウトした場合、同一署名済み取引の再送と、成否不明のまま別の取引を作ることはどう違うか。
5. `NodeGateway`を抽象化する利点は何か。
6. 秘密鍵をWASM内で扱うだけでは安全性が保証されない理由は何か。
7. 再接続後にイベント購読だけを再開しても不十分な場合があるのはなぜか。
8. 単体テストと統合テストで確認できる範囲と、その限界は何か。
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

[次章「Wallet Architecture & Key Management」](chapter-24.md)では、鍵生成、HDウォレット、アドレス、署名、バックアップと復旧をさらに掘り下げる。

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

本文・固定ソース・独自教材の確認範囲は `experiments/chapter-23-review-2026-11-01.md`。全SDK、実際のRust RPCアダプター、ブラウザーDOM、HD復旧、全Scriptを検証したものではない。
