# SUMMARY順レビュー 09：第5章「Bitcoinのアーキテクチャ」

確認日：2026-10-31。対象は `src/part-01-foundations/chapter-05.md`。Bitcoinの基礎概説と固定したCoreソースの該当箇所を照合し、本文から抽出したUTXO教材をテストした。

## 修正内容

- C4モデルという見出しを役割と接続の概念図へ変更し、実モジュールの境界・実行順ではないことを明記した。
- ライフサイクルは一般的な経路の例とし、受信ブロック内のTXが自分のMempoolにある必要はないことを補った。
- 手数料を独立した出力として描いていた図を修正。1.3 BTCの入力、1.299 BTCの出力、0.001 BTCの差額という計算例へ整理した。
- Script・署名は支出権限の説明とし、法的な所有権の判定と混同しない見出しにした。
- ヘッダーのtxidのMerkle木とwitnessのコミットメントを区別。包含証明は全TXの有効性検証ではないことを補った。
- PoWの二重SHA-256、80バイトのシリアライズ、targetの範囲・期待難易度を補った。chainworkは実際の試行回数を計測した値ではなくtargetから求める指標とした。
- Coinbase請求額の上限と、採用チェーンから外れたブロックの報酬の扱いを整理。確認数は採用チェーン内の包含を1とする説明へ揃えた。
- Bitcoinでも側枝を管理し、再編成では旧枝を戻して新枝を適用することを補った。
- UTXO教材は空入出力と既存Outpointの再作成を拒否し、出力インデックスの変換も状態変更前に行うよう修正した。消費する入力と同じOutpointなら再作成できるという例外を削除した。
- 教材の数値txidの一意割り当ては呼び出し側の前提と明記。完全消費後のID再利用の検出、Bitcoinの金額上限、Script等は未実装として範囲を限定した。

## 参照版と一次資料

Bitcoin Coreの参照版はv28.0、SHA `110183746150428e6385880c79f8c5733b1361ba`。公開APIでタグの参照コミットを取得し、そのSHAのファイルを取得した。説明を固定するための参照版であり、最新版・運用推奨版ではない。Kaspa側は従来の基準版v1.0.1の概説と比較する。

| 固定ソース | SHA-256 |
|---|---|
| [src/consensus/tx_check.cpp](https://github.com/bitcoin/bitcoin/blob/110183746150428e6385880c79f8c5733b1361ba/src/consensus/tx_check.cpp) | `495df9a5b341bb22a3651e5738b6628253c6b6f1fbef804213be2310eec7bf95` |
| [src/consensus/tx_verify.cpp](https://github.com/bitcoin/bitcoin/blob/110183746150428e6385880c79f8c5733b1361ba/src/consensus/tx_verify.cpp) | `8fbc9bde843c2cfb0577ae7d179ad11f928b09bf8b0fc3e43a9ede7dc34a7370` |
| [src/validation.cpp](https://github.com/bitcoin/bitcoin/blob/110183746150428e6385880c79f8c5733b1361ba/src/validation.cpp) | `cd62a42980f56311f9de661a531e996ca264b8c7800b71dffb3f3dc1173625ee` |
| [src/chain.cpp](https://github.com/bitcoin/bitcoin/blob/110183746150428e6385880c79f8c5733b1361ba/src/chain.cpp) | `bbc1aa18b580d8a36ce7c416ee6cadee2c89686905912fc9a1edf6b3873992a5` |
| [src/pow.cpp](https://github.com/bitcoin/bitcoin/blob/110183746150428e6385880c79f8c5733b1361ba/src/pow.cpp) | `0431aede1a788821e9f67660b2928ed8408b5098d1300ef2a5c3204bcc0be7a8` |
| [src/consensus/merkle.cpp](https://github.com/bitcoin/bitcoin/blob/110183746150428e6385880c79f8c5733b1361ba/src/consensus/merkle.cpp) | `69e5bebee3d98f4ef711c1970d5814950310c4d6e5ee524625570354df0d18ad` |

CheckTransactionの空入出力・金額・重複入力、CheckTxInputsのUTXO・成熟・金額差、GetBlockProof、CheckProofOfWork、validation.cppの期待nBits・Coinbase上限・チェーン切り替えの入口を確認した。merkle.cppはBlockMerkleRootとwitnessの別関数の入口を読んだ。各ファイル全体の監査やCoreの実行結果ではない。

その他の一次資料（確認日に参照）：

- [Bitcoin Developer Reference：Block Chain](https://developer.bitcoin.org/reference/block_chain.html)：80バイトのヘッダー、フィールド、PoW。
- [Bitcoin Developer Guide：Transactions](https://developer.bitcoin.org/devguide/transactions.html#transaction-fees-and-change)：手数料とお釣り。古い手数料水準は転用しない。
- [BIP 141](https://github.com/bitcoin/bips/blob/master/bip-0141.mediawiki)：txid/wtxidとCoinbaseを用いたwitnessコミットメント。

## 教材のテスト

Rust 1.90.0、macOS ARM64、edition 2021、標準ライブラリのみ。本文のコードに `chapter-05-utxo-tests.rs` を連結し、rustc --testで2テスト成功。書式検査も成功した。

- 正常な送金で差額1を返し、入力消費・送金先/お釣り出力・無関係なUTXOの保持を集合全体で確認。
- 同じ送金の再適用はMissingInputとなり、集合を変更しない。
- 拒否ケース9件：空入力、空出力、存在しない入力、重複入力、入力合計のoverflow、出力合計のoverflow、超過支出、無関係な既存出力の上書き、消費する入力と同じOutpointの再作成。
- 各拒否ケースでエラーの種類と、UTXO集合全体が変更されないことを確認。

u64::MAXはモデルの算術を検査するための架空の値であり、Bitcoinの有効な金額ではない。出力インデックスの上限を超える巨大な出力配列は、実際には確保・実行していない。これらのテストは公式Coreとの差分比較、永続化・クラッシュ耐久性、並行処理、署名や全Scriptの検証ではない。

### 再実行

リポジトリのルートで実行する。一時ディレクトリにだけ教材を展開する。

```bash
python3 - <<'PYCODE'
from pathlib import Path
import re
import subprocess
import tempfile

chapter = Path("src/part-01-foundations/chapter-05.md").read_text()
blocks = re.findall(r"^```rust\n(.*?)^```", chapter, re.M | re.S)
assert len(blocks) == 1
root = Path(tempfile.mkdtemp(prefix="chapter-05-utxo-"))
source = root / "utxo.rs"
source.write_text(blocks[0] + "\n" + Path("experiments/chapter-05-utxo-tests.rs").read_text())
binary = root / "utxo-tests"
subprocess.run(["rustc", "--edition", "2021", "--test", str(source), "-o", str(binary)], check=True)
subprocess.run([str(binary)], check=True)
subprocess.run(["rustfmt", "--edition", "2021", "--check", str(source)], check=True)
PYCODE
```

本文のコードとテストを使う手動検査であり、CIの2教材の同期検査対象には加えていない。新たなexamplesフォルダ、Core本体、依存コード、ビルド生成物は公開物へ追加していない。

次はSUMMARYの10ページ目、第6章 `src/part-01-foundations/chapter-06.md`。

## 本文反映後の確認

記録に掲載した再実行手順で2テストと書式検査が成功。mdBookビルド、生成HTML 53ページの内部リンク・アンカー、SUMMARY全49ページと進捗表の順・ラベル、既存2教材の本文一致、差分の空白検査も成功。公開候補155ファイルの秘密情報・個人ホームパスのパターン検査で検出なし。Coreのノード起動・採掘・送金は行っていない。
