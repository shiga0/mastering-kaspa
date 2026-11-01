# SUMMARY順レビュー 14：第10章「KaspaノードとP2Pネットワーク」

確認日：2026-11-01。対象は `src/part-02-consensus/chapter-10.md`。前日に開始したレビューを継続し、基準版の主要な接続・伝播経路と教材を確認した。

## 修正内容

- 基準版v1.0.1と完全SHAを冒頭・ソース探索へ固定。第1章で取得するリポジトリ外のソースを使う手順へ揃えた。
- 接続管理の候補選択・DNS、Version/Verackの並行交換、ネットワーク等の確認、Ready後のフロー起動を一次ソースへ対応づけた。互換性の確認は相手の正直さの証明ではない。
- relayの通知・要求・検証への引き渡し、MissingParentsからの取得・再投入を示した。重複排除を「IDを一度見たら永久に無視」としない説明へ修正した。
- 分かりにくかったDAG図を親子関係が明確な配置へ変更。構造上のtip、同期、確定性を区別した。
- Safetyの定義を確定条件と対応づけ、一時的な未確定状態の差を安全性の破綻と同一視しない説明を補った。
- 伝播教材の接続を集合で管理して重複接続を排除し、双方向リンクを削除するdisconnectを追加した。
- 再接続時の古いメッセージの未補完をテストし、実ノードの同期を再現していない限界として説明した。既存メッセージの照会・再送を発展課題へ移した。
- 接続グラフの循環とBlockDAGの非巡回性、文字列IDとデータ内容の検証、開始ノードも受信数に含むことを区別した。
- 誤った論文題名をePrintレコードへ揃え、次章へのリンクを追加した。

## 基準版ソース

v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。読んだ主要な入口とSHA-256を記録する。

| ソース | SHA-256 |
|---|---|
| [components/connectionmanager/src/lib.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/components/connectionmanager/src/lib.rs) | `2aacd6cefc20f7e4504da42883fe443373a55ab595aed3f2520c87a4297f7105` |
| [protocol/p2p/src/handshake.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/protocol/p2p/src/handshake.rs) | `d97be8c9861a999448cd03ce11c01c6a6fe8ae5b6601eb76628a5cdeaf81c535` |
| [protocol/flows/src/flow_context.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/protocol/flows/src/flow_context.rs) | `6f1ed18826f7d9c261ee8bb65e6602f919bfbdca5314673fb733776a4e737014` |
| [protocol/flows/src/lib.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/protocol/flows/src/lib.rs) | `e26f9617637d9210d305135bbd12571fa8f3656bc2c19257aeb9b5d04335072e` |
| [protocol/flows/src/v6/mod.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/protocol/flows/src/v6/mod.rs) | `003ac003991cc41946ee4b6c138409de16199c710603a90ca1ec176a260b1940` |
| [protocol/flows/src/v5/blockrelay/flow.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/protocol/flows/src/v5/blockrelay/flow.rs) | `f6574e5c5b943277660768105f82228b9af1e3c6047d2724fd2e2526d836cabc` |

HandshakeはKaspadHandshakeの送受信・Ready交換、FlowContextはネットワーク・バージョンの判定とフロー起動、relayはInvRelayBlock・RequestRelayBlocks・validate_and_insert_block・親不足時の再投入を確認した。

基準版は `pub use v6 as v7` とし、v6の登録でv5のrelay等を再利用する。したがってディレクトリ名v5だけを見て、交渉されたプロトコルがv5だとは判断しない。全プロトコル版・全メッセージ・全IBD経路を照合したものではない。

[PHANTOM and GHOSTDAG](https://eprint.iacr.org/2018/104)のレコードで題名を確認した。既存の `devnet-partition-2026-10-31.md`、`core-validation-review-2026-10-31.md` は、実測・親不足の扱いを読むための記録として利用し、新たなノード実験の結果にはしていない。

[Eclipse Attacks on Bitcoin’s Peer-to-Peer Network](https://www.usenix.org/conference/usenixsecurity15/technical-sessions/presentation/heilman)の著者・要旨も確認し、攻撃の背景資料として追加した。Bitcoinを対象とした研究であり、Kaspaの攻撃耐性の評価には転用していない。

## 教材の実行

Rust 1.90.0、macOS ARM64、edition 2021、外部依存なし。本文のRustブロックを一時ファイルへ抽出し、rustcの通常実行・rustc --test・rustfmt --checkを確認した。

5テスト成功：

1. 接続に循環があっても同じIDは一度だけ受理し、別IDは改めて届く。
2. 分断された別の連結成分へは届かない。
3. 同じ接続を逆向きから繰り返しても隣接集合が増えない。
4. 切断は双方向に反映され、その先には届かない。
5. 再接続後も既知IDを元のノードで再送するだけでは欠落を補完できない。一方、新しいIDは全4ノードへ届く。

通常実行の標準出力は本文と一致した。

```text
first delivery: 4
second delivery: 0
after disconnect: 3
```

一時ファイル・バイナリは公開物へ追加していない。本文のコードが検査対象であり、実KaspaのP2Pテストではない。実行したのはrustc経由で、cargo newからの新規環境手順全体を別途実行したものではない。CI同期対象も増やしていない。

## 残る範囲

ソケット通信、遅延・帯域・損失、攻撃耐性、全IBD/Pruning後IBD、親の遅延到着の保留・再評価は教材の対象外。実ノードの新たな起動・接続・採掘・送金は今回行っていない。接続後の不足データ補完が未実装である点は、モデルの限界として残す。

次はSUMMARYの15ページ目、第11章 `src/part-02-consensus/chapter-11.md`。

## 本文反映後の確認

mdBookビルド、生成HTML 53ページの内部リンク・アンカー、SUMMARY全49ページと進捗表の順・ラベル、既存2教材の本文一致、差分の空白検査は成功。公開候補160ファイルの秘密情報・個人ホームパスのパターン検査で検出なし。
