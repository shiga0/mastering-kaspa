# Appendix E — プロトコルパラメータ一覧

> **Mastering Kaspa / 付録E**  
> 原稿ステータス：**v0.1（技術レビュー前）**  
> 確認基準：2026年10月30日に閲覧した `rusty-kaspa` の公開 `master` ブランチ。**特定リリースの実行結果ではない**。出版時にはコミットハッシュと実機の値で再検証すること。

## E.1 この付録の目的

Kaspaのブロック生成速度、GHOSTDAG、難易度調整、UTXO、ブロック報酬、Pruningなどを理解する際、「パラメータ名」「単位」「どのネットワークの値か」「どのバージョンで有効か」を混同すると、実装を誤読する。

この付録は、**固定値を暗記するためではなく、仕様を追跡して検証するための索引**である。以下を区別する。

1. **コンセンサスパラメータ**：一致しなければノード間で合意できない可能性がある値。
2. **ネットワーク設定**：P2Pポート、DNSシーダーなど。合意ルールとは別。
3. **ノード運用設定**：ログ、RPC待受、データ保存先など。ローカルで変更できる設定。
4. **実測値**：観測BPS、手数料、ハッシュレートなど。プロトコル定数ではない。
5. **フォーク依存値**：有効化前後で意味・数値が変わる値。

> **重要**：`master` は将来の変更を含み得る。以下の「ソース上の値」は、現在稼働中のメインネットにすべて適用済みだという意味ではない。

## E.2 一次資料と確認の優先順位

| 優先 | 資料 | 確認するもの |
|---|---|---|
| 1 | [rusty-kaspa: `params.rs`](https://github.com/kaspanet/rusty-kaspa/blob/master/consensus/core/src/config/params.rs) | `MAINNET_PARAMS`、`TESTNET_PARAMS`、`DEVNET_PARAMS`、`BlockrateParams` |
| 2 | [rusty-kaspa: `constants.rs`](https://github.com/kaspanet/rusty-kaspa/blob/master/consensus/core/src/config/constants.rs) | 時間・サンプリング・セキュリティ関連定数 |
| 3 | [rusty-kaspa: パラメータ上書き文書](https://github.com/kaspanet/rusty-kaspa/blob/master/docs/override-params.md) | 実験ネットワーク用の設定項目 |
| 4 | [rusty-kaspa: mass 実装](https://github.com/kaspanet/rusty-kaspa/blob/master/consensus/core/src/mass/mod.rs) | compute / storage / transient mass の計算 |
| 5 | [Kaspa KIPs](https://github.com/kaspanet/kips) | 変更の提案・動機・互換性 |
| 6 | [rusty-kaspa Releases](https://github.com/kaspanet/rusty-kaspa/releases) | 配布版・変更履歴・アップグレード |

**実装の値 > 二次解説の値**ではあるが、実装の `master` と稼働中リリースも同一ではない。検証時は**コミットIDとネットワーク**を必ず記録する。

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

## E.4 Mainnetの主要パラメータ（ソース確認値）

次の値は上記 `params.rs` の **`MAINNET_PARAMS` 定義**から読めるもの。**2026年10月30日時点の公開 `master` に関する記述**であり、稼働リリースとの一致は別途確認する。

| 分類 | Rustのフィールド | ソース上の値 | 単位・意味 | 出典 |
|---|---|---:|---|---|
| ネットワーク | `net` | Mainnet | ネットワーク識別 | `MAINNET_PARAMS` |
| 目標ブロック時間 | `blockrate.target_time_per_block` | **100** | ms、10 BPS相当 | `BlockrateParams::new::<10>()` |
| Crescendo前の目標時間 | `pre_crescendo_target_time_per_block` | **1000** | ms、1 BPS相当 | `MAINNET_PARAMS` |
| Crescendo有効化条件 | `crescendo_activation` | **110,165,000** | DAA score | `MAINNET_PARAMS` |
| 時刻許容幅 | `timestamp_deviation_tolerance` | **132** | 秒 | `constants.rs` |
| Coinbaseペイロード上限 | `max_coinbase_payload_len` | **204** | byte | `MAINNET_PARAMS` |
| Coinbase内Script公開鍵上限 | `coinbase_payload_script_public_key_max_len` | **150** | byte | `MAINNET_PARAMS` |
| Tx入力数上限 | `max_tx_inputs` | **1000** | 個 | `MAINNET_PARAMS` |
| Tx出力数上限 | `max_tx_outputs` | **1000** | 個 | `MAINNET_PARAMS` |
| Script公開鍵長上限 | `max_script_public_key_len` | **10,000** | byte | `MAINNET_PARAMS` |
| 1バイトあたりの計算Mass | `mass_per_tx_byte` | **1** | mass / byte | `MAINNET_PARAMS` |
| Script公開鍵1バイトあたりの計算Mass | `mass_per_script_pub_key_byte` | **10** | mass / byte | `MAINNET_PARAMS` |
| SigOpあたりの計算Mass | `mass_per_sig_op` | **1000** | mass / SigOp | `MAINNET_PARAMS` |
| ブロック計算Mass上限 | `block_mass_limits.compute` | **500,000** | mass | `MAINNET_PARAMS` |
| ブロックStorage Mass上限 | `block_mass_limits.storage` | **500,000** | mass | `MAINNET_PARAMS` |
| ブロックTransient Mass上限 | `block_mass_limits.transient` | **1,000,000** | mass | `MAINNET_PARAMS` |
| 最大ブロックレベル | `max_block_level` | **225** | level | `MAINNET_PARAMS` |
| Pruning Proofパラメータ | `pruning_proof_m` | **1000** | M | `MAINNET_PARAMS` |
| PoW省略 | `skip_proof_of_work` | **false** | 真偽値 | `MAINNET_PARAMS` |
| デフレ期開始DAA score | `deflationary_phase_daa_score` | **15,519,600** | DAA score | `MAINNET_PARAMS` |
| デフレ期前基本報酬 | `pre_deflationary_phase_base_subsidy` | **50,000,000,000** | sompi | `MAINNET_PARAMS` |

**未転記の値**：`max_signature_script_len`、`storage_mass_parameter`、`max_difficulty_target`、`blockrate.ghostdag_k`、`max_block_parents`、`mergeset_size_limit`などは、参照先の別定義やジェネリック計算を要する。根拠なしに数値を埋めず、E.6の手順で計算元を確認する。

### E.4.1 10 BPSの意味

```text
目標時間 = 100 ms / block
目標BPS  = 1000 ms / 100 ms = 10 blocks/s
```

これは**長期平均の目標**であって、任意の1秒に必ず10個のブロックが生成される保証ではない。また、BlockDAGでは「ブロック数」と「トランザクションが最終的に採用される順序」を同一視しない。

### E.4.2 時間単位とブロック単位

`finality_depth`、`pruning_depth`、`coinbase_maturity`は `BlockrateParams` で目標BPSを使って算出される。**秒単位の定数をそのままブロック数として記載しないこと**。

## E.5 時間・難易度・Pruningに関する定数

[`constants.rs`](https://github.com/kaspanet/rusty-kaspa/blob/master/consensus/core/src/config/constants.rs) に定義されている値。

| 定数 | 値 | 単位 | 解説 |
|---|---:|---|---|
| `NETWORK_DELAY_BOUND` | 5 | 秒 | ネットワーク遅延に関する推定上限 |
| `GHOSTDAG_TAIL_DELTA` | 0.01 | 確率 | 大きなAnticoneに関する設計上の確率パラメータ |
| `TIMESTAMP_DEVIATION_TOLERANCE` | 132 | 秒 | タイムスタンプの許容幅 |
| `PAST_MEDIAN_TIME_SAMPLE_INTERVAL` | 10 | 秒 | 中央時刻用サンプリング間隔 |
| `DIFFICULTY_WINDOW_DURATION` | 2641 | 秒 | 難易度計算の目標窓長 |
| `DIFFICULTY_WINDOW_SAMPLE_INTERVAL` | 4 | 秒 | 難易度サンプリング間隔 |
| `FINALITY_DURATION` | 43,200 | 秒 | 12時間 |
| `PRUNING_DURATION` | 108,000 | 秒 | 30時間 |
| `MERGE_DEPTH_DURATION` | 3600 | 秒 | 1時間 |
| `COINBASE_MATURITY_SECONDS` | 100 | 秒 | Coinbase成熟の基準時間 |
| `PRUNING_PROOF_M` | 1000 | — | Pruning Proof用のパラメータ |

**注意**：`FINALITY_DURATION = 12時間` は「すべての送金で12時間待たなければならない」という意味ではない。プロトコル内部のFinalityと、アプリケーションが設定する決済確定ポリシーは別の概念である。

## E.6 `BlockrateParams` から導出される値

`BlockrateParams::new::<10>()` は、ブロック生成レートに依存するパラメータを組み立てる。主なフィールドは以下。

| フィールド | 役割 | 直接確認すべき場所 |
|---|---|---|
| `ghostdag_k` | GHOSTDAGの分類パラメータ | `Bps::<BPS>::ghostdag_k()` |
| `past_median_time_sample_rate` | Median Timeのサンプリング率 | `Bps::<BPS>::past_median_time_sample_rate()` |
| `difficulty_sample_rate` | DAAのサンプリング率 | `Bps::<BPS>::difficulty_adjustment_sample_rate()` |
| `max_block_parents` | ブロックの親数上限 | `Bps::<BPS>::max_block_parents()` |
| `mergeset_size_limit` | Merge Setのサイズ制限 | `Bps::<BPS>::mergeset_size_limit()` |
| `merge_depth` | Merge Depth | `Bps::<BPS>::merge_depth_bound()` |
| `finality_depth` | Finality Depth | `Bps::<BPS>::finality_depth()` |
| `pruning_depth` | Pruning Depth | `Bps::<BPS>::pruning_depth()` |
| `coinbase_maturity` | Coinbase成熟深さ | `Bps::<BPS>::coinbase_maturity()` |

例：単純な時間換算として、100秒 × 10 BPS = 1000ブロック相当。ただし**実際の `coinbase_maturity` の計算式は実装を確認**し、単純換算を公式値と取り違えない。

### E.6.1 サンプリング窓と実ブロック数

`difficulty_window_size` は「**サンプリング済みの窓の大きさ**」。10 BPSで生成されるすべてのブロック数と同一ではない。

`params.rs` には次のような関係が実装されている。

```rust
// 原典の考え方を簡略化した説明用の式
let expected_duration_ms =
    target_time_per_block_ms * difficulty_sample_rate * difficulty_window_size;
```

ここで各変数の単位を確認する。単位を無視してBPSを乗算すると、10倍・100倍の誤りが起こる。

## E.7 難易度調整（DAA）のパラメータ

DAAを調査する際は、以下をセットで読む。

- `difficulty_window_size`
- `min_difficulty_window_size`
- `blockrate.difficulty_sample_rate`
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

Kaspaのリソース制限には、複数のMass次元がある。公開ソースの `MAINNET_PARAMS` には次の上限が定義されている。

```text
compute mass   <=   500,000
storage mass   <=   500,000
transient mass <= 1,000,000
```

**注意**：これは`master`にある定義の転記であり、古い資料の「1ブロックのMassは○○」と機械的に比較できない。ルールやフォークが異なる可能性がある。

`mass_per_tx_byte`、`mass_per_script_pub_key_byte`、`mass_per_sig_op` は計算Massの構成要素だが、**トランザクションMassは単なるTxバイト数ではない**。Storage MassもUTXO集合への影響を評価する。詳しい算出経路は [`consensus/core/src/mass/mod.rs`](https://github.com/kaspanet/rusty-kaspa/blob/master/consensus/core/src/mass/mod.rs) を参照。

### E.9.1 検証の観点

| 検証項目 | 期待すること |
|---|---|
| Txサイズだけ増加 | 計算Massの変化を説明できる |
| 出力数増加 | Storage Massなどへの影響を説明できる |
| Script公開鍵長増加 | 上限・計算Massへの影響を確認できる |
| 複数Txを1ブロックへ追加 | 各Mass次元の上限を越えない |
| 境界値 | 上限ちょうど／1超過で結果が分かれる |

## E.10 トランザクション・Scriptの制限

公開 `MAINNET_PARAMS` から直接読めるもの：

| 項目 | 値 | 注意 |
|---|---:|---|
| 最大入力数 | 1000 | 別のMass制限が先に効く場合がある |
| 最大出力数 | 1000 | 同上 |
| Script公開鍵長 | 10,000 byte | 上限内でもMass制限がある |
| Coinbaseペイロード長 | 204 byte | 通常Txの任意データ上限ではない |
| Coinbaseペイロード内Script公開鍵長 | 150 byte | 通常のScript公開鍵上限と別 |

`max_signature_script_len` の数値は、参照先の定数定義を確認してから記載する。**この表にない値は無制限という意味ではない**。

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

`params.rs` の `bps_history()` は、Crescendo前後のBPS履歴を報酬月の計算に使うと説明している。**現在報酬を単一の定数で掲載しない**。

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

- Simnetまたは隔離されたDevnetを使用する。
- 変更前の値をJSON等で保存する。
- Genesisと既存データベースの互換性を確認する。
- ノードA/Bで設定が一致しているか検証する。
- 実験結果にソースコミット・設定ファイル・ログを添付する。

公式文書：[Parameter Overrides](https://github.com/kaspanet/rusty-kaspa/blob/master/docs/override-params.md)。

## E.17 ローカルで一次資料を固定する

以下は**ユーザー自身のローカル環境で実行するコマンド例**。本書作成時点での実行検証はしていない。

```bash
git clone https://github.com/kaspanet/rusty-kaspa.git
cd rusty-kaspa

git rev-parse HEAD
git status --short

grep -n 'pub const MAINNET_PARAMS' consensus/core/src/config/params.rs
grep -n 'pub const TESTNET_PARAMS' consensus/core/src/config/params.rs
grep -n 'pub const DEVNET_PARAMS' consensus/core/src/config/params.rs
grep -n 'pub const SIMNET_PARAMS' consensus/core/src/config/params.rs
```

調査対象を再現可能にするには、`master` のままではなく**検証済みタグまたはコミットに固定**する。

```bash
# 実際に存在するタグまたはコミットを指定する
# git checkout <verified-tag-or-commit>

git rev-parse HEAD
```

## E.18 コード上の参照箇所を調べる

```bash
rg 'ghostdag_k|target_time_per_block|pruning_depth' consensus/
rg 'max_tx_inputs|max_tx_outputs|block_mass_limits' consensus/
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
struct BlockRate {
    target_ms: u64,
}

impl BlockRate {
    fn bps(self) -> Option<u64> {
        if self.target_ms == 0 || 1000 % self.target_ms != 0 {
            return None;
        }
        Some(1000 / self.target_ms)
    }

    fn blocks_for_seconds(self, seconds: u64) -> Option<u64> {
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

`rustc --test example.rs -o example-tests && ./example-tests` で実行できる形式を意図しているが、**本書作成時点では実行未検証**。なお、この関数は整数BPSに限定した教材であり、任意のブロック時間を扱う汎用モデルではない。

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

`MAINNET_PARAMS.blockrate` から `ghostdag_k` の定義までたどり、数値・単位・導出元を記録する。

### 演習2：Massの上限を調べる

`block_mass_limits` がどの検証処理で使用されるか検索し、上限ちょうどと上限超過のテストを見つける。

### 演習3：フォーク前後を比較する

Crescendo有効化前後のBPS履歴と報酬計算の関係をコードで確認する。

### 演習4：Devnetで設定変更を試す

隔離されたDevnetでパラメータを変更し、2ノードの設定不一致が起こす問題を観察する。**Mainnetでは実施しない**。

### 演習5：変更影響表を作る

`max_tx_outputs` が変更されたと仮定し、Wallet、Indexer、RPCクライアント、テストにどのような変更が必要か列挙する。

## E.28 出版前の技術レビュー・チェックリスト

- [ ] 全数値に対象ネットワークを記載した。
- [ ] 単位（ms、秒、ブロック数、DAA score、sompi）を明記した。
- [ ] `master` と配布リリースを区別した。
- [ ] 検証済みコミットSHAを記録した。
- [ ] 10 BPSへの移行条件を一次資料で確認した。
- [ ] `BlockrateParams` の導出値を実装と照合した。
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

- [kaspanet/rusty-kaspa — Consensus Parameters](https://github.com/kaspanet/rusty-kaspa/blob/master/consensus/core/src/config/params.rs)
- [kaspanet/rusty-kaspa — Consensus Constants](https://github.com/kaspanet/rusty-kaspa/blob/master/consensus/core/src/config/constants.rs)
- [kaspanet/rusty-kaspa — Parameter Overrides](https://github.com/kaspanet/rusty-kaspa/blob/master/docs/override-params.md)
- [kaspanet/rusty-kaspa — Mass Calculation](https://github.com/kaspanet/rusty-kaspa/blob/master/consensus/core/src/mass/mod.rs)
- [kaspanet/kips — Kaspa Improvement Proposals](https://github.com/kaspanet/kips)
- [kaspanet/rusty-kaspa — Releases](https://github.com/kaspanet/rusty-kaspa/releases)
- [Kaspa Wiki](https://wiki.kaspa.org/)

---

**次の付録：Appendix F — 論文・KIP・実装対応表**
