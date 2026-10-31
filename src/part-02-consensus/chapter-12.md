# Chapter 12 — GHOSTDAG Internals（GHOSTDAGの内部構造）

> **Part II — Kaspa Consensus & Architecture**  
> 原稿ステータス: v0.2（GHOSTDAGの主要規則を固定実装と照合、章全体の技術レビューは継続中）


> 主要なGHOSTDAG経路を[本書の基準版](../baseline.md)v1.0.1で再照合した。小規模DAGの24ケースで公式実装との実行時差分を比較した。章全体のレビューは未完了。

## 照合した実装

2026年10月31日に `rusty-kaspa` のコミット `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` を参照した。本章の親選択・merge set・blue判定・score/workの式は、下記の固定ソースを読み取って照合した（Source-checked）。さらに `experiments/ghostdag/` で公式の `GhostdagManager::ghostdag` と全blue集合のanticoneを直接数えるモデルを比較し、24ケース・各親配列順で712ブロックの親選択、順序付きblue/red、score/workが一致した。到達可能性は共通のDFSアダプターで提供し、公式Reachability・PoW・ヘッダー検証は対象外。詳細は `experiments/ghostdag-comparison-2026-10-31.md`。メインネットの稼働リリースとの一致や章全体の検証を意味しない。

- [親選択・分類・スコア計算](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/ghostdag/protocol.rs)
- [blue workとハッシュの比較](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/ghostdag/ordering.rs)
- [merge setの列挙](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/ghostdag/mergeset.rs)
- [GHOSTDAGメタデータ](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/model/stores/ghostdag.rs)

## この章で学ぶこと

前章では、受信したブロックが検証され、BlockDAGへ登録されるまでを学んだ。本章では、登録された複数のブロックを**どのように評価し、合意可能な順序へ変換するか**を掘り下げる。中心となるのがGHOSTDAGである。

本章の到達目標は次のとおり。

1. BlockDAGの構造とGHOSTDAGのアルゴリズムを区別する。
2. selected parent、blue/red、blue score、blue work、anticone、merge setを説明できる。
3. パラメータ `k` が表す制約と、その設計上の意味を理解する。
4. 選択された過去と並行ブロックを使って、決定的な順序を構成する考え方を説明できる。
5. GHOSTDAGの結果とUTXOの受理・拒否が異なるレイヤーに属することを理解する。
6. 小さなDAGを使ってPast・Anticone・k-cluster条件と、追加順による結果の不変性をRustで検証できる。

> **対象範囲:** 本章はGHOSTDAGの概念・データ構造・主要な不変条件を説明する。後半のRustコードは**教育用のDAG解析器**であり、GHOSTDAGそのものの実装ではない。照合した規則とソースは冒頭に示し、残る未確認事項は章末にまとめる。

---

## 12.1 なぜGHOSTDAGが必要なのか

Bitcoinのブロックチェーンでは、ほぼ同時に発見されたブロックが競合すると、最終的な主チェーンから外れるブロックが生じる。生成間隔を短くすると、ネットワーク伝播に要する時間が相対的に大きくなり、この競合が起こりやすくなる。

Kaspaは、競合したブロックを単に「一方だけを残す」のでなく、**複数の親を持てるBlockDAG**に保持する。ただし、DAGに記録するだけではトランザクションの競合や順序は解決しない。

```text
Bitcoinの概念図

       B ── C
      /
A ───
      \
       D       ← 分岐したブロックの一部は主チェーンから外れる

BlockDAGの概念図（矢印は子 → 親）

       C ──→ B ──→ A
        \         ↑
         └──→ D ──┘

C は B と D を参照できる（実際の親関係は図の配置に依存しない）。
```

GHOSTDAGは、DAGにあるブロックのうち、ネットワーク遅延による自然な並行生成と整合する集合を評価し、後続の**合意順序**を導くための仕組みである。

### 12.1.1 データ構造と合意規則の分離

| レイヤー | 主な問い | 主な成果物 |
|---|---|---|
| P2P | ブロックは届いたか | 受信データ |
| ブロック検証 | 形式やPoWなどは妥当か | 検証済み候補 |
| BlockDAG | 親子関係は何か | 有向非巡回グラフ |
| GHOSTDAG | どのブロックをどのように評価するか | selected parent、blue/redなどのメタデータ |
| 合意順序 | ブロック・トランザクションをどう並べるか | 決定的な順序 |
| UTXO状態 | どのトランザクションが適用可能か | Virtual UTXO state |

**DAGに存在すること、blueに分類されること、トランザクションが受理されることは同義ではない。**

---

## 12.2 GHOST、PHANTOM、GHOSTDAGの位置づけ

### 12.2.1 GHOST

GHOST（Greedy Heaviest-Observed Sub-Tree）は、ブロックツリーにおけるチェーン選択を、単純な最長チェーンとは異なる形で考える研究である。ブロックツリーの重みを使って枝を評価する。

### 12.2.2 PHANTOM

PHANTOMはBlockDAGに対して、並行性が一定の制約内に収まるブロック集合（`k`-cluster）を考える。理想的な集合を求める問題は一般に計算量の課題がある。

### 12.2.3 GHOSTDAG

GHOSTDAGはPHANTOMの考え方に基づく**貪欲（greedy）な近似手法**として設計された。各ブロックに対してselected parentを選び、その履歴を踏まえながらblue/red分類を構築する。

| 概念 | 主に扱う対象 | 注目点 |
|---|---|---|
| Nakamoto consensus | ブロックチェーン | 累積PoWに基づくチェーン選択 |
| GHOST | ブロックツリー | サブツリーの重み |
| PHANTOM | BlockDAG | `k`-clusterという集合の性質 |
| GHOSTDAG | BlockDAG | 効率的な貪欲分類・順序構築 |

> **注意:** 名前が似ていても、GHOSTとGHOSTDAGを同じアルゴリズムとして扱わない。DAGKnightも別の研究・設計として第29章で扱う。

---

## 12.3 DAGの関係を正確に定義する

以下では、矢印を **子ブロック → 親ブロック** とする。

```text
         E
        / \
       C   D
        \ /
         B
         |
         A
```

この例で、`E` は `C` と `D` を親に持ち、`C` と `D` はともに `B` を参照する。

### 12.3.1 Parent / Child

- **Parent（親）:** ブロックヘッダーで直接参照する先。
- **Child（子）:** そのブロックを親として参照する後続ブロック。

### 12.3.2 Past / Future

- `Past(B)`: `B` から親方向へたどって到達できる祖先の集合。
- `Future(B)`: `B` を祖先として参照する子孫の集合。

ここでは `B` 自身は両集合に含めない。

### 12.3.3 Anticone

`Anticone(B)` は、`B` の祖先でも子孫でもなく、`B` 自身でもないブロックの集合である。

```text
     C      D
      \    /
        B
        |
        A
```

`C` と `D` は互いに祖先・子孫の関係がないため、互いのanticoneに属する。ただし、どのDAG部分集合を対象にするかによって、anticoneの計算対象は変わる。

### 12.3.4 Merge set

あるブロックが複数の親を参照するとき、selected parentの過去にまだ含まれていない、他の親から新たに取り込まれる過去のブロックが存在する。これらを扱う概念が**merge set**である。

本章では `Past` が自分自身を含まないため、通常のブロック `B` とselected parent `P` について、selected parentを除いた候補集合を次のように表す。

```text
M = Past(B) \ (Past(P) ∪ {P})
```

候補には他の親自身も含まれる。確認した実装は `ordered_mergeset_without_selected_parent` でこの候補を列挙する。一方、保存する `mergeset_blues` は先頭に `P` を含む。「merge set」がselected parentを含むかは、関数・データ構造ごとに区別する。根拠は[集合の列挙](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/ghostdag/mergeset.rs)と[初期化処理](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/model/stores/ghostdag.rs)を参照する。

---

## 12.4 Selected Parentとは何か

DAG上のブロックは複数の親を持てる。しかし、GHOSTDAGはそれらの中から**selected parent**を一つ選び、再帰的な評価の基準にする。

```text
             E
            / \
           C   D
            \ /
             B
             |
             A

E の親は C と D。
GHOSTDAGは規則に従い、そのうち一方を selected parent にする。
```

確認した実装の `find_selected_parent` は、親の `(blue_work, hash)` を比較して最大の親を選ぶ。まずblue workが大きい親を優先し、同じなら `Hash` 型の比較で大きい親を選ぶ。blue scoreや到着順を親選択の比較キーには使わない。ハッシュの大小は実装の型の比較であり、表示文字列の比較へ置き換えない。[親選択](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/ghostdag/protocol.rs)と[比較規則](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/ghostdag/ordering.rs)を参照する。

**selected parentは唯一の有効な親という意味ではない。** selected parentでない親もDAGの一部であり、merge-set処理の対象になる。

### 12.4.1 Selected-parent chain

各ブロックからselected parentをたどると一本の系列を得る。この系列を**selected-parent chain**と呼ぶ。

これは「BlockDAGを普通のブロックチェーンに戻した」という意味ではない。系列以外のブロックも評価対象となり、合意順序に組み込まれうる。

---

## 12.5 blueとredをどう理解するか

GHOSTDAGは、あるブロックを評価する文脈に対してblue/redの分類を構築する。分類を文脈なしの永久的なブロック属性として扱わない。

- **blue:** 選択した集合における並行性の制約と整合するものとして採用されるブロック。
- **red:** その制約のもとでblue集合に含まれなかったブロック。

重要なのは、**redは「PoWが無効」「不正ブロック」「破棄されたブロック」を意味しない**こと。redでも構造上有効なブロックはあり、合意順序の扱いとトランザクションの受理判定は別問題である。

### 12.5.1 なぜ並行性を制限するのか

伝播遅延があるネットワークでは、正直なマイナーが同時期に異なるブロックを見て採掘することがある。一方、攻撃者が意図的に大量の並行ブロックを作る可能性も考えなければならない。

GHOSTDAGは、並行ブロックを無制限に同等視するのでなく、パラメータ `k` を用いて一定の構造的制約を課す。

---

## 12.6 `k`-clusterとパラメータ`k`

PHANTOM/GHOSTDAGを理解するうえで、`k` は重要なパラメータである。

直感的には、選ばれた集合の各ブロックについて、**その集合内で互いに順序関係を持たないブロック数（anticoneサイズ）を制限する**。

集合 `S` が `k`-clusterである条件は、概念的に次のように書ける。

```text
任意の b ∈ S について

|Anticone(b) ∩ S| ≤ k
```

ここで `Anticone(b)` は評価対象のDAGに対して定義される。

基準版v1.0.1の通常 `GhostdagManager` は、`k` を `ForkedParam<KType>` として保持する。分類に使う値は **selected parentのDAA score** で選び、activation前後で切り替わる。新ブロック自身のDAA scoreやblue scoreで選ぶのではない。[基準版のDevnet設定](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/params.rs)はCrescendo無効で、既定値はpriorの `k = 18`。値の定義は[constants.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/constants.rs)を参照する。別ネットワークや現在のMainnetの設定へ一般化しない。

上位レベル用の `with_level` はkを定数にする別の構築経路である。既存照合版との主な差はこのフォーク対応であり、固定kだけの説明をすべての版へ当てはめない。照合記録は `experiments/ghostdag-baseline-review-2026-10-31.md`。

### 12.6.1 直感的な例

- `k = 0`: 集合内に互いに並行なブロックを含められない。
- `k = 1`: 各ブロックは、集合内で最大1個の並行ブロックを許容する。
- `k = 2`: 各ブロックは、集合内で最大2個の並行ブロックを許容する。

これは**説明用の小規模な例**であり、Kaspaメインネットの実パラメータを表していない。

### 12.6.2 `k`を大きくすれば常によいのか

そうではない。ネットワーク遅延・ブロック生成率・攻撃モデル・セキュリティ解析と結びつくため、`k`を単なるパフォーマンス設定として変更してはならない。

| 観点 | `k`に関連する論点 |
|---|---|
| 正直な並行ブロック | どの程度の並行性を許容するか |
| 攻撃耐性 | 敵対的なブロック生成をどう評価するか |
| 生成レート | ネットワーク伝播との関係 |
| 計算資源 | 集合評価や関連メタデータの処理 |

> `k`、DAA score、blue score、blue workは異なる概念。名前が似ていても混同しない。

---

## 12.7 GHOSTDAGの計算を概念的に追う

実際のアルゴリズムは詳細な整合性判定を含む。ここでは考え方だけを段階に分ける。

```text
新しいブロック B
       │
       ▼
B の親候補を取得
       │
       ▼
GHOSTDAGの規則で selected parent を決定
       │
       ▼
selected parent のメタデータを基礎にする
       │
       ▼
merge set にあるブロックを決定的な順序で評価
       │
       ▼
blue/red分類を更新
       │
       ▼
blue score / blue work などのメタデータを導出
```

候補の評価順は `(blue_work, hash)` の昇順であり、親選択の最大値取得とは向きが異なる。候補をblueに加えるには、候補自身のblue anticoneサイズだけでなく、候補と並行な既存blueブロックのanticoneサイズも `k` 以下に保つ必要がある。候補だけを検査する実装では不十分である。[候補のソート](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/ghostdag/mergeset.rs)、[比較](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/ghostdag/ordering.rs)、[blue判定](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/ghostdag/protocol.rs)を参照する。

### 12.7.1 小さなDAGを手で追う

Genesis `G` の子として `A`、`B`、`C` が並行に生成され、`X` が3つを親に持つとする。これは教材用の `k = 1` の例であり、メインネットの設定ではない。各ブロックの仕事量を同じ `w`、ハッシュの比較順を `A < B < C` と仮定する。

```text
         G
       / | \
      A  B  C
       \ | /
         X
```

| 手順 | 判断 | 結果 |
|---|---|---|
| 1 | A・B・CはいずれもGだけを親に持つ | 各blue scoreは1、blue workはw |
| 2 | Xの親のblue workが同じなのでハッシュで比較 | selected parentはC |
| 3 | C以外のmerge set候補を昇順で並べる | A、Bの順に評価 |
| 4 | Aと並行な既存blueはCだけ | A自身とCのanticoneサイズが1となり、Aをblueに追加 |
| 5 | Bと並行な既存blueはCとA | 候補のサイズが2になりkを超えるため、Bはred |
| 6 | Xのmergeset_bluesはCとA | blue score(X) = 1 + 2 = 3、blue work(X) = w + 2w = 3w |

`B` のred判定はXを評価する文脈での分類である。Bを無効ブロックと判断したり、その中のトランザクションの受理可否をこの表だけで決めたりはしない。

この表は上記の固定ソースの比較・分類・加算規則を手で適用したもの。公式実装を実行して得たテストベクトルではなく、公式実装との実行時の差分照合は未実施である。

### 12.7.2 なぜ順序が決定的でなければならないのか

同じDAGを見たノードが、受信順やハッシュマップの走査順に依存して異なる分類を行うと、合意が壊れる。

したがって、同じ有効なDAGと同じプロトコル規則を入力すれば、**ノードの受信順序に依存しない結果**が得られなければならない。

### 12.7.3 擬似コード（概念図）

```text
function conceptual_ghostdag(block):
    parents = get_parents(block)
    selected_parent = select_by_consensus_rules(parents)
    metadata = inherit_relevant_metadata(selected_parent)

    candidates = deterministic_merge_set_order(block, selected_parent)
    for candidate in candidates:
        if satisfies_blue_constraints(candidate, metadata, k):
            metadata.add_blue(candidate)
        else:
            metadata.add_red(candidate)

    return finalize_metadata(metadata)
```

**これは仕様を満たす実装可能な擬似コードではない。** とくに候補集合の定義、blue anticoneサイズの検査、選択基準、重みとタイブレークは省略している。

---

## 12.8 blue scoreとblue work

### 12.8.1 blue score

通常のブロック `B` では、selected parentを `P`、`B` のメタデータに保存する `mergeset_blues` を `MB(B)` とすると、確認した実装は次の式を使う。

```text
blue_score(B) = blue_score(P) + |MB(B)|
```

`MB(B)` は `P` 自身を含み、`B` 自身は含まない。Genesisのblue scoreは0である。単純な「DAG内の全ブロック数」や「トランザクション数」とは異なる。根拠は[計算処理](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/ghostdag/protocol.rs)と[初期化処理](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/model/stores/ghostdag.rs)。

### 12.8.2 blue work

通常のGHOSTDAG計算では次のように、`MB(B)` に含まれるブロックのヘッダー `bits` から計算した仕事量を加算する。

```text
blue_work(B) = blue_work(P) + Σ calc_work(bits(X))
                              X ∈ MB(B)
```

これは実際にマイナーが試行したハッシュ回数の実測値ではない。ブロックごとの難易度が異なれば、ブロック数と仕事量は一致しない。実装は高レベルsub-DAG用に `max(calc_work(bits(X)), level_work)` という下限も備える。通常の `new` は `level_work = 0` とする。Genesis・ORIGINには特別な初期化がある。[計算と初期化](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/ghostdag/protocol.rs)を参照する。

| 指標 | 概念的に表すもの | 注意点 |
|---|---|---|
| Block count | 観測されたブロック数 | 有効性・blue分類を区別しない |
| blue score | blueブロックの累積的な数に関する指標 | 全ブロック数ではない |
| blue work | blueブロックの累積PoWに関する指標 | 単純な個数比較ではない |
| DAA score | 難易度調整等で使われる別の指標 | blue scoreと同一ではない |

---

## 12.9 GHOSTDAGと合意順序の関係

BlockDAGは部分順序を持つ。親子関係にあるブロックの前後関係は明確だが、互いに並行なブロック間には自然な先後関係がない。

```text
      C       D
       \     /
          E

C と D は並行している。
E は C と D の両方を参照する。
```

トランザクション処理には、並行ブロックを含めて**全ノードで一致する順序**が必要になる。GHOSTDAG由来のメタデータと順序付け規則は、その決定的な列を構築するために使われる。

### 12.9.1 トポロジカル順序だけでは不十分

トポロジカルソートは親を子より前に置くことができる。しかし、`C` と `D` の順序は一意ではない。

```text
A → B → C → D → E
A → B → D → C → E
```

どちらも親子関係を満たす可能性があるため、コンセンサスには**追加の決定的な選択規則**が必要である。

### 12.9.2 GHOSTDAGとトランザクション順序

ブロック順序が定まると、各ブロック内のトランザクションを含めて状態を評価できる。ただし、二重支払いがある場合、**ブロックが順序に入ることと、そのトランザクションが状態に適用されることは別**である。

---

## 12.10 Virtual Block / Virtual Stateとの接続

Kaspaのノードは、DAGの先端（tips）を踏まえて仮想的な合意状態を評価する。これがVirtual BlockとVirtual Stateの概念につながる。

```text
BlockDAG の tips
       │
       ▼
Virtual Block の構成
       │
       ▼
GHOSTDAGに基づく評価・順序
       │
       ▼
トランザクションの適用可能性を判定
       │
       ▼
Virtual UTXO State
```

これは**概念上の依存関係**であり、実装の厳密な関数呼び出し順ではない。

### 12.10.1 二重支払いの例

```text
UTXO X
  ├─ Tx1: X を Alice → Bob に使用
  └─ Tx2: X を Alice → Carol に使用
```

Tx1とTx2が異なる並行ブロックに含まれていても、両方が同時に有効な支払いとして適用されるわけではない。合意順序とUTXO検証の結果、競合する消費の扱いが決まる。

---

## 12.11 Reorganizationと安定性

既存の通常ブロックについて、その親集合と親のメタデータが同じなら、算出済みのselected parentが新しい子孫の到着だけで変わるわけではない。一方、DAGの先端が更新されると、virtual側のselected parentや合意ビューは変わりうる。したがって、現在の先端付近の合意順序を、不可逆な確定履歴と同一視してはいけない。

ここでは次の区別が重要である。

- **受信済み:** ピアからデータを受け取った。
- **検証済み:** ブロックが該当する検証を通過した。
- **DAG登録済み:** DAGの一部として管理されている。
- **順序付け済み:** 現時点の合意ビューで順序が定まっている。
- **状態適用済み:** 対応するトランザクションがUTXO状態の評価に反映された。
- **十分に安定:** アプリケーションがリスクに応じた確認基準を満たした。

**「何秒で絶対に確定する」という一般化は避ける。** 実際の確定性・確認ポリシーはネットワーク状態、攻撃モデル、利用するアプリケーションのリスク許容度に依存する。

---

## 12.12 安全性・活性・ネットワーク仮定

分散コンセンサスでは二つの性質を区別する。

- **Safety（安全性）:** 正直なノード同士が矛盾した確定履歴を採用しないこと。
- **Liveness（活性）:** ネットワーク条件が許せば処理が進み続けること。

GHOSTDAGの安全性は「DAGを使うから自動的に保証される」ものではない。PoWの攻撃者比率、ネットワーク遅延、ブロック生成率、プロトコルパラメータなどの仮定とともに評価される。

### 12.12.1 攻撃モデルの例

| シナリオ | 起こりうる問題 | 観察すべき点 |
|---|---|---|
| 大きな伝播遅延 | 正直な並行ブロック増加 | anticoneの分布 |
| ネットワーク分断 | 異なるDAGビュー | 再接続後の合意更新 |
| 敵対的なブロック生成 | 並行構造の悪用 | blue/red分類と仕事量 |
| Eclipse攻撃 | 一部ノードの情報隔離 | ピア構成と到達性 |
| 実装の非決定性 | ノード間の結果不一致 | ソート順・タイブレーク |

---

## 12.13 Rust実験：DAGのPastとAnticoneを計算する

ここからは、GHOSTDAGを理解するための**最小のDAG解析器**をRustで作る。GHOSTDAGのblue/red分類を再現するものではない。

### 12.13.1 作成するプロジェクト

```bash
cargo new dag-relations
cd dag-relations
```

依存クレートは標準ライブラリのみ。

### 12.13.2 `src/lib.rs`

<!-- example:dag-relations:start -->

```rust
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default)]
pub struct Dag {
    // child -> parents
    parents: BTreeMap<String, BTreeSet<String>>,
}

impl Dag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, id: &str, parents: &[&str]) -> Result<(), String> {
        if self.parents.contains_key(id) {
            return Err(format!("duplicate block: {id}"));
        }
        if parents.contains(&id) {
            return Err("self-parent is not allowed".into());
        }
        if parents.iter().any(|p| !self.parents.contains_key(*p)) {
            return Err("unknown parent".into());
        }
        self.parents.insert(
            id.to_owned(),
            parents.iter().map(|p| (*p).to_owned()).collect(),
        );
        Ok(())
    }

    pub fn past(&self, id: &str) -> Result<BTreeSet<String>, String> {
        if !self.parents.contains_key(id) {
            return Err(format!("unknown block: {id}"));
        }
        let mut visited = BTreeSet::new();
        let mut stack: Vec<String> = self.parents[id].iter().cloned().collect();
        while let Some(next) = stack.pop() {
            if visited.insert(next.clone()) {
                stack.extend(self.parents[&next].iter().cloned());
            }
        }
        Ok(visited)
    }

    pub fn anticone(&self, id: &str) -> Result<BTreeSet<String>, String> {
        let past = self.past(id)?;
        let mut result = BTreeSet::new();
        for other in self.parents.keys() {
            if other == id || past.contains(other) {
                continue;
            }
            // id が other の祖先なら、other は id の future に属する。
            if self.past(other)?.contains(id) {
                continue;
            }
            result.insert(other.clone());
        }
        Ok(result)
    }

    pub fn is_k_cluster(&self, selected: &BTreeSet<String>, k: usize) -> Result<bool, String> {
        for id in selected {
            let anticone = self.anticone(id)?;
            let within = anticone.intersection(selected).count();
            if within > k {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Dag {
        let mut dag = Dag::new();
        dag.insert("A", &[]).unwrap();
        dag.insert("B", &["A"]).unwrap();
        dag.insert("C", &["B"]).unwrap();
        dag.insert("D", &["B"]).unwrap();
        dag.insert("E", &["C", "D"]).unwrap();
        dag
    }

    #[test]
    fn past_contains_transitive_ancestors() {
        let dag = sample();
        assert_eq!(
            dag.past("E").unwrap(),
            BTreeSet::from(["A", "B", "C", "D"].map(str::to_owned))
        );
    }

    #[test]
    fn parallel_blocks_are_in_each_others_anticone() {
        let dag = sample();
        assert_eq!(dag.anticone("C").unwrap(), BTreeSet::from(["D".to_owned()]));
        assert_eq!(dag.anticone("D").unwrap(), BTreeSet::from(["C".to_owned()]));
    }

    #[test]
    fn k_cluster_constraint() {
        let dag = sample();
        let selected = BTreeSet::from(["C".to_owned(), "D".to_owned()]);
        assert!(!dag.is_k_cluster(&selected, 0).unwrap());
        assert!(dag.is_k_cluster(&selected, 1).unwrap());
    }

    #[test]
    fn rejects_missing_parent() {
        let mut dag = Dag::new();
        assert!(dag.insert("B", &["A"]).is_err());
    }
    #[test]
    fn parallel_insertion_order_preserves_relations() {
        let forward = sample();
        let mut reverse = Dag::new();
        reverse.insert("A", &[]).unwrap();
        reverse.insert("B", &["A"]).unwrap();
        reverse.insert("D", &["B"]).unwrap();
        reverse.insert("C", &["B"]).unwrap();
        reverse.insert("E", &["D", "C"]).unwrap();
        for id in ["A", "B", "C", "D", "E"] {
            assert_eq!(forward.past(id).unwrap(), reverse.past(id).unwrap());
            assert_eq!(forward.anticone(id).unwrap(), reverse.anticone(id).unwrap());
        }
    }
}
```

<!-- example:dag-relations:end -->

実行:

```bash
cargo test
```

**期待される学び:** `C` と `D` は互いにanticoneに含まれ、`E` の過去には両方が入る。`{C, D}` は `k=0` の条件を満たさず、`k=1` では満たす。

> 本サンプルは `examples/dag-relations/` に収録し、単体テストを実行している。条件と結果は `experiments/ghostdag-review-2026-10-31.md` に記録した。`insert`が既知の親しか受け取らないため、追加順に制約があり、実ネットワークの孤児ブロック・Pruning・到達可能性インデックス・並行処理には対応していない。

---

## 12.14 実験を拡張する

### 実験A：到達可能性と非巡回性

任意の異なる `x`, `y` について、`x ∈ Past(y)` なら `y ∉ Past(x)` が成立することをテストする。これはDAGの非巡回性と関係する。

### 実験B：Anticoneの対称性

`x ∈ Anticone(y)` と `y ∈ Anticone(x)` が同値であることを確認する。

### 実験C：受信順と決定性

親子関係を守りつつ、`C` と `D` の追加順を入れ替える。`past`と`anticone`の結果が変わらないことを確認する。

### 実験D：`k`-cluster

`B`を共通祖先に持つ並行ブロックを3つ以上追加し、選択集合に対する `k` の境界を観察する。

### 実験E：計算量

このサンプルは到達可能性を繰り返し探索するため、大きなDAGでは効率が悪い。ブロック数を増やし、計算時間とメモリ使用量を計測する。

---

## 12.15 `rusty-kaspa`で対応箇所を調べる

実装の理解では、**名前の一致ではなく責務の一致**を確認する。

公式リポジトリ:

- [kaspanet/rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)

### 12.15.1 ソースコード調査の入口

```bash
git clone https://github.com/kaspanet/rusty-kaspa.git
cd rusty-kaspa

git rev-parse HEAD
rg -n 'ghostdag|Ghostdag|selected_parent|blue_score|blue_work' .
rg -n 'merge_set|anticone|reachability|ordering' .
```

リポジトリ構造や識別子は変更される可能性がある。以下のような責務を探す。

| 探す責務 | 調査したいこと |
|---|---|
| GHOSTDAGメタデータ | selected parent・blue/red・score/workの保持 |
| GHOSTDAG計算 | 親選択とmerge-set評価 |
| Reachability | 祖先・子孫の効率的な判定 |
| DAG traversal | 依存関係を守る走査 |
| Ordering | 決定的なブロック順序 |
| Virtual state | 合意順序をUTXO状態に反映する境界 |
| Pruning | 古いDAG履歴を削減する際の前提 |

### 12.15.2 コードリーディングの記録テンプレート

```markdown
## 調査対象
- Repository: kaspanet/rusty-kaspa
- Commit: <git rev-parse HEAD>
- Module: <path>
- Function / Type: <name>

## 入力と出力
- Input:
- Output:

## 不変条件
- 

## 依存するコンポーネント
- 

## 例外・失敗時の扱い
- 

## 原論文との対応
- 

## 確認できた事実 / 未確認の推測
- Fact:
- Hypothesis:
```

**実装にない概念を、論文の記述だけから実装済みと断定しない。**

---

## 12.16 実装を追うときの責務境界

本章で固定した実装では、次の境界を追うと計算と状態反映を混同しにくい。

| 対象 | 確認する処理 |
|---|---|
| `protocol.rs` | 親選択、候補のblue/red判定、score/workの算出 |
| `ordering.rs` | blue work、同値時のハッシュによる比較 |
| `mergeset.rs` | selected parentの過去を除いた候補の列挙 |
| `GhostdagData` | selected parentを含むblue列と分類結果の保持 |

UTXOの適用やvirtualの更新経路は第14章で扱う。本章の分類処理の照合だけで、それらの状態遷移まで検証できたとは扱わない。

---

## 12.17 不変条件を整理する

| 不変条件 | なぜ重要か | テスト例 |
|---|---|---|
| DAGは循環しない | Past/Futureが矛盾しない | 自己参照・循環の拒否 |
| 親参照は既知のブロックを指すか、未解決として管理する | 依存関係を壊さない | 不足親の保留 |
| 同じDAGから同じGHOSTDAG結果が得られる | コンセンサスの決定性 | 受信順を入れ替える |
| selected parentは親の一つ | 関係の整合性 | メタデータ検証 |
| blue/redの扱いは合意規則に従う | ノード間の一致 | 既知テストベクトル |
| 合意順序は親子関係に反しない | 過去を未来より後に置かない | トポロジカル検証 |
| UTXOの同一出力を二重消費しない | 状態の整合性 | 競合トランザクションの評価 |

> 上表は学習上の設計・検証観点であり、現行実装が同じ名称のアサーションを備えていることを意味しない。

---

## 12.18 よくある誤解

**Q1. BlockDAGならGHOSTDAGは不要？**  
いいえ。DAGはブロック間の関係を表すデータ構造であり、どの順序を合意するかは別途決める必要がある。

**Q2. blueブロックは「有効」、redブロックは「無効」？**\
いいえ。blue/redはGHOSTDAGによる分類であり、ブロックの形式・PoW・トランザクションの有効性とは異なる。

**Q3. Selected parent以外の親は無視される？**  
いいえ。selected parentは評価の基準だが、他の親が導入するブロックもmerge-setの処理などに関係する。

**Q4. blue scoreが高ければ、必ずトランザクション数も多い？**\
いいえ。blue scoreはトランザクション数ではない。

**Q5. `k`を大きくすればセキュリティと性能が両方上がる？**  
いいえ。安全性の前提とネットワーク条件を含む設計パラメータである。

**Q6. GHOSTDAGが二重支払いを直接すべて解決する？**  
GHOSTDAGは合意順序を導く重要な構成要素だが、UTXOの適用可能性の判定も必要になる。

**Q7. 本章のRustコードはKaspaのGHOSTDAG実装？**  
いいえ。Past・Anticone・`k`-clusterという基礎概念の実験器である。

---

## 12.19 理解度チェック

1. BlockDAGとGHOSTDAGの役割の違いを説明せよ。
2. `Past(B)`、`Future(B)`、`Anticone(B)`をそれぞれ定義せよ。
3. selected parentが一つでも、なぜ他の親を保持する必要があるのか。
4. blueとredが「有効／無効」と一致しない理由を説明せよ。
5. `k`-clusterの条件を集合記法で書け。
6. blue scoreとblue workの違いを説明せよ。
7. トポロジカルソートだけで全ノードの順序が一致するとは限らないのはなぜか。
8. 競合する二つのトランザクションが異なるブロックに含まれるとき、どのレイヤーで最終的な適用可否を判断するか。
9. 同じDAGを異なる順番で受信しても結果が変わらないことを、どうテストするか。
10. 原論文と`rusty-kaspa`を対応づけるとき、コミットハッシュを記録する理由は何か。

### 発展課題

- 小規模DAGの全ての部分集合を列挙し、`k`-cluster条件を満たす集合を探す。**これはPHANTOM/GHOSTDAGの実装ではない**ことを明記する。
- 既知のDAGテストベクトルを収集し、実装のblue/red分類と順序を比較する。
- GHOSTDAGメタデータとVirtual Stateの依存関係をC4コンポーネント図で表す。
- 遅延・分断・並行生成のシナリオを作り、合意結果の変化を記録する。

---

## 12.20 まとめ

本章の核心は、**DAGを保持することと、DAGに合意可能な意味を与えることは別**だという点である。

1. Kaspaは複数の親を持つBlockDAGを使用する。
2. GHOSTDAGはselected parentを基準に、並行ブロックをblue/redとして評価する。
3. `k`-clusterは並行性の制約を考える基礎概念である。
4. blue scoreとblue workは異なる指標である。
5. 決定的な順序が得られても、トランザクションの適用可能性は別に検証する。
6. 実装を読む際は、DAG・Reachability・GHOSTDAG・Ordering・Virtual Stateの責務を分ける。

次章では **Chapter 13 — Difficulty Adjustment Algorithm（難易度調整アルゴリズム）** に進み、PoW難易度がどのようにネットワークのブロック生成状況に適応するかを学ぶ。

---

## 参考資料

### 原論文・研究

1. Yonatan Sompolinsky, Aviv Zohar, **[Secure High-Rate Transaction Processing in Bitcoin](https://eprint.iacr.org/2013/881)** — GHOSTの研究背景。
2. Yonatan Sompolinsky, Yoad Lewenberg, Aviv Zohar, **[SPECTRE: A Fast and Scalable Cryptocurrency Protocol](https://eprint.iacr.org/2016/1159)** — BlockDAG系の関連研究。
3. Yonatan Sompolinsky, Yoad Lewenberg, Aviv Zohar, **[PHANTOM, GHOSTDAG: Two Scalable BlockDAG Protocols](https://eprint.iacr.org/2018/104)** — PHANTOM/GHOSTDAGの原論文。
4. Satoshi Nakamoto, **[Bitcoin: A Peer-to-Peer Electronic Cash System](https://bitcoin.org/bitcoin.pdf)** — 比較対象となるNakamoto consensus。

### 実装・公式資料

5. [Kaspa — rusty-kaspa](https://github.com/kaspanet/rusty-kaspa) — Rustノード実装。
6. [Kaspa Improvement Proposals](https://github.com/kaspanet/kips) — プロトコル変更・提案の調査。
7. [Kaspa Wiki](https://wiki.kaspa.org/en/home) — 技術用語と関連資料の入口。

> **未確認事項:** 原論文の証明との完全照合、今回の24ケース以外の分類結果、メインネットの稼働バージョン・パラメータ、Virtual Stateで使われる最終的な取引処理順序は継続レビューの対象とする。本章の候補列挙順を、そのまま取引の最終処理順と同一視しない。
