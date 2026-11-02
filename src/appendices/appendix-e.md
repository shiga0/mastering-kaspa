# Appendix E — プロトコルパラメータ一覧

> **Mastering Kaspa / 付録E**  
> 目次順の本文レビュー：2026-11-01。基準版の主要定義/利用箇所と単位教材を確認。\
> 確認基準：rusty-kaspa v1.0.1 / `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。ソース定義の照合であり、現行Mainnetの実測ではない。

対象版は[本書の対象バージョン](../baseline.md)、この回の結果は `experiments/appendix-e-review-2026-11-01.md` を参照。旧稿の別版master由来のフィールド/値を基準版へ整理した。

## E.1 この付録の目的

Kaspaのブロック生成速度、GHOSTDAG、難易度調整、UTXO、ブロック報酬、Pruningなどを理解する際、「パラメータ名」「単位」「どのネットワークの値か」「どのバージョンで有効か」を混同すると、実装を誤読する。

この付録は、**固定値を暗記するためではなく、仕様を追跡して検証するための索引**である。以下を区別する。

1. **コンセンサスパラメータ**：一致しなければノード間で合意できない可能性がある値。
2. **ネットワーク設定**：P2Pポート、DNSシーダーなど。合意ルールとは別。
3. **ノード運用設定**：ログ、RPC待受、データ保存先など。ローカルで変更できる設定。
4. **実測値**：観測BPS、手数料、ハッシュレートなど。プロトコル定数ではない。
5. **フォーク依存値**：有効化前後で意味・数値が変わる値。

> **重要**：以下は固定コミットの定義であり、現在稼働中のMainnetの設定・ソフトウェア・有効化状況を確認した値ではない。

## E.2 一次資料と確認の優先順位

| 優先 | 資料 | 確認するもの |
|---|---|---|
| 1 | [rusty-kaspa: `params.rs`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/params.rs) | `MAINNET_PARAMS`、`TESTNET_PARAMS`、`DEVNET_PARAMS`、`CrescendoParams` / `Bps` |
| 2 | [rusty-kaspa: `constants.rs`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/constants.rs) | 時間・サンプリング・セキュリティ関連定数 |
| 3 | [rusty-kaspa: `bps.rs`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/bps.rs) | `TenBps = Bps<10>` の導出式 |
| 4 | [rusty-kaspa: mass 実装](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/mass/mod.rs) | compute / storage / transient mass の計算 |
| 5 | [Kaspa KIPs](https://github.com/kaspanet/kips) | 変更の提案・動機・互換性 |
| 6 | [rusty-kaspa Releases](https://github.com/kaspanet/rusty-kaspa/releases) | 配布版・変更履歴・アップグレード |

**実装の値 > 二次解説の値**ではあるが、固定版の定義と稼働中リリース/有効な分岐も同一ではない。検証時は**コミットIDとネットワーク**を必ず記録する。

## E.3 パラメータを読むための用語

| 用語 | 意味 | 注意点 |
|---|---|---|
| BPS | Blocks per second：目標ブロック数/秒 | 観測値ではない |
| DAA score | 難易度調整に関係するスコア | 単純な「ブロック高さ」ではない |
| GHOSTDAG `k` | blue/red分類に関係する許容パラメータ | 「最大同時ブロック数」ではない |
| blue score | GHOSTDAGで得られるスコア | blue workと別 |
| blue work | 累積作業量に関係する量 | 単純なブロック数ではない |
| Finality depth | 合意ルールにおける深さ | 決済事業者の承認待ち時間とは別 |
| Pruning depth | 過去データを整理するための深さ | すべての履歴を消す設定ではない |
| Mass | トランザクションやブロックの資源コスト | バイト数と同義ではない |

## E.4 Mainnetの主要パラメータ（基準版ソース）

MAINNET_PARAMSの直接定義と、同設定が参照するCRESCENDOを分ける。変更される値はForkedParam::getへ渡すDAA scoreで選ばれる。どのブロック/親のスコアを使うかは呼出箇所を確認する。

| 取得元・フィールド | Crescendo前 | Crescendo後 | 単位・意味 |
|---|---:|---:|---|
| `target_time_per_block()` | 1000 | 100 | ms、目標1/10 BPS |
| `ghostdag_k()` | 18 | 124 | blue anticoneに関係するk |
| `max_block_parents()` | 10 | 16 | 直接の親の数 |
| `mergeset_size_limit()` | 180 | 248 | merge set制限 |
| `merge_depth()` | 3600 | 36,000 | 深さ |
| `finality_depth()` | 86,400 | 432,000 | blue scoreによる深さ |
| `pruning_depth()` | 185,798 | 1,080,000 | 深さ |
| `coinbase_maturity()` | 100 | 1000 | 検証ではDAA score差と照合 |
| `max_tx_inputs()` / `max_tx_outputs()` | 各1,000,000,000 | 各1000 | 個。他の制限も適用 |
| `max_signature_script_len()` | 1,000,000,000 | 10,000 | byte |
| `max_script_public_key_len()` | 1,000,000,000 | 10,000 | byte |
| `difficulty_window_size()` | 2641 | 661 | 前はFull窓、後はSampled窓 |
| `difficulty_sample_rate()` | 1 | 40 | サンプリング間隔のブロック単位 |

前の大きな入力/Script長上限は、巨大な取引を実際に受理できるという意味ではない。mass・Script engine・通信等の制限を別途満たす必要がある。

| MAINNET_PARAMSのフィールド | 値 | 単位・意味 |
|---|---:|---|
| `crescendo_activation` | 110,165,000 | DAA score。境界を含めて有効 |
| `timestamp_deviation_tolerance` | 132 | 秒 |
| `max_coinbase_payload_len` | 204 | byte |
| `coinbase_payload_script_public_key_max_len` | 150 | byte |
| `mass_per_tx_byte` | 1 | mass / byte |
| `mass_per_script_pub_key_byte` | 10 | mass / byte |
| `mass_per_sig_op` | 1000 | mass / SigOp |
| `max_block_mass` | 500,000 | mass。適用次元はE.9 |
| `max_block_level` | 225 | level |
| `pruning_proof_m` | 1000 | M |
| `skip_proof_of_work` | false | 真偽値 |
| `deflationary_phase_daa_score` | 15,519,600 | DAA score |
| `pre_deflationary_phase_base_subsidy` | 50,000,000,000 | sompi、現在の報酬ではない |

### E.4.1 10 BPSの意味

目標100 ms/blockは長期平均で10 blocks/s相当。任意の1秒に必ず10個生成される保証ではなく、取引の受理/確定とも別である。

### E.4.2 時間単位と深さ

秒と深さ・DAA score・blue scoreを区別する。BPSによる時間換算は設計上の目安で、時計による待ち時間の保証ではない。finality、pruning、coinbase maturityで実際に比較する値と分岐は各処理を追う（第19〜21章）。

## E.5 時間・難易度・Pruningに関する定数

[`constants.rs`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/constants.rs) に定義されている値。

| 定数 | 値 | 単位 | 解説 |
|---|---:|---|---|
| `NETWORK_DELAY_BOUND` | 5 | 秒 | ネットワーク遅延に関する推定上限 |
| `GHOSTDAG_TAIL_DELTA` | 0.01 | 確率 | 大きなAnticoneに関する設計上の確率パラメータ |
| `TIMESTAMP_DEVIATION_TOLERANCE` | 132 | 秒 | タイムスタンプの許容幅 |
| `PAST_MEDIAN_TIME_SAMPLE_INTERVAL` | 10 | 秒 | 中央時刻用サンプリング間隔 |
| `NEW_DIFFICULTY_WINDOW_DURATION` | 2641 | 秒 | 難易度計算の目標窓長 |
| `DIFFICULTY_WINDOW_SAMPLE_INTERVAL` | 4 | 秒 | 難易度サンプリング間隔 |
| `NEW_FINALITY_DURATION` | 43,200 | 秒 | 12時間 |
| `NEW_PRUNING_DURATION` | 108,000 | 秒 | 30時間 |
| `MERGE_DEPTH_DURATION` | 3600 | 秒 | 1時間 |
| `LEGACY_COINBASE_MATURITY` | 100 | 深さ | Bpsの成熟深さ導出の基準 |
| `PRUNING_PROOF_M` | 1000 | — | Pruning Proof用のパラメータ |

**注意**：`NEW_FINALITY_DURATION = 12時間` は「すべての送金で12時間待たなければならない」という意味ではない。プロトコル内部のFinalityと、アプリケーションが設定する決済確定ポリシーは別の概念である。

## E.6 `CRESCENDO` と `Bps<10>` の導出値

基準版はBlockrateParamsではなく、CRESCENDO: CrescendoParamsでTenBpsの関数を呼ぶ。以下は固定ソースの式からの計算で、公式関数を今回実行比較した結果ではない。

| 項目 | 10 BPS側の値 | 導出元 |
|---|---:|---|
| ghostdag_k | 124 | Bpsの定数表 |
| max_block_parents | 16 | k/2を10〜16に制限 |
| mergeset_size_limit | 248 | 2kを180〜512に制限 |
| past_median_time_sample_rate | 100 | BPS×10 |
| past_median_time_sampled_window_size | 27 | ceil((2×132−1)/10) |
| difficulty_sample_rate | 40 | BPS×4 |
| sampled_difficulty_window_size | 661 | ceil(2641/4) |
| merge_depth | 36,000 | BPS×3600 |
| finality_depth | 432,000 | BPS×43,200 |
| pruning_depth | 1,080,000 | 下記max式 |
| coinbase_maturity | 1000 | BPS×LEGACY_COINBASE_MATURITY |

pruning_depthは `max(BPS×108000, finality_depth + 2×merge_depth + 4×mergeset_size_limit×k + 2×k + 2)`。10 BPSの後者は627,258で、採用値は1,080,000。単なる30時間換算だけが定義ではない。

本書のDevnet既定値はCrescendoがneverなので、CRESCENDOに10 BPS値が入っていても前の設定を使う。Simnetはalways、Mainnet/Testnet-10はそれぞれ設定のactivationを確認する。

### E.6.1 サンプリング窓と実ブロック数

Crescendo後の期待DAA窓時間は `100 ms × 40 × 661 = 2,644,000 ms`、2644秒。ceilにより設計期間2641秒と同じにはならない。前は `1000 ms × 2641 = 2,641,000 ms`。期待期間と、実際に取得できた窓/経過時間は別である。

difficulty_window_size()は前後の窓構成で意味が異なる。sampled窓を構成する際のnon-DAA除外とsamplingも区別する（[第13章](../part-02-consensus/chapter-13.md)）。

## E.7 難易度調整（DAA）のパラメータ

DAAを調査する際は、以下をセットで読む。

- `difficulty_window_size`
- `min_difficulty_window_size`
- `difficulty_sample_rate()`
- `max_difficulty_target`
- ブロックヘッダのDifficulty関連フィールド
- DAA scoreとフォーク有効化条件

**よくある誤解**：目標BPSが10になったからといって、単純にPoW難易度を「以前の10倍」に固定するわけではない。実際の難易度はネットワークの計算能力や履歴に応じて調整される。

## E.8 GHOSTDAGとBlockDAGのパラメータ

GHOSTDAGの理解では、`k`だけを見るのでは不十分である。

```text
ブロック受信
   │
   ├─ 親ブロック集合 ── max_block_parents
   │
   ├─ Merge Set ───── mergeset_size_limit
   │
   ├─ Blue/Red分類 ── ghostdag_k
   │
   └─ 深さの制約 ──── merge_depth / finality_depth
```

`k`はネットワーク遅延・ブロック生成レートなどの想定と関連するが、実際の安全性は単一の数値で決まらない。論文上の仮定とコードのチェックを分けて読む。

## E.9 Mass：ブロック容量をバイトだけで判断しない

基準版にblock_mass_limits.compute/storage/transientというフィールドはない。Params::max_block_massは500,000で、body_validation_in_isolation.rsのcheck_block_massはCrescendo後に各次元を独立に集計し、それぞれこの値と比較する。

```text
compute mass   <= 500,000
storage mass   <= 500,000
transient mass <= 500,000
```

Crescendo前のこの処理はcompute massを集計する。後のstorage集計は取引のmass commitmentを使い、UTXO文脈で計算した値との整合は別の検証段階で確認する。ここだけでmass全規則を説明・検証したことにはならない。

mass_per_tx_byte/script_pub_key_byte/sig_opはcompute massの構成要素で、単純な生Txバイト数と同じではない。Storage MassはUTXOへの影響を評価する。算出式は[基準版mass実装](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/mass/mod.rs)、適用は[body検証](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/body_processor/body_validation_in_isolation.rs)を参照。

### E.9.1 検証の観点

| 検証項目 | 期待すること |
|---|---|
| Txサイズだけ増加 | 計算Massの変化を説明できる |
| 出力数増加 | Storage Massなどへの影響を説明できる |
| Script公開鍵長増加 | 上限・計算Massへの影響を確認できる |
| 複数Txを1ブロックへ追加 | 各Mass次元の上限を越えない |
| 境界値 | 上限ちょうど／1超過で結果が分かれる |

## E.10 トランザクション・Scriptの制限

E.4の入力/出力/Script長はMainnetのCrescendo前後で異なる。CRESCENDOで各入力/出力1000、署名Script/公開鍵Scriptは各10,000 byte。coinbase payload全体204 byteと内部Script公開鍵150 byteは別の制限で、通常Txの任意データ上限ではない。

この表にない値は無制限という意味ではない。個数/長さが範囲内でもmass・Script engine等で拒否され得る。

## E.11 PoWと難易度ターゲット

PoWの基本条件を概念的に表すと次のとおり。

```text
hash(block_header, nonce, ...) <= target
```

これは説明用の表現であり、Kaspa固有のPoWハッシュ計算・ヘッダ直列化・比較方法の完全な実装仕様ではない。

`max_difficulty_target` は最大ターゲットを定義する。ターゲットの値と「難易度」は逆方向の関係になるため、**ターゲットが大きい＝難易度が高い**と説明しない。

## E.12 発行量とブロック報酬

### E.12.1 通貨単位

Kaspaでは通常、**1 KAS = 100,000,000 sompi** と表現する。実装で整数単位を扱うときは、表示用の小数と内部値を分離する。

```text
50,000,000,000 sompi = 500 KAS
```

これは上記の`pre_deflationary_phase_base_subsidy`の換算例であり、**現在のブロック報酬を意味しない**。

### E.12.2 報酬は固定値ではない

現在の補助金額を求めるには、少なくとも以下が必要。

1. 対象ネットワーク
2. 対象ブロックのDAA score
3. 発行スケジュールの実装
4. フォーク前後のBPS履歴の扱い
5. 実際のCoinbaseトランザクションと手数料

基準版のCoinbaseManagerは前後のBPSとCrescendo activation DAA scoreを用いて補助金表と月を扱う。基準版params.rsにbps_history()という関数はない（[第19章](../part-03-transactions/chapter-19.md)）。**現在報酬を単一の定数で掲載しない**。

## E.13 Crescendo：フォーク前後の比較

| 項目 | ソース上のフォーク前 | ソース上のフォーク後 |
|---|---:|---:|
| 目標時間 | 1000 ms | 100 ms |
| 目標BPS | 1 | 10 |
| 有効化 | — | Mainnet DAA score 110,165,000 |

この表は`MAINNET_PARAMS`の定義に基づく。フォークによって影響を受ける**他のパラメータまで不変だったと断定するものではない**。

## E.14 Mainnet・Testnet・Devnet・Simnetの違い

| ネットワーク | 主目的 | 例：公開ソースの特徴 |
|---|---|---|
| Mainnet | 実資産の移転 | `skip_proof_of_work = false` |
| Testnet | 公開テスト | ネットワークサフィックスがある場合がある（例：Testnet 10） |
| Devnet | 開発・統合テスト | 独立したGenesisと設定 |
| Simnet | ローカルシミュレーション | `skip_proof_of_work = true` が定義されている |

**Testnetは1つとは限らない**。Testnet番号、Genesis、対応ノードバージョンを指定しないまま「テストネットでは…」と書かない。

### E.14.1 DevnetはMainnetのコピーではない

Devnetは独立した実験環境であり、次の違いがあり得る。

- Genesis
- 難易度とハッシュレート
- フォーク有効化条件
- Coinbase報酬
- ネットワーク参加ノード
- ブロック履歴

したがってDevnetで動いた処理が、そのままMainnetで実行可能とは限らない。

## E.15 コンセンサスパラメータとRPC設定の違い

```text
┌──────────────────────────────┐
│ コンセンサス                 │
│ 目標BPS / GHOSTDAG k / Mass  │
│ 同じネットワークの合意に影響 │
└──────────────────────────────┘
               ≠
┌──────────────────────────────┐
│ ノードの運用                 │
│ RPC待受 / ログ / 保存先      │
│ ローカルの動作・公開範囲     │
└──────────────────────────────┘
```

RPCポートやP2Pポートは接続上重要だが、ブロックの有効性判定そのものではない。ポート番号はノード実装・ネットワーク・起動オプションで変わるため、固定表だけで判断しない。

## E.16 パラメータ上書きは実験環境だけで行う

`rusty-kaspa` にはパラメータ上書きの仕組みがある。ただし、**合意に影響する値をMainnetで勝手に変えると、他のノードと整合しなくなる**。

実験では以下を徹底する。

- 基準版の `--override-dag-params-file` はDevnet用。対応する項目/制限を確認し、隔離したDevnetで使う。Simnetでも同じCLI上書きが可能と仮定しない。
- 変更前の値をJSON等で保存する。
- Genesisと既存データベースの互換性を確認する。
- ノードA/Bで設定が一致しているか検証する。
- 実験結果にソースコミット・設定ファイル・ログを添付する。

基準版にdocs/override-params.mdはない。入口は[kaspadの引数定義](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/kaspad/src/args.rs)。これは上書き機能の実行試験ではない。

## E.17 ローカルで一次資料を固定する

本書の外の調査用ディレクトリで固定ソースを取得する読者用手順。今回は既存の固定archiveを参照し、新しいcloneやGit HEAD取得はしていない。

```bash
git clone --branch v1.0.1 --depth 1 https://github.com/kaspanet/rusty-kaspa.git rusty-kaspa-v1.0.1
cd rusty-kaspa-v1.0.1

git rev-parse HEAD
git status --short

grep -n 'pub const MAINNET_PARAMS' consensus/core/src/config/params.rs
grep -n 'pub const TESTNET_PARAMS' consensus/core/src/config/params.rs
grep -n 'pub const DEVNET_PARAMS' consensus/core/src/config/params.rs
grep -n 'pub const SIMNET_PARAMS' consensus/core/src/config/params.rs
```

HEADが `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` と一致することを確認する。既存の変更済み作業コピーをresetして合わせない。

## E.18 コード上の参照箇所を調べる

```bash
rg 'ghostdag_k|target_time_per_block|pruning_depth' consensus/
rg 'max_tx_inputs|max_tx_outputs|max_block_mass' consensus/
rg 'deflationary_phase_daa_score|base_subsidy' consensus/
```

`rg` はripgrep。未導入なら `grep -R -n` でもよい。

### E.18.1 値だけでなく利用箇所を追う

例えば `coinbase_maturity` を見つけたら、次の順番で調べる。

1. **定義**：どこで値を設定するか。
2. **導出**：BPSなどから計算していないか。
3. **検証**：どの関数でルールを適用するか。
4. **テスト**：境界値テストがあるか。
5. **履歴**：過去のフォークで変わったか。

## E.19 Rustで単位を間違えないための最小例

以下はKaspaのコードではなく、**単位変換の教育用プログラム**。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockRate {
    pub target_ms: u64,
}

impl BlockRate {
    pub fn bps(self) -> Option<u64> {
        if self.target_ms == 0 || 1000 % self.target_ms != 0 {
            return None;
        }
        Some(1000 / self.target_ms)
    }

    pub fn blocks_for_seconds(self, seconds: u64) -> Option<u64> {
        seconds.checked_mul(self.bps()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ten_bps() {
        let rate = BlockRate { target_ms: 100 };
        assert_eq!(rate.bps(), Some(10));
        assert_eq!(rate.blocks_for_seconds(100), Some(1000));
    }

    #[test]
    fn one_bps() {
        assert_eq!(BlockRate { target_ms: 1000 }.bps(), Some(1));
    }

    #[test]
    fn reject_invalid_rate() {
        assert_eq!(BlockRate { target_ms: 0 }.bps(), None);
        assert_eq!(BlockRate { target_ms: 300 }.bps(), None);
    }

    #[test]
    fn avoid_overflow() {
        let rate = BlockRate { target_ms: 100 };
        assert_eq!(rate.blocks_for_seconds(u64::MAX), None);
    }
}
```

本書の外の作業ディレクトリで、Rust 1.90.0を用意して次の手順で実行する。

```bash
cargo +1.90.0 new --lib --edition 2021 protocol-units-lab
cd protocol-units-lab
# src/lib.rs を上記コード全体に置き換える
cargo +1.90.0 test --offline
cargo +1.90.0 fmt --all -- --check
cargo +1.90.0 clippy --offline --all-targets -- -D warnings
```

`cargo test`では4テストが成功することを確認する。外部crateは不要で、`main`関数ではなく単体テストを実行する例である。

既存Rust 1.90.0 / edition 2021で4テスト・書式/Clippyを確認済み。新規toolchain導入はしていない。なお、この関数は整数BPSに限定した教材であり、任意のブロック時間を扱う汎用モデルではない。

## E.20 実装で守るべき不変条件

| 不変条件 | 理由 |
|---|---|
| ネットワークを明示する | MainnetとTestnetの混同を防ぐ |
| 単位を明示する | 秒・ミリ秒・ブロック数の混同を防ぐ |
| DAA scoreをブロック高さとして扱わない | 意味が異なる |
| Massを単純なバイト上限に還元しない | 多次元の制限がある |
| 過去フォークの値を現在値と混同しない | 履歴の検証に必要 |
| 通貨を内部では整数で扱う | 丸め誤差を避ける |
| 上限を検証するときは境界値を試す | `<=` と `<` の差が重大 |

## E.21 設計：パラメータをアプリケーションに埋め込まない

### E.21.1 避けたい例

```rust
// 説明用：運用中のネットワーク変更に弱い
const BLOCKS_PER_SECOND: u64 = 10;
const PAYMENT_CONFIRMED_AFTER_BLOCKS: u64 = 10;
```

BPSを10と仮定すること自体より、**プロトコル定数と事業上の承認ルールを同一視すること**が問題。

### E.21.2 推奨する分離

```text
Kaspa Node / RPC
      │
      ▼
Network Observation
      │
      ▼
Protocol Adapter ─── version / network / capabilities
      │
      ▼
Domain Policy ─────── 入金判定・再編成時の扱い
      │
      ▼
Application
```

アプリケーションの確認ポリシーは、ノードのコンセンサス設定とは独立して管理する。

## E.22 パラメータ変更のレビュー手順

1. **何が変わるか**：旧値と新値、単位を確認。
2. **なぜ変わるか**：KIP、設計議論、ベンチマークを確認。
3. **どこに影響するか**：合意、RPC、Wallet、Indexer、Miningを分類。
4. **いつ変わるか**：フォーク条件と対象ネットワークを確認。
5. **どのリリースに入ったか**：タグ・コミットを確認。
6. **どう検証するか**：境界値テスト、統合テスト、Devnet実験を設計。
7. **どう公開するか**：互換性と運用上の注意を記録。

## E.23 実験：BPS変更による換算の違い

| 基準時間 | 1 BPS相当 | 10 BPS相当 |
|---|---:|---:|
| 10秒 | 10 | 100 |
| 100秒 | 100 | 1000 |
| 1時間 | 3600 | 36,000 |
| 12時間 | 43,200 | 432,000 |
| 30時間 | 108,000 | 1,080,000 |

これは**単純な時間×BPS換算表**である。`finality_depth`や`pruning_depth`の実装値をすべてこの式だけで導けると断定しない。正式な計算は `Bps::<BPS>` の定義を参照。

## E.24 実験：境界値のテスト計画

| ケース | 入力 | 期待する検証 |
|---|---|---|
| 入力数 | 999 / 1000 / 1001 | 上限付近の扱い |
| 出力数 | 999 / 1000 / 1001 | 上限付近の扱い |
| Script長 | 9999 / 10000 / 10001 | Script長制限 |
| 計算Mass | 上限−1 / 上限 / 上限＋1 | Mass上限 |
| ブロック時間 | 100 ms / 1000 ms | 単位変換 |
| フォーク境界 | 有効化前 / 境界 / 後 | 分岐条件 |

実際には他の制約が先に失敗することがある。**単一ルールのテスト**と**トランザクション全体の妥当性テスト**を分ける。

## E.25 記録用テンプレート

```markdown
# Kaspa Protocol Parameter Verification

- 確認日（UTC）：
- ネットワーク：Mainnet / Testnet-XX / Devnet / Simnet
- ノード実装：rusty-kaspa
- バージョン／タグ：
- コミットSHA：
- 参照ファイル：
- 変更対象パラメータ：
- 値：
- 単位：
- 導出式：
- フォーク条件：
- 検証方法：コード確認 / テスト / 実機RPC
- テスト結果：
- 注意事項：
```

## E.26 よくある誤解

**Q. Kaspaは常に10ブロック/秒を出す？**  
A. 10 BPSは目標値であり、短時間の実測値とは異なる。

**Q. GHOSTDAGの `k` は親ブロック数の上限？**  
A. 違う。`ghostdag_k` と `max_block_parents` は別フィールド。

**Q. Pruningすると過去のトランザクションが無効になる？**\
A. 違う。Pruningはデータ保持と検証の仕組みに関わる。トランザクションの有効性を消す操作ではない。

**Q. Finalityが12時間なら決済も12時間待つ？**  
A. 同義ではない。プロトコル内部のFinalityと決済アプリケーションのリスク管理を分ける。

**Q. Testnetで動けばMainnetでも動く？**  
A. 断定できない。ネットワーク・フォーク・Mass・利用可能機能を確認する。

**Q. `master` の値が現行Mainnetの値？**  
A. 必ずしもそうではない。実行中ノードのリリースとフォーク状態を確認する。

## E.27 演習

### 演習1：値の出典を追う

`MAINNET_PARAMS.crescendo` から `ghostdag_k` の定義までたどり、数値・単位・導出元を記録する。

### 演習2：Massの上限を調べる

`max_block_mass` がどの検証処理で使用されるか検索し、上限ちょうどと上限超過のテストを見つける。

### 演習3：フォーク前後を比較する

Crescendo有効化前後のBPS履歴と報酬計算の関係をコードで確認する。

### 演習4：Devnetで設定変更を試す

隔離されたDevnetで、設定が一致する場合と異なる場合を比較する。**Mainnetでは実施しない**。

1. E.16の引数定義から、基準版の `--override-dag-params-file` が受け付ける項目・JSON形式・制限を調べる。E.4の全項目を上書きできるとは仮定しない。
2. [第28章28.8「2ノード構成とピア接続」](../part-04-development/chapter-28.md#288-2ノード構成とピア接続)を参考に、同じ版・同じ設定の2ノードで接続とブロック同期を確認し、比較用の結果を記録する。
3. 対応する項目を一つ選び、どの検証処理に影響するかを先に調べる。両ノードへ同じ変更を適用した構成で起動・接続を確認してから、一方だけ値を変えた構成と比較する。各構成には専用のデータディレクトリを用意し、既存DBの再利用可否を推測で判断しない。
4. 設定ファイル・コミット・ピア状態・同期状況・検証エラーを記録する。差が出なければ、変更したルールが適用されるフォーク条件と、差を生むブロックが実際に生成されたかを調べる。接続できたことだけで合意ルールの互換性を結論づけない。

これは追加実験の課題であり、本付録で実行確認済みの手順ではない。結果はE.25のテンプレートに残す。

### 演習5：変更影響表を作る

`max_tx_outputs` が変更されたと仮定し、Wallet、Indexer、RPCクライアント、テストにどのような変更が必要か列挙する。

## E.28 検証を広げる際のチェックリスト

今回確認した主要定義/算術/教材と、全境界値・実ノードの実行は分ける。以下をすべて完了したという意味ではない。

- [ ] 全数値に対象ネットワークを記載した。
- [ ] 単位（ms、秒、ブロック数、DAA score、sompi）を明記した。
- [ ] `master` と配布リリースを区別した。
- [ ] 検証済みコミットSHAを記録した。
- [ ] 10 BPSへの移行条件を一次資料で確認した。
- [ ] `CrescendoParams` / `Bps` の導出値を実装と照合した。
- [ ] Massの多次元制限を反映した。
- [ ] 報酬の履歴と現在値を混同していない。
- [ ] Testnetのサフィックスを明記した。
- [ ] フォークの変更履歴とKIPを突き合わせた。
- [ ] コード例を実行し、結果を記録した。
- [ ] ノードのRPC出力と整合性を確認した。

## E.29 まとめ

Kaspaのプロトコルパラメータを扱ううえで重要なのは、**数字そのものより、その数字が適用される条件を理解すること**である。

```text
パラメータ名
    │
    ▼
単位・導出式
    │
    ▼
ネットワーク
    │
    ▼
フォーク条件
    │
    ▼
実装コミット
    │
    ▼
テスト・実機確認
```

数値を見つけたら、**「どのネットワークの、どの実装の、いつ有効な、何を制限する値か」**を必ず問う。この習慣は、Kaspaへのコントリビュートだけでなく、ブロックチェーン以外の分散システム開発にも役立つ。

## E.30 参考資料

- [kaspanet/rusty-kaspa — Consensus Parameters](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/params.rs)
- [kaspanet/rusty-kaspa — Consensus Constants](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/constants.rs)
- [kaspanet/rusty-kaspa — BPS derivation](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/bps.rs)
- [kaspanet/rusty-kaspa — Mass Calculation](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/mass/mod.rs)
- [kaspanet/kips — Kaspa Improvement Proposals](https://github.com/kaspanet/kips)
- [kaspanet/rusty-kaspa — Releases](https://github.com/kaspanet/rusty-kaspa/releases)
- [Kaspa Wiki](https://wiki.kaspa.org/)

---

**次の付録：** [Appendix F — 論文・KIP・実装対応表](appendix-f.md)
