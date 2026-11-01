# Chapter 30 — Kaspa Improvement Proposals（Kaspa改善提案・KIP）

> **目次順の本文レビュー：2026-11-01。固定KIP文書・主要出典と規則選択教材を確認。全実装/本番有効化の照合ではない。**
> 本章のKIP文書はコミット `e4ae2332117b5cb68bd6188e065ef885b6d17939`（2026-07-15）を2026-11-01に取得して確認した。ノードの基準版v1.0.1のSHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` とは別の参照軸である。KIPの状態、実装、ネットワークへの有効化状況は変化する。必ず参照時点の一次資料・リリース・実ノードで再確認する。

確認記録は `experiments/chapter-30-review-2026-11-01.md`。文書上のStatusと、対応実装・リリース・ネットワーク有効化の証拠を分ける。

## 30.1 本章の目的

Kaspa Improvement Proposal（**KIP**）は、Kaspaのプロトコル、ネットワーク更新、ノードやAPIなどに関する技術提案を文書化する仕組みである。KIPを読むことは、機能を暗記することではない。**なぜ変更が必要で、どの不変条件が変わり、どのソフトウェアに影響するのか**を追跡する作業だ。

本章を終えたら、次を説明できることを目指す。

1. 提案・仕様・実装・有効化を区別する。
2. コンセンサス変更とAPI変更の互換性リスクを評価する。
3. KIPからRustのコード、テスト、ネットワークでの挙動まで追跡する。
4. 小さなKIP相当の提案書と検証計画を作る。

## 30.2 KIPが必要な理由

分散ネットワークでは、単一の開発者がコードを変更しても、すべてのノードが同じルールを採用するとは限らない。特にコンセンサス規則の変更は、ノード間で結果が食い違えばチェーン分岐を引き起こし得る。

```text
問題の発見
    │
    ▼
変更案と根拠の文書化（KIP）
    │
    ▼
技術レビュー・議論
    │
    ▼
実装・テスト・監査
    │
    ▼
リリース・有効化計画
    │
    ▼
ネットワークでの運用・観測
```

これは**理解のための一般化した流れ**であり、すべてのKIPに必須の公式ワークフローを意味しない。提案が承認されたように見えても、そのままメインネットで動作するとは限らない。

## 30.3 一次資料を起点にする

- 本章の固定KIP一覧：[README](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/README.md)
- 固定版の本文：[ファイル一覧](https://github.com/kaspanet/kips/tree/e4ae2332117b5cb68bd6188e065ef885b6d17939)
- 以後の改訂の入口：<https://github.com/kaspanet/kips>
- KIPのPull Request：<https://github.com/kaspanet/kips/pulls>
- Rust実装：<https://github.com/kaspanet/rusty-kaspa>
- Kaspa研究フォーラム：<https://research.kas.pa/>

固定リンクと、状態が変化するPR・リポジトリの入口を区別する。KIP番号だけで内容を推測しない。**`master`上の文書、未マージのPR、フォーク内の文書は別物**である。同じ番号を用いた競合案がPR上に存在する場合もある。

## 30.4 状態を読むときの注意

公式一覧には `Implemented`、`Proposed`、`Draft`、`Rejected`、`Active` などの表示がある。これらは**KIP文書上の状態**を表すもので、必ずしも「現在のメインネットで有効」という意味ではない。

| 観点 | 確認する質問 | 証拠の例 |
|---|---|---|
| 提案 | 何を変えたいのか | KIP本文、議論 |
| 仕様 | 変更条件が十分に定義されているか | 数式、疑似コード、テストベクトル |
| 実装 | 対応コードは存在するか | PR、コミット、テスト |
| リリース | どのバージョンに含まれるか | リリースノート |
| 有効化 | どのネットワークでいつ有効か | アクティベーション条件、ノードの状態 |
| 運用 | 期待通り動いているか | メトリクス、ログ、障害報告 |

**`Active = mainnet activated` と読み替えない**こと。以下のStatusは冒頭の固定版で読んだ文書情報であり、観測日時のMainnet対応表ではない。一覧の題名と本文のTitleも一致するとは限らず、それぞれの出典を残す。

## 30.5 KIPの分類を考える

KIPは変更対象によって検証方法が異なる。

| 分類 | 主な変更対象 | 特に重要なテスト |
|---|---|---|
| Consensus | ブロック・トランザクションの妥当性、順序、状態遷移 | 同じ入力と規則で決定的に一致するか |
| Node / P2P | 同期、メッセージ、ピア管理 | 旧版・新版間の接続、耐障害性 |
| API / RPC | 外部クライアントのインターフェース | 後方互換性、エラー、型 |
| Script Engine | スクリプト命令と実行規則 | 検証結果、計算量、資源上限 |
| Applications | ウォレット・署名等の規約 | 相互運用性、鍵管理、安全性 |

これは本書の変更対象による整理であり、公式Layerフィールドの列挙型ではない。一つのKIPが複数の分類にまたがることもある。

## 30.6 KIP-1：Rustによるフルノードの書き直し

[KIP-1](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0001.md) はKaspaフルノードをRustで再構築する提案であり、公式一覧では `Implemented` とされる。目的は単なる言語変更ではなく、保守性・拡張性・性能の改善だった。

ここで重要なのは、**性能改善が将来のプロトコル変更を可能にする基盤になる**という関係だ。

```text
ノードの設計・性能改善
          │
          ▼
より高い処理負荷への対応
          │
          ▼
新しいプロトコル機能の検討余地
```

ただし「Rustだから自動的に安全」「Rustだからコンセンサスが正しい」とは言えない。メモリ安全性とプロトコルの正しさは別の性質である。

## 30.7 KIP-2：DAGKnightへの移行提案

[KIP-2](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0002.md) はGHOSTDAGからDAGKnightへのコンセンサス更新を提案し、公式一覧では `Proposed` と表示される。

論点は、事前の遅延想定に基づく `k` を用いるGHOSTDAGに対して、DAGKnightがネットワークの実際の状況に応じた確認ポリシーを目指すこと。KIP本文は研究、効率的実装、ウォレット向け確認ポリシーAPIなどを課題に挙げている。

**重要：KIPが存在することはDAGKnightがメインネットに導入済みである証拠ではない。**

基準版GHOSTDAGのkはネットワーク更新で切り替わり得るため、「全リリースで一定の値」とも解釈しない。KIP-2はウォレット側の確認条件に加え、difficulty・発行・Pruning等のためのグローバル遅延上限への適応も研究課題に挙げる。parameterlessを全用途で遅延仮定不要という意味にはしない。

[第29章](chapter-29.md)のセキュリティモデルと照らし合わせ、論文上の性質・提案された仕様・実装の状態を別々に読む。

## 30.8 KIP-4：難易度調整のウィンドウ

[KIP-4](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0004.md) はSparse Difficulty Windowsを扱う。高いブロック生成率で難易度調整の計算・サンプリングをどう効率化するかという問題に関係する。

評価するポイント：

- 入力に使うブロック集合の定義は決定的か。
- 遅延・並行ブロックの多い状況でもノード間で同じ結果になるか。
- 計算量・メモリ使用量はどう変わるか。
- 境界条件や極端なネットワーク状況のテストがあるか。

## 30.9 KIP-9・KIP-13：状態肥大化と資源制約

[KIP-9](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0009.md) はストレージ負担を考慮したmassの仕組み、[KIP-13](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0013.md) は一時的なデータ量への制約を扱う。

ここでの設計問題は「トランザクションを処理できるか」だけではない。**UTXOの長期的な保持コスト**、転送・検証コスト、Pruningまでのブロック本体等の保持コストを分けて考える必要がある。

```text
トランザクション
    ├── 計算コスト
    ├── 永続的なUTXOコスト
    └── 一時的な転送・処理・本体保持コスト
              │
              ▼
       ノード資源の制約
```

KIP-13の「一時的」は転送中のRAMだけではなく、Pruningまで保持するブロック等のディスク負担も含む。KIP-9の永続的なUTXO負担と、同じ上限・同じ尺度だと決めつけない。

アプリケーション開発者は、送金額や手数料だけでなく、出力数・出力額・トランザクションサイズに伴う制限を考慮する。

## 30.10 KIP-10・KIP-17・KIP-20：スクリプトの表現力

- [KIP-10](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0010.md)：トランザクション情報を参照する新しいスクリプト命令。
- [KIP-17](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0017.md)：Covenantsを含むスクリプト機能の拡張。
- [KIP-20](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0020.md)：Covenant IDに関する提案。

固定KIP文書には基準版v1.0.1より新しい改訂も含まれる。Activeの表示だけを根拠に、本書のSDK/Devnetでその命令やフィールドを使えるとは判断しない。

「高度なスクリプトを提案している」ことと「任意のスマートコントラクトをL1で実行できる」ことは同義ではない。追加命令の仕様、計算資源制約、利用可能なネットワークを確認する。

## 30.11 KIP-14：複数の変更を束ねるハードフォーク

[KIP-14](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0014.md) はCrescendoハードフォークを記述する。1 BPSから10 BPSへの変更に加え、複数の関連する改善と移行上の論点を扱う。

一つの機能だけを見ると単純でも、ネットワーク更新では依存関係が発生する。

```text
ブロック生成率の変更
   ├── DAAの負荷
   ├── ブロック・トランザクション検証
   ├── ネットワーク伝播
   ├── Mempoolの処理
   ├── ストレージ・Pruning
   └── ウォレットやインデクサの追従
```

このような変更は、**KIP単体ではなく依存するKIP群・実装・テストネット運用**を合わせて検証する。

## 30.12 KIP-15：順序のコミットメント

[KIP-15](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0015.md) は、受理されたトランザクションの順序を検証可能な形でコミットする設計を扱う。L2がL1の順序を根拠として利用する場合に重要な論点となる。

本文のTitleはCanonical Transaction Ordering and SelectedParent Accepted Transactions Commitmentであり、一覧のSequencing Commitmentsという題名と区別する。受理順序のコミットメントがあることだけで、Pruning後の任意の取引・L2状態・復旧素材が自動取得できるとも判断しない。

検討する問い：

1. 何の順序をコミットするのか（ブロック順序とトランザクション受理順序を混同しない）。
2. そのコミットメントを第三者がどう検証するのか。
3. Pruningされた履歴に対して何を証明できるのか。
4. Reorganization時にどのように追従するのか。

KIPの擬似コードだけでなく、関連する実装PR・実際のヘッダ形式を照合する。

## 30.13 KIP-16・KIP-21：高度な検証とコミットメント

[KIP-16](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0016.md) はZK証明検証のための命令を扱う。本文のTitleはNew Transaction Opcodes for Verifiable Computationで、一覧のZK Precompile Opcodeとは異なる。[KIP-21](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0021.md) はPartitioned Sequencing Commitmentに関する設計を扱う。

KIP-21の本文はkip-0021.mdで、同名ディレクトリは補足資料の入口である。本文の題名はPartitioned Sequencing Commitment with O(activity) Proving。accepted_id_merkle_rootという既存フィールドを使いつつ、activation後にレーンごとのコミットメント等へ意味を変える設計を記述する。同じフィールド名でもコミット対象・hashの組み立てが同じとは限らない。

いずれも「検証できること」「証明に含めること」「誰が何を信頼するか」を明示して読む。提案内容は、L1・L2・証明者・検証者の責務を混同せずに整理する。

## 30.14 KIP-22を例にしたPRと正式文書の違い

2026-11-01に確認した[PR #37](https://github.com/kaspanet/kips/pull/37)はP2MR ScriptPublicKey for quantum resistance、[PR #48](https://github.com/kaspanet/kips/pull/48)はPay-to-Public-Key-Hash via P2SHを題名に含み、どちらもKIP 22という番号を使ってOpenだった。一方、冒頭の固定masterのREADMEとファイル一覧にはKIP-22がない。

これは番号の一致から仕様を一意に決められない具体例である。PR番号、対象リポジトリ、base/head、取得日時・コミット、レビュー/マージ状態を記録し、正式文書・対応実装・ネットワーク有効化と分ける。PR一覧に表示された関連リンクのMerged表示を、そのPR自身のマージ状態と取り違えない。

これらのPRの暗号方式や安全性を今回検証したわけではない。後で状態が変わる場合も、固定文書での不在と現在のPRの存在は別の証拠として扱う。

## 30.15 互換性を4つに分解する

**互換性**は一つの属性ではない。

| 互換性 | 例 | 失敗すると |
|---|---|---|
| Consensus | ブロックの妥当性規則 | ノード間の合意不一致 |
| Wire / P2P | メッセージ形式 | 接続失敗・同期失敗 |
| API / RPC | フィールド、型、イベント | アプリケーション障害 |
| Data / Storage | 永続化形式・インデックス | 移行失敗・データ再構築 |

加えて、ウォレットの署名形式やアドレス規約、インデクサのスキーマなど、エコシステム側の互換性も評価する。

## 30.16 KIPを読むための質問テンプレート

KIPを一つ読むごとに、以下を埋める。

```markdown
# KIP調査ノート

- KIP番号・正式名称：
- 文書URL・コミット：
- 調査日：
- 文書上のStatus：
- 問題（Why）：
- 変更する規則（What）：
- 変更しない規則（Non-goals）：
- 前提条件・脅威モデル：
- 互換性への影響：
- 対応PR・コミット：
- テストネット／メインネットの状態：
- 検証可能な受け入れ条件：
- 未解決の疑問：
```

調査中のメモは本書の外へ置き、公開用に整理した記録だけを追加する。experiments/kip-reviews/は保存先の提案例で、本章で新規作成したディレクトリではない。

## 30.17 KIPからRust実装を追跡する

```bash
# 本書とは別の調査用ディレクトリで実行する
git clone https://github.com/kaspanet/kips.git kips-review
git -C kips-review checkout --detach e4ae2332117b5cb68bd6188e065ef885b6d17939
git -C kips-review rev-parse HEAD

# ノード側は第1章の固定版checkoutを再利用する
git -C rusty-kaspa-v1.0.1 rev-parse HEAD
grep -n -E 'calc_contextual_masses|calc_storage_mass' rusty-kaspa-v1.0.1/consensus/core/src/mass/mod.rs
grep -n -E 'struct ForkActivation|fn is_active|struct ForkedParam' rusty-kaspa-v1.0.1/consensus/core/src/config/params.rs
```

ノード側のHEADは冒頭の完全SHAと照合する。KIP文書とソースのコミットを別々に固定し、検索ヒットだけで対応機能の有効化を確認したことにしない。未コミット変更のあるcheckoutを移動する場合は別の調査コピーを用意し、変更を消すreset等は行わない。

### 追跡の順序

1. KIPのMotivationで「問題」を確認する。
2. Specificationで「規則」を抽出する。
3. Backwards Compatibilityで「破壊的変更」を確認する。
4. 関連PRの差分を読む。
5. テストとテストベクトルを探す。
6. ノードのリリース・有効化条件を確認する。
7. Devnet/Testnetで期待挙動を再現する。

**PRがマージされても、その規則がすべてのネットワークで有効とは限らない。**

## 30.18 C4モデルで変更範囲を可視化する

```text
[Wallet / Application]
        │ RPC / wRPC
        ▼
[Kaspa Node]
   ├── RPC Adapter
   ├── Mempool
   ├── Consensus
   │     ├── Block Validation
   │     ├── GHOSTDAG / Ordering
   │     ├── Transaction Validation
   │     └── Virtual State
   ├── P2P ◀────────────▶ [Other Nodes]
   └── Storage
```

これは変更対象を整理する概念図であり、各処理の実行順や単一の原子的更新を示すものではない。

たとえばScript Engineを変えるKIPは、スクリプト実行だけでなく、トランザクション検証、Mempoolポリシー、ウォレットのトランザクション構築に影響し得る。

## 30.19 DDDで考える「ルールの所有者」

設計上、どのコンポーネントが最終的なルールを持つべきかを考える。

- **Consensus**：ネットワーク全体で一致しなければならない妥当性規則。
- **Mempool**：未確定トランザクションの受け入れ・優先順位などのポリシー。
- **Wallet**：ユーザーの意図から署名可能なトランザクションを構築する。
- **RPC**：外部から観測・操作するためのインターフェース。

同じ「トランザクションを拒否する」でも、コンセンサス違反なのか、Mempoolのローカルポリシーなのかで意味が違う。

## 30.20 ステートマシンとしてのアップグレード

以下はアップグレードを考えるための**概念モデル**であり、Kaspaの正式な状態遷移仕様ではない。

```text
NotScheduled
     │ リリース・有効化条件を確定
     ▼
Scheduled
     │ 条件を満たす
     ▼
Activated
     │ 観測・運用
     ▼
Operational
```

必要な不変条件の例：

- 同じ入力DAG・UTXO文脈・ネットワーク設定・規則セットなら、正直なノードの判定結果は一致する。
- 有効化前のブロックを、新規則で誤って再判定しない。
- 有効化後のブロックは新規則に照らして検証する。旧形式のデータを一律に無効にするという意味ではない。
- 永続化されたデータが、使用した規則セットと矛盾しない。

実際の有効化方法はKIP・実装ごとに確認する。基準版の[ForkActivation / ForkedParam](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/params.rs)は渡されたDAA scoreを使って比較・規則選択を行う。何のDAA scoreを渡すかは呼び出し元も読む。例えばGHOSTDAGのkはselected parentのDAA scoreが文脈となる。現在のRPC値をすべての過去ブロックの判定へ流用しない。

## 30.21 Rustで変更境界を表現する

次のコードは**アップグレード時の規則選択を学ぶための教育用モデル**であり、Kaspaの有効化規則を再現するものではない。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleSet {
    Legacy,
    Upgraded,
}

#[derive(Debug, Clone, Copy)]
pub enum Activation {
    Never,
    Always,
    AtScore(u64),
}

impl Activation {
    pub fn rules_at(&self, context_score: u64) -> RuleSet {
        match self {
            Self::Never => RuleSet::Legacy,
            Self::Always => RuleSet::Upgraded,
            Self::AtScore(at) if context_score >= *at => RuleSet::Upgraded,
            Self::AtScore(_) => RuleSet::Legacy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn switches_at_boundary() {
        let activation = Activation::AtScore(100);
        assert_eq!(activation.rules_at(99), RuleSet::Legacy);
        assert_eq!(activation.rules_at(100), RuleSet::Upgraded);
        assert_eq!(activation.rules_at(101), RuleSet::Upgraded);
    }
    #[test]
    fn explicit_always_and_never() {
        for score in [0, 100, u64::MAX] {
            assert_eq!(Activation::Never.rules_at(score), RuleSet::Legacy);
            assert_eq!(Activation::Always.rules_at(score), RuleSet::Upgraded);
        }
    }
    #[test]
    fn thresholds_at_integer_extremes_are_defined() {
        assert_eq!(Activation::AtScore(0).rules_at(0), RuleSet::Upgraded);
        assert_eq!(
            Activation::AtScore(u64::MAX).rules_at(u64::MAX - 1),
            RuleSet::Legacy
        );
        assert_eq!(
            Activation::AtScore(u64::MAX).rules_at(u64::MAX),
            RuleSet::Upgraded
        );
    }
    #[test]
    fn historical_context_is_not_a_sticky_activation_flag() {
        let activation = Activation::AtScore(100);
        assert_eq!(activation.rules_at(150), RuleSet::Upgraded);
        assert_eq!(activation.rules_at(99), RuleSet::Legacy);
        assert_eq!(activation.rules_at(100), RuleSet::Upgraded);
    }
}
```

本書の外で `cargo new --lib --edition 2021 activation-boundary-lab` を作り、src/lib.rsへ配置する。Rust/Cargo 1.90.0、外部依存なしで4テスト・書式・Clippyを確認した。

context_scoreは教材独自の文脈値であり、単純なブロック高でも実ノードの現在値でもない。Never/Alwaysのenum表現を含め、公式ForkActivationの内部表現・番兵値・全ネットワークの設定を再現したコードではない。文脈を取り直して過去の規則を選べることと、実ノードの移行・データ互換性の検証を分ける。

## 30.22 テスト戦略：何を証明したいのか

| テスト | 確認事項 |
|---|---|
| 単体テスト | 変更された関数・規則の境界条件 |
| Property-based | 入力を変えても保たれる不変条件 |
| 差分テスト | 旧版・新版の意図した差だけが生じるか |
| 統合テスト | Consensus・Mempool・RPC間の整合性 |
| 複数ノードテスト | 同じ入力集合・文脈へ到達後、受信順によらず一致するか |
| 障害注入 | 遅延・再起動・分断・不正入力への耐性 |
| 性能テスト | CPU・メモリ・ディスク・帯域への影響 |
| 移行テスト | 旧データ・旧ウォレット・旧クライアントの扱い |

### 境界条件・敵対的な入力・観測の例

境界付近のブロックや再編成イベント自体は正当な場合もある。正常系と資源攻撃等のケースを分けて期待結果を定義する。

- 境界値の直前・直後に作られたブロック。
- 大量の小額UTXOを生成するトランザクション。
- スクリプト実行資源を極端に消費する入力。
- 順序や親参照の異なる並行ブロック。
- 再編成の前後で観測されるRPCイベント。

## 30.23 KIPを提案するための最小テンプレート

```markdown
# Proposal: <短いタイトル>

## Abstract
何を変えるか（数行）

## Motivation
現状の問題と、その再現条件

## Specification
変更する規則、入力、出力、エラー、境界条件

## Rationale and Alternatives
他の案とトレードオフ

## Backwards Compatibility
Consensus / P2P / RPC / Storageへの影響

## Security Considerations
攻撃面、資源制約、想定外の挙動

## Test Plan
テストベクトル、複数ノード試験、性能測定

## Rollout / Activation
対象ネットワーク、移行方法、監視項目

## Open Questions
未決定事項
```

これは**学習用テンプレート**。正式な提出時は公式リポジトリの既存KIPと投稿ルールに合わせる。

## 30.24 実践演習A：KIP-9の影響を整理する

1. KIP-9を読み、問題設定を100〜200字でまとめる。
2. 「計算コスト」と「長期ストレージコスト」を分けて図示する。
3. 小額UTXOを大量に作るケースを考える。
4. Wallet・Mempool・Consensusの各責務を表にする。
5. 対応するRust実装とテストを探し、コミットを記録する。

**成果物：** `experiments/kip-reviews/kip-0009.md`

## 30.25 実践演習B：KIP-2とKIP-14を比較する

比較する軸は、**変更対象、研究段階、実装状況、ネットワークへの影響、検証方法**。

| 質問 | KIP-2 | KIP-14 |
|---|---|---|
| 主要な目的 | DAGKnightへのコンセンサス移行 | Crescendoハードフォーク |
| 文書上の状態（冒頭の固定版） | Proposed | Active |
| 主な論点 | セキュリティ・確認ポリシー | BPS・性能・関連変更の移行 |
| 調査の中心 | 研究・実装・有効化の区別 | 関連KIP群と運用上の依存関係 |

ここでの表は**学習用の入口**であり、最新の導入状況を確定するものではない。

## 30.26 実践演習C：架空のKIPを設計する

課題：**RPCでトランザクション検証失敗の理由を、より構造化して返す**改善案を考える。

1. 現在のエラー表現を調べる。
2. 変更対象をRPC境界に限定できるか考える。
3. 旧クライアントの互換性を検討する。
4. Rustの型でエラー分類を表す。
5. 契約テストを設計する。
6. 「コンセンサス規則を変更しない」ことを明記する。

フィールドの追加でも、旧クライアントのデコードや未知enumの扱いを確認する。Mempoolのローカルポリシーによる拒否と、コンセンサス上の無効も別の理由にする。

重要なのは、最初から大きなコンセンサス変更を提案することではなく、**変更範囲を限定し、検証可能な改善を記述すること**。

## 30.27 よくある誤解

**誤解1：KIP番号があるなら採用済み。**  
番号付きの未マージPRもある。公式一覧・PR・実装を区別する。

**誤解2：`Active`ならメインネットで有効。**  
文書の状態とネットワークの状態は別に確認する。

**誤解3：Rustに実装されていれば安全。**  
実装の存在と正しさは別。仕様適合テストと運用検証が必要。

**誤解4：新機能はすべてコンセンサス変更。**  
RPC・ウォレット規約・ノード内部改善など、合意規則を変えない変更もある。

**誤解5：ハードフォークは単一機能の切り替え。**  
実際には複数コンポーネントと外部サービスへの影響を調整する必要がある。

## 30.28 理解度チェック

1. KIPとGitHub PRは何が違うか。
2. 提案・実装・有効化を分ける理由は何か。
3. コンセンサス規則とMempoolポリシーの違いは何か。
4. KIP-1はなぜ単なる言語変更ではないのか。
5. KIP-2の存在からDAGKnightの導入済みを断定できないのはなぜか。
6. KIP-9とKIP-13が対象とする資源問題はどう異なるか。
7. KIP-15で「順序」が重要なのはなぜか。
8. 新しいRPCフィールドを追加する際、何をテストすべきか。
9. 30.14節の観測時点でOpenだったKIP-22のPRを技術書でどう扱うべきか。
10. 一つのKIPを実装まで追うとき、どんな証拠を残すべきか。

## 30.29 本章のまとめ

KIPを読む目的は、Kaspaの新機能を列挙することではない。**問題から仕様へ、仕様から実装へ、実装からネットワーク上の検証へ**と、変更の根拠を追跡することにある。

- KIPは技術提案を記述する文書である。
- 文書上の状態と、実際のネットワークへの有効化は別である。
- コンセンサス変更は決定性・互換性・移行条件を厳密に扱う。
- APIやアプリケーションの改善も、責務境界と互換性が重要である。
- 良い提案には、問題・仕様・トレードオフ・検証計画がある。

[次章](chapter-31.md)では、これらの改善提案と関係する **Covenants & Advanced Scripting** を学ぶ。

## 30.30 参考資料

1. [Kaspa Improvement Proposals — 固定版一覧](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/README.md)
2. [KIP Pull Requests](https://github.com/kaspanet/kips/pulls)
3. [KIP-1: Rust Rewrite](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0001.md)
4. [KIP-2: DAGKnight](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0002.md)
5. [KIP-4: Sparse Difficulty Windows](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0004.md)
6. [KIP-9: Extended Mass](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0009.md)
7. [KIP-10: Script Opcodes](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0010.md)
8. [KIP-13: Transient Storage](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0013.md)
9. [KIP-14: Crescendo](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0014.md)
10. [KIP-15: Canonical Transaction Ordering and SelectedParent Accepted Transactions Commitment](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0015.md)
11. [KIP-16: New Transaction Opcodes for Verifiable Computation](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0016.md)
12. [KIP-17: Covenants](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0017.md)
13. [KIP-20: Covenant IDs](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0020.md)
14. [KIP-21: Partitioned Sequencing Commitment with O(activity) Proving](https://github.com/kaspanet/kips/blob/e4ae2332117b5cb68bd6188e065ef885b6d17939/kip-0021.md)
15. [rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)

---

**確認範囲：** 固定KIPの主要説明・Title/Status、PRの区別、基準版の規則選択と独自教材を確認した。全KIPの仕様・暗号/証明・実装PR・全ネットワークでの有効化を照合した結果ではない。記録は `experiments/chapter-30-review-2026-11-01.md`。
