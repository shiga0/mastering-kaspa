# Chapter 34 — vProgs & Programmability（vProgsとKaspaのプログラマビリティ）

> **原稿ステータス：v0.1（2026-10-30、技術レビュー前）**  
> 本章は研究構想・実装中の機能・メインネットで利用可能な機能を区別する。vProgsのAPIやプロトコルは変更され得る。

## 34.1 本章の目的

Kaspaのプログラマビリティを「EVMのような汎用VMがL1ですべてを実行する」という先入観から切り離し、**順序付け（sequencing）・実行（execution）・検証（verification）・状態保存（state storage）**を別々の責務として理解する。

学習目標：

1. Covenants、Based Apps、Inline ZK、Full vProgsの違いを説明できる。
2. 「証明できる計算」と「誰でも状態を再現できること」の違いを説明できる。
3. 複数アプリを横断する同期的・原子的な合成が難しい理由を説明できる。
4. Rustで状態遷移と不変条件を実装し、テストできる。
5. 研究仕様と実装・稼働状況を一次資料で追跡できる。

## 34.2 2026年10月時点の位置付け

Kaspaの公式開発者向け案内では、**Covenants**、**Based Apps**、専門用途の**Inline ZK**を区別している。さらに、独立したアプリ間の合成を目指す**Full vProgs**は将来の方向性として説明されている。

| 技術・モデル | 主な役割 | 2026年10月時点の読み方 |
|---|---|---|
| Covenants | L1上のUTXO支出条件・状態遷移制約 | Toccata以降のメインネット機能と、実際に利用可能なopcodeを確認する |
| Inline ZK | アクションごとの証明をL1で検証 | ZK検証機能と、アプリの完成度は別 |
| Based Apps | 共有状態を持つアプリの実行モデル | 開発者向け環境・ツールチェーンの成熟度を個別に確認する |
| Full vProgs | 独立アプリ間の同期的な合成 | 研究・開発段階。完成した汎用メインネット基盤とみなさない |

公式サイトによれば、ToccataによりCovenants、トランザクション内省、ZK検証、Based Apps向けの順序付け支援が導入され、2026年6月30日からメインネットで稼働している。一方、公開 `kaspanet/vprogs` リポジトリはプロトタイプ段階と明記する。**基盤機能の有効化 ≠ Full vProgsの完成**である。

参照：[Kaspa Programmability](https://docs.kaspa.org/programmability)、[Kaspa Build](https://kaspa.org/build)、[vprogs repository](https://github.com/kaspanet/vprogs)。

## 34.3 なぜプログラマビリティを分離するのか

汎用的なL1実行では、すべての検証ノードにアプリの計算や状態管理を要求しやすい。複雑なアプリが増えるほど、実行・ストレージ・帯域の負担が拡大する。

vProgs研究の方向性は、アプリ側で計算を行い、**計算が規則に従ったことを暗号学的に検証する**構造を通じて、アプリごとの主権と合成可能性の両立を探ることにある。

```text
利用者の操作
    |
    v
Kaspa L1: 取引の順序・利用可能なコミットメント
    |
    +-------------------+-------------------+
    |                   |                   |
    v                   v                   v
App A の実行         App B の実行        App C の実行
    |                   |                   |
    v                   v                   v
状態差分・証明        状態差分・証明        状態差分・証明
    \                   |                  /
     +------------------+-----------------+
                        |
                        v
              L1上の検証・状態コミット
```

この図は**研究構想を理解するための概念図**であり、稼働中の完全なプロトコル構成図ではない。

## 34.4 4つの責務

| 責務 | 問い | 代表的な検証項目 |
|---|---|---|
| Sequencing | 操作はどの順序で並ぶか | 順序の一意性、再編成への対応 |
| Execution | 操作で状態はどう変わるか | 決定性、入力検証、資源制限 |
| Proving / Verification | 遷移が正しいとどう示すか | 証明鍵、公開入力、証明検証 |
| State / Data Availability | 必要なデータを誰が保持・取得できるか | データ欠損、再構築、可用性 |

重要なのは、**証明が正しくても、ユーザーが必要なデータを取得できるとは限らない**こと。また、ゼロ知識証明を使うからといって、トランザクション内容が自動的に非公開になるわけではない。

## 34.5 vProgの抽象モデル

アプリ `p` を、状態集合 `S_p` と状態遷移関数 `F_p` で表す。

```text
S_p(t+1) = F_p(S_p(t), operation, context)
```

`context` には、確定した操作順序、参照状態、プロトコルが定める環境情報などが含まれ得る。ここで必要な不変条件は次のとおり。

- 同じ状態・入力・コンテキストから同じ結果が得られる（決定性）。
- 所有していないアプリ状態を無断で書き換えない（状態所有権）。
- 失敗した操作が中途半端な更新を残さない（原子性）。
- 公開されたコミットメントと検証対象の状態が一致する（整合性）。

## 34.6 同期的コンポーザビリティとは

例えば、DEXの交換と融資アプリの担保預け入れを一つの操作として扱いたいとする。

```text
操作 T:
  (1) App A: 100 token X を token Y に交換
  (2) App B: 受け取った token Y を担保に預ける

望ましい性質:
  両方成功 → 一括確定
  どちらか失敗 → 両方取り消し
```

別々のアプリが独立に状態を管理すると、単に「同じ順序を見ている」だけでは原子性を保証できない。読み書き依存関係、証明の結合、確定ルールを定義する必要がある。

**非同期メッセージング**では、Aの処理完了後にBへメッセージを送るため、途中失敗や補償処理が問題となる。**同期的・原子的合成**では、関連する操作を一体として扱うため、相互依存の追跡と検証負担が問題となる。

## 34.7 Computation DAG（計算依存DAG）

BlockDAGはブロック間の参照関係を表す。一方、vProgs研究で論じられる**Computation DAG**は、アプリ間の操作や状態参照による計算依存関係を表す。**両者を同じDAGとして扱わない。**

```text
BlockDAG                       Computation DAG（概念例）
  block A    block B             A:注文作成
      \      /                       |
       block C                   B:残高更新
                                    |
                               C:担保判定
```

計算依存の履歴が長くなるほど、後続の操作を検証するために必要な計算範囲（computational scope）が増える可能性がある。研究モデルでは、定期的な証明・状態コミットが依存履歴を圧縮し、再計算範囲を縮める役割を担う。

参照：[Computation DAGの形式モデル](https://research.kas.pa/t/zoom-in-a-formal-backbone-model-for-the-vprog-computation-dag/407)。

## 34.8 Proof Stitching（証明の結合）

アプリAの状態を参照するアプリBの操作を検証するには、個々の証明が正しいだけでなく、**それらが同じ履歴・同じ参照状態を扱っていること**も確認しなければならない。

研究仕様のProof Stitchingは、個別の状態遷移証明や状態コミットメントを、依存関係と整合する形で結び付けることを目指す。

```text
Proof A ──┐
          ├── 状態参照の整合性 ──> 合成結果の検証
Proof B ──┘
```

これは単純な証明ファイルの連結ではない。公開入力、参照する状態ルート、順序、所有権、対象操作を検証する必要がある。

## 34.9 L1で何を検証するのか

KIP-16は、Kaspa ScriptにZK検証のための `OpZkPrecompile` を導入する提案で、公開仕様上のステータスは **Active** である。これはオフチェーン計算の正しさをL1上で検証するための基盤だが、**任意のvProgアプリが完成していることを意味しない**。

一般的な証明検証は次の形で表せる。

```text
Verify(verifying_key, public_inputs, proof) -> true / false
```

公開入力には、旧状態コミットメント、新状態コミットメント、対象操作のコミットメントなどが含まれ得る。ただし、**実際のフィールドや証明形式は採用された仕様・実装に従う**。

参照：[KIP-16](https://github.com/kaspanet/kips/blob/master/kip-0016.md)。

## 34.10 Based AppsとFull vProgs

**Based Apps**は、Kaspaの順序付けを利用して共有状態アプリを構築する方向性として案内されている。**Full vProgs**は、アプリごとに独立した状態管理を維持しながら、アプリ横断の同期的合成を目指す、より広い研究目標である。

「Based AppsでRustアプリが書ける」という説明から、「Full vProgsの合成プロトコルがすべて実装済み」と推論してはいけない。

## 34.11 既存L2・EVMとの比較

| 観点 | 一般的なEVM型L2 | vProgs研究が目指す方向 |
|---|---|---|
| アプリ実行 | 共通VM内のコントラクト実行が中心 | アプリごとの独立実行を重視 |
| 状態 | L2内の共有状態が中心 | アプリごとの状態所有権 |
| 合成 | 同じ実行環境内では比較的直接的 | 独立アプリ間の依存・証明結合が課題 |
| L1との関係 | 決済・DA・検証等の設計次第 | Kaspaを共有順序付け・検証の基盤にする構想 |
| 実装成熟度 | 個別チェーンごとに異なる | Full vProgsは研究・プロトタイプ段階 |

「L2だからL1の安全性を継承する」「ZKだから必ず安全」という短絡は避ける。証明システム、データ可用性、アップグレード権限、資産管理、退出方法まで評価する。

## 34.12 C4モデル：責務と境界

### Context

```text
利用者 → Wallet / Client → App Node
                             |
                             +→ Kaspa Node
                             +→ Prover
                             +→ State Store
```

### Container（概念モデル）

```text
Client
  |
  v
API / Transaction Gateway
  |
  +--> Sequencing Adapter ----> Kaspa RPC
  |
  +--> Application Runtime ---> State Store
  |          |
  |          v
  |        Prover
  |
  +--> Proof / Commitment Publisher
```

### 変更理由による責務分離

- `domain`：残高・注文・担保などの業務ルール。
- `runtime`：決定的な状態遷移の実行。
- `sequencing`：順序・再編成の追跡。
- `proving`：証明生成・検証のアダプター。
- `storage`：状態の永続化・スナップショット。
- `api`：外部からの操作受け付け。

これは学習用の設計例であり、公式リポジトリのディレクトリ構造をそのまま再現したものではない。

## 34.13 Rustで最小の状態遷移を実装する

まずZK証明を実装する前に、**決定的な業務ルール**を作る。以下は標準ライブラリだけで動く教育用モデルであり、Kaspa SDKやvProgsランタイムとの互換性はない。

```rust
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct State {
    balances: BTreeMap<String, u64>,
}

#[derive(Clone, Debug)]
struct Transfer {
    from: String,
    to: String,
    amount: u64,
}

#[derive(Debug, PartialEq, Eq)]
enum Error {
    ZeroAmount,
    InsufficientFunds,
    Overflow,
}

fn apply(state: &State, tx: &Transfer) -> Result<State, Error> {
    if tx.amount == 0 {
        return Err(Error::ZeroAmount);
    }
    let mut next = state.clone();
    if tx.from == tx.to {
        let balance = *next.balances.get(&tx.from).unwrap_or(&0);
        if balance < tx.amount {
            return Err(Error::InsufficientFunds);
        }
        return Ok(next);
    }
    let sender = *next.balances.get(&tx.from).unwrap_or(&0);
    let receiver = *next.balances.get(&tx.to).unwrap_or(&0);
    if sender < tx.amount {
        return Err(Error::InsufficientFunds);
    }
    let new_receiver = receiver.checked_add(tx.amount).ok_or(Error::Overflow)?;
    next.balances.insert(tx.from.clone(), sender - tx.amount);
    next.balances.insert(tx.to.clone(), new_receiver);
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn initial() -> State {
        State { balances: BTreeMap::from([
            ("alice".into(), 100), ("bob".into(), 20)
        ]) }
    }

    #[test]
    fn transfer_preserves_total() {
        let before = initial();
        let after = apply(&before, &Transfer {
            from: "alice".into(), to: "bob".into(), amount: 30
        }).unwrap();
        assert_eq!(after.balances["alice"], 70);
        assert_eq!(after.balances["bob"], 50);
        assert_eq!(after.balances.values().sum::<u64>(),
                   before.balances.values().sum::<u64>());
    }

    #[test]
    fn rejected_transfer_does_not_change_input() {
        let before = initial();
        let snapshot = before.clone();
        let result = apply(&before, &Transfer {
            from: "alice".into(), to: "bob".into(), amount: 101
        });
        assert_eq!(result, Err(Error::InsufficientFunds));
        assert_eq!(before, snapshot);
    }

    #[test]
    fn self_transfer_does_not_mint_money() {
        let before = initial();
        let after = apply(&before, &Transfer {
            from: "alice".into(), to: "alice".into(), amount: 10
        }).unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn overflow_is_rejected() {
        let before = State { balances: BTreeMap::from([
            ("alice".into(), 1), ("bob".into(), u64::MAX)
        ]) };
        assert_eq!(apply(&before, &Transfer {
            from: "alice".into(), to: "bob".into(), amount: 1
        }), Err(Error::Overflow));
    }
}
```

### 実行方法

`src/main.rs` にコードを貼り付け、末尾に `fn main() {}` を追加する。

```bash
cargo new minimal-vprog-model
cd minimal-vprog-model
cargo test
```

この実装が保証するのは、**この関数に渡された状態と操作についてのローカルな規則**だけである。署名、二重実行、順序、永続化、ZK証明、Kaspa L1へのコミットは未実装。

## 34.14 次の段階：証明対象の仕様を定義する

ZK回路を選ぶ前に、以下を固定する。

1. `old_state_root`：どの旧状態を使うか。
2. `operation_commitment`：何を実行したか。
3. `new_state_root`：実行後の状態は何か。
4. `program_id`：どの規則で検証するか。
5. `sequence_reference`：どの順序に基づくか。

```text
Public Inputs:
  old_root, operation_hash, new_root, program_id, sequence_ref

Private / Witness Inputs:
  必要な状態データ、実行の中間値など

Constraint:
  new_state = F(old_state, operation, context)
  commit(old_state) == old_root
  commit(new_state) == new_root
```

**注意**：これは証明システム設計の一般的な擬似仕様であり、Kaspa vProgsの正式な回路仕様ではない。

## 34.15 ステートマシンと原子性

```text
Received
   |
   v
Ordered
   |
   v
Executed ----失敗----> Rejected
   |
   v
Proving
   |
   v
ProofVerified
   |
   v
Committed
```

この状態遷移図も教育用の概念モデル。実際にはチェーン再編成、証明生成の遅延、データ欠損、タイムアウト、再送、重複操作を扱う必要がある。

原子性の不変条件：

```text
commit(A) && commit(B)  または  !commit(A) && !commit(B)
```

実際のプロトコルでこれを満たすには、アプリをまたぐ共通の確定条件が必要となる。

## 34.16 セキュリティと脅威モデル

| 脅威 | 失敗例 | 検証方法 |
|---|---|---|
| 不正な状態遷移 | 残高の不正増加 | 不変条件・証明検証 |
| データ欠損 | 状態を再構築できない | DA・復元試験 |
| 証明の参照ずれ | 異なる旧状態を参照 | 公開入力と状態ルートの照合 |
| 再送・リプレイ | 同じ操作を重複適用 | nonce・操作ID・順序検証 |
| 再編成 | 未確定順序を確定と誤認 | rollback / replay |
| アプリ間の競合 | 同じ資源を同時に更新 | 依存関係とスケジューラ検証 |
| DoS | 過大な証明・依存範囲 | 資源計量・上限・タイムアウト |
| 運用者への依存 | 単一prover停止 | 冗長化・退出設計 |

## 34.17 公開 `vprogs` Rustリポジトリの読み方

公式の公開リポジトリは**初期開発・プロトタイプ**と明記され、概ね以下のレイヤーを説明している。

| レイヤー | 役割 |
|---|---|
| `core/` | 基本型・共通抽象 |
| `storage/` | 永続化 |
| `state/` | 状態・バージョン管理 |
| `scheduling/` | 実行順序・依存関係の調整 |
| `transaction-runtime/` | トランザクション実行の意味論 |
| `node/` | 全体統合 |

READMEはtrait境界、バッチ処理、`StateDiff`、`Rollback`などを説明している。ただし、**実際の最新ソースを開いて確認するまで、型名・公開API・互換性を保証しない**。

```bash
git clone https://github.com/kaspanet/vprogs.git
cd vprogs
git rev-parse HEAD
cargo test
```

ビルドに失敗したら、まずRust toolchain、READMEの要件、Cargo workspace構成を確認する。実行結果を記録し、リポジトリのコミットハッシュを原稿に残す。

## 34.18 テスト戦略

### 単体テスト

- 状態遷移の決定性。
- 資産総量保存（発行・焼却のないモデル）。
- 失敗時の状態不変。
- 所有していない状態への書き込み拒否。

### 性質ベーステスト

- 任意の送金列でも残高が負にならない。
- 任意の入力順序で、**同じ確定順序**を与えた場合の結果は一致する。
- 証明対象の公開入力が変われば、元の証明を再利用できない。

### 統合テスト

- L1順序参照 → アプリ実行 → 証明 → コミットメント検証。
- 依存する2アプリの片方が失敗した場合の原子性。
- Prover停止、ノード再起動、再編成、データ欠損。

### 差分テスト

素朴な参照実装と最適化した実装に同じ操作列を入力し、状態ルートや結果を比較する。

## 34.19 最小プロダクトを設計する

学習用の題材として「残高移動アプリ」と「注文アプリ」を考える。

```text
Account App                 Order App
  残高管理                     注文管理
      |                           |
      +-----------+---------------+
                  |
             共通操作ID
                  |
           Sequencing Adapter
                  |
               Kaspa L1
```

**MVPの段階では、まずオフチェーンの単一プロセスで原子性を実現し、テストする。** 次にアプリを分割し、順序付けと証明の境界を抽象化する。実際のvProgs互換性を主張するのは、正式な実装・仕様で動作を確認してからにする。

## 34.20 実践課題

**課題A：ドメインモデル**  
2つのアプリが持つ状態、書き込み権限、不変条件を表にする。

**課題B：Rust実装**  
第34.13節の `apply` を、複数操作のバッチに拡張する。途中で1件失敗した場合、バッチ全体を拒否する。

**課題C：再編成**  
順序が `T1 → T2` から `T2 → T1` に変わった場合、どの状態を巻き戻し、何を再実行するかを設計する。

**課題D：証明境界**  
`StateTransition`, `Prover`, `Verifier`, `StateStore`, `Sequencer` のtraitを設計する。実装ではなく責務と依存関係を先に定義する。

**課題E：一次資料調査**  
`vprogs` のコミットハッシュを固定し、READMEの各レイヤーに対応するRustコードとテストを1つずつ特定する。

## 34.21 理解度チェック

1. BlockDAGとComputation DAGは何が違うか。
2. 順序付けが共通でも、なぜ複数アプリの原子性は自動的に保証されないか。
3. ZK証明の検証とデータ可用性は何が違うか。
4. CovenantsとBased Appsはどのような要件で使い分けるか。
5. Full vProgsが研究段階であることは、アプリ設計にどのような影響を与えるか。
6. 状態ルートを公開入力に含める理由は何か。
7. `Rollback` が必要になる障害シナリオを3つ挙げられるか。

## 34.22 まとめ

Kaspaのプログラマビリティは、L1のCovenantsやZK検証などの**既存の基盤**と、Based AppsやFull vProgsなどの**発展中の実行モデル**を分けて考えると理解しやすい。

vProgs研究の中心課題は、アプリの独立性を保ちながら、共有順序付けと暗号学的検証を用いてアプリ間の合成を成立させることにある。設計では、状態所有権、計算依存DAG、Proof Stitching、データ可用性、再編成、原子性が重要になる。

Rustで学ぶ際は、**決定的な状態遷移 → 不変条件のテスト → バッチ原子性 → 順序・巻き戻し → 証明境界**の順に実装すると、概念と実践を結び付けやすい。

## 34.23 参考資料・一次資料

- [Kaspa公式：Programmability Overview](https://docs.kaspa.org/programmability)
- [Kaspa公式：Build / 開発者向け案内](https://kaspa.org/build)
- [kaspanet/vprogs：公開Rustプロトタイプ](https://github.com/kaspanet/vprogs)
- [Kaspa Research：vProg Computation DAGの形式モデル](https://research.kas.pa/t/zoom-in-a-formal-backbone-model-for-the-vprog-computation-dag/407)
- [Kaspa Research：同期的合成可能なvProgsアーキテクチャ提案](https://research.kas.pa/t/concrete-proposal-for-a-synchronously-composable-verifiable-programs-architecture/387)
- [Kaspa Research：Proof Stitchingの基本フレームワーク](https://research.kas.pa/t/a-basic-framework-for-proofs-stitching/323)
- [kaspanet/research：vProgs Yellow Paper（草案）](https://github.com/kaspanet/research/blob/main/vProgs/vProgs_yellow_paper.pdf)
- [KIP-16：New Transaction Opcodes for Verifiable Computation](https://github.com/kaspanet/kips/blob/master/kip-0016.md)
- [Kaspa KIPs一覧](https://github.com/kaspanet/kips)

---

**次章：Chapter 35 — Building a Kaspa Application（Kaspaアプリケーションの構築）**
