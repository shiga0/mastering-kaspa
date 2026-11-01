# SUMMARY順レビュー 27：第23章「Rust SDKとWASM」

確認日：2026-11-01。Kaspaの基準版v1.0.1、SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063`。本文・図・独自教材・主要バインディングと実習への導線を確認した。全SDKの検証ではない。

## 本文と固定ソース

- kaspa-wrpc-clientのKaspaRpcClient、kaspa-wrpc-wasmのRpcClient、kaspa-wasmの集約とfeatureを具体的な入口として示した。Rust crate、JS公開名、配布物、独自教材を区別した。
- KaspaRpcClient::newはencoding / url / resolver / network_id / subscription_contextを受け取る。WASM側はRpcConfigを使う別の境界であり、Rustの呼び出しをそのままJSへ写す例ではない。
- WASM RpcClient.connectは通知タスクを起動して内部client.connectをawaitする。subscribe_utxos_changedはlistener_idがあればアドレスを変換してstart_notifyをawaitするが、なければログを出してOkを返す分岐がある。正常終了だけでは購読の生存を保証しないと補った。この分岐の実通信による再現は今回行っていない。
- wasm/Cargo.tomlとwasm/src/lib.rsでcdylib、wasm32-sdk / wasm32-rpc等の分岐と再公開を確認した。workspaceのwasm-bindgen指定0.2.100は互換範囲の指定であり、本教材の厳密指定=0.2.100と同じ依存固定を意味しない。
- u64/BigIntとu32/Numberの境界を補足。Rustの整数型やchecked_addは、JS値が変換される前の不正値を自動拒否しない。教材にはJSの型・整数・範囲検査と、初期化を待つHTML例を追加した。
- 金額の例はSompi型を実際の算術へ使い、ゼロ額、不足、加算overflow、差額、u64最大値をテストした。単位の型と値の正当性検査は別である。
- Gateway例は完全なTestnet番号を照合し、通信・復号エラーを保持する。実RPCアダプターではなく、鮮度・必要なindex等は第22章の要件を引き継ぐと明記した。
- 送信前からTX IDと支払い意図の対応を保持し、成否不明、MempoolとVirtualの受理、取り消し、業務確定を区別した。再接続・現在UTXO照会を全履歴/会計台帳の復元と同一視しない。
- ソースの探索手順を固定版へ合わせ、2つの教材プロジェクトを本書の外で作る手順にした。次章、参考資料、既存実測の説明を整理した。

一次資料はこれまで基準に使ったソースアーカイブ。Git checkoutではないため、この回にgit rev-parseでHEADを取得したものではない。次のSHA-256はファイルの識別用であり、全行・全機能の監査を表さない。

| ファイル | SHA-256 |
|---|---|
| `Cargo.toml` | `a4ac0081533750ca27a929b80575463ee59439c412cae748212e24b3bc07cada` |
| `wasm/Cargo.toml` | `26091500c8ff3fad21ecc58249c12cc2faa637b2d8871892c0d05d9f6d826b58` |
| `wasm/src/lib.rs` | `f0da746ac61a497ddc81acbab7ff93338e100f9518f4134de37199972e5ac62b` |
| `rpc/wrpc/client/Cargo.toml` | `ac507909bb92c5a54defbb8412721d7a54bb9d5338849d638e2afc3e973b9a9e` |
| `rpc/wrpc/client/src/client.rs` | `881d126e34b4eb44c3166766a197cb6177ff7d571cd80774f472894d8a94d28f` |
| `rpc/wrpc/wasm/Cargo.toml` | `c7a3fa140c3a624ed8fb308aeca1f3c27e08ab639fb67a53d6f1a850f5a0e56f` |
| `rpc/wrpc/wasm/src/client.rs` | `8d6ebcb66df9ab817d0baea2af1c2fd56d443f7a614dae6b44cc7d747bc5a338` |

## 参照した数値境界・読み込みの一次資料

[wasm-bindgen Numbers](https://wasm-bindgen.github.io/wasm-bindgen/reference/types/numbers.html)で数値型のJS表現と変換時の切り捨て/循環を、[Without a Bundler](https://wasm-bindgen.github.io/wasm-bindgen/examples/without-a-bundler.html)でweb target・ES module・初期化待ち・HTTP配信を確認した。SDKが独自に行う変換まで一律に保証する資料ではない。

## 実行した教材

macOS ARM64、Rust/Cargo 1.90.0、wasm32-unknown-unknown、wasm-pack 0.12.1、wasm-bindgen crate/CLI 0.2.100、Node.js 20.0.0。

リポジトリ外の一時領域へ本文のコードを抽出。23.3の2断片と23.8を連結したnativeプロジェクトは外部依存なし。23.6は別のwasmプロジェクトとした。rustfmtの結果を本文へ反映した。

両プロジェクトで実行：

```bash
cargo test --offline
cargo fmt --all -- --check
cargo clippy --offline --all-targets -- -D warnings
```

- native：9テスト成功（金額4、Gateway5。通信/復号の2エラーは同じテスト内で検査）。
- wasm教材のホスト側単体テスト：6テスト成功。
- 両プロジェクトともfmtとClippy成功。

WASMビルドの初回は、対象targetのRustコンパイルは成功したが、利用可能なwasm-bindgen CLIがなくwasm-packのno-installモードで失敗した。wasm-pack 0.12.1はこのプラットフォームのprebuilt binaryを認識できず、ローカルCLIも見つけられなかった。

対応版CLIだけをcargo installで一時領域へ導入し、そのbinをPATHへ追加したうえで再実行：

```bash
# --rootにはリポジトリ外の専用一時ディレクトリを指定する
cargo install wasm-bindgen-cli --version 0.2.100 --locked --bin wasm-bindgen --root <一時ツールディレクトリ>
# そのbinをPATHへ追加した環境で、wasm教材内から実行する
wasm-pack build --target web --dev --mode no-install -- --offline
```

終了コード0でweb向けJS・WASM・型宣言を生成した。任意のpackage metadata不足とwasm-pack version取得の警告は出た。CLIの導入にはlocked依存のyanked/future-incompatibility警告も出ており、補助ツールの依存監査を済ませたとの意味ではない。これらのCLI依存は教材の依存グラフとは別であり、リポジトリへ追加していない。

教材の一時Cargo.lockのSHA-256：`3b9972514bf5c1dd1c2b97caa387b328b3d6db8aa4e202703aae971b5cf04eda`。

解決した外部依存：bumpalo 3.20.3, cfg-if 1.0.5, log 0.4.35, once_cell 1.21.4, proc-macro2 1.0.107, quote 1.0.47, rustversion 1.0.23, syn 2.0.119, unicode-ident 1.0.27, wasm-bindgen 0.2.100, wasm-bindgen-backend 0.2.100, wasm-bindgen-macro 0.2.100, wasm-bindgen-macro-support 0.2.100, wasm-bindgen-shared 0.2.100。

直接依存の版だけでは推移的依存は固定されない。記載は今回の解決結果であり、新規環境で同じ解決になる保証ではない。Cargo.lock・生成pkg・ビルド成果物・ツールはリポジトリへ追加していない。

## JS/WASM境界の再実行コード

次をWASM教材のルートにcheck-boundary.mjsとして置き、生成済みpkgと本文のpayment.jsを使って `node check-boundary.mjs` を実行する。

Nodeでは生成JSをES moduleとしてdata URLから読み、WASMのバイト列を明示して初期化する。生成JS本文は変更せず、payment.jsのimport先だけを同じdata URLへ置き換える。ブラウザーでのURL/fetch・DOMの検査ではない。

```javascript
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
const glue = readFileSync(new URL("./pkg/kaspa_wasm_lab.js", import.meta.url));
const glueUrl = "data:text/javascript;base64," + glue.toString("base64");
const bindings = await import(glueUrl);
await bindings.default({ module_or_path: readFileSync(new URL("./pkg/kaspa_wasm_lab_bg.wasm", import.meta.url)) });
const wrapper = readFileSync(new URL("./payment.js", import.meta.url), "utf8").replace("./pkg/kaspa_wasm_lab.js", glueUrl);
const { canCoverPayment } = await import("data:text/javascript;base64," + Buffer.from(wrapper).toString("base64"));
const max = 0xffff_ffff;
const cases = [
    [[1000, 900, 100], true],
    [[1000, 901, 100], false],
    [[1000, 0, 10], false],
    [[max, max, 1], false],
    [[max, max, 0], true],
    [[1, 1, 0], true],
    [[0, 1, 0], false],
    [[0, 0, 0], false],
];
for (const [args, expected] of cases) assert.equal(canCoverPayment(...args), expected);
const invalid = [-1, 0x1_0000_0000, NaN, Infinity, -Infinity, 0.5, "1", 1n, undefined, null, true, {}, Symbol("amount")];
let rejected = 0;
for (let position = 0; position < 3; position++) {
    for (const value of invalid) {
        const args = [1000, 900, 100];
        args[position] = value;
        assert.throws(() => canCoverPayment(...args), RangeError);
        rejected++;
    }
}
// Raw bindings convert first: checked_add cannot reject the original JS value.
assert.equal(bindings.can_cover_payment(1000, 0x1_0000_0001, 0), true);
assert.equal(bindings.can_cover_payment(-1, 1, 0), true);
assert.equal(bindings.can_cover_payment(1, 1.9, 0), true);
assert.equal(bindings.can_cover_payment(1, 1, NaN), true);
console.log(`WASM/JS checks: ${cases.length} valid cases, ${rejected} rejected inputs, 4 coercion checks`);
```

実行結果：`WASM/JS checks: 8 valid cases, 39 rejected inputs, 4 coercion checks`。計51項目成功。

不正値13種類をavailable / amount / feeの各位置へ与え、すべてRangeErrorになることを確認。生バインディングでは2^32+1、負数、小数、NaNの変換後にtrueになる例を実際に確認し、JS検査が必要な理由とした。u64のJS境界は本教材で実行しておらず、数値の説明は上記の一次資料に基づく。

## 範囲と制限

公式Node.js/WASM配布SDKを使う過去の送金・通知実測は `devnet-send-review-2026-10-31.md`、`devnet-notifications-2026-10-31.md` 等の記録を参照。本章の独自WASMを使った結果へ読み替えない。

今回はノード起動・採掘・鍵生成・署名・送金・通知購読を再実行していない。ブラウザーのHTTP/MIME/CORS/DOM、実際のRust RPCアダプター、全SDK/feature、無期限自動再接続、HD復旧、全Script、鍵保護の監査は未検証。独自モジュールの6ホストテストとwebビルド、Node境界51項目を確認した範囲である。

## 書籍・公開対象の検査

- mdBook 0.5.4のビルド成功。生成HTML 53ページの内部ファイル・アンカー切れ0件。
- 既存の教材2組がscripts/check-book-examples.pyで一致。手動教材をCIへ追加していない。
- SUMMARYとreview-statusの本文表49項目が同じ順番。次は第24章。
- 実行したRust・JSラッパー・HTMLの内容と本文のコードが一致。HTML自体はブラウザー実行をしていない。
- git diff --check成功。追跡対象・未追跡候補173ファイルの秘密鍵ブロック、既知トークン、認証付きURL、個人ホームパス等のパターン検出0件。gitleaks/trufflehogは未導入で、秘密情報の不存在を保証する監査ではない。
- 新規公開候補はレビュー記録のMarkdownだけ。examplesに新規フォルダーは作成しておらず、鍵・DB・生ログ・生成JS/WASM・ビルドツール・アーカイブは追加していない。コミット・pushはしていない。
