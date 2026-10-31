# Argent — KaspaのActorベース・アプリケーション言語

> **位置づけ**：KaspaのCovenant対応UTXO上で、状態を持つ複数のコントラクトを1つのトランザクションとして組み合わせるための言語・コンパイラ・Rustランタイム。
>
> **開発状況（2026年10月時点）**：開発中。主要機能とE2Eサンプルは存在するが、Argent自体は一般向けの本番リリース前で、監査・堅牢化が必要。以下の機能説明はプロジェクトが想定するCovenant機能を前提とし、現在のKaspaメインネットでそのまま利用可能であることを意味しない。

## 1. Argentとは

Argentは、**Actorを状態付きUTXOとして表現する**アプリケーション言語である。複数のActorを1つのトランザクション内で消費・再生成することで、状態遷移を原子的に実行する。

通常の非同期Actorモデルと異なり、ArgentのActorにはメールボックスや常駐プロセスがない。状態はUTXOに保持され、更新時には古いUTXOを消費して後継UTXOを作る。

```text
Argent (.ag)
   │
   ▼
Argent Compiler
   ├── SilverScript (.sil)
   └── artifact.json
          │
          ▼
     argent-runtime (Rust)
          │
          ▼
    Kaspa transaction
          │
          ▼
    Kaspa Scriptによる検証
```

SilverScriptは単一コントラクトの言語・コンパイラ基盤であり、Argentはその上で複数Actor、状態遷移、アプリケーション間連携を記述する。Argentが生成する`.sil`は監査可能な通常のSilverScriptである。

## 2. なぜ必要なのか

Covenantを使ったアプリでは、単に「署名が正しいか」だけでなく、次のような条件を検証する必要がある。

- 更新後のUTXOが正しいコントラクトのテンプレートを使っているか
- 状態のシリアライズとハッシュが正しいか
- 許可された数・順番・種類の出力だけが生成されたか
- 他のActorと同じトランザクションで整合的に遷移したか
- 別アプリケーションのCovenantを安全に参照できるか

これらを低レベルのScriptで個別に記述すると、出力インデックス、テンプレートコミットメント、Witnessの組み立てが複雑になる。Argentは**アプリケーションの状態遷移をソースコードに明示し、検証用コードと実行用メタデータを生成する**。

## 3. UTXOとActor

例えばチケット発行システムでは、イベント管理Actorがチケットを発行する。

```text
取引前
  [Event UTXO: 残り100枚]
               │
               ▼ buy()
      1つのトランザクション
               │
         ┌─────┴──────────┐
         ▼                ▼
  [Event UTXO]       [Ticket UTXO]
   残り99枚           所有者 Alice
```

これは既存のEvent UTXOを書き換える処理ではない。**古いEvent UTXOを消費し、新しいEvent UTXOとTicket UTXOを出力する**。トランザクション全体が成立しなければ、片方だけが確定することはない。

### Actorの状態所有権

Actorは自身の状態レイアウトと、許可された状態遷移を定義する。別のActorが勝手に状態を書き換えるのではなく、対象UTXOを消費する際にそのActorのScriptによる検証を通す。

## 4. 言語の基本要素

| 要素 | 役割 |
|---|---|
| `state` | 永続状態の型・レイアウト |
| `actor` | 状態を所有するコントラクトのテンプレート |
| `entry` | 外部から呼び出す状態遷移 |
| `emits` | 遷移で生成可能なActor出力の宣言 |
| `become` | 後継Actorとその状態の指定 |
| `consumes` | 同一トランザクションで消費する他Actorの指定 |
| `delegate` | 協調遷移に参加する非主導Actorの検証 |
| `app` | 同じCovenant境界に属するActor集合 |
| `observes` | 別Covenantの状態・出力を参照するICC宣言 |
| `spawns` | 新しいCovenantの初期出力グループを宣言 |
| `actor_type<State>` / `actor enum` | 実行時に選択するActor型を制約付きで扱う |
| `virtual` / `expands` | 共通状態レイアウトと非公開の拡張状態を扱う |

### 例：チケット購入（学習用）

以下は公式READMEの構造を参考にした簡略例である。実行前には使用するArgentバージョンで検証すること。

```text
state EventState {
    int remaining_tickets;
    int price;
}

state TicketState {
    byte[32] owner;
}

actor Event owns EventState {
    entry buy(byte[32] buyer)
    emits {
        event: Event,
        ticket: Ticket,
    } {
        require(remaining_tickets > 0);
        require(event.value == self.value + price);
        unrestricted(ticket.value);

        EventState next_event = {
            remaining_tickets: remaining_tickets - 1,
            price: price,
        };
        TicketState new_ticket = { owner: buyer };

        become {
            event <- Event(next_event),
            ticket <- Ticket(new_ticket),
        };
    }
}

actor Ticket owns TicketState {
    entry transfer(byte[32] next_owner, sig owner_sig, pubkey owner_pk)
    emits next: Ticket {
        require(blake2b(byte[](owner_pk)) == owner);
        require(checkSig(owner_sig, owner_pk));
        unrestricted(next.value);
        TicketState new_state = { owner: next_owner };
        become next <- Ticket(new_state);
    }
}

app Tickets {
    actor Event;
    actor Ticket;
}
```

`require`は守るべき条件、`emits`は生成される出力の形、`become`は出力Actorの種類と状態を定義する。`unrestricted`は特定の出力値を制約しないという**明示的な宣言**であり、安全性の観点から意図を確認する必要がある。

## 5. 複数Actorの原子的な遷移

1つのトランザクションで複数のActor入力を消費する場合、`consumes`と`delegate`を使って協調する。中心となる`entry`が入力グループ全体の構成を検証し、参加するActor側の`delegate`も整合性を確認する。

例：ゲームの対戦結果確定、複数資産の交換、チケットとイベント在庫の同時更新。

**重要な不変条件**：許可していないActor入力・出力が紛れ込まないこと、各出力の承認元が明確であること、各後継Actorのテンプレートと状態が検証されること。

## 6. ICC：Inter-Covenant Communication

通常の`app`は閉じたActor集合を扱う。ICCは、**別々にコンパイルされたアプリケーション間でCovenantを参照・協調する仕組み**である。

- **Closed ICC**：参照先のActorやアプリの型を固定して連携する。
- **Open ICC**：状態に保持したActor型ハンドルなどを用いて、実行時に参照先を選択する。
- `observes`：外部Covenantの入力・出力を検証し、状態を読むための宣言。
- 依存するアプリの**artifact ID**を照合し、想定と異なる依存物を使わないようにする。

ICCは任意のActorへ自由にメッセージを送る機能ではない。**同一トランザクション内のCovenant入力・出力を検証可能な形で結び付ける**仕組みである。

## 7. コンパイラと生成物

Argentは`.ag`から次を生成する。

```text
build/
├── artifact.json    # Script・型・状態・経路・Witness等の情報
├── manifest.json    # ビルドメタデータ
└── sil/
    └── *.sil         # ActorごとのSilverScript
```

コンパイラは、後継Actorのテンプレートをあらかじめコミットできるよう、Actor間の遷移グラフを解析する。`become`は出力の検証に、`observes`は外部Covenantの照合に変換される。必要に応じて非公開のWitness情報やテンプレートの部分情報が生成される。

`artifact.json`は、Rustランタイムがトランザクションを組み立てるための可搬な契約情報である。**生成された`.sil`とArtifactを監査対象に含める**ことが重要。

## 8. Rustランタイム

`argent-runtime`はコンパイラ本体に依存せず、Artifactを読み込んでトランザクションを構築する。

主な役割：

1. Actorの初期状態とUTXOを読み込む。
2. 実行する`entry`と引数を指定する。
3. 消費するActor入力・通常入力を指定する。
4. 後継Actor出力と値を指定する。
5. 隠れたWitnessやScript情報を補完する。
6. 完成したトランザクションを組み立てる。

アプリケーションの業務ロジックはArgentで定義し、**UTXO取得、署名、トランザクション送信、状態追跡**などの周辺処理はRustアプリケーションが担う。

## 9. GenesisとCovenant ID

Covenantの開始時には、初期Actor群とその状態を確定させる必要がある。ArgentはGenesis証明の仕組みを用意しており、Covenant IDがどの初期構成に対応するかを検証できる。

Covenant IDや初期状態を単なる外部データベースの申告として信頼せず、**オンチェーンで観測されたIDと初期化データの整合性を検証する**ことが目的。

## 10. 高度な機能

### `spawns`

既存の遷移の中から、新たなCovenantのGenesis出力を制約付きで作る。単なる出力追加ではなく、生成されるグループの形とIDを検証する。

### `actor enum`と`actor_type<State>`

実行時に選択されるActorの実装を、互換な状態型とテンプレート情報で制約する。DEXやルーティングのように選択肢がある場合に役立つ。

### `virtual`と`expands`

共通の基本状態を維持しながら、具体的なActorが追加の状態を持つための機構。拡張部分をDigestと事前イメージで扱う設計がある。

## 11. セキュリティ上の論点

Argentを学ぶ際には、コードがコンパイルできるだけでなく、次を検証したい。

- **後継出力の完全性**：不正なActorや余分な出力が混ざらないか。
- **テンプレートの一致**：想定したScriptへ遷移するか。
- **状態の一貫性**：残高・所有者・在庫などの不変条件が守られるか。
- **署名・認可**：操作権限が正しく検証されるか。
- **ICCの依存関係**：参照するArtifactとCovenant IDが正しいか。
- **Genesisの真正性**：初期ActorとCovenant IDの関係を証明できるか。
- **実行コスト**：Scriptサイズ、opcode数、Witnessサイズが実用範囲か。

開発元は、生成Scriptのセキュリティ不変条件を個別の設計文書で説明している。Argentは開発中なので、生成コードを無条件に信用して本番資産を扱うべきではない。

## 12. ハンズオン：理解する順序

### 演習A：Counter

1. `argent-playground`の`basic_counter`を動かす。
2. Actorの状態を1増やすトランザクションを追う。
3. 生成された`.sil`と`artifact.json`を確認する。
4. 不正な次状態を指定して失敗を観察する。

### 演習B：Ticket

1. 公式の`examples/tickets.ag`を読む。
2. Issuerの発行前後の状態とTicket出力を図にする。
3. 署名検証と出力値の制約を確認する。
4. 二重発行や不正所有者を想定したテストを考える。

### 演習C：複数ActorとICC

1. `two_actor_exchange`で原子的な交換を確認する。
2. `multiapp_badge`で別アプリ間の連携を追う。
3. `open_icc_agent`で実行時Actorハンドルを確認する。
4. `dex_asset`で資産・ペア・レジストリの関係を整理する。

### ローカルでの確認コマンド

```bash
# Argentリポジトリ内
./check.sh
cargo run -- build examples/tickets.ag --out examples/build/tickets
cargo run --bin argentc -- inspect examples/build/tickets

# argent-playgroundリポジトリ内
cargo run --bin basic_counter
cargo run --bin two_actor_exchange
cargo run --bin multiapp_badge
```

コマンドや生成物は開発中に変更され得るため、実行時は各リポジトリのREADMEを優先する。

## 13. SilverScript・Argent・vProgsの違い

| 技術 | 主な関心 | 位置づけ |
|---|---|---|
| Kaspa Script | 個々のUTXOの支出条件を検証 | 基盤となるScript実行層 |
| SilverScript | 単一コントラクトの型付き高水準記述 | Scriptへコンパイルする言語 |
| Argent | 複数Actorの状態遷移と連携 | SilverScript上のアプリケーション言語・ランタイム |
| vProgs | 別系統のプログラマビリティ研究・設計 | Argentと同一技術ではない |

Argentは**EVMそのものでも、L2そのものでもない**。KaspaのCovenant対応UTXOを組み合わせるL1アプリケーションモデルとして理解する。

## 14. 制約と今後の課題

- Argentは一般向けの正式リリース前で、監査・堅牢化が継続中。
- 一部の範囲指定機能（`observes`や`spawns`のrangeなど）は実装途中。
- 生成Scriptのコストや複雑さはアプリケーション規模に依存する。
- 実際のネットワーク利用可能性は、対象Kaspaネットワークで有効なCovenant関連機能・コンセンサス仕様に依存する。

**本章の要点**：Argentは「UTXOをActorとして扱い、トランザクション全体の状態遷移を型付きソースで定義する」ための仕組みである。理解の核心は、`state` → `actor` → `entry` → `emits` → `become`と、生成されたSilverScriptによる検証を対応付けることにある。

## 参考資料（一次資料）

- [Argent公式README](https://github.com/argent-lang/argent/blob/master/README.md)
- [Argent公式ソース](https://github.com/argent-lang/argent)
- [Ticketsサンプル](https://github.com/argent-lang/argent/blob/master/examples/tickets.ag)
- [ICC semantics](https://github.com/argent-lang/argent/blob/master/docs/icc-semantics.md)
- [Security invariants](https://github.com/argent-lang/argent/tree/master/docs/security-invariants)
- [Range設計と実装状況](https://github.com/argent-lang/argent/blob/master/docs/entry-clause-ranges.md)
- [Argent Playground](https://github.com/argent-lang/argent-playground)
