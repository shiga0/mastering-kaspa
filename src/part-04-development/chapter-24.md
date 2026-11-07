# Chapter 24 — Wallet Architecture & Key Management（ウォレットアーキテクチャと鍵管理）

<p class="chapter-part">Part IV — Kaspa Development &amp; Implementation（Kaspaの開発と実装）</p>

> **対象読者：** Rust・バックエンド開発の経験があり、Kaspaウォレットの設計と安全性を理解したい人

## 照合した実装

基準版は[本書の基準版](../baseline.md) v1.0.1、コミット `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。主要経路の初回照合は2026-10-31、本文・教材の目次順レビューは2026-11-01に行った。今回の記録は `experiments/chapter-24-review-2026-11-01.md`、以前の照合は `experiments/signing-wallet-review-2026-10-31.md` に示す。

ランダム鍵による単一署名Devnet送金・Virtual受理は過去に実測した（`experiments/devnet-send-review-2026-10-31.md`）。今回再実行したものではない。2026-11-02には公式v1.0.1 SDKでgen1の鍵・アドレスのオフライン復旧とメッセージ署名を確認したが、同日、別プロセスで復旧した受取鍵・変更鍵によるDevnet送金2件も受理まで確認した。残高/履歴の未知index探索・全異常系の検証は完了していない。

## 24.1 この章で学ぶこと

ウォレットは「コインを保存する箱」ではない。ブロックチェーン上のUTXOを発見・追跡し、それを支出するための**鍵と署名機能を管理するソフトウェア**である。本章では次を扱う。

- ウォレット、ノード、UTXO、鍵の責務を分離する。
- 秘密鍵・公開鍵・アドレス・導出パスの関係を理解する。
- HDウォレット、バックアップ、復旧の境界条件を整理する。
- オフライン署名、ハードウェアウォレット、マルチシグなどの設計を比較する。
- Rustでウォレットの状態管理と不変条件を実験する。
- 教材の入力予約と、公式SDKによるHD鍵導出・取引構築・署名の責務を対応付ける。

> **重要：** 本章のRustコードは**鍵を生成・署名する実装ではない**。秘密鍵やシードを本番資産で試さず、実際の署名には監査されたKaspa対応ライブラリと、対象ネットワーク・バージョンで検証された手順を用いる。

## 24.2 ウォレットは何を管理するのか

KaspaのKASはウォレットの内部に存在しない。UTXOはネットワークが合意する状態に存在し、ウォレットは自分が支出できるUTXOを識別する。

```text
┌──────────────────────────────────────────────┐
│ Wallet Application                           │
│ ┌─────────────┐ ┌────────────┐ ┌───────────┐ │
│ │ Key Manager │ │ UTXO Index │ │ Tx Builder│ │
│ └──────┬──────┘ └──────┬─────┘ └─────┬─────┘ │
│        │               │             │       │
│        └───────────────┼─────────────┘       │
│                        ▼                     │
│                 Node Gateway                 │
└────────────────────────┬─────────────────────┘
                         │ RPC / wRPC
                         ▼
                   Kaspa Full Node
                         │
                         ▼
                    Kaspa Network
```

| 構成要素 | 責務 | 管理すべきでないもの |
|---|---|---|
| Key Manager | 鍵の生成・導出・署名の権限管理 | ネットワークのコンセンサス |
| Address Manager | 受取・変更アドレスの導出と使用履歴 | 秘密鍵のログ出力 |
| UTXO Tracker | 自分のUTXO、残高、利用可能性の追跡 | 未確認残高の確定扱い |
| Coin Selector | 入力UTXOと変更出力の選択 | 鍵素材の直接保持 |
| Transaction Builder | 支出内容と署名対象の構築 | ネットワークの検証規則の独自変更 |
| Signer | 承認された内容への署名 | 未承認の任意トランザクション署名 |
| Node Gateway | ノード照会・送信・イベント購読 | ウォレットの業務ルール |

**境界を明確にすること**が、安全性と変更容易性の両方につながる。

## 24.3 秘密鍵・公開鍵・アドレス

秘密鍵は支出権限の根拠となる秘密の値である。公開鍵は秘密鍵から導かれ、署名の検証に利用される。アドレスは受取先を人間やアプリケーションが扱いやすい形式で表現する識別子であり、公開鍵そのものと常に同一ではない。

```text
Secret Key ──(楕円曲線演算)──> Public Key
    │                             │
    │                             └──(アドレス形式に応じた符号化)──> Address
    │
    └──(署名)──> Signature ──(公開鍵で検証)──> true / false
```

Kaspaのアドレス形式・署名方式・Scriptバージョンの詳細は、利用するウォレット種別と現行の実装で確認する。Bitcoinの`bc1...`やEthereumの`0x...`をそのままKaspaアドレスとして扱ってはならない。基準版の[Address Prefix](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/crypto/addresses/src/lib.rs)はkaspa / kaspatest / kaspasim / kaspadevを区別する。ただし接頭辞は完全なNetworkIdではなく、kaspatestだけではTestnet-10とTestnet-11を識別できない。アドレスの検査に加え、接続ノードのNetworkIdを番号まで照合する。

> **注意：** 秘密鍵を知っていることと、任意のUTXOを支出できることは同義ではない。対象UTXOのロック条件、署名対象データ、トランザクションの有効性が一致する必要がある。

## 24.4 シードフレーズとHDウォレット

### 24.4.1 なぜ階層的に鍵を導出するのか

受取先ごとに独立した鍵を生成する方式では、鍵を増やすたびにバックアップ対象も増える。HD（Hierarchical Deterministic）ウォレットは、共通のルートから複数の鍵を決定的に導出する。

```text
Recovery Mnemonic
       │
       ▼
Seed / Root Key Material
       │
       ▼
Hierarchical Key Derivation
       ├── Account 0
       │    ├── Receive 0, 1, 2, ...
       │    └── Change  0, 1, 2, ...
       └── Account 1
            ├── Receive 0, 1, 2, ...
            └── Change  0, 1, 2, ...
```

「シードフレーズが同じなら必ず同じアドレスになる」とは限らない。**導出方式、導出パス、パスフレーズ、アドレス形式、ネットワーク**が異なると、復旧結果も変わりうる。

### 24.4.2 BIP-39・BIP-32・BIP-44を混同しない

| 仕様 | 主な役割 | 注意点 |
|---|---|---|
| BIP-39 | ニーモニックとシード生成 | 任意のウォレットが完全互換とは限らない |
| BIP-32 | 階層的な鍵導出 | 導出パスや曲線・鍵形式の互換性を要確認 |
| BIP-44 | アカウント階層の慣習 | coin type とウォレット実装の対応が必要 |

基準版の[gen1のWalletDerivationManager](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/wallet/keys/src/derivation/gen1/hd.rs)は単一署名のアカウント経路に `m/44'/111111'/account'`、マルチシグにpurpose `45'` を使う。cosigner indexや受取・変更の枝が必要な場合は、その後へ追加する。これはgen1の経路生成を読んだ結果であり、legacy/gen0を含むすべてのKaspaウォレットの共通仕様とは扱わない。復旧では利用した方式・account・cosigner・受取/変更の枝・アドレスindexも記録する。

同じファイルの `create_address` はSchnorr用にはx-only公開鍵、ECDSA用には圧縮公開鍵をpayloadとして使い、指定prefixで符号化する。ニーモニックが一致しても、公開鍵やアドレスの種類を取り違えると期待する復旧結果にはならない。

### 24.4.3 パスフレーズ

[BIP-39](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki)の追加パスフレーズは、同じニーモニックから異なるシードを導出する。誤ったパスフレーズでも別の有効なシードになり得るため、エラーが出ないことは復旧成功の証拠ではない。既知の受取・変更アドレス等を照合する。ウォレットファイルの解除・暗号化パスワードとも役割を分ける。パスフレーズを失うと、ニーモニックが残っていても目的のウォレットを復旧できない可能性がある。パスフレーズの有無もバックアップ仕様の一部である。

**固定版SDKのUnicode入力にも注意が必要。** BIP-39のseed生成はニーモニックと追加パスフレーズをUTF-8 NFKDで処理する。一方、公式v1.0.1 SDKの `Mnemonic.toSeed` は入力を正規化せず、基準ソースの [`to_seed`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/wallet/bip32/src/mnemonic/phrase.rs) でもそのままPBKDF2へ渡す。

英語ニーモニックと追加パスフレーズ `caf\u00e9`（合成形）/ `cafe\u0301`（分解形）を使うと、NFKDでは同じになるのに、固定版SDKへそのまま渡したseed・アドレスは一致しなかった。明示的にNFKD化するとNode cryptoで求めたBIP-39の参照値と一致した。既存ウォレットがraw入力から作られていた場合、復旧時だけ正規化を追加すると別の鍵になる。作成時の版・入力処理と既知アドレスを照合する。この限定試験を、現行SDKや全ウォレットの挙動へ一般化しない。

## 24.5 アドレスの再利用と変更出力

UTXOモデルでは、入力の合計が支払額と手数料を超える場合、差額を**変更出力（change output）**として自分の管理するアドレスへ戻す。

```text
Inputs:  12 KAS
          │
          ▼
Transaction
  ├── Recipient: 8 KAS
  ├── Change:    3.9 KAS
  └── Fee:       0.1 KAS
```

数値は説明用。実際の手数料計算はトランザクションのmassやネットワーク規則に依存する。

ウォレットは受取アドレスだけでなく変更アドレスも追跡する必要がある。変更アドレスを復旧対象から外すと、残高の一部を見失う可能性がある。アドレスの使い回しを減らすことは、トランザクションの関連付けを難しくするための基本的なプライバシー対策となる。ただし、UTXOの統合などで関連性が推測されることはある。

## 24.6 残高は一つの数字ではない

ウォレット画面に表示する「残高」には異なる意味がある。

| 表示 | 意味 | 典型的な注意点 |
|---|---|---|
| 観測残高 | ノード・インデクサが観測したUTXOの合計 | 同期遅延や再編成 |
| 利用可能残高 | 現時点で支出候補にできるUTXOの合計 | ロック・予約・ポリシー |
| 送信可能額 | 手数料・変更出力・mass制約を考慮した額 | 残高と一致しないことがある |
| 保留中 | Mempoolや未確定ブロックに関連する額 | コンセンサス受理とは別 |

**不変条件：** `available_balance <= observed_balance` のような関係は、同じスナップショットと定義に基づく場合に限って期待できる。異なる時刻のRPC応答を混ぜると一時的に矛盾して見える。

## 24.7 ウォレットの送金ライフサイクル

```text
User Intent
    │
    ▼
Validate Destination / Amount / Network
    │
    ▼
Query & Select Spendable UTXOs
    │
    ▼
Construct Outputs / Change / Fee
    │
    ▼
Present Exact Transaction for Approval
    │
    ▼
Sign (Local or Hardware Signer)
    │
    ▼
Broadcast via Node
    │
    ▼
Track Mempool / BlockDAG / Virtual State
    │
    ▼
Update Wallet View
```

重要なのは、**送信成功 ≠ コンセンサス受理 ≠ アプリケーション上の確定**であること。送信APIが成功を返しただけで「支払い完了」と断定しない。

## 24.8 Coin Selectionと入力予約

複数のUTXOからどれを選ぶかは、手数料・プライバシー・UTXOの断片化に影響する。

- **少数入力優先：** トランザクションが小さくなりやすいが、特定のUTXOへ依存する。
- **小額UTXO統合：** 将来の管理を簡単にできる一方、手数料やプライバシーへの影響がある。
- **プライバシー配慮：** 無関係なUTXOを不用意に結び付けない。

並行送金で同じUTXOを二度選ばないよう、ウォレット内部で**入力予約**を設ける。ただしローカル予約はネットワーク上の支出確定を保証しない。

```text
Available -> Reserved -> Broadcast -> Accepted
                  │           │
                  └─ Cancelled└─ Rejected / Reorg -> Reconcile
```

状態の戻し方は、再照会したコンセンサス状態と照合して決める。単に通信がタイムアウトしただけなら、**送信が失敗したと断定せず**トランザクションID等で再確認する。

### 24.8.1 基準版のGeneratorとSigner

[Generator](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/wallet/core/src/tx/generator/generator.rs)はUTXOを集め、mass境界に達した場合に中継トランザクションを作り、複数段階のトランザクションへ分割する。送金要求1件が必ず単一のトランザクションになるとは限らない。各段階の入力・手数料と、最終の支払い先を分けて追う。

[SignerT::try_sign](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/wallet/core/src/tx/generator/signer.rs)は `SignableTransaction` とアドレス群を受け、対応する鍵を取得し、`sign_with_multiple_v2` と `fully_signed` を呼ぶ。署名用の一時的な鍵配列に対するzeroizeは、fully_signed()?の後にある。そこでErrになると通常の早期returnで消去行へ到達しない。成功経路に消去処理があることと、全終了経路・鍵キャッシュ・プロセスメモリ全体の消去を保証することは別である。これはソース上の制御フローの確認で、実鍵を使ったメモリ残留の測定ではない。multisig向けのTODOもあるため、この一般Signerが任意のマルチシグを扱えるとは断定しない。

この経路のソース確認は、生成したトランザクションがノードで受理された実験とは別である。ランダム鍵で単一署名送金した過去の記録は24.17に示す。gen1 HD鍵の復旧と所定の送金は24.12の別実験で確認した。任意のGenerator/Signer経路、全失敗分岐を実測した意味ではない。

## 24.9 オフライン署名とハードウェアウォレット

署名をネットワーク接続部分から分離すると、秘密鍵をオンラインのバックエンドへ渡さずに済む。

```text
Online Host                          Isolated Signer
┌───────────────────┐                ┌──────────────────┐
│ UTXO Discovery    │                │ Key Storage      │
│ Tx Construction   │── Sign Request▶│ Review & Sign    │
│ Broadcast         │◀─ Signature ──│                  │
└───────────────────┘                └──────────────────┘
```

ただしオフライン署名も万能ではない。署名要求の参照UTXOの金額・Script、変更出力が自分の導出経路に属すること、[第16章](../part-03-transactions/chapter-16.md)のSIGHASHフラグも確認対象になる。署名デバイスが**受取先・金額・手数料・ネットワーク・変更出力**を正しく解釈し、利用者に確認できなければ、悪意あるホストによる誘導を防げない。

ハードウェアウォレット対応は製品・ファームウェア・Kaspa機能の実装状況によって異なる。購入前に公式の対応状況を確認する。

### 24.9.1 アドレス案・Provider探索・部分署名の互換性

以下は2026-11-03に確認した提案であり、本章のv1.0.1実習の機能追加ではない。

**公開鍵を隠すアドレス案：** [KIP-22案／PR #48](https://github.com/kaspanet/kips/pull/48)はDraft・Open・未マージで、[固定本文](https://github.com/kaspanet/kips/blob/b031344f82f4491b9066ef556163509f93833cd6/kip-0022.md)は公開鍵を含むredeem scriptのハッシュへ支払うP2SH方式を提案する。支出前の公開鍵露出を抑えるが、支出時には鍵が明らかになる。公開鍵・xpubを別経路で公開した場合や支出時の攻撃リスクも残り、量子耐性署名への置換ではない。提案の導出purposeと基準版gen1の導出パスを混ぜて復旧しない。同番号の別案との区別は[30.14](../part-05-advanced/chapter-30.md#3014-kip-22を例にしたprと正式文書の違い)を参照する。

**Wallet Discovery：** [KCC-12案／PR #24](https://github.com/kaspanet/kccs/pull/24)もDraft・Open・未マージである。ブラウザー内のウォレットProviderを見つける仕組みであり、24.12のHDアカウント・使用済みアドレス探索とは異なる。提案では`kaspa:requestProvider`と`kaspa:announceProvider`を用いる。Providerの自己紹介だけで信頼性は確定しないため、接続先、ネットワーク、ユーザー承認、対応メソッドを確認する。

**部分署名：** 同案の[固定本文](https://github.com/saefstroem/kccs/blob/8e1053ec112b7e69504f4c26cba910ec0071ea0f/kcc-0012.md)は、指定入力だけを署名する取引APIと、PSKBへの部分署名APIを区別する。署名追加と、最終Scriptの構築・送信は別の処理である。別署名者のデータを保持し、入力・出力を勝手に並べ替えず、参照UTXOや対応しない署名方式が不明なら拒否する。PSKBをBitcoinのPSBTと同一形式だと仮定しない。各ウォレット間の実機互換性は未検証である。

`SIGHASH_ALL`は出力を拘束するが、それだけで任意の操作引数や承認意図の認証が完了するわけではない（[31.10.1](../part-05-advanced/chapter-31.md#31101-idプログラム状態の出所を分ける)）。通常のALLで署名した後にスポンサー入力を追加すれば、その署名はそのまま使えない。必要な入力・出力・手数料を先に確定してから署名を集めるか、再構築・再署名する。異なるSIGHASHを使う設計は拘束範囲を別途検証する。[第35章](../part-06-contributing/chapter-35.md#課題eprogrammabilityへの拡張)のスポンサー決済は、この課題を持つ設計例である。

## 24.10 Watch-onlyウォレットと署名権限の分離

Watch-onlyウォレットは、公開情報を用いて残高やトランザクションを監視し、秘密鍵を保持しない構成である。会計システムや入金監視に向いている。

| 構成 | できること | できないこと |
|---|---|---|
| Watch-only | 受取アドレス監視・残高確認 | 秘密鍵を要する署名 |
| Hot wallet | 自動署名・自動送金 | 鍵漏えいリスクをゼロにすること |
| Cold signer | オフライン承認・署名 | 単独でネットワーク照会・送信（通常） |
| Multisig / 条件付き支出 | 複数承認や条件の表現（対応方式による） | 実装非対応のスクリプト条件の実行 |

Kaspaのスクリプト機能で実現できる条件と、特定ウォレットがUIとして提供する機能は分けて確認する。

Watch-onlyでも拡張公開鍵（xpub）はアドレス群の関連付けを可能にし、プライバシー上の機微情報になる。[BIP-32](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki)では、親の拡張公開鍵と対応する非hardened子秘密鍵が揃うと親秘密鍵を求められる性質もある。xpubだけで署名できるわけではないが、公開リポジトリやログへ無条件に載せてよい情報とは扱わない。

## 24.11 マルチシグと組織の承認フロー

「複数人の承認が必要」という業務要件と、「オンチェーンで複数署名を検証する」という技術要件は同じではない。

```text
Payment Request
     │
     ▼
Business Approval (Policy)
     │
     ▼
Transaction Construction
     │
     ▼
Cryptographic Signing (Script / Signer Policy)
     │
     ▼
Broadcast
```

業務承認をDB上で2人分記録しても、単一秘密鍵が流出すれば不正送金される可能性がある。逆に、複数署名条件だけでは経理上の請求書照合を保証しない。**業務ポリシーと暗号学的な支出条件を両方設計する**。

## 24.12 バックアップと復旧

ウォレット設計では、作成時より**復旧時**に設計ミスが発覚しやすい。

### 復旧に必要な情報

1. ニーモニック／秘密鍵などの復旧素材
2. 追加パスフレーズの有無
3. 使用したウォレットの導出方式とアカウント構成
4. アドレス形式・ネットワークの情報
5. 必要に応じてアドレス探索範囲やウォレット固有のメタデータ

探索範囲・停止条件も復旧仕様である。[BIP-44](https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki)のアカウント探索は残高だけでなく取引履歴に基づく。残高ゼロを未使用と決めると、過去に使われたアドレスやアカウントを見落とし得る。gap limitを超えて離れたindexを使っていた場合などは探索範囲の調整が必要になるが、BIP-44の探索規則をすべてのKaspaウォレットへ当てはめない。

### 固定版のアカウント探索とgap limit

基準版の [`scan_bip44_accounts`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/wallet/core/src/wallet/mod.rs) は、各accountについて受取・変更の両枝の `0..address_scan_extent` を導出し、現在のUTXOを照会する。例えばextent 20はindex 0〜19で、20を含まない。継続条件は `account_index < last_account_index + account_scan_extent`。BIP-44の履歴ベースの探索や連続未使用20件の停止と同じ処理ではない。

探索範囲の端は結果に影響する。例えば、receive/change index 20のUTXOはextent 20では範囲外で、21へ広げると検出できる。**戻り値 `lastAccountIndexFound=0` は、未発見の場合とaccount 0を検出した場合の両方で返る。** この値だけで資金がないとは判断できない。

現在UTXOが空でも、支出済みの履歴がないとは限らない。教育用gapモデルでも、履歴を使うか現在UTXOだけを見るかで探索結果が変わった。探索上限を広げても、その範囲外に資金がない証拠にはならない。具体的な境界条件と実測は、下表の探索検査の記録を参照する。

### 復旧テスト

- 本番資産とは切り離したテスト用ウォレットを用意する。
- 少額・テストネットなど安全な環境で受取アドレスが再現されることを確かめる。
- 受取だけでなく変更アドレスや複数アカウントも確認する。
- 署名と送金が再現できることを確認する。
- 手順を記録する。ただし**秘密情報そのものをログや手順書に書かない**。

### 既存の検査で確認した範囲

2026-11-02に公式v1.0.1 SDKで行った検査を、目的別に整理する。鍵が再構成できること、資金を探索できること、復旧鍵で送金できることは、それぞれ別の確認である。

| 検査 | 確認したこと | 対象外・限界 | 実行手順・詳細記録 |
|---|---|---|---|
| 鍵・アドレスの再構成 | 独立プロセス間で160個の鍵由来の公開鍵・アドレスが一致。メッセージ署名、gen1導出パス、xpubとの一致と所定の不正入力の拒否 | 既知indexのオフライン検査。残高・履歴探索や取引送金は含まない | [実行手順](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/hd-recovery/README.md#実行)・[条件と結果](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/hd-recovery-review-2026-11-02.md) |
| Unicode入力 | 合成形・分解形の追加パスフレーズで、固定版SDKのraw入力とNFKD化後の結果の違いを確認（24.4.3） | 現行SDK・全ウォレットへ一般化しない。既存ウォレットの入力処理と照合が必要 | [Unicodeの結果](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/hd-recovery/README.md#unicodeの結果) |
| アドレス・アカウント探索 | 合成RPC応答による公式APIの14ケースと、教育用gapモデルの12,288比較で探索境界を確認 | 実ノード・Indexerからの支出済み履歴の復元ではない | [実行手順](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/hd-recovery/README.md#アドレスアカウント探索の境界)・[条件と結果](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/hd-discovery-review-2026-11-02.md) |
| 復旧鍵によるDevnet送金 | 別workerで復旧した受取鍵で送金し、さらに復旧した変更鍵でお釣りを支出。2件のMempool掲載・virtual受理・入力消費・出力と手数料を確認 | 既知パスでの内部送金。未知indexの再発見や全ウォレット復旧ではない | [送金手順](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/hd-recovery/README.md#復旧した鍵でdevnet送金する)・[条件と結果](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/hd-send-review-2026-11-02.md) |

復旧素材はテスト用にランダム生成し、メモリと匿名パイプだけで受け渡した。Devnet送金では、誤った追加パスフレーズによる既知アドレスとの不一致を署名前に検出し、署名以外のTX項目が未署名TXと一致することも照合した。実コードと受取枝・変更枝の使い分けは[24.14.1](#24141-教材の入力予約と公式sdkをつなぐ)で追う。

これらを合わせても、暗号化バックアップ・DB・未知の残高や履歴の探索を含むウォレット全体の復旧を検証したことにはならない。

> **注意：** ニーモニックをWebフォームやAIチャットに入力して「復旧できるか確認」してはいけない。スクリーンショット・クラウド同期・クリップボード履歴にも注意する。

## 24.13 鍵管理の脅威モデル

| 脅威 | 典型的な失敗 | 対策の例 |
|---|---|---|
| 秘密鍵漏えい | ログ・Git・環境変数の露出 | 非保持設計、秘密情報スキャン、アクセス制御 |
| フィッシング | 偽ウォレットへのシード入力 | 公式配布元の検証、シード非入力原則 |
| マルウェア | 送金先アドレスの置換 | 署名デバイス上で宛先確認 |
| サプライチェーン | 改ざんされた依存ライブラリ | 依存固定、署名・リリース検証、監査 |
| バックアップ紛失 | シードが一箇所だけ | オフライン冗長化、定期的な復旧訓練 |
| ネットワーク誤送信 | mainnet/testnet混同 | ネットワークIDの型とUI確認 |
| 再編成・同期遅延 | 受信済みを確定扱い | 状態追跡、入金確認ポリシー |
| 内部不正 | 単一担当者が全権限を保持 | 職務分離、複数承認、監査ログ |

鍵素材を扱うプロセスの権限は最小化する。バックエンドの一般的なAPIサーバーが常時ホットウォレットの全秘密鍵を保持する設計は、必要性と被害範囲を慎重に検討する。

## 24.14 Rustで設計するウォレットの境界

ウォレットのロジックをSDKの具体型へ直接結び付けると、ネットワーク接続・鍵保管方式の変更が難しくなる。

```text
Application Service
    ├── WalletPolicy
    ├── UtxoRepository (trait)
    ├── TransactionBuilder (trait)
    ├── Signer (trait)
    └── NodeGateway (trait)
```

以下は**秘密鍵を持たない**入力予約の教育用実装である。外部クレート不要で、Rust標準ライブラリのみを利用する。本書のリポジトリとは別の作業用ディレクトリで、後述のプロジェクトを作る。Outpointの文字列も独自モデルであり、実際のTX ID・Script・成熟・NetworkIdは検証しない。

<details class="code-details">
<summary>ウォレット境界モデルの実装とテストを見る</summary>

```rust
use std::collections::{hash_map::Entry, HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Outpoint {
    pub txid: String,
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utxo {
    pub outpoint: Outpoint,
    pub amount: u64, // sompi
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReservationId(u64);

#[derive(Debug, PartialEq, Eq)]
pub struct Reservation {
    pub id: ReservationId,
    pub total: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum WalletError {
    EmptyInput,
    UnknownUtxo,
    AlreadyReserved,
    DuplicateInput,
    DuplicateUtxo,
    AmountOverflow,
    ReservationIdExhausted,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WalletState {
    utxos: HashMap<Outpoint, Utxo>,
    reserved: HashMap<Outpoint, ReservationId>,
    last_reservation_id: u64,
}

impl WalletState {
    // 初期投入用。同じOutpointのデータを黙って上書きしない。
    pub fn insert(&mut self, utxo: Utxo) -> Result<(), WalletError> {
        match self.utxos.entry(utxo.outpoint.clone()) {
            Entry::Occupied(_) => Err(WalletError::DuplicateUtxo),
            Entry::Vacant(entry) => {
                entry.insert(utxo);
                Ok(())
            }
        }
    }

    // 全入力・合計・IDを検証してから更新する。Errなら既存状態を維持する。
    pub fn reserve(&mut self, inputs: &[Outpoint]) -> Result<Reservation, WalletError> {
        if inputs.is_empty() {
            return Err(WalletError::EmptyInput);
        }
        let mut seen = HashSet::new();
        let mut total = 0_u64;
        for input in inputs {
            if !seen.insert(input) {
                return Err(WalletError::DuplicateInput);
            }
            let utxo = self.utxos.get(input).ok_or(WalletError::UnknownUtxo)?;
            if self.reserved.contains_key(input) {
                return Err(WalletError::AlreadyReserved);
            }
            total = total
                .checked_add(utxo.amount)
                .ok_or(WalletError::AmountOverflow)?;
        }
        let next = self
            .last_reservation_id
            .checked_add(1)
            .ok_or(WalletError::ReservationIdExhausted)?;
        let id = ReservationId(next);
        self.reserved
            .extend(inputs.iter().cloned().map(|input| (input, id)));
        self.last_reservation_id = next;
        Ok(Reservation { id, total })
    }

    // 同一インスタンス内の対象予約だけを解除する。古いIDの再適用は0件。
    pub fn release(&mut self, id: ReservationId) -> usize {
        let before = self.reserved.len();
        self.reserved.retain(|_, owner| *owner != id);
        before - self.reserved.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn add(wallet: &mut WalletState, name: &str, amount: u64) -> Outpoint {
        let op = Outpoint {
            txid: name.into(),
            index: 0,
        };
        wallet
            .insert(Utxo {
                outpoint: op.clone(),
                amount,
            })
            .unwrap();
        op
    }
    fn sample() -> (WalletState, Outpoint) {
        let mut wallet = WalletState::default();
        let op = add(&mut wallet, "tx-a", 100);
        (wallet, op)
    }

    #[test]
    fn reserve_once() {
        let (mut wallet, op) = sample();
        assert_eq!(
            wallet.reserve(std::slice::from_ref(&op)).unwrap().total,
            100
        );
        let before = wallet.clone();
        assert_eq!(wallet.reserve(&[op]), Err(WalletError::AlreadyReserved));
        assert_eq!(wallet, before);
    }
    #[test]
    fn duplicate_input_does_not_mutate_state() {
        let (mut wallet, op) = sample();
        let before = wallet.clone();
        assert_eq!(
            wallet.reserve(&[op.clone(), op]),
            Err(WalletError::DuplicateInput)
        );
        assert_eq!(wallet, before);
    }
    #[test]
    fn missing_input_does_not_mutate_state() {
        let (mut wallet, op) = sample();
        let missing = Outpoint {
            txid: "missing".into(),
            index: 0,
        };
        let before = wallet.clone();
        assert_eq!(
            wallet.reserve(&[op, missing]),
            Err(WalletError::UnknownUtxo)
        );
        assert_eq!(wallet, before);
    }
    #[test]
    fn empty_input_does_not_allocate_an_id() {
        let (mut wallet, _) = sample();
        let before = wallet.clone();
        assert_eq!(wallet.reserve(&[]), Err(WalletError::EmptyInput));
        assert_eq!(wallet, before);
    }
    #[test]
    fn release_allows_retry() {
        let (mut wallet, op) = sample();
        let first = wallet.reserve(std::slice::from_ref(&op)).unwrap();
        assert_eq!(wallet.release(first.id), 1);
        let second = wallet.reserve(&[op]).unwrap();
        assert_eq!(second.total, 100);
        assert_ne!(first.id, second.id);
    }
    #[test]
    fn stale_release_cannot_clear_a_new_reservation() {
        let (mut wallet, op) = sample();
        let old = wallet.reserve(std::slice::from_ref(&op)).unwrap();
        wallet.release(old.id);
        let current = wallet.reserve(&[op]).unwrap();
        let before = wallet.clone();
        assert_eq!(wallet.release(old.id), 0);
        assert_eq!(wallet, before);
        assert_eq!(wallet.release(current.id), 1);
    }
    #[test]
    fn release_affects_only_the_matching_owner() {
        let (mut wallet, a) = sample();
        let b = add(&mut wallet, "tx-b", 200);
        let c = add(&mut wallet, "tx-c", 300);
        let first = wallet.reserve(&[a, b]).unwrap();
        assert_eq!(first.total, 300);
        let second = wallet.reserve(std::slice::from_ref(&c)).unwrap();
        assert_eq!(wallet.release(first.id), 2);
        assert_eq!(wallet.reserved.get(&c), Some(&second.id));
    }
    #[test]
    fn partially_reserved_input_set_preserves_all_state() {
        let (mut wallet, a) = sample();
        let b = add(&mut wallet, "tx-b", 200);
        wallet.reserve(std::slice::from_ref(&b)).unwrap();
        let before = wallet.clone();
        assert_eq!(wallet.reserve(&[a, b]), Err(WalletError::AlreadyReserved));
        assert_eq!(wallet, before);
    }
    #[test]
    fn amount_overflow_preserves_existing_reservations() {
        let (mut wallet, a) = sample();
        let b = add(&mut wallet, "tx-b", u64::MAX);
        let c = add(&mut wallet, "tx-c", 1);
        wallet.reserve(&[a]).unwrap();
        let before = wallet.clone();
        assert_eq!(wallet.reserve(&[b, c]), Err(WalletError::AmountOverflow));
        assert_eq!(wallet, before);
    }
    #[test]
    fn duplicate_insert_cannot_change_reserved_amount() {
        let (mut wallet, op) = sample();
        wallet.reserve(std::slice::from_ref(&op)).unwrap();
        let before = wallet.clone();
        assert_eq!(
            wallet.insert(Utxo {
                outpoint: op,
                amount: 999
            }),
            Err(WalletError::DuplicateUtxo)
        );
        assert_eq!(wallet, before);
    }
    #[test]
    fn exhausted_ids_do_not_wrap_or_mutate_state() {
        let (mut wallet, op) = sample();
        wallet.last_reservation_id = u64::MAX;
        let before = wallet.clone();
        assert_eq!(
            wallet.reserve(&[op]),
            Err(WalletError::ReservationIdExhausted)
        );
        assert_eq!(wallet, before);
    }
}
```

</details>

本文から抽出した教材は11テスト成功。既存の予約を含む失敗時の状態維持、古い解除要求、複数予約の分離、ID上限も検査した。実UTXO・署名・送金・永続DBの検査ではない。

ローカルでの実行例：

```bash
cargo new --lib --edition 2021 wallet-state-lab
cd wallet-state-lab
# 上のコードを src/lib.rs に配置
cargo test
```

この実装の予約IDは、同じWalletStateインスタンスのメモリ内でだけ有効である。別のウォレット・ネットワーク・再起動後に通用するIDではなく、認証トークンでもない。Cloneはテストの状態比較用であり、複製を別の送金ワーカーとして動かしてはいけない。insertは初期投入用で、RPCのスナップショットを反映するupsertや再編成処理ではない。

この実装は**&mut selfで直列に操作するメモリ内の予約**しか扱わない。複数プロセスや複数デバイスでの送金には、DBトランザクション、排他制御、再起動後の復元、ネットワークとの再照合が必要である。`release`は送信前の取消や、既存の送信結果を解決して解除できると判断した場合に呼ぶ。通信タイムアウトや期限切れだけでの無条件解除はしない。期限は再照合を始める契機として扱う。Err時の状態維持はこのメモリ内の通常の戻り値についての性質であり、クラッシュやDBの原子性の保証ではない。

### 24.14.1 教材の入力予約と公式SDKをつなぐ

前節の`WalletState`が管理するのは、「このアプリ内で、どの処理が入力を予約しているか」である。公式SDKは鍵導出・取引構築・署名・RPCの機能を提供するが、この独自の予約IDを受け取って管理するわけではない。

実装を読む際は、次の二つを分けて追う。

| アプリ側の責務 | 対応する実データ・SDKの操作 |
|---|---|
| 候補UTXOの取得・選択 | `getUtxosByAddresses`で取得したOutpoint・金額・成熟条件を確認する |
| 同じ入力の並行使用を防ぐ | 選択したOutpointを予約する。前節の`reserve`はこの部分だけを表す |
| 取引構築と変更出力 | `createTransactions`へ入力候補と`changeAddress`を渡す |
| 支出する鍵の用意 | 入力アドレスに対応する受取枝・変更枝の鍵を導出する |
| 署名と送信 | `pending.sign`と`pending.submit`、または復旧実習の`signTransaction`とRPC送信 |
| 送信後の追跡 | TX ID・現在UTXO・virtual受理を照会し、アプリの状態を更新する |

[第15章15.17.1](../part-03-transactions/chapter-15.md#15171-教材から公式sdkの送金処理へ)と[第16章16.12.3](../part-03-transactions/chapter-16.md#16123-fakeverifierから公式sdkの署名へ)は、ランダムな一時鍵による送金を扱う。ここでは既存のHD復旧実習を使い、同じ鍵を再び導出する処理と、変更枝のお釣りを使う処理まで追う。

#### まず、ノードを使わず鍵・アドレスの再構成を試す

[第28章28.6.2](chapter-28.md#2862-公式nodejswasm-sdk)で公式v1.0.1 SDKを取得・SHA-256照合・展開したら、リポジトリのルートで次を実行する。このオフライン検査にはDocker・ノード・WebSocket依存は不要である。

```bash
KASPA_SDK_ROOT="$PWD/experiments/devnet/.sdk" node experiments/hd-recovery/run.cjs
```

スクリプトはテスト用ニーモニックを内部生成し、作成用と復旧用の独立プロセスで同じ鍵・アドレスを得られるか比較する。`allChecksPassed: true`と`recoveredKeys: 160`が確認点になる。個人の復旧素材を渡すインターフェースではなく、`--worker`も内部通信用なので手動実行しない。

`allChecksPassed`には、24.12で説明したUnicode正規化の既知の差が再現されたことも含む。BIP-39全面適合やウォレット全体の復旧成功を示す値ではない。試験条件は[実習README](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/hd-recovery/README.md)にまとめている。

#### HD導出を使う実コードを読む

`experiments/hd-recovery/send.cjs`の復旧workerは、親プロセスから匿名パイプで受け取ったテスト素材を使い、鍵とDevnetアドレスを再構成する。以下はその抜粋で、`input`・`paths`・オブジェクト管理用の`own`はスクリプト全体で定義している。

```javascript
{{#include ../../experiments/hd-recovery/send.cjs:restore-hd-keys}}
```

この実習の`paths`は次の3本である。一般のウォレットすべてに使える設定ではなく、固定したgen1単一署名の試験条件として読む。

| 用途 | 導出パス |
|---|---|
| 資金を受け取る鍵 | `m/44'/111111'/1'/0/21` |
| お釣りを受け取る変更鍵 | `m/44'/111111'/1'/1/20` |
| 試験用の送金先 | `m/44'/111111'/0'/0/0` |

追加パスフレーズが違っても、別の鍵を導出できてしまう場合がある。そこで署名前に、作成時に得た既知アドレスと復旧結果が一致することを確認する。

```javascript
{{#include ../../experiments/hd-recovery/send.cjs:sign-restored-transaction}}
```

`signTransaction`の第3引数`true`でローカルの署名検証を要求する。親プロセスは返された取引について、署名以外の項目が元の未署名取引と一致するかを確認してからノードへ送る。ローカル検証成功と、ノードでの受理は別々に確認する。

#### 受取鍵の送金から、変更鍵の送金へ進む

実ノードで試す場合は、SDKに加えてWebSocket依存とDockerを用意し、[HD復旧実習READMEの送金手順](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/hd-recovery/README.md#復旧した鍵でdevnet送金する)に従う。既存の単一ノード実習との同時実行は避け、起動・停止・ボリュームの扱いもその手順を使う。

処理は次の順で進む。

1. 作成用プロセスで受取・変更・送金先アドレスを求め、そのプロセスを終了する。
2. 受取アドレスへ採掘し、成熟した報酬から1 KASを送る取引を構築する。お釣りの宛先は変更枝にする。
3. 新しい復旧workerで受取鍵を再構成し、既知アドレスとの一致を確認して署名する。
4. 送信後、入力消費・宛先出力・変更出力・手数料・virtual受理を確認する。
5. さらに別の復旧workerで変更鍵を再構成し、一回目のお釣りのOutpointを入力にして、もう1 KASを送る。

2026-11-02の実測では、復旧署名プロセス2件、誤った追加パスフレーズの署名前拒否、変更出力を使った次の送金を確認した。機械可読な結果は`experiments/hd-recovery/send-observed-2026-11-02.json`に保存している。

#### 入力予約はどこへ組み込むか

この送金実習は逐次実行であり、前節の`WalletState`と接続した実装ではない。並行送金を追加するなら、構築に使う入力候補を予約し、生成された各取引の実入力との対応を管理する。SDKが複数取引を生成する場合には、最終取引だけでなく中継段階も追う必要がある。

署名前に取り消した処理と、送信後に応答を失った処理は区別する。後者で`release`を即座に呼ぶと、既に送信された取引の入力を次の送金が選ぶ可能性がある。TX IDによる照会とUTXOの再照合を先に行う。受理・消費済みの入力は候補から外す必要もあるが、前節の教材にはその同期処理やDB永続化は含まれていない。

## 24.15 ウォレットのステートマシン

| 状態 | イベント | 次の状態 | 必須チェック |
|---|---|---|---|
| Draft | UTXO選択 | Prepared | 入力の重複なし |
| Prepared | 利用者承認 | Approved | 宛先・金額・手数料一致 |
| Approved | 署名 | Signed | 署名対象と承認内容一致 |
| Signed | 送信 | Broadcast / SubmitUnknown | NetworkId照合、送信前からTX ID・予約ID・支払い意図を関連付け |
| Broadcast | コンセンサス観測 | Observed | トランザクション識別子一致 |
| Observed | 確認ポリシー充足 | Settled | 同期・再編成条件を考慮 |
| SubmitUnknown | 成否の照会 | Broadcast / Observed / Reconcile | 既知のTX IDを使い、別取引を直ちに構築しない |
| Broadcast / Observed / Settled | 競合・再編成 | Reconcile | 受理状態・UTXOと業務記録を再評価 |

**不変条件の例：**

1. 秘密鍵はネットワーク層へ渡さない。
2. 署名対象の受取先・金額・手数料は承認後に黙って変更しない。
3. 同一ウォレット内の未完了送金が同じUTXOを同時予約しない。
4. 送信タイムアウトだけで未送信と断定しない。
5. 同期が不完全なノードの観測を確定判定の唯一の根拠にしない。
6. 失敗した操作は、既存の鍵情報・UTXO状態を壊さない。

## 24.16 `rusty-kaspa` のコードを読む

公式リポジトリ： [kaspanet/rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)

調査の際は**特定のコミットに固定**し、次の観点でコードを追う。

1. Wallet関連のRustクレートとWASM公開APIの境界。
2. アドレス生成、鍵導出、トランザクション構築の責務分担。
3. RPC / wRPCとの接続部分と再接続処理。
4. UTXOイベントの反映、残高更新、再編成時の挙動。
5. 鍵保管・暗号化・シークレット管理の実装。
6. 署名方式・Script・ネットワーク別のテストベクトル。

調査用コマンド例：

```bash
# 第1章で取得した固定版の親ディレクトリから実行する
cd rusty-kaspa-v1.0.1
git rev-parse HEAD
grep -n -E 'build_derivate_path|create_address' wallet/keys/src/derivation/gen1/hd.rs
grep -n -E 'try_sign|fully_signed|zeroize' wallet/core/src/tx/generator/signer.rs
grep -n -E 'stage|relay' wallet/core/src/tx/generator/generator.rs
```

未取得なら[第1章](../part-01-foundations/chapter-01.md)の手順を使い、HEADを本章冒頭の完全SHAと照合する。検索は固定版の主要箇所への入口であり、全APIを列挙するものではない。

## 24.17 Devnetでの実験計画

**目的：** ウォレットの状態とネットワークの状態が一致しないケースを観察する。

| 実験 | 操作 | 確認項目 |
|---|---|---|
| 1 | Devnetウォレットで受取アドレスを生成 | ネットワークとアドレス形式 |
| 2 | テスト資金を受け取る | UTXO検出までの時間 |
| 3 | 送金トランザクションを構築 | 入力・出力・変更・手数料 |
| 4 | 送信直後にノードを切断 | タイムアウト時の再照合 |
| 5 | 再接続して履歴を照会 | 二重送信・重複表示の防止 |
| 6 | 同期の遅いノードを参照 | 残高・確定判定の変化 |

**記録すべき項目：** `rusty-kaspa`コミットID、ネットワーク、ノード起動引数、SDKバージョン、操作時刻、トランザクションID、観測結果。秘密鍵・ニーモニックは記録しない。

### 実測と未実施の境界

[第28章](chapter-28.md)の `send.cjs` は、公式v1.0.1 SDKでランダムな一時鍵からDevnetアドレスを生成し、成熟した報酬から単一署名送金する。`--checks` では入力・出力から手数料とお釣りを検算し、残高不足・不正署名・競合入力の拒否を確認した。`--retry-checks` の障害注入は送信成功応答の喪失であり、上表のノード停止・遅延をすべて試したものではない。

この実習は秘密鍵をメモリ内だけで扱い、保存も表示もしない。gen1のHD導出・同じ素材からの鍵/アドレス再構成・メッセージ署名は24.12の別のオフライン試験で確認した。24.12のDevnet試験では復旧受取鍵と復旧変更鍵による送金2件も確認した。実ノードからの未知indexのアドレス再発見、ウォレット履歴の復旧は未実施。UTXOの現在状態を補完する実習の成功も、ウォレット復旧の成功とは別である。

## 24.18 よくある誤解

<details class="misconception-details">
<summary>よくある誤解を見る</summary>

**1. ウォレットを削除するとKASも消える？**

UTXO自体はネットワーク上に残る。ただし復旧素材を失えば支出権限を再現できなくなる。

**2. シードフレーズがあればどのウォレットでも必ず復旧できる？**

いいえ。導出方式・パス・パスフレーズ・アドレス形式の互換性が必要。

**3. 送信APIが成功すれば決済完了？**

いいえ。Mempool、ブロックへの取り込み、コンセンサス受理、サービスの確認ポリシーは別。

**4. ハードウェアウォレットならすべて安全？**

いいえ。署名内容の確認、復旧素材の保護、ファームウェアやサプライチェーンへの信頼が必要。

**5. UTXOの合計額がそのまま送金可能額？**

いいえ。手数料、入力選択、mass、予約・未確定状態などの制約がある。

</details>

## 24.19 実践課題

<details class="exercise-details">
<summary>実践課題を見る</summary>

1. **C4モデル：** Wallet UI、Wallet Service、Signer、Node Gateway、Kaspa Nodeの境界を描く。
2. **脅威分析：** ホットウォレットの秘密鍵が漏えいした場合の被害範囲と防止策を表にする。
3. **Rust：** 24.14の予約IDをTX ID・支払い意図へ関連付け、永続化と再起動後の再照合を設計する。期限切れだけで自動解除しない。
4. **テスト：** 送信が成功したのにレスポンスだけ失われたケースをモデル化する。
5. **復旧：** テスト用のHDウォレットで受取アドレスと変更アドレスを再現する。実資産を使わない。

</details>

## 24.20 理解度チェック

<details class="quiz-answers">
<summary>理解度チェックを見る</summary>

1. ウォレットがKASそのものを保存していないとはどういう意味か。
2. ニーモニックと導出パスはそれぞれ何を決めるか。
3. 変更出力を追跡しなければ何が起こるか。
4. Watch-onlyウォレットができること・できないことは何か。
5. 送信タイムアウト時に入力予約を即時解除してはいけない理由は何か。
6. 業務上の複数承認と暗号学的マルチシグはどう違うか。
7. 「残高」「送信可能額」「確定済み額」を分ける理由は何か。
8. 署名デバイスが宛先を表示・確認することにはどんな意味があるか。

**解答例**

1. ウォレットが管理するのは鍵・アドレス・取引情報等で、支出可能な出力の状態はネットワークの合意で管理されるという意味である。
2. ニーモニックは所定の方式で復旧用シードへ結び付く語列、導出パスは鍵の階層上の位置を指定する。方式・パス・追加パスフレーズ等も合っていなければ同じ鍵・残高を復元できない。
3. 自分に戻した出力を見落とし、残高・履歴・復旧対象を過小評価する可能性がある。変更側アドレスも追跡する。
4. 公開情報を使ったアドレス導出・観測等は可能だが、支出に必要な秘密鍵がなければ署名できない。利用できる公開情報の範囲も確認する。
5. 応答が失われただけで取引は受理されている可能性があるため。同じ入力を別の取引へ再使用する前に、TX IDと現在UTXO・予約状態を再照合する。
6. 業務上の複数承認はアプリ内の手続き、暗号学的マルチシグは所定の署名条件を支出検証で強制する仕組みである。前者だけではアプリを迂回した支出を防げない。
7. 所有する観測額、成熟・予約等を考慮した利用可能額、所定の確認ポリシーを満たす額は一致しないことがあるから。表示条件を明示する。
8. ホスト側の表示が改ざんされても、署名対象の送金先・金額等を独立した経路で確認するため。表示と実際に署名する内容の一致が必要である。

</details>

## 24.21 まとめ

<details class="summary-details">
<summary>まとめを見る</summary>

ウォレットの本質は、**鍵による支出権限の管理**と**UTXOの状態追跡**を、安全な境界で結び付けることにある。暗号技術だけでは十分ではない。導出互換性、バックアップ、承認フロー、並行送金、ノード同期、再編成、秘密情報の保護まで含めて設計する必要がある。

Kaspaアプリケーションでは、ウォレットが観測する状態とコンセンサス上の状態を混同せず、RPC・SDK・Signer・業務ロジックの依存方向を明確にすることが重要である。

</details>

[次章 Chapter 25 — Exploring the rusty-kaspa Codebase](chapter-25.md) では、これらの責務が実際のRustコードでどのように分割されているかを調査する。

## 24.22 参考資料

<details class="reference-details">
<summary>参考資料を見る</summary>

- [Kaspa公式ドキュメント](https://docs.kaspa.org/)
- [Kaspa公式リポジトリ — rusty-kaspa](https://github.com/kaspanet/rusty-kaspa)
- [Kaspa公式Wiki](https://wiki.kaspa.org/en/home)
- [BIP-32: Hierarchical Deterministic Wallets](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki)
- [BIP-39: Mnemonic code for generating deterministic keys](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki)
- [BIP-44: Multi-Account Hierarchy for Deterministic Wallets](https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki)
- [OWASP Cryptographic Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html)

> **残る確認：** legacy/gen0を含む導出互換性、暗号化バックアップ/履歴探索を含むHDウォレット全体の復旧、鍵キャッシュの寿命と全終了経路の消去、マルチシグ・ハードウェア機器の対応、全Generator経路は未検証。過去の単一署名Devnet送金・所定の拒否・応答喪失と、今回の入力予約教材は別の検査である。記録は `experiments/chapter-24-review-2026-11-01.md`。

</details>

<nav class="chapter-next" aria-label="次のページ">

[次章：Chapter 25 — Exploring the rusty-kaspa Codebase（rusty-kaspaのコードベースを読む）](chapter-25.md)

</nav>

<details class="verification-details">
<summary>検証状況を見る</summary>

[対象バージョン](../baseline.md)・[このページのレビュー記録](../review-status.md#review-chapter-24)

</details>
