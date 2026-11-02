# Finality pointの実行時境界比較

実施日：2026-11-02（日本時間）。基準版v1.0.1 / `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。macOS ARM64、Rust 1.90.0、Python 3.11。

## 方法と固定したもの

公式ソースアーカイブのSHA-256は `4585123204609640af4f7482ea8851ced912c2bfae5f36955c808c14210fbe2a`。公式Cargo.lockを変更せず、リポジトリ外の展開先へexampleを追加した。既存Cargoキャッシュを使い、2並列・offlineで実行。初回の依存ビルドは約4分、展開先/targetの合計は約3.4GB。公式ライブラリにはRust 1.90でlifetime表記の警告が出たが、改変していない。

対象は [`BlockDepthManager::calc_finality_point`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/block_depth.rs)。合成selected parent/blue score/DAA scoreを与え、公式MemoryGhostdagStore、公式MemoryReachabilityStore、公式 `inquirer::add_block` と `MTReachabilityService` を利用した。HeaderStoreとDepthStoreの必要な読み書きだけをハーネスのメモリ上に用意した。

別のPythonモデルは親ポインタから祖先を列挙し、blue scoreの条件を満たす点と以前の点から期待値を求める。公式のinterval・forward iteratorは使わない。連続計算では公式側もモデル側も、それぞれ自分が返した点を次の計算へ渡す。単発22ケースは、入力へ手で付けた期待IDとも照合する。

[再実行手順](finality/README.md)、[入力](finality/vectors.json)、[公開結果と識別値](finality/observed-2026-11-02.json)。公式関数・Reachability/Params・Cargo.lock、ハーネス・モデル・入力・runnerのSHA-256を結果JSONに保存した。

## 結果

25ケース、計112回の公式関数呼び出しがモデルと一致した。

| カバーした分岐・境界 | 件数 |
|---|---:|
| selected parentがORIGIN | 1 |
| blue scoreがdepth未満でGenesisを返す | 12 |
| Pruning pointとの距離不足でORIGIN | 1 |
| Pruning pointがchain上にないためORIGIN | 1 |
| chain上で点を選択/保持 | 97 |
| 保存されたORIGINからPruning pointへフォールバック（上の97の内数） | 3 |
| 以前の点が閾値以上で保持（上の97の内数） | 10 |
| chainに閾値と同じscoreの点がある（上の97の内数） | 65 |
| 選択親DAAがactivation直前 / 一致 / 直後 | 5 / 4 / 5 |
| 後退しないことを確認した連続計算列 | 3 |

depthは3/7/11などの合成値とDevnetのprior値86400を使った。Devnet相当の値の試験でもヘッダー/scoreは合成で、長時間採掘を行っていない。

Pruning pointがoff-chainのケースでは、公式Reachabilityで「DAG祖先=true、chain祖先=false」を先にassertした。単に無関係な枝を与えた例とは分け、DAG祖先であることだけではchain条件を満たさないことを確認した。

## 本文へ反映した境界

`required_blue_score = candidate.blue_score - depth` に対し、探索はscoreが **以上（>=）** の点で止まる。以前の点が閾値より手前なら、返るのは閾値未満のchain上の点。score 0/2/5/9/14のchain、候補score 20、depth 11、以前の点Genesisでは、閾値9と一致する点へは進まずscore 5を返す。

以前の点が閾値以上でも後退しない。depth 3→7、選択親DAA 10でactivationする合成列では、候補score 10時点のfinality point score 6を、候補score 11〜14でも保持し、score 15で点がscore 7へ進んだ。depth減少7→3と、scoreに飛びのある列も含め、連続3列で後退しなかった。

第20章20.9.1へこの具体例と範囲を追記し、進捗表/ROADMAPの「公式Finality実行比較が未実施」を関数単位の部分成功へ更新した。

## 残る範囲

これはメタデータを明示した関数検査で、任意のDAGについての正しさの証明ではない。GHOSTDAGによるblue score/選択親の導出、PoW・ヘッダー検証、実際のCrescendo/Mainnet有効化、VirtualProcessorのsink候補拒否・状態遷移、Pruning point一覧検査、Pruning Proof/UTXO転送・Pruning後IBD、DB永続化、攻撃者モデルの安全性証明は未検証。C20全体をVerifiedにしない。

鍵・シード・ノード起動・採掘・送金は今回不要。生のビルドログ、固定ソースのコピー、target、一時DBは公開物へ含めない。CIの実行対象は変更しない。

## 収録時の検査

結果JSONのハーネス/モデル/runner/入力のハッシュが現在のファイルと一致し、入力生成スクリプトの再実行でvectors.jsonが同一になることを確認した。実測の112出力を1件ずつ意図的に誤ったIDへ置き換える負例は、すべてモデル比較で拒否された。rustfmt --check、mdBook build、既存2教材の本文/ソース同期、git diff --checkが成功した。
