# SUMMARY順レビュー 37：Argent補足

確認日：2026-11-01。目次途中で見落としていた補足ページを確認し、固定文書・主要実装と学習用チケット例・ローカルruntimeデモを照合した。全security invariant・暗号方式・実ネットワーク・業務アプリの監査ではない。これでPart Vの全ページの本文確認は一区切りとなり、第35章は確認済みなので次は第36章。

## 固定資料・依存

- [Argent](https://github.com/argent-lang/argent/tree/2796809f167144902f04a4b3333a4c144f91fdaf): HEAD 2796809f167144902f04a4b3333a4c144f91fdaf、2026-10-29。
- [Playground](https://github.com/argent-lang/argent-playground/tree/86d5453ee525e7aa743d0b3bb21d1bb8d96c3558): HEAD 86d5453ee525e7aa743d0b3bb21d1bb8d96c3558、2026-10-23。
- 公開リポジトリを本書外へcloneし、ls-remote/実HEADを照合。実行後も両リポジトリの追跡ファイルに差分はなかった。独自probeは本書外のPlayground checkoutに未追跡のbook_invalid_counter.rsとして追加しただけで、本書へは収録していない。
- READMEはnot yet release-readyと説明する。SilverScript v1.0.0へpinというREADME表記に対して、manifest/lockfileのsilverscript-lang/abiは1.0.1。Kaspaの主要依存はcrates.ioの2.1.0。第32章の3ed9733 / SilverScript v1.0.0や本書基準ノードv1.0.1とは別の資料集合。
- MSRVはRust 1.94.0 / edition 2024。第32章で追加済みのcargo +1.94.0を使用し、既定Rustを変更していない。Playgroundは../argentへのpath依存なので、兄弟checkoutの配置を本文へ追加した。
- 最初のoffline CLIビルドは2.1.0のキャッシュが足りず失敗し、承認されたネットワークアクセスで固定依存を取得した。Playgroundの最初のoffline実行もregistry展開先へのsandbox書込制限で失敗し、承認された実行へ切り替えた。その後の再実行はofflineで可能になった。

## 主要説明・コードの対応

固定README、ICC semantics、range設計、security invariantsの主要説明、CLI・runtime/Genesisの主要入口を確認した。

- ICCは同じ取引でCovenantが消費されたことと、その操作を認可したことを区別する。id.co_spentだけで認可範囲は決まらず、署名対象や観測/出力条件と意図して結び付ける。
- artifact IDは依存の同定・整合性に必要だが、それだけでチェーン上の正当なGenesisや業務上の認可を保証しない。
- コンパイラーが補うhidden/internal witnessと、暗号学的な秘密/ゼロ知識性を区別した。signatureScript等へ渡すデータの公開範囲は別に確認する。virtual/expandsのDigestと前像も同様。
- runtimeのTxBuilder::buildはresolve.rs末尾でexecute_transaction_with_covenantsを呼ぶ。Script/Covenant実行、compute budgetの反映、非文脈mass制約等を扱うが、合成UTXOを受け取れるローカル検査であり、チェーン上の存在/未消費・資金調達・Finalityまでは保証しない。
- Genesis crateはConsensus/Sil/Argentという階層の初期化proofを持つ。ZK証明や、チェーン包含/Finalityの証明と同義ではない。今回は全proof検証アルゴリズム・Genesis実行の監査はしていない。
- 固定range文書ではleader consumes/current-entry emitsの範囲を先行実装し、observed/spawned rangesは生成時に拒否する後続段階と説明する。機能表からすべての組合せが使えるとは推定しない。
- leader/delegateやtemplate frameのsecurity invariant文書は主要定義を参照しただけで、全ルールの証明・実装との全面照合ではない。

## 本文チケット例のbuild/inspect

元の本書例は価格の正値条件がなく、Ticketの値をunrestrictedとしていた。require(price > 0)、ticket.value > 0、transfer時のnext.value == self.valueを加えた。公式examples/tickets.agのIssuer/issue/redeemとは別の、購入/譲渡を説明する本書独自の学習例である。

本文コードを本書外lab/tickets.agへ抽出し、完全一致を確認した。

```bash
cargo +1.94.0 run --locked --bin argentc -- --help
cargo +1.94.0 run --offline --locked --bin argentc -- build ../lab/tickets.ag --out ../lab/tickets
cargo +1.94.0 run --offline --locked --bin argentc -- inspect ../lab/tickets
```

いずれも終了コード0。artifact.json、manifest.json、sil/Event.sil、sil/Ticket.silを生成した。

- Event: Script 276 B / state 51 B / template 225 B / opcodes 205 / entry 1。
- Ticket: Script 149 B / state 33 B / template 116 B / opcodes 101 / entry 1。
- inspect上のsignature scriptサイズはEvent::buy 436 B、Ticket::transfer 288 B。あらゆる実引数・実UTXOでの測定値やMainnet資源上限の保証ではない。

価格・値の条件を置いても、返金/期限・初期設定・署名認可・在庫の業務要件を網羅する例ではない。本文Ticketの署名や資金支払いを実行したのではなく、コンパイル・生成物の検査まで。

## Playgroundと負例

PlaygroundのCargo.lockはArgent側と一部の依存versionが異なり、同じ外部targetを共有しても一部が再ビルドされた。依存は更新せず、--lockedで実行。下記の実際の実行ではCARGO_TARGET_DIRを本書外のArgent targetへ向けた。

```bash
cargo +1.94.0 run --locked --bin basic_counter
cargo +1.94.0 run --offline --locked --bin two_actor_exchange
```

- basic_counter: count=2、delta=3、後継count=5を指定。入力1/出力1の取引構築と、build内のローカルScript/Covenant検査が成功。
- two_actor_exchange: Ping→PongとPong→Pingの2取引が成功。名前から同時の複数入力交換や、2取引全体の原子性を証明したと誤解しないよう演習の説明を訂正した。
- 独自book_invalid_counter probe: basic_counterをコピーして後継countだけ6へ変更。buildがBuilderError::InputScript { input_index: 0, .. }を返すことをassertし、想定した拒否で終了コード0。不正状態が成功した場合や別の種類のerrorなら失敗するprobeとした。
- 上記は合成outpoint/UTXOと固定の仮想covenant IDを与えたローカル試験。RPC接続・送信・実チェーン受理・FinalityのE2Eではない。鍵生成・署名・送金は実行していない。

公式runtimeの次の単体テスト1件も成功した。

```bash
cargo +1.94.0 test --offline --locked -p argent-runtime --lib tests::transaction_execution_reports_the_failing_input_index -- --exact
```

1 passed / 27 filtered out。最初にmodule接頭辞なしで--exactを指定した試行は0件だったため、成功件数に算入せず、上の完全名へ訂正して1件の実行を確認した。OpFalseによる失敗入力番号の確認であり、全runtimeテストではない。

## 今回行っていないこと

check.sh全体/--full・全テスト/全デモ、署名を使うCounter/Ticket、ICCの全組合せ・Genesisの実行、実資金・実ノード/E2E、全暗号方式/Scriptの監査は行っていない。check.sh --fullは追跡済み生成例も再生成するため、今回の部分確認と区別した。本書に新規examples・Cargoプロジェクト・鍵・生成物を追加していない。

## SHA-256による資料と生成物の識別

ファイルの主要箇所を読んだことと、全ファイルを監査したことは異なる。生成物は本書外に保持し、hashだけ記録する。

| ファイル | SHA-256 |
|---|---|
| Argent: README.md | `bf8b4218368bfbecf8b80757adc689de7d99b27fdf933f53d7dcfec8f0aa396e` |
| Argent: Cargo.toml | `29c3fd8a59b4e7a3f12f2e0e861792cdd652a3fd91597a1d6894279255f18e0a` |
| Argent: Cargo.lock | `cb63247f489ea39d9273944db91c422909041808fed0db65f9ab5d4c8bbde83d` |
| Argent: check.sh | `01878ae65b95f94c6d105164b01a3622627c2a4588bf76bdc8236b27fe68530f` |
| Argent: examples/tickets.ag | `4d235e56800b8ee3057be7731deb17ad1e061fbc58d86e011ba83c4c0f9c3388` |
| Argent: docs/icc-semantics.md | `b29919095c2273b1c8d91e12fe2da43c52f878386313869368a51ec420b389a6` |
| Argent: docs/entry-clause-ranges.md | `a2de43e54e47a95c85a02fa0ea2babc46e0c2aee639ca5b7c53d15cf73888755` |
| Argent: docs/security-invariants/leader-delegate-input-groups.md | `ee92eafccd98ed8870d4098f52fe357fe84fc4a22492880401a022b97820fdfa` |
| Argent: docs/security-invariants/template-frame-identity.md | `4f7e633e094c80c91f00603e4f4ba73e5a338534acee4267ecccb4bd5bb9c11f` |
| Argent: crates/argent-runtime/src/resolve.rs | `53f6731b1bba76feba1e4a01c15a5f5b10efe6f2f4e40ce9406a6c9b53a2f5b0` |
| Argent: crates/argent-runtime/src/lib.rs | `76a0224636b5e6ddc702cc55d0de42e67e4cfc8db10a5fc4cdc4adcf11936032` |
| Argent: crates/argent-genesis/src/lib.rs | `ff5d8038ff94eda1403577971e1652f14fdcf7c3d6b5f5ec6dcc6ed03f755833` |
| Playground: Cargo.toml | `3c491138d611d775ee1d105613a1a780188250e8c428e22df5e6d0f3a7dc33e0` |
| Playground: Cargo.lock | `846868b50b671032fc27b188fdea3dbfc10eba11e6fa909ae8038c658033cccb` |
| Playground: src/bin/basic_counter.rs | `7c331d57470853452c00e7f6df1cb537bd2688d6f56d11e3578abfce24196f20` |
| Playground: src/bin/two_actor_exchange.rs | `3caf4950b0ffafeb2a1b99fe6b3b14e80524246ec587a0de1bb24d7a00546799` |
| Playground: src/bin/book_invalid_counter.rs | `05878f8c3e7ad3dc7f5063d3d016f1ea9c14c30ea9433449ddacaf2841673050` |
| Lab: tickets.ag | `ae84db69f614f518502fcfbff30f3ecf29471789839c81aa292310413ed39ee4` |
| Lab: tickets/artifact.json | `8e5ee712727c2cb7299ad1d31e104339bb705feba12064c7714dfa8d95315333` |
| Lab: tickets/manifest.json | `d927b92f9f71ce34647d093fd4538f0c556c0ff026931b13d7be2e30da85e2de` |
| Lab: tickets/sil/Event.sil | `a03f2653c25157db5ee61cf90683126f4bdbaf6607bd4daa112deb9134df46c3` |
| Lab: tickets/sil/Ticket.sil | `89028133548fee8a9f3d69a948d72a52ce59ce2ade963bf6f3731df1783c45c9` |

inspectのArtifact ID（ファイル全体のSHA-256とは別）: `1d607e29773af3f51f01d2688c2eb456ed32c3db2a5aa5818559dbb0d1c27f99`。

## 書籍・公開対象の検査

- mdBook 0.5.4でビルド成功。生成HTML 53ページの内部ファイル・アンカー切れ0件。
- scripts/check-book-examples.pyで既存教材2組が一致。ArgentのCLI/ローカル例は本書外での手動検査で、CI対象を増やしていない。
- SUMMARYとreview-statusの本文表49項目、進捗表1〜41の番号/順序が一致。Argentの未確認を回収し、次は第36章、その次は付録A。git diff --check成功。
- 追跡対象・未追跡候補186ファイルの秘密鍵ブロック・既知トークン・認証付きURL・個人ホームパス等のパターン検出0件。gitleaks/trufflehogは未導入で、秘密情報の不存在を保証する監査ではない。
- 本書に追加した公開候補は本文・進捗・レビュー記録のMarkdown。コンパイラーclone・生成Script/Artifact・独自probe・鍵・DB・生ログ・バイナリー・アーカイブは追加していない。コミット・pushもしていない。
