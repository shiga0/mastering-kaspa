# Chapter 26 — Consensus Implementation in Rust（Rustによるコンセンサス実装）

<p class="chapter-part">Part IV — Kaspa Development &amp; Implementation（Kaspaの開発と実装）</p>

> **前提知識：** 第7章（BlockDAG）、第12章（GHOSTDAG）、第14章（Virtual State）、第25章（コードベース調査）

<a id="261-この章の目的"></a>

## 26.1 この章で学ぶこと

KaspaのコンセンサスをRustで理解するには、アルゴリズムだけでなく、**データの表現・検証順序・永続化・状態更新・並行処理**を一体として捉える必要がある。本章では、実装を読むための責務分解を示し、固定した公式実装で親選択からメタデータ算出までを追う。自作モデルの実装は第27章で扱う。

本章を終えたら、次の問いに答えられることを目指す。

- BlockDAGを保持する処理とGHOSTDAGの順序付けは何が違うか。
- ブロックの構造的な妥当性とトランザクションの受理をなぜ分離するのか。
- Virtual Stateの再計算とストレージ更新にどんな不変条件が必要か。
- `rusty-kaspa`のどこから読み始め、どのように実装の根拠を残すか。

> **対象範囲：** 26.7と26.10の固定ソースへの参照は主要経路の読み取り記録である。その他の型・図は責務を説明する概念モデルであり、公式実装との一対一の対応やノード全体の検証を意味しない。

## 26.2 コンセンサスの責務を分ける

「コンセンサス」という語は広い。ノード内の実装では、少なくとも次の関心事を区別する。

| 関心事 | 入力 | 主な責務 | 出力 |
|---|---|---|---|
| Block ingestion | ネットワークからのブロック | 重複判定、依存する親の確認、受信管理 | 処理対象の候補 |
| Block validation | ブロックとコンセンサス文脈 | ヘッダー、PoW、トランザクションなどの規則検証 | 検証結果 |
| DAG storage | ブロックと親子関係 | 祖先関係・インデックス・永続化 | DAGの参照可能な状態 |
| GHOSTDAG | DAGとパラメータ | Selected parent、blue/redなどの計算 | コンセンサス用メタデータ |
| Ordering | GHOSTDAGの結果 | 処理に用いる順序の決定 | ブロックの順序 |
| UTXO state | 順序付けられたトランザクション | 支出可能性・競合・状態遷移の評価 | UTXO集合 |
| Virtual state | DAGの現在のtipsなど | 仮想的な視点での状態の評価 | ノードが参照する現在状態 |
| Persistence | 状態変更 | 一貫性を保った記録・復旧 | 永続データ |

これらは**概念的な責務**であり、実際の`rusty-kaspa`に同名のモジュールが一対一で存在することを意味しない。

```text
P2P / RPC
   │
   ▼
Block processing ──────► Validation
   │                         │
   └──────────┬──────────────┘
              ▼
          BlockDAG
              │
              ▼
           GHOSTDAG
              │
              ▼
       Consensus ordering
              │
              ▼
        UTXO evaluation
              │
              ▼
         Virtual state
              │
              ▼
         Storage / API
```

基準版ではヘッダー処理中にGHOSTDAGを計算し、ヘッダー・本体・Virtualの各段階へ進む。GHOSTDAGが全検証・DAG保存の後に一度だけ走るという意味ではない。実際には検証と状態更新の順番、キャッシュ、並列処理などが入り組む。上図を**ソースコードの呼び出し順序そのもの**として読んではいけない。

## 26.3 Rustでのドメインモデル

設計の出発点は、型によって混同を防ぐことだ。

- `BlockHash`：ブロックの識別子。単なる文字列や整数と区別する。
- `BlockHeader`：親参照、タイムスタンプ、PoW関連フィールドなど。
- `Block`：ヘッダーとトランザクション群。
- `Outpoint`：トランザクションIDと出力インデックスの組。
- `UtxoEntry`：金額、支出条件など。
- `ValidationError`：無効な入力と一時的な依存不足を区別する。

たとえば「親がまだ届いていない」は、必ずしも「ブロックが無効」という意味ではない。**再試行可能な不足状態**と**確定的な検証失敗**を同じエラーにしない。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InsertError {
    Duplicate,
    MissingParent(BlockId),
    SelfReference,
}
```

この型は説明用であり、実際のKaspaのブロックハッシュは`u64`ではない。Rust/Cargo 1.90.0、edition 2021の一時libでコンパイル・Clippyを確認した。挿入処理や検証アルゴリズムを実装したコードではない。

## 26.4 DAGの基本不変条件

BlockDAGを保持する部分には、最低限次のような不変条件が必要になる。

1. 同一識別子のブロックを重複登録しない。
2. 親参照は自己参照しない。
3. 親を参照できるか、依存不足として保留する。
4. DAGに循環を作らない。
5. どの単位の更新をまとめ、失敗時に何を保存・維持するかを明示する。

実際のコンセンサスでは、これに加えてPoW、親集合の制約、ブロックのコンテキスト依存規則などが必要となる。

### 26.4.1 Past・Future・Anticone

ここではPast/Futureを自分自身を除くstrictな集合として使う。ブロック`A`がブロック`B`の祖先であれば、`A`は`B`のPastに属する。`B`は`A`のFutureに属する。異なる既知の2ブロックで、どちらも他方の祖先ではない場合、互いはAnticoneの関係にある。公式Reachability APIには自分自身もancestorとする問い合わせがあるため、教材のstrictな定義と呼び出し条件を合わせる。あるブロックのAnticone集合の要素同士まで、互いに無関係であるとは限らない。

```text
       G
      / \
     A   B
      \ /
       C
```

矢印を省略した図で、上が親、下が子である。子から親への参照方向は下から上になる。`A`と`B`は並行して生成された例であり、`C`は両方を親として参照する。この図は概念図であり、表示方向に普遍的な規約はない。

## 26.5 GHOSTDAGは単純なトポロジカルソートではない

DAGは一般に複数のトポロジカル順序を持つ。どれか一つを選ぶだけでは、敵対的なネットワーク条件下で全ノードが同じコンセンサス結果に至るための規則として不十分だ。

KaspaではGHOSTDAGにより、Selected parentやblue/redの分類、blue score/blue workなどの情報を計算し、コンセンサス順序の基礎とする。

**注意すべき違い:**

| 用語 | 意味 | 誤解しやすい点 |
|---|---|---|
| DAGの親子関係 | ブロックが参照する先 | それだけでトランザクション順序は確定しない |
| Selected parent | GHOSTDAGの規則に基づく選択 | 「最初に受信した親」ではない |
| blue / red | GHOSTDAGの分類 | red = 無効なブロック、ではない |
| Consensus ordering | 規則に従って決まる順序 | 全トランザクションが必ず受理されるわけではない |
| Virtual state | DAGの現在の視点から評価する状態 | 永久不変な最終状態とは限らない |

GHOSTDAGの厳密な実装を作るには、論文の定義、ネットワークパラメータ、参照実装を突き合わせる必要がある。ここでは第12章で確認した規則を、公式実装の関数・データ構造に対応づける。

## 26.6 Virtual Stateと状態遷移

コンセンサス上のブロック順序と、UTXOの受理可否は別の段階である。二つのトランザクションが同じUTXOを消費しようとする場合、順序付けの結果に従って一方が適用され、他方は競合として適用されない場合がある。

```text
BlockDAG
   ↓
Consensus ordering
   ↓
Transaction evaluation
   ├─ Input exists?
   ├─ Already spent?
   ├─ Authorization valid?
   └─ Amount / other rules valid?
   ↓
UTXO state transition
```

上記は**概念的なチェック項目**。実際のKaspaにはさらにコンセンサス固有の規則がある。

### 26.6.1 状態遷移の原子性

複数入力を持つトランザクションで、最初の入力を消費した後に別の入力で検証が失敗したとする。このとき、最初の入力だけが消費された状態を残してはいけない。

- **検証段階:** すべての入力と出力を確認する。
- **コミット段階:** 全検証が成功した場合にのみ状態を変更する。
- **障害復旧:** 永続化が失敗しても不整合を残さない。

本番実装ではデータベースのトランザクション、書き込みバッチ、ロールバックなどの設計も重要になる。これは設計時に確認する性質であり、バッチを見つけただけで電源断後の耐久性まで実証したことにはならない。ヘッダー等の既に完了した段階や無効ステータスが、後続エラーで一律に取り消されるとも限らない（[第25章](chapter-25.md)）。

## 26.7 公式実装を追う：親選択からメタデータ保存まで

この章では公式実装の入力・出力・依存先を読む。教育用DAG・UTXOコードを一から組み立てる作業は第27章にまとめる。第12章のDAG解析器で集合の意味を確認してから、以下の経路を追うとよい。

参照コミットは `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。以下の表は固定ソースの読み取りによる確認である。追加の `experiments/ghostdag/` では公式の分類関数を直接呼び、24ケースで親選択・順序付きblue/red・score/workの差分を検査した。固定ソースの取得・ビルド環境・実行コマンドは[比較実験の再実行手順](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/ghostdag/README.md#実行)、条件と結果は[検証記録](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/ghostdag-comparison-2026-10-31.md)を参照する。合成ヘッダーとDFS到達可能性を使う関数単位の試験であり、公式ノード全体の実行結果ではない。

| 読む箇所 | 入力と処理 | 確認する出力 |
|---|---|---|
| [`find_selected_parent`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/ghostdag/protocol.rs) | 親一覧を受け、ストアから各親のblue workを取得 | `(blue_work, hash)` が最大の親 |
| [`ordered_mergeset_without_selected_parent`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/ghostdag/mergeset.rs) | 他の親から過去を探索し、selected parent自身とその過去を除外 | blue work、hashの昇順の候補列 |
| [`GhostdagData::new_with_selected_parent`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/model/stores/ghostdag.rs) | selected parentを最初のblueとして設定 | 分類計算の初期メタデータ |
| [`check_blue_candidate`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/ghostdag/protocol.rs) | 到達可能性と既存blueのanticoneサイズを調べる | 候補のblue/red判定 |
| [`ghostdag`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/ghostdag/protocol.rs) | 判定を反映し、score/workを算出 | `GhostdagData` |

これらの関数には非空の親やストア上の親メタデータ等の前提があり、assert / unwrapも使われる。未検証のネットワーク入力をそのまま渡す汎用バリデーターではない。Genesisは通常のghostdag呼び出しと別に初期化し、ORIGINの子にも特別な分岐がある。

通常経路のblue scoreはselected parentのscore + mergeset_bluesの長さ。blue workはselected parentのworkに、mergeset_blues内の各ブロックのcalc_work(bits)とlevel_workの大きい方を合計する。先頭のselected parentもこの列に含まれ、計算対象ブロック自身のPoWをここで加える式ではない。workはブロック数だけでは決まらない。

基準版の通常 `GhostdagManager` は `ForkedParam<KType>` を持ち、`ghostdag` 内でselected parentのDAA scoreからkを取得する。旧照合版の固定kとは異なる。上位レベル用 `with_level` はkを定数化する。選択・候補順・blue判定・score/workの主要規則は再照合したが、任意のk減少まで扱えるという意味ではない。パラメータと適用範囲は[第12章](../part-02-consensus/chapter-12.md)、比較の記録は `experiments/ghostdag-baseline-review-2026-10-31.md` を参照する。

### 26.7.1 呼び出し元と保存の境界

[HeaderProcessor](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/header_processor/processor.rs)の通常validate_headerは、[ヘッダー単独検査](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/header_processor/pre_ghostdag_validation.rs)（PoW・block levelの算出を含む）、親関係、文脈作成、GHOSTDAGを経てpre_pow_validation / post_pow_validationへ進む。関数名のpre_powだけから「ここまではPoWをまだ調べていない」と判断しない。trusted headerの経路も通常経路と検査範囲が異なる。

processor側のghostdagは既存メタデータがあれば再利用し、なければGhostdagManagerを呼んで文脈へ保持する。分類関数自体がDBへ保存するわけではない。

commit_headerはGHOSTDAGデータ、ヘッダー、DAA除外・depth、親関係、Reachability、StatusHeaderOnly等をWriteBatchへ追加し、関係するwrite guardを保持したままdb.write(batch)を呼ぶ。difficulty/median-time window cacheへの更新もある。StatusHeaderOnlyは本体・UTXOの処理完了を意味しない。

ここで読んだバッチとロックの範囲はヘッダーの保存境界であり、全キャッシュの巻き戻し、後続の本体/Virtual更新、I/O失敗・電源断後の耐久性を検証した結果ではない。

### 26.7.2 読み取りの課題

1. 親一覧の順を変えても同じ親が選ばれる理由を、比較処理から説明する。
2. `mergeset_blues` の先頭要素がscore/workの式にどう影響するか確認する。
3. 分類関数のどこがストアの読み取りを行い、何を戻り値として返すか記録する。
4. 呼び出し元を検索し、戻り値を保存する経路を追う。**分類関数を読んだだけで永続化の原子性まで確認したと扱わない。**

第12章は規則と集合の意味、本章は関数と依存先、第27章は自作モデルの接続を担当する。公式実装に対応しない教材の型やアルゴリズムを、実装の説明へ混ぜない。

## 26.8 何がまだ足りないのか

教育用モデルからKaspa互換のコンセンサスに進むには、大きな隔たりがある。

| 未実装の領域 | なぜ必要か |
|---|---|
| 実際のブロックハッシュとシリアライズ | ノード間で同じ識別子を計算するため |
| PoW・Difficulty | ブロックが必要な作業量を満たすか判断するため |
| GHOSTDAGの厳密な計算 | DAGの選択・分類・順序付けを合意するため |
| コンセンサスのトランザクション規則 | 支出権限や金額・その他の制約を検証するため |
| Virtual State・Reorganization | 新しいDAGの情報に応じて現在状態を更新するため |
| Pruning・同期証明 | 全履歴なしで検証と同期を成立させるため |
| 永続化と障害復旧 | 再起動後も状態の整合性を維持するため |
| 並行処理とキャッシュ | 高頻度のブロック処理を安全に行うため |

ここで重要なのは、**「動くDAG」と「合意可能なBlockDAGコンセンサス」は同義ではない**ことだ。

## 26.9 並行処理とRustの所有権

Kaspaノードでは、P2P受信、ブロック検証、同期、RPCなどが同時進行する。Rustの所有権・借用・`Send`/`Sync`はデータ競合を減らす助けになるが、**論理的な競合やコンセンサスの誤りを自動で防ぐわけではない**。

例として、二つのタスクが同じUTXOを使うトランザクションを同時に検証すると、どちらも「未使用」と読める可能性がある。したがって、検証とコミットを整合的に扱う設計が必要になる。

検討事項：

- どのコンポーネントが状態変更の所有者か。
- 読み取り専用スナップショットをどう扱うか。
- 書き込みの順序をどう制御するか。
- キャッシュと永続データの不整合をどう検出するか。
- タスクのキャンセルやノード停止時に中途半端な更新を残さないか。

### 26.9.1 性能と正しさの優先順位

まず**単一スレッドで決定的に正しいモデル**を作り、テストを充実させてから、並列化・キャッシュ・非同期化を検討する。コンセンサスの実装では、速く間違うことより、遅くても正しいことの方が重要だ。

## 26.10 Reorganizationをどう考えるか

新しいブロックが届くと、Virtual Stateの評価結果が変わることがある。そのため、現在の状態を一度計算したら永久に固定できるとは限らない。

状態管理には、概念的に次の選択肢がある。

1. **再計算:** 既知の基準状態から順序に従って再評価する。
2. **差分適用:** 適用・取り消しの差分を保持する。
3. **スナップショット:** ある時点の状態を保存し、以降の変更を再生する。

基準版の[commit_virtual_state](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/virtual_processor/processor.rs)は、蓄積したUTXO差分・新Virtual state・selected chainの変更を1つのWriteBatchへ積み、関連write guardを保持したままdb.writeを行う。これは26.7のヘッダー保存とは別の呼び出し・バッチである。呼び出し前の差分計算、通知や索引、Pruningを含む全操作が1つのトランザクションだとは扱わない。

### 26.10.1 Reorganizationの不変条件

- 親、基準UTXO、適用順、ネットワーク規則・activation条件等を固定した同じ評価から、同じ結果を得る。
- 取り消されたトランザクションの出力を誤って残さない。
- 同じUTXOを二度消費しない。
- 状態更新の途中でRPCに矛盾した結果を公開しない。
- 再起動後の状態が、正しく再生した状態と一致する。

これらは確認すべき性質であり、全経路で検証済みという列挙ではない。基準版のpick_virtual_parentsには候補をランダムに選ぶ処理があり、ノードの観測時点や選ばれた親を固定せずに「同じDAGだから現在のVirtualも完全一致する」とは言えない。更新途中の公開境界はAPI・索引ごとに調べ、複数RPCを続けて呼ぶだけで同じスナップショットを得たとも判断しない。

## 26.11 エラー設計と失敗の分類

すべての失敗を「invalid block」にまとめると、運用も実装も難しくなる。

| 分類 | 例 | 基本的な対応 |
|---|---|---|
| 入力が無効 | フォーマットやコンセンサス規則に違反 | 拒否・必要に応じてピア評価 |
| 依存不足 | 親ブロックが未到着 | 保留・依存を取得 |
| 一時的な資源不足 | キュー上限、タイムアウト | バックプレッシャー・再試行 |
| 永続化失敗 | ディスクI/O障害 | 安全に停止・復旧 |
| 内部不変条件違反 | 想定外の状態不整合 | エラー報告・原因調査 |

ログにはブロック識別子、処理段階、依存先、エラー分類、実行時間などを残すと追跡しやすい。ただし、秘密情報や過剰なユーザーデータを記録しない。

## 26.12 C4とDDDで実装を読む

C4モデルでは、まずノードを一つのシステムとして捉え、次に内部の主要コンポーネントを分ける。DDDでは「何を正しいと判断する責務がどこにあるか」を確認する。

```text
System: Kaspa node

Components (conceptual):
  Network ingress
       │
       ▼
  Block processor ──────► Consensus rules
       │                        │
       ▼                        ▼
  DAG repository ◄──────── State manager
       │                        │
       └────────► Storage ◄─────┘
```

**調査の問い:**

- ブロックを「受信した」と「受理した」の境界はどこか。
- どの型が未検証データを表すか。
- どこでコンセンサス規則が適用されるか。
- 状態変更をコミットする境界はどこか。
- 外部RPCは検証済みデータだけを返すか。

## 26.13 `rusty-kaspa`を実際に読む手順

### 26.13.1 リビジョンを固定する

```bash
# 第1章の固定版checkoutの親ディレクトリから実行
cd rusty-kaspa-v1.0.1
git rev-parse HEAD
git status --short
KASPA_CONSENSUS_REVIEW_DIR="$(mktemp -d)"
cargo metadata --locked --offline --no-deps --format-version 1 > "$KASPA_CONSENSUS_REVIEW_DIR/metadata.json"
```

未取得なら[第1章](../part-01-foundations/chapter-01.md)の手順を使い、HEADを26.7の完全SHAと照合する。metadataには絶対パスが含まれるため生JSONを公開しない。--no-depsの情報は外部依存の完全な解決グラフではない（[第25章](chapter-25.md)）。

本の執筆では、調査時の**コミットハッシュ、実行したコマンド、観察した結果**を必ず残す。`master`ブランチの内容は更新されるため、行番号だけに依存した説明は古くなりやすい。

### 26.13.2 エントリーポイントを探す

```bash
grep -n -E 'find_selected_parent|check_blue_candidate|pub fn ghostdag' consensus/src/processes/ghostdag/protocol.rs
grep -n -E 'validate_header|fn ghostdag|commit_header|insert_batch' consensus/src/pipeline/header_processor/processor.rs
grep -n -E 'commit_virtual_state|pick_virtual_parents' consensus/src/pipeline/virtual_processor/processor.rs
```

これらは基準版で確認した入口である。検索で見つけただけでは、前提条件・エラー・公開境界まで検証したことにはならない。検索結果が空なら、`cargo metadata`で実際のクレート名を確認し、関連する型やテストから辿る。

### 26.13.3 1機能を縦に追う

たとえば「ブロック受理」を調べる場合：

1. P2P側の受信処理を探す。
2. どのAPIでコンセンサスへ渡すか調べる。
3. 返り値・エラー型・非同期境界を確認する。
4. どの検証がどの順序で行われるか確認する。
5. 状態更新と永続化を追う。
6. 同じ経路を検証するテストを探す。

次のような表を残すと、調査が再現可能になる。

| 観点 | 記録する内容 |
|---|---|
| Source revision | Git commit hash |
| Entry point | ファイル、関数名、呼び出し元 |
| Input | 型、検証前提 |
| State mutation | 変更されるデータ、トランザクション境界 |
| Errors | 分類と呼び出し元の対応 |
| Tests | テスト名、再現条件 |
| Unverified | 未確認の仮説 |

## 26.14 テスト戦略

コンセンサス実装では、正常系よりも**境界条件と不変条件のテスト**が重要になる。

| レベル | テスト内容 |
|---|---|
| Unit | DAGの祖先関係、重複検出、UTXO更新 |
| Property | 入力順序を変えても維持されるべき性質 |
| Golden vector | 既知の入力に対する期待結果との一致 |
| Differential | 参照実装との結果比較 |
| Integration | ブロック受信から状態更新まで |
| Fault injection | ディスク障害、通信遅延、再起動 |
| Multi-node | 複数ノードが同じ結果に収束するか |

### 26.14.1 具体的な不変条件

- **Determinism:** 同じ有効な入力と規則から同じ結果を得る。
- **No double spend:** 一つのUTXOを同じ状態で二重に消費しない。
- **Atomicity:** 失敗した状態遷移が一部だけ反映されない。
- **DAG integrity:** 親子関係の参照が破損しない。
- **Crash recovery:** 再起動後に一貫した状態へ戻れる。

> 性質ベーステストや差分テストは強力だが、**テストが通ることはコンセンサス互換性の証明ではない**。特に敵対的なネットワーク条件では、仕様・論文・実装の三者を照合する必要がある。

## 26.15 実践課題

<details class="exercise-details">
<summary>実践課題を見る</summary>

### 課題A：DAGの問い合わせを実装する

第12章の `examples/dag-relations/` にある `Dag` を題材に、次のメソッドを確認・追加する。

- `past(id)`：既存メソッドで、指定ブロックの祖先集合を返すことを確認する。
- `is_ancestor(a, b)`：異なる既知の2ブロックについて、`a`が`b`のstrictな祖先か判定する。同一ブロックはfalse、不明IDはエラーとする。
- `is_in_anticone(a, b)`：異なる既知の2ブロックが互いに祖先・子孫でないか判定する。既存の `anticone(id)` は集合を返すため、その名前と区別する。

**確認:** 2本の並行枝を作り、互いに祖先でないブロックがAnticoneに入ることをテストする。同じ親を共有するだけでは不十分で、片方がもう片方の子・子孫になっている例も区別する。不明IDや同一IDをAnticoneに含めない。既存メソッドはsrc/lib.rsにあり、8テストを今回一時コピーで再実行した。追加メソッドの実装はこの課題で行う。

### 課題B：状態差分を実装する

第27章の `examples/minimal-node/src/main.rs` にある `UtxoSet::apply` を対象にする。トランザクションの適用結果として `UtxoDiff { removed, added }` を返すよう変更する。差分には削除したOutputの内容も保持し、逆適用で元の状態へ戻せることを検証する。複数取引なら依存関係に従い逆順へ戻す。後続更新が入った状態へ古い差分を無条件に適用せず、状態の識別と対象出力の一致を検査する。この逆適用は今回実装・検証した機能ではない。

### 課題C：参照実装の経路を調査する

`rusty-kaspa`の一つのブロック受理経路について、次をMarkdownに記録する。

1. エントリーポイント
2. 検証と状態更新の境界
3. 関係するストレージ
4. 代表的なエラー
5. 対応するテスト

**完了条件：** 読者が同じコミットをcheckoutし、同じ場所を確認できること。

</details>

## 26.16 よくある誤解

<details class="misconception-details">
<summary>よくある誤解を見る</summary>

**1. Rustで書けばコンセンサスのバグはなくなる？**

ならない。Rustはメモリ安全性やデータ競合の防止に役立つが、合意規則の論理的な誤りは別問題だ。

**2. トポロジカルソートができればGHOSTDAGを実装したことになる？**

ならない。GHOSTDAGにはSelected parent、blue/red分類、blue workなどの規則がある。

**3. ブロックが有効なら中の全トランザクションも受理される？**

必ずしもそうではない。ブロックの妥当性と、コンセンサス順序に基づくトランザクションの受理は分けて考える必要がある。

**4. テストが全部通ればメインネット互換？**

ならない。網羅性、仕様との一致、敵対的条件、参照実装との比較などが必要だ。

</details>

## 26.17 理解度チェック

<details class="quiz-answers">
<summary>理解度チェックを見る</summary>

1. DAGストレージとGHOSTDAG計算の責務をそれぞれ説明せよ。
2. 「親ブロック未到着」と「無効なPoW」を同じエラーにすべきでない理由は何か。
3. blue/red分類とトランザクションの受理可否を混同すると何が起こるか。
4. UTXO状態遷移を原子的に行う理由を具体例で説明せよ。
5. Reorganizationが発生したときに保持すべき不変条件を三つ挙げよ。
6. Rustの所有権システムが防げる問題と、防げない問題を区別せよ。
7. `rusty-kaspa`の実装調査でコミットハッシュを記録する理由は何か。
8. 第27章の教育用モデルに不足しているKaspa固有の要素を五つ挙げよ。

**解答例**

1. DAGストレージはブロック・親子関係等を保存・提供し、GHOSTDAG計算はその関係と合意規則から分類・スコア等を求める。保存しただけで合意状態への適用が完了するわけではない。
2. 不足親は取得・再投入で解消し得るが、無効PoWは規則違反だから。原因を混ぜると再試行・隔離・ピア評価を誤る。
3. redの取引を一律に捨てたり、blue内の不正・競合取引を無条件に適用したりする誤りにつながる。取引は合意順序とUTXO等の検証で判断する。
4. 一例として入力を消費した後で出力追加に失敗すると資金が消えた状態になるため。検証と変更を分け、対象状態の整合性を保つ単位で更新する。
5. 一例は、競合UTXOを二重消費しないこと、新しい合意文脈に整合するUTXO・メタデータへ戻ること、同じ変更を再適用して二重計上しないことである。
6. 所有権はメモリの寿命や不正な共有・借用等の問題を防ぐ助けになるが、誤った署名規則、金額計算、合意順序、業務ポリシー等の論理誤りまでは証明しない。
7. 規則・型・呼び出し経路とテストが変更され得るため。同じ版で追跡・再現できる根拠を残す。
8. 一例は正式なPoW・ハッシュ、署名とScript検証、GHOSTDAG、Virtual State、難易度調整である。複数親を持つ教材のDAGだけでKaspa互換にはならない。

</details>

## 26.18 まとめ

<details class="summary-details">
<summary>まとめを見る</summary>

Kaspaのコンセンサス実装を理解するには、**BlockDAG・GHOSTDAG・順序付け・UTXO評価・Virtual State・永続化**を分離し、それぞれの入力、出力、不変条件を明確にする必要がある。

Rustはこうした境界を型で表現しやすい一方、正しいコンセンサス規則そのものを保証するわけではない。学習用の小さな実装で基本を理解した後は、参照実装のリビジョンを固定し、関数・テスト・仕様を突き合わせて検証する。

</details>

[次章](chapter-27.md)では、この考え方を踏まえて、**最小Kaspa型ノード**を組み立てる。

## 26.19 参考資料

<details class="reference-details">
<summary>参考資料を見る</summary>

- [Kaspa公式リポジトリ：rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)
- [Kaspa公式ドキュメント](https://docs.kaspa.org/)
- [Kaspa Improvement Proposals (KIPs)](https://github.com/kaspanet/kips)
- [PHANTOM and GHOSTDAG: A Scalable Generalization of Nakamoto Consensus](https://eprint.iacr.org/2018/104.pdf)
- [Rust公式ドキュメント](https://doc.rust-lang.org/book/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)

</details>

<nav class="chapter-next" aria-label="次のページ">

[次章：Chapter 27 — Building a Minimal Kaspa-like Node in Rust（Rustによる最小Kaspa型ノードの構築）](chapter-27.md)

</nav>

<details class="verification-details">
<summary>検証状況を見る</summary>

[対象バージョン](../baseline.md)・[このページのレビュー記録](../review-status.md#review-chapter-26)

> **確認範囲:** 固定版のGHOSTDAG主要経路、ヘッダー保存とVirtual更新、既存DAG教材と型断片を確認した。全規則・全キャッシュ・公式Reachability・永続化の障害注入・原論文の証明を検証した結果ではない。今回の記録は `experiments/chapter-26-review-2026-11-01.md`。


</details>
