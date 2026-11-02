# Chapter 34 — vProgs & Programmability（vProgsとKaspaのプログラマビリティ）

> **目次順の本文レビュー：2026-11-01。主要な公開設計・固定プロトタイプと独自Rust教材を確認。Full vProgsや本番動作の検証ではない。**
> 本章は研究構想・実装中の機能・メインネットで利用可能な機能を区別する。vProgsのAPIやプロトコルは変更され得る。

公開vprogsは `f9b84a863a7c7c20586a9cf947550475e894f72e` へ固定した。本書の基準ノードv1.0.1、固定KIP、Silverscriptと同一時点の機能集合ではない。確認記録は `experiments/chapter-34-review-2026-11-01.md`。

## 34.1 本章の目的

Kaspaのプログラマビリティを「EVMのような汎用VMがL1ですべてを実行する」という先入観から切り離し、**順序付け（sequencing）・実行（execution）・検証（verification）・状態保存（state storage）**を別々の責務として理解する。

学習目標：

1. Covenants、Based Apps、Inline ZK、Full vProgsの違いを説明できる。
2. 「証明できる計算」と「誰でも状態を再現できること」の違いを説明できる。
3. 複数アプリを横断する同期的・原子的な合成が難しい理由を説明できる。
4. Rustで状態遷移と不変条件を実装し、テストできる。
5. 研究仕様と実装・稼働状況を一次資料で追跡できる。

## 34.2 2026-11-01に確認した公式案内の位置付け

Kaspaの公式開発者向け案内では、**Covenants**、**Based Apps**、専門用途の**Inline ZK**を区別している。さらに、独立したアプリ間の合成を目指す**Full vProgs**は将来の方向性として説明されている。

| 技術・モデル | 主な役割 | 確認時点の読み方 |
|---|---|---|
| Covenants | L1上のUTXO支出条件・状態遷移制約 | Toccata以降のメインネット機能と、実際に利用可能なopcodeを確認する |
| Inline ZK | アクションごとの証明をL1で検証 | ZK検証機能と、アプリの完成度は別 |
| Based Apps | 共有状態を持つアプリの実行モデル | 公式案内では構築中。環境・ツールチェーンを個別に確認する |
| Full vProgs | 独立アプリ間の同期的な合成 | 研究・開発段階。完成した汎用メインネット基盤とみなさない |

公式の[Toccata Dev Guide](https://docs.kaspa.org/toccata)は、2026-06-30、DAA score 474_165_565でMainnet有効化と説明する。これは確認日に読んだ公式文書の記載であり、このレビューで本番ノードを観測した結果ではない。Based Appsの案内は構築中、Full vProgsは将来の方向性と明記する。一方、公開 `kaspanet/vprogs` リポジトリはプロトタイプ段階と明記する。**基盤機能の有効化 ≠ Full vProgsの完成**である。

参照：[Kaspa Programmability](https://docs.kaspa.org/programmability)、[Kaspa Build](https://kaspa.org/build)、[固定vprogs README](https://github.com/kaspanet/vprogs/blob/f9b84a863a7c7c20586a9cf947550475e894f72e/README.md)、[Based Apps](https://docs.kaspa.org/programmability/based-apps)、[Full vProgs](https://docs.kaspa.org/programmability/full-vprogs)。

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

計算依存の履歴が長くなるほど、後続の操作を検証するために必要な計算範囲（computational scope）が増える可能性がある。2025-08-17の形式モデルでは、read/write集合と状態Witnessを持つ操作列を定義し、証明・状態コミットで計算上の依存scopeを縮めることを論じる。履歴データを永久保存したり、必要な状態/Witnessを自動復元したりする仕組みと同一視しない。証明生成・検証やDAも含む全体コストが常に減るという結論でもない。

参照：[Computation DAGの形式モデル](https://research.kas.pa/t/zoom-in-a-formal-backbone-model-for-the-vprog-computation-dag/407)。

## 34.8 Proof Stitching（証明の結合）

アプリAの状態を参照するアプリBの操作を検証するには、個々の証明が正しいだけでなく、**それらが同じ履歴・同じ参照状態を扱っていること**も確認しなければならない。

2025-04-02のProof Stitchingの投稿は、logic zoneごとのsubtransactionにindex・入出力状態・call stack・証明を対応させるproof of conceptを説明する。2025-08-07の全体構想やComputation DAGの形式モデルと、単一の完成した仕様であるかのように混ぜない。研究仕様のProof Stitchingは、個別の状態遷移証明や状態コミットメントを、依存関係と整合する形で結び付けることを目指す。

```text
Proof A ──┐
          ├── 状態参照の整合性 ──> 合成結果の検証
Proof B ──┘
```

これは単純な証明ファイルの連結ではない。公開入力、参照する状態ルート、順序、所有権、対象操作を検証する必要がある。

## 34.9 L1で何を検証するのか

KIP-16は、Kaspa ScriptにZK検証のための `OpZkPrecompile` を導入する提案で、本書の固定KIP文書上のステータスは **Active** である。これはオフチェーン計算の正しさをL1上で検証するための基盤だが、**任意のvProgアプリが完成していることを意味しない**。

一般的な証明検証は次の形で表せる。

```text
Verify(verifying_key, public_inputs, proof) -> true / false
```

公開入力には、旧状態コミットメント、新状態コミットメント、対象操作のコミットメントなどが含まれ得る。ただし、**実際のフィールドや証明形式は採用された仕様・実装に従う**。

参照：[KIP-16](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0016.md)。

## 34.10 Based AppsとFull vProgs

**Based Apps**は、Kaspaの順序付けを利用して共有状態アプリを構築する方向性として案内されている。**Full vProgs**は、アプリごとに独立した状態管理を維持しながら、アプリ横断の同期的合成を目指す、より広い研究目標である。

公式Based Appsの案内は単一アプリ内の共有状態・組み込みアカウントと、app-localな合成を説明している。独立アプリ間のネットワークとしての合成とは範囲が異なる。

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

これは処理順や単一の原子的保存を表す図ではない。学習用の設計例であり、公式リポジトリのディレクトリ構造をそのまま再現したものではない。

## 34.13 第33章の状態遷移で順序依存を確かめる

[第33章33.20の残高移転教材](chapter-33.md#3320-rustで最小の状態遷移を作る)を再利用し、この章では**操作の順序と、複数操作をまとめる境界**に注目する。Kaspa SDKやvProgsランタイムとの互換性を持つコードではなく、単一プロセス内のアカウント残高モデルである。

### 同じ操作でも順序によって成否が変わる

第33章の `Balances` と `apply_transfer` を使い、初期状態をAlice=100、Bob=20とする。次の2操作を用意する。

```text
T1: AliceからBobへ80を送る
T2: BobからAliceへ50を送る
```

| 順序 | 1件目の結果 | 2件目の結果 | 成功した操作だけを反映した最終残高 |
|---|---|---|---|
| T1 → T2 | T1成功：Alice=20、Bob=100 | T2成功 | Alice=70、Bob=50 |
| T2 → T1 | T2失敗：Bobの残高不足 | 元の状態からT1成功 | Alice=20、Bob=100 |

この表では、失敗した操作を記録して処理を続ける教材上の方針を使う。第33章のプロジェクトへテストを追加し、順序ごとの結果と、失敗時に入力状態が変わらないことを確認する。同じ初期状態・操作列・規則なら同じ結果になることと、操作順を変えても同じ結果になることは別である。

### 順序の一致とバッチ原子性を分ける

複数アプリが同じ順序を見ていても、一方だけ状態をコミットすれば、操作全体の原子性は成立しない。34.20の課題Bでは、同じ移転関数を使い、複数操作を一つのバッチとして扱う。

候補状態へ順に適用し、全操作が成功した場合だけ新状態を返す。途中で失敗したら候補全体を破棄する方針なら、上の `T2 → T1` はT2でバッチ全体を拒否し、公開状態をAlice=100、Bob=20のまま維持する。操作単位で失敗を飛ばす場合とは結果が異なるため、どちらの規則かを明示してテストする。

順序が後から変わった場合は、既に反映した状態へ新しい順序の操作を追加するのではなく、共通の基準状態へ戻して再評価する。巻き戻しの範囲と再実行は課題Cで設計する。

### この比較で確認できる範囲

学ぶのは、決定的なローカル状態遷移・順序依存・メモリ内のバッチ境界である。署名、権限、nonce、永続化、証明生成、Kaspa L1へのコミット、独立アプリ間の原子性は実装しない。第33章のnonce追加課題は別の拡張であり、この比較がリプレイ防止を備えるという意味ではない。

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
  commit(operation) == operation_hash
  contextの順序・program/domain/versionを公開入力へ結び付ける
```

検証側もold_rootが現在受理している状態と一致すること、sequence_refが正しい履歴/順序に属すること、正しいプログラム/検証鍵であることを照合する。公開入力に名前を並べるだけでは、回路がその値を拘束したことにはならない。

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

これは同じ論理操作・整合した観測に対する安全性条件であり、全処理がいつか完了するという進行性は別に必要になる。単一関数がcloneを返す原子性だけでは、独立ノード/アプリ・永続化・外部作用の原子性は保証しない。

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

### 34.16.1 証明が正しくても資金を取り出せない場合

**問題：** ZK Proofが存在することだけで、アプリの業務ルールと運用まで安全だと思ってしまう。

**攻撃・停止例：** 送金者の承認を確認し忘れたプログラムに対する正しい証明は、不正な送金を排除しない。また、Operator（実行・取引集約・証明提出等を担当する運用者）が停止すれば、偽の状態を受理しなくても利用者の処理が止まり得る。これは証明の検証とは別の進行性の問題である。

**原因：** 証明対象のプログラム、公開入力、状態の出所、提出・退出経路の前提を一つの「ZKで安全」という主張にまとめている。

**設計上の対策：** 証明対象のコード・旧状態・操作・新状態の結び付きを固定し、承認と保存則をプログラム自身に含める。証明生成のCPU/GPU・メモリ・時間・費用を測り、誰が負担するかを決める。Operatorが停止したとき、他者が必要データを入手して証明を提出できるかも検討する。資金を預ける設計では、Escape Hatch（通常処理が止まった場合の退出経路）について、引き出せる主体、参照状態、必要なデータ・証拠、待機条件、手数料を定義する。DA（データ可用性）がなければ証明や退出の材料を再構成できない場合がある。[第33章](chapter-33.md)のDA・資産管理・退出の比較と併せて読む。

**検証方法：** 不正な承認・別旧状態・再送を拒否する試験に加え、Operator停止、データ未公開、証明生成失敗、通常経路停止中の引き出しを試す。Tic-Tac-Toeのような小さなゲームでも、証明を通す試験と、手番違反・使用済みマス・終局後の手を拒否する業務ルールの試験を分ける。これは検証課題の例であり、本書がゲームの証明や退出コントラクトを実装・実行したという意味ではない。

以上は設計時の評価項目で、現在のFull vProgsに完成したEscape Hatchや引き出しAPIがあるという主張ではない。Igra等の個別L2も、実行系・資産ブリッジ・DA・運用者・退出条件をそれぞれ調べる必要がある。Kaspaを使うことだけで、Full vProgsと同じ実行・証明・信頼モデルにはならない。

## 34.17 公開 `vprogs` Rustリポジトリの読み方

READMEはbased computationのframeworkと記述する。リポジトリ名vprogsだけでFull vProgsの合成プロトコルが完成したとは判断しない。

固定[README](https://github.com/kaspanet/vprogs/blob/f9b84a863a7c7c20586a9cf947550475e894f72e/README.md)は初期開発・プロトタイプと明記する。確認したHEADは冒頭の完全SHA、コミット日は2026-07-28。READMEの層構造に加え、現在の主要配置を照合した。

| 対象 | 固定版で読んだ入口 | 確認した役割 |
|---|---|---|
| core | core/types/src/lib.rs | ResourceId / AccessMetadata / SchedulerTransaction等 |
| storage | storage/types/src/lib.rs | ReadStore / Store / WriteBatchの境界 |
| state | state/version/src/lib.rs | StateVersionとバージョン付き状態 |
| scheduling | scheduling/scheduler/src/lib.rs、rollback.rs | ScheduledBatch / StateDiff / Rollback等 |
| transaction-runtime | transaction-runtime/program/src/lib.rs | ProgramTypeと実行用ELF bytes |
| node | node/vm/src/lib.rs | 統合用VMのProcessor実装。ただし下記の未実装部分がある |
| L1接続 | l1/bridge/src/lib.rs | チェーン追跡・再編成をschedulerへ接続する責務 |
| zk / runner等 | Cargo.toml、zk/abi/src/lib.rs | 証明用ABI・実行/統合の追加領域 |

[core/types](https://github.com/kaspanet/vprogs/blob/f9b84a863a7c7c20586a9cf947550475e894f72e/core/types/src/lib.rs)の実際の型名はSchedulerTransactionで、README中のTransactionという略記と区別する。Rollbackの型は存在するが、scheduling/schedulerのlib.rsはそれを公開再exportしていない。文書にある名前を外部からそのまま使えるAPIだと判断しない。

[node/vm](https://github.com/kaspanet/vprogs/blob/f9b84a863a7c7c20586a9cf947550475e894f72e/node/vm/src/lib.rs)のprocess_transactionはこの固定版でtodo!を含み、receipt用image IDも0を返す。この経路を完成した汎用VMと扱わない。一方、別のrunner等の存在もあるため、この1か所からリポジトリ全体が動かないと一般化もしない。

Cargo metadataを--offline --locked --no-depsで取得し、61パッケージを列挙できた。依存解決・ビルド/全テストの成功ではない。Cargo.tomlはKaspa依存をmasterのbranch指定で記述するが、固定Cargo.lockは `9fdbaf1b1886fb5742783d2da73c36239bd6f272` を指す。ソースとlockfileを一緒に固定し、本書基準版やSilverscriptの依存へ置き換えない。

```bash
# 本書とは別の調査用ディレクトリで取得する
git clone https://github.com/kaspanet/vprogs.git vprogs-review
git -C vprogs-review checkout --detach f9b84a863a7c7c20586a9cf947550475e894f72e
git -C vprogs-review rev-parse HEAD
cd vprogs-review
cargo +1.94.0 metadata --offline --locked --no-deps --format-version 1
```

metadataの生JSONにはローカルパスが含まれるため、本書の公開物へ追加しない。公式テストの場所はcore/atomics/src/atomic_ring.rs、scheduling/scheduler/tests/e2e.rs、node/framework/tests/e2e.rs等を確認した。ファイルやテスト名の存在と、実行成功は別である。全体のcargo testを無条件に動かす前に、対象crate・必要なRocksDB/RISC Zero等のビルド環境・ネットワークやノードの使用を確認する。今回、公式テスト・実ノード・証明生成は実行していない。

## 34.18 テスト戦略

### 単体テスト

- 状態遷移の決定性。
- 資産総量保存（発行・焼却のないモデル）。
- 失敗時の状態不変。
- 所有していない状態への書き込み拒否。

### 性質ベーステスト

- 任意の送金列でも残高が負にならない。
- 任意の入力順序で、**同じ確定順序**を与えた場合の結果は一致する。
- 証明対象の公開入力に結び付けた値が変われば、元の証明を受理しない（方式が許す同値表現や別の有効証明と区別する）。

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
34.13で再利用する第33章の `apply_transfer` を、複数操作のバッチへ拡張する。候補状態に順に適用し、途中で1件失敗した場合はバッチ全体を拒否する。先行操作が成功した後に失敗する例も用意し、公開する残高集合が全体として維持されることを確認する。これは単一プロセスのメモリ内モデルであり、複数アプリ・永続DB・外部作用の原子性を実装したものではない。

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
- [kaspanet/vprogs：固定Rustプロトタイプ](https://github.com/kaspanet/vprogs/tree/f9b84a863a7c7c20586a9cf947550475e894f72e)
- [Kaspa Research：vProg Computation DAGの形式モデル](https://research.kas.pa/t/zoom-in-a-formal-backbone-model-for-the-vprog-computation-dag/407)
- [Kaspa Research：同期的合成可能なvProgsアーキテクチャ提案](https://research.kas.pa/t/concrete-proposal-for-a-synchronously-composable-verifiable-programs-architecture/387)
- [Kaspa Research：Proof Stitchingの基本フレームワーク](https://research.kas.pa/t/a-basic-framework-for-proofs-stitching/323)
- [kaspanet/research：vProgs Yellow Paper（草案）](https://github.com/kaspanet/research/blob/main/vProgs/vProgs_yellow_paper.pdf)（追加調査先。今回PDF本文は未照合）
- [KIP-16：New Transaction Opcodes for Verifiable Computation](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0016.md)
- [Kaspa KIPs一覧](https://github.com/kaspanet/kips)

---

次は[第35章 — Kaspaアプリケーションの設計と部分実装](../part-06-contributing/chapter-35.md)。

**確認範囲：** 公開案内/研究投稿・固定プロトタイプの主要箇所・独自状態遷移教材。全論文/証明、公式テスト全体、実アプリ間原子性・DA/退出・現行Mainnet動作は未検証。記録は `experiments/chapter-34-review-2026-11-01.md`。
