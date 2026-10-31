# Appendix F — 論文・KIP・実装対応表

> **Mastering Kaspa / 付録F**  
> 原稿: v0.1（技術レビュー前）｜確認日: 2026-10-30  
> 対象: Kaspaの理論・仕様・実装・検証をつなぐための調査ガイド  
> **注意:** GitHubの`master`は変化する。以下のファイルパスは探索の入口であり、固定コミットでの完全照合を意味しない。KIPの状態は執筆時点のリポジトリREADME表示を参考にしたもので、ネットワークの稼働状況とは区別する。

## F.1 この付録の目的

Kaspaの理解には、次の4層を区別することが重要である。

1. **論文** — 何を解決し、どのような仮定で正しさを論じるか。
2. **KIP（Kaspa Improvement Proposal）** — どの変更を、どの条件で提案・仕様化するか。
3. **実装（rusty-kaspa）** — 実際のノードがどのような処理をするか。
4. **検証** — テスト、実験、リリース記録、実ネットワークで何を確認したか。

この4層は一致するとは限らない。論文のアイデアが実装されていない場合、KIPが改訂された場合、コードが複数のアップグレードを扱う場合がある。

```text
研究論文 ── 原理・安全性・仮定
   │
   ▼
KIP ─────── 仕様・互換性・導入条件
   │
   ▼
rusty-kaspa ─ 実装・ネットワーク別パラメータ
   │
   ▼
テスト・実験 ─ 再現性・境界条件・動作確認
```

**目標:** 「この仕様はどの論文に由来し、どのKIPで議論され、どのコードで処理され、何のテストで確認できるか」を説明できるようになること。

## F.2 一次資料の入口

| 種別 | 一次資料 | 調査対象 |
|---|---|---|
| Kaspa実装 | [kaspanet/rusty-kaspa](https://github.com/kaspanet/rusty-kaspa) | Rustノード、コンセンサス、RPC、テスト |
| KIP一覧 | [kaspanet/kips](https://github.com/kaspanet/kips) | 提案番号、状態、仕様文書 |
| KIP索引 | [kips/README.md](https://github.com/kaspanet/kips/blob/master/README.md) | KIPの表題とステータス |
| Kaspaドキュメント | [kaspanet/docs](https://github.com/kaspanet/docs) | 論文、設計・技術文書の探索 |
| Kaspa Wiki | [wiki.kaspa.org](https://wiki.kaspa.org/) | 用語と周辺解説（必ず一次実装へ戻る） |
| 旧Go実装 | [kaspanet/kaspad](https://github.com/kaspanet/kaspad) | 歴史的な実装比較（現行仕様の根拠にしない） |

`README`の状態表示だけでは、実装・アクティベーション・採用ネットワークは確定しない。

## F.3 対応表の読み方と証拠レベル

| 記号 | 意味 | 記録例 |
|---|---|---|
| P | 論文に記述がある | 論文の節・定理・図 |
| K | KIPに仕様がある | KIP番号・節・状態 |
| I | 実装に対応する処理がある | コミットSHA・関数・行 |
| T | テストを確認した | テスト名・実行コマンド・結果 |
| N | 対象ネットワークで有効と確認した | リリース・アクティベーション条件・ノード結果 |
| ? | 未確認 | 次に調べる事項 |

例えば`P/K/I`だけでは、**Mainnetで有効**とは言えない。`N`の根拠が必要である。

## F.4 理論・実装の主要対応マップ

次の表の「探索先」は**候補**であり、ファイル・関数を固定コミットで精査したことを示すものではない。

| テーマ | 研究・理論 | KIPとの関係 | rusty-kaspaの探索先 | 確認すべき性質 |
|---|---|---|---|---|
| BlockDAG | DAGの因果関係・部分順序 | 基本プロトコル | `consensus/`、`consensus/core/` | 祖先・過去・未来・anticone |
| GHOSTDAG | PHANTOM/GHOSTDAG系列 | 既存コンセンサスの基盤 | `consensus/`で`ghostdag`を検索 | selected parent、blue/red、順序 |
| DAGKnight | DAGKnight研究 | **KIP-0002**（索引ではProposed） | 現行実装と研究コードを区別 | 提案内容と有効化の差 |
| DAA | PoWの難易度調整 | **KIP-0004**（索引ではActive） | `consensus/`で`daa`を検索 | サンプル窓、境界条件 |
| UTXO | UTXO状態遷移 | 関連KIP複数 | `consensus/`で`utxo`を検索 | 二重支払いの拒否、保存則 |
| Script | 条件付き支出 | **KIP-0010、0017、0020** | `crypto/`、`consensus/`で`script`を検索 | オペコード、検証規則 |
| Mass | リソース計測と状態肥大化 | **KIP-0009、0013** | `consensus/core/src/config/params.rs` | 制限値、計算、拒否条件 |
| 高BPS | ブロック生成速度と遅延 | **KIP-0014**（Crescendo） | `params.rs`、関連テスト | activation前後の値 |
| 順序コミットメント | 正規順序・コミットメント | **KIP-0015** | `sequencing`等を検索 | 順序の決定性 |
| ZK検証 | 検証可能計算 | **KIP-0016** | `script`、`precompile`等を検索 | 検証の入力・費用・失敗 |
| Pruning | 履歴削減と安全な同期 | pruning関連文書 | `consensus/`で`pruning`を検索 | proofの検証、境界条件 |
| RPC / wRPC | ノード外部インターフェース | API関連KIP | `rpc/`、`wallet/`、`wasm/` | 接続・通知・再同期 |

**重要:** KIP番号はプロジェクトの索引で照合する。番号の連続性や、番号の大小から導入順を推測しない。

## F.5 論文を読むときの対応表

| 論文・研究テーマ | 最初に理解する概念 | コード側の探索語 | 実験アイデア |
|---|---|---|---|
| PHANTOM / GHOSTDAG | anticone、k-cluster、blue set | `ghostdag`, `anticone`, `blue_score` | 小規模DAGの色分け |
| SPECTRE | ペア比較による投票と順序 | 実装されていると仮定しない | 小規模DAGの投票モデル |
| DAGKnight | 適応的な安全性・ネットワーク条件 | `dagknight`、KIP-0002 | 論文とKIPの仮定を比較 |
| Pruning / Prunality | 過去の削減と安全性 | `pruning`, `proof` | 切り詰め前後の検証 |
| PoW・DAA | 目標間隔、観測窓、難易度 | `daa`, `difficulty` | ブロック間隔の分布 |

論文の表題・版・公開年・節番号は、書籍の正式な参考文献一覧（付録H）で**実際に読んだ版**に合わせて固定する。ここでは曖昧な論文名からDOIや著者を推測していない。

## F.6 KIPの対応表（公開索引の例）

2026-10-30に参照したKIP索引では、少なくとも以下の対応が掲載されている。**索引上のStatusは、稼働ネットワークでの機能提供を単独で証明しない。**

| KIP | 表題（要旨） | 索引上のStatus | 本書の対応章 | 調査ポイント |
|---|---|---|---|---|
| 0001 | Rustフルノードへの書き換え | Implemented | 10, 25, 36 | Go→Rustの移行目的 |
| 0002 | DAGKnightへのコンセンサス変更 | Proposed | 29, 30 | 提案と稼働実装の区別 |
| 0003 | 高BPS向けブロックサンプリング | Rejected | 13, 30 | 不採用の理由と代替 |
| 0004 | Sparse Difficulty Windows | Active | 13 | DAAのサンプル設計 |
| 0005 | メッセージ署名 | Active | 16, 24 | ドメイン分離・検証 |
| 0006 | Proof of Chain Membership | Draft | 21, 30 | 構想・証明の対象 |
| 0009 | 拡張Mass計算 | Active | 15, 19, 30 | ストレージ負荷 |
| 0010 | Script機能拡張のオペコード | Active | 16, 31, 32 | VMと互換性 |
| 0013 | Transient Storage Handling | Active | 15, 30 | 一時リソースの扱い |
| 0014 | Crescendo Hardfork | Active | 9, 13, 30 | 有効化前後のパラメータ |
| 0015 | Canonical Transaction Ordering | Active | 14, 34 | 正規順序とコミットメント |
| 0016 | ZK Precompile Opcode | Active | 33, 34 | 証明検証の費用・境界 |
| 0017 | Covenants and Improved Scripting | Active | 31, 32 | 支出条件とScript検証 |
| 0020 | Covenant IDs | Active | 31, 32 | IDと参照の仕様 |
| 0021 | Partitioned Sequencing Commitment | Active | 14, 34 | 証明対象とスケーリング |

> これは**網羅的なKIP一覧ではない**。最新の提案や番号の欠番を含め、出版時にREADMEと各KIP本文を再確認する。

## F.7 `rusty-kaspa`の探索ルート

```text
rusty-kaspa/
├── consensus/       # 合意規則・状態処理を探す入口
├── crypto/          # 暗号関連を探す入口
├── rpc/             # ノードAPIを探す入口
├── wallet/          # ウォレット関連を探す入口
├── wasm/            # WASM統合を探す入口
└── ...              # 実際のディレクトリ構成は対象コミットで確認
```

実装の所在は移動することがある。最初にディレクトリ構造を確認し、**名前だけで役割を断定しない**。

### F.7.1 探索コマンド

```bash
# ソース取得
git clone https://github.com/kaspanet/rusty-kaspa.git
cd rusty-kaspa

# 調査対象を固定する
git rev-parse HEAD
git status --short

# 高速検索（ripgrepがある場合）
rg -n 'ghostdag|selected_parent|blue_score' consensus
rg -n 'daa|difficulty' consensus
rg -n 'pruning|utxo' consensus
rg -n 'script|opcode' consensus crypto

# ripgrepがない場合
git grep -n 'ghostdag'

# 変更の経緯
git log --oneline -- consensus
```

**注意:** コマンドは探索用であり、全パス・検索語が全バージョンで一致するとは限らない。

## F.8 コンセンサスパラメータの追跡例

[`consensus/core/src/config/params.rs`](https://github.com/kaspanet/rusty-kaspa/blob/master/consensus/core/src/config/params.rs)には、ネットワークごとのパラメータや`ForkActivation`、`BlockrateParams`、`Params`などの定義が見られる。たとえば`BlockrateParams`には、目標ブロック時間、`ghostdag_k`、難易度サンプリング、finality・pruning深度などがまとまっている。

```text
Params
  ├── network ID
  ├── genesis
  ├── blockrate: BlockrateParams
  │      ├── target_time_per_block
  │      ├── ghostdag_k
  │      ├── difficulty_sample_rate
  │      ├── finality_depth
  │      └── pruning_depth
  └── crescendo_activation
```

調査では次を分ける。

- **定義**: どの構造体に値があるか。
- **選択**: Mainnet/Testnet/Devnetでどの値を使うか。
- **有効化**: アクティベーション条件をどの処理が参照するか。
- **使用**: 実際の検証・順序付けでどこから呼ばれるか。
- **テスト**: 境界値の直前・直後で何を検証しているか。

特に`ForkActivation`の存在は、**実行時の判定を追跡すべき**ことを示している。値の定義を見ただけでは仕様の全体像は分からない。

## F.9 GHOSTDAGの対応調査

GHOSTDAGを調べる際は、以下の依存関係を整理する。

```text
DAGの親子関係
     │
     ▼
Past / Anticone
     │
     ▼
Selected Parent
     │
     ▼
Merge Set
     │
     ▼
Blue / Red分類
     │
     ▼
Blue Score / Blue Work
     │
     ▼
順序付けとVirtual State
```

この図は**概念的な学習順**であり、実装の厳密な関数呼び出し順ではない。

### 調査質問

1. selected parentを選ぶ際の比較規則は何か。
2. anticoneの定義をどのデータ構造で実現するか。
3. `k`をどの設定から取得するか。
4. blue scoreとblue workの役割は何か。
5. ブロックのDAG順序とトランザクションの受理順序をどう区別するか。
6. 再編成に相当する状況でvirtual stateはどう更新されるか。

## F.10 DAAの対応調査

**KIP-0004**を入口として、難易度計算の実装を追う。

| 確認点 | 論文・KIPで読むこと | 実装で探すこと | テストで確認すること |
|---|---|---|---|
| サンプル | 観測対象・抽出条件 | `difficulty`, `sample` | 少数ブロック時の処理 |
| 時間 | timestampの扱い | `timestamp`, `median` | 異常な時刻 |
| 目標 | 目標生成速度 | `target_time_per_block` | アップグレード前後 |
| 境界 | 丸め・上限・下限 | 整数演算、clamp | overflow、極端な値 |

PoWの期待ブロック間隔と、個々のブロックの発生間隔は一致しない。**確率的な揺らぎ**と**アルゴリズムの不具合**を混同しない。

## F.11 UTXO・Scriptの対応調査

| 概念 | 実装で確認する責務 | 典型的な不変条件 |
|---|---|---|
| UTXO入力 | 存在確認・重複支出防止 | 同一UTXOを二度消費しない |
| UTXO出力 | 新しい支出可能状態の生成 | 金額が範囲内 |
| 手数料 | 入出力差分と制約 | 負の手数料を許さない |
| Script | 署名・条件評価 | 無効な支出を受理しない |
| Mass | リソース消費の制限 | 上限を超えるトランザクションを拒否 |
| Covenant | 支出先・出力等の制約 | 指定条件を満たさない支出を拒否 |

**KIP-0010、0017、0020**を調べる際は、Scriptの構文・VMの挙動・トランザクションの検証ルール・有効化条件を別々に記録する。

## F.12 L2・ZK・vProgsの位置づけ

ここでは、**L1の仕様**と**L2や研究プロトタイプ**を混同しない。

```text
Kaspa L1: 順序付け・検証・合意規則
     │
     ├── Script / Covenant / ZK関連機能（対象リリース要確認）
     │
     └── L2 / vProgsなどの周辺構想
            ├── オフチェーン実行
            ├── 状態・証明の生成
            └── L1への提出・検証（設計によって異なる）
```

- **KIP-0016**が扱う検証機能と、任意のZKアプリが利用可能なことは別。
- **vProgs**の研究資料と、稼働中のL1コンセンサスコードは別。
- **EVM互換L2**の資産管理方式と、Kaspa L1が保証する範囲は別。

調査記録には「どのネットワーク・どのソフトウェア・どのブリッジ・どの検証者か」を必ず記載する。

## F.13 実装追跡の5ステップ

### Step 1 — 問題を一文にする

例: 「なぜGHOSTDAGの`k`はこのネットワークでこの値なのか」。

### Step 2 — 論文の前提を読む

ネットワーク遅延、ブロック生成率、攻撃者モデルなどを整理する。

### Step 3 — KIPの状態を確認する

関連提案があれば、本文・議論・改訂履歴を確認する。関連KIPがないことも記録する。

### Step 4 — コードを固定して追う

```bash
git rev-parse HEAD
rg -n 'ghostdag_k|target_time_per_block' consensus
```

定義→呼び出し元→利用箇所→テストの順に調べる。

### Step 5 — 小さな実験で検証する

境界値、想定外入力、ネットワーク差分をテストする。実行結果と使用コミットを記録する。

## F.14 Gitの履歴から設計意図を読む

```bash
# 変更を検索
git log --oneline --all --grep='ghostdag' -i

# ファイルの変更履歴
git log --follow --oneline -- consensus/core/src/config/params.rs

# 特定コミットの変更内容
git show --stat <COMMIT_SHA>
git show <COMMIT_SHA> -- consensus/core/src/config/params.rs

# 行ごとの変更由来
git blame consensus/core/src/config/params.rs
```

コミットメッセージは**設計意図の手がかり**であり、正式仕様そのものではない。関連PR、レビュー、テストも読む。

## F.15 追跡カードのテンプレート

```markdown
### 調査ID: F-001
- 問い:
- 関連章:
- 論文: URL / 版 / 節
- KIP: 番号 / URL / 状態 / 更新日
- 実装: リポジトリ / コミットSHA / ファイル / 関数
- テスト: テスト名 / コマンド / 結果
- ネットワーク: Mainnet / Testnet / Devnet / Simnet
- 有効化条件:
- 確認できたこと:
- 未確認事項:
- 最終確認日:
```

このカードを`experiments/`に保存すると、本文に根拠を戻しやすくなる。

## F.16 対応表をCSVとして管理する

書籍の規模が大きくなると、対応表をCSVで管理すると便利である。

```csv
id,topic,paper,kip,source_path,commit,test,status
F-001,GHOSTDAG,,,consensus/,未記録,未記録,調査中
F-002,DAA,,KIP-0004,consensus/,未記録,未記録,調査中
F-003,Script,,KIP-0017,consensus/,未記録,未記録,調査中
```

ここでの`未記録`は、**未実装**を意味しない。

## F.17 Rustによる対応表の整合性チェック

次は外部クレートなしで動く、**書籍管理用**の教育コードである。Kaspaのコンセンサス実装ではない。

```rust
#[derive(Debug)]
struct Evidence<'a> {
    topic: &'a str,
    paper: Option<&'a str>,
    kip: Option<&'a str>,
    commit: Option<&'a str>,
    test: Option<&'a str>,
    mainnet_verified: bool,
}

fn is_reproducible(e: &Evidence<'_>) -> bool {
    e.commit.is_some() && e.test.is_some()
}

fn can_claim_mainnet(e: &Evidence<'_>) -> bool {
    e.mainnet_verified && is_reproducible(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposal_alone_does_not_prove_deployment() {
        let e = Evidence {
            topic: "DAGKnight",
            paper: Some("research document"),
            kip: Some("KIP-0002"),
            commit: None,
            test: None,
            mainnet_verified: false,
        };
        assert!(!can_claim_mainnet(&e));
    }

    #[test]
    fn code_and_test_alone_do_not_prove_mainnet() {
        let e = Evidence {
            topic: "Example feature",
            paper: None,
            kip: Some("KIP"),
            commit: Some("example-commit"),
            test: Some("example-test"),
            mainnet_verified: false,
        };
        assert!(is_reproducible(&e));
        assert!(!can_claim_mainnet(&e));
    }
}
```

実行方法:

```bash
rustc --test evidence.rs -o evidence_tests
./evidence_tests
```

`mainnet_verified`は人間が記録するフラグであり、このコードがネットワークを自動検証するわけではない。

## F.18 変更追跡と差分レビュー

論文やKIPが更新されたら、次を順に確認する。

1. どの定義が変わったか。
2. 安全性の前提が変わったか。
3. 実装のどの関数が影響を受けるか。
4. 既存のテストは新しい境界をカバーするか。
5. 本書のどの章・図・数値が古くなるか。

**例:** ブロック生成速度の変更は、目標時間だけでなく、DAA、finalityの時間換算、pruning、報酬成熟、監視メトリクスの解釈にも影響し得る。実際の影響範囲は対象バージョンで調査する。

## F.19 C4モデルによる調査対象の境界

```text
[研究者・開発者]
        │
        ▼
[仕様・文書] ───── [KIPリポジトリ]
        │                  │
        └──────┬───────────┘
               ▼
        [rusty-kaspa]
          │        │
          ▼        ▼
      [Consensus] [RPC / Wallet]
          │
          ▼
       [Tests]
```

これは**調査のための概念図**であり、実際のモジュール依存を正確に表すC4コンポーネント図ではない。実装図にするには、Cargo workspaceとクレート依存を確認する。

## F.20 DDD・不変条件・ステートマシンで読む

### DDD的な問い

- 「ブロックの有効性」はどのコンテキストの責務か。
- 「トランザクションの受理」と「Mempoolへの格納」は同じ概念か。
- 「確定」と「アプリケーションの入金完了」は同じ概念か。

### 不変条件の問い

- 同じUTXOを二度消費できないか。
- 無効なScriptでUTXOが消費されないか。
- 順序付けの結果は同じDAGから決定的に得られるか。

### 状態遷移の問い

```text
Received → Validated → Accepted → Applied
              │            │
              ▼            ▼
           Rejected      Reconsidered
```

この図は一般化した**調査用モデル**であり、Kaspa内部の正式な状態名や全遷移を示すものではない。

## F.21 テストを読む順序

| 順序 | 種類 | 目的 |
|---|---|---|
| 1 | Unit | 個々の計算や境界条件 |
| 2 | Property / Invariant | 広い入力集合での性質 |
| 3 | Integration | 複数コンポーネント間の整合性 |
| 4 | Regression | 過去の不具合の再発防止 |
| 5 | Devnet | ネットワーク動作と運用条件 |

テストの**存在**と**現在の環境での成功**は区別する。実行したら日時・環境・SHA・コマンド・結果を記録する。

## F.22 再現実験: パラメータから処理を追う

**目的:** `ghostdag_k`の定義から利用箇所まで追跡する。

```bash
git clone https://github.com/kaspanet/rusty-kaspa.git
cd rusty-kaspa
git rev-parse HEAD
rg -n 'ghostdag_k' .
rg -n 'ForkActivation|crescendo_activation' consensus
```

記録すること:

1. `ghostdag_k`の定義箇所。
2. MainnetとDevnetでの設定箇所。
3. 値が参照される関数。
4. 関連するテスト。
5. 値の変更が与える影響について、コードから確実に言えること。

**禁止する推論:** 「この値があるから、ある研究提案がMainnetで稼働している」。

## F.23 再現実験: KIPとコードの差分

**題材:** KIP-0014（Crescendo）。

1. KIP本文で、導入理由と仕様変更点を抽出する。
2. `params.rs`で`crescendo_activation`と前後のブロック時間を調べる。
3. `ForkActivation`を利用する処理を検索する。
4. テストが境界の直前・直後を確認しているか調べる。
5. ネットワーク別の導入状況を別途検証する。

**成果物:** 「KIPの節 → ソースの関数 → テスト名」の3列対応表。

## F.24 再現実験: 未採用提案を調べる

**題材:** KIP-0003（索引上Rejected）。

調べる内容:

- 解決しようとした問題。
- 提案されたアルゴリズム。
- 不採用とされた理由（議論・履歴で確認）。
- 実装に残っている類似の考え方があるか。

**学習上の意義:** 「採用された仕様」だけでなく、なぜ別案が選ばれなかったかを学べる。

## F.25 セキュリティ上の注意

- コンセンサスコードの変更は、ローカルでテストが通ってもネットワーク互換性を保証しない。
- 公開ネットワークの資金を使って未検証コードを試さない。
- Devnetの成功をMainnetの安全性の証拠としない。
- 提案段階の機能を、ユーザーに「利用可能」と案内しない。
- 署名、鍵、UTXO、Script、ZK検証は特に境界値と失敗時挙動を調べる。

## F.26 AIを調査に使う際のルール

AIは調査の補助にはなるが、証拠にはならない。

**よい依頼:**

> このコミットの`Params`定義と`ForkActivation`の利用箇所を列挙し、根拠となるファイル・行を示して。推測は別欄に分けて。

**避ける依頼:**

> KaspaのすべてのKIPがMainnetで使えるか教えて。

生成された関数名・パス・仕様・テスト名は、必ずリポジトリで確認する。

## F.27 本書とのクロスリファレンス

| 本書 | この付録の調査対象 |
|---|---|
| 第9〜14章 | BlockDAG、GHOSTDAG、DAA、順序付け |
| 第15〜20章 | UTXO、Script、Mass、マイニング、安全性 |
| 第21〜28章 | Pruning、RPC、Rust実装、Devnet |
| 第29〜30章 | DAGKnight、KIPの評価 |
| 第31〜34章 | Covenants、ZK、プログラマビリティ |
| 第35〜36章 | アプリ開発、コントリビュート |
| 付録E | 数値・単位・ネットワーク設定 |
| 付録G | 実験結果・テストベクトル |
| 付録H | 文献の版・コミット・変更履歴 |

## F.28 出版前の確認チェックリスト

- [ ] 各論文の正式名称・著者・版・節番号を固定した。
- [ ] 各KIPの状態を最新版で確認した。
- [ ] 各KIPの状態とMainnetでの有効化を混同していない。
- [ ] 各ソースのコミットSHAを記録した。
- [ ] 実装のファイルパスと関数名を対象SHAで確認した。
- [ ] テストコマンドを実行し、結果を保存した。
- [ ] ネットワーク別の差異を記録した。
- [ ] 論文の主張とコードの挙動が異なる箇所を明示した。
- [ ] 本文中の表・図・数値に根拠を付けた。
- [ ] 未確認事項を「確認済み」と書いていない。

## F.29 理解度チェック

1. 論文・KIP・実装・テストは、それぞれ何を証明するか。
2. KIPが`Active`であることだけで、Mainnet稼働を断定できるか。
3. `params.rs`に値が定義されていることと、実際に使われていることの違いは何か。
4. GHOSTDAGとDAGKnightを混同しないために、何を確認するか。
5. コミットSHAを固定する必要があるのはなぜか。
6. Devnetのテスト成功をどこまで一般化できるか。
7. AIが提示した関数名を検証する最短手順は何か。
8. コンセンサス変更のレビューで不変条件が重要なのはなぜか。

## F.30 まとめ

Kaspaを深く理解するには、論文を読むだけでも、Rustコードを読むだけでも十分ではない。**理論が仕様になり、仕様が実装になり、実装がテストで検証されるまでのつながり**を追うことが重要である。

```text
「なぜそう設計したか」── 論文・議論
          ↓
「何を約束するか」──── KIP・仕様
          ↓
「どう実現したか」──── Rustコード
          ↓
「本当に動くか」───── テスト・再現実験
          ↓
「どこで有効か」───── ネットワーク・リリース
```

この調査手順は、Kaspaへのコントリビュートだけでなく、他の分散システムやOSSの設計を学ぶ際にも応用できる。

## F.31 参考資料

1. [Kaspa Improvement Proposals — GitHub](https://github.com/kaspanet/kips)
2. [KIP索引（README）](https://github.com/kaspanet/kips/blob/master/README.md)
3. [rusty-kaspa — GitHub](https://github.com/kaspanet/rusty-kaspa)
4. [Consensus Parameters (`params.rs`)](https://github.com/kaspanet/rusty-kaspa/blob/master/consensus/core/src/config/params.rs)
5. [Consensus Constants (`constants.rs`)](https://github.com/kaspanet/rusty-kaspa/blob/master/consensus/core/src/config/constants.rs)
6. [Kaspa Documents](https://github.com/kaspanet/docs)
7. [Kaspa Wiki](https://wiki.kaspa.org/)
8. [旧Go実装 `kaspad`](https://github.com/kaspanet/kaspad)

**書籍編集メモ:** 参考URLは追跡の入口。出版用の厳密な引用には、対象コミットのパーマリンク、参照日、論文の版情報を追加すること。
