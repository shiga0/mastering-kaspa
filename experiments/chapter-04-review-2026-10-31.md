# SUMMARY順レビュー 08：第4章「分散システムとP2Pネットワーク」

確認日：2026-10-31。対象は `src/part-01-foundations/chapter-04.md`。基礎概説・近似式・伝播教材・基準版の接続管理の入口を確認した。

## 修正内容

- 複数コンピューターへの分散と管理主体の分散を区別し、同じ情報の同時受信を「保証できない」と記載した。
- 物理時計の同時性と因果的な並行を区別。図の横向き矢印を伝播時間ゼロと読ませない説明を補った。
- TCPの接続内のバイト列順序・再送と、複数接続にまたがる順序やアプリケーション要求の完了を区別した。
- ポアソン近似のレートを未受信マイナー群の一定レートとして定義し、単位と一定の未受信時間を明記。数式の不要なアンダースコアのエスケープを修正し、P(0)からの導出と仮の計算例9.52%を補った。Kaspa実測へ読み替えない。
- CAPのCを線形化可能性、Aを故障していないノードでの要求の最終的な応答として説明。定理の対象と稼働率・応答時間・PoWの確定性を区別した。
- DNS候補取得・アドレス管理・アドレス交換を基準版へ対応づけ、隔離DevnetでもDNSを使うという説明にはしないようにした。
- 伝播教材に重複受信の表示を追加し、初回受信時だけ再中継する挙動を観察できるようにした。保存ファイル名、edition、仮想時刻、固定遅延・損失なし等の前提を明記した。
- 複数ブロックを扱う課題ではメッセージIDをイベント・重複判定へ追加すること、無向辺の変更は両方向で行うことを明記した。
- 既存の隔離Devnet分断記録を観測例として案内した。新たなネットワーク実測ではない。

## 一次資料と照合範囲

| 出典 | 確認した範囲 |
|---|---|
| [Lamport：Time, Clocks, and the Ordering of Events](https://lamport.azurewebsites.net/pubs/time-clocks.pdf) | happened-beforeの定義、因果的な並行と論理時計。全アルゴリズムの検証ではない |
| [Gilbert/Lynch：Brewer’s Conjecture](https://www.comp.nus.edu.sg/~gilbert/pubs/BrewersConjecture-SigAct.pdf) | 第2節の定義と第3節の非同期モデルでの不可能性結果。全証明・部分同期モデル全体のレビューではない |
| [Brewer：CAP Twelve Years Later](https://www.infoq.com/articles/cap-twelve-years-later-how-the-rules-have-changed/) | 著者による補足記事を参照。定義の直接の根拠は上記論文へ対応づけた |
| [RFC 9293](https://www.rfc-editor.org/rfc/rfc9293) | 第2.2節の順序を保つバイトストリーム・再送・接続の性質。Kaspaのトランスポート全体を実測したものではない |
| `devnet-partition-2026-10-31.md` | 既存2ノードでの分断中のRPC応答、再接続後のDAG・指定報酬先UTXOの収束と、その記録の限界 |

HTTP資料は確認日に参照。既存のDevnet記録・観測JSONを改変したり、同じ実験を再実行したりしていない。

## 固定ソース

対象はv1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。以下のSHA-256を記録した。

| ファイル | SHA-256 |
|---|---|
| [components/connectionmanager/src/lib.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/components/connectionmanager/src/lib.rs) | `2aacd6cefc20f7e4504da42883fe443373a55ab595aed3f2520c87a4297f7105` |
| [protocol/flows/src/v5/address.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/protocol/flows/src/v5/address.rs) | `980c69da1382a44b4b9b1943cc722546cbd8822abec385f265d92737afb1e8e7` |
| [protocol/p2p/src/handshake.rs](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/protocol/p2p/src/handshake.rs) | `d97be8c9861a999448cd03ce11c01c6a6fe8ae5b6601eb76628a5cdeaf81c535` |

接続管理の候補選択・接続不足時のDNS問い合わせと、RequestAddresses/Addressesの受信・送信を確認。HandshakeはVersion/VerackとReadyの交換の入口を読んだ。接続制御の全条件・全メッセージ・攻撃耐性を照合したものではない。

## 教材と計算の実行確認

本文のRustブロックを一時ファイルへ抽出し、Rust 1.90.0、macOS ARM64で `rustc --edition 2021` によりビルド・実行した。標準ライブラリのみで外部依存はない。`rustfmt --check` も成功した。

期待する標準出力と機械比較し、次の全5行が一致した。

```text
A received at 0 ms
B received at 20 ms
D received at 50 ms
C received at 60 ms
C duplicate at 80 ms (ignored)
```

Pythonの `-expm1(-1 * 0.1)` で数式の計算例を確認し、百分率で小数第2位まで9.52%となった。仮想の到着時刻・単純化した確率モデルの確認であり、実ネットワークの測定ではない。

一時ファイルのコード・バイナリは公開物に追加していない。教材のCI同期対象も増やしていない。発展課題の変更版、ソケット通信、帯域幅・損失・ノード停止・合意・最終確定の実行試験は今回の対象外。

次はSUMMARYの9ページ目、第5章 `src/part-01-foundations/chapter-05.md`。

## 本文反映後の確認

mdBookビルド、生成HTML 53ページの内部リンク・アンカー、SUMMARY全49ページと進捗表の順・ラベル、既存2教材の本文一致、差分の空白検査は成功。公開候補153ファイルの秘密情報・個人ホームパスのパターン検査で検出なし。実ノードへの新たな接続・採掘・送金・分断試験は行っていない。
