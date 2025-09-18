# Pgbouncer Config Rust

PgBouncer の設定ファイル（pgbouncer.ini）を、ライブラリと CLI の両方から安全に生成・変更・比較できる Rust ワークスペースです。

このワークスペースは次の 2 クレートで構成されています。

- pgbouncer-config: PgBouncer 設定（[pgbouncer.ini] の内容）を型安全に表現・構築・解析（パース）・差分取得・入出力するためのライブラリ
- pgbouncer-generator: 上記ライブラリを用いた CLI。中間定義ファイル（TOML/JSON）から pgbouncer.ini を生成したり、差分表示を行えます


## 特徴

- PgBouncer 設定（[pgbouncer] と [databases] セクション）を Rust の型として安全に扱える
- pgbouncer.ini 文字列の生成・既存 pgbouncer.ini のパースに対応
- TOML/JSON 形式の中間定義ファイル（構造化データ）を読み書き可能
- 複数の PostgreSQL ホストからデータベース一覧を取り込み（import）可能
- 現在の pgbouncer.ini と中間定義ファイルの差分を JSON で取得・表示可能
- CLI から一連の操作（初期化、テンプレート追加、DB 追加、インポート、差分、生成）を実行


## ワークスペース構成

- pgbouncer-config: ライブラリ crate（edition = 2024）
- pgbouncer-generator: CLI crate（edition = 2024）
- generated/: サンプルの中間定義ファイルと pgbouncer.ini 出力先

ワークスペースのメタデータは Cargo.toml（workspace）をご参照ください。


## 必要要件

- Rust 1.80 以降（edition 2024 対応の安定版を推奨）
- PgBouncer のドメイン知識（項目名や意味）
- PostgreSQL へ接続して情報を取得する場合は適切なネットワーク/資格情報


## ビルドとテスト

- 全体ビルド
  - cargo build
- ライブラリのテスト
  - cargo test -p pgbouncer-config


## インストール（CLI のみ）

ワークスペースから pgbouncer-generator をインストール（ローカル）

- cargo install --path pgbouncer-generator

インストールせずに実行する場合は cargo run を使います。


## 使い方（CLI: pgbouncer-generator）

次のサブコマンドを提供します。引数のデフォルトはソースをご参照ください（src/main.rs）。

- init: 中間定義ファイル（TOML）を初期生成
- add-empty-pg-template: 空の Postgres テンプレートを中間定義に追加
- add-pg: 1 台の Postgres 情報（ホスト、ポート、資格情報、DB 一覧など）を中間定義に追加
- import: 指定した Postgres ホスト群からデータベース名を取り込み、中間定義へ反映
- diff: 現在の pgbouncer.ini と中間定義の差分を JSON で表示
- generate: 中間定義から pgbouncer.ini を生成

基本パス（既定値）

- 中間定義: ./generated/pgbouncer_definition.toml
- 出力 pgbouncer.ini: ./generated/pgbouncer.ini

実行例

1) 初期化（definition を生成）

- cargo run -p pgbouncer-generator -- init

既存ファイルを上書きする場合

- cargo run -p pgbouncer-generator -- init --force-overwrite

2) 空の Postgres テンプレートを追加

- cargo run -p pgbouncer-generator -- add-empty-pg-template

3) Postgres 情報を追加（ユーザー/パスワード、対象 DB、無視する DB、資格情報の出力有無など）

- cargo run -p pgbouncer-generator -- add-pg \
    --host localhost \
    --port 5432 \
    --user postgres \
    --password postgres \
    --databases db1 db2 db3 \
    --ignore-databases template0 template1 \
    --is-output-credentials-to-config false \
    --allow-not-exist true

4) Postgres ホストから DB 一覧を取り込み

- cargo run -p pgbouncer-generator -- import \
    --target-postgres-host 10.0.0.10 10.0.0.11

5) 差分表示（definition と現在の pgbouncer.ini を比較）

- cargo run -p pgbouncer-generator -- diff \
    --path-def-file ./generated/pgbouncer_definition.toml \
    --path-pgbouncer-ini ./generated/pgbouncer.ini

6) 生成（definition から pgbouncer.ini を作成）

- cargo run -p pgbouncer-generator -- generate

上書きしたくない場合は --allow-overwrite false を指定してください。


## ライブラリ利用例（pgbouncer-config）

依存関係（Cargo.toml）

```toml
[dependencies]
pgbouncer-config = { version = "0.1", git = "https://github.com/SHIMA0111/pgbouncer-config-rs" }
# もしくはローカルパスを指定
# pgbouncer-config = { path = "../pgbouncer-config" }
```

サンプルコード

```rust,no_run
use pgbouncer_config::builder::PgBouncerConfigBuilder;
use pgbouncer_config::pgbouncer_config::databases_setting::{Database, DatabasesSetting};
use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
use pgbouncer_config::pgbouncer_config::PgBouncerConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // databases セクションの組み立て
    let mut db_setting = DatabasesSetting::new();
    let db = Database::new("localhost", 5432, "postgres", "postgres", None);
    db_setting.add_database(db);

    // pgbouncer セクションの組み立て
    let mut pgbouncer_setting = PgBouncerSetting::default();
    pgbouncer_setting.set_listen_addr("0.0.0.0");

    // 全体設定を構築
    let config: PgBouncerConfig = PgBouncerConfigBuilder::builder()
        .set_databases_setting(db_setting)?
        .set_pgbouncer_setting(pgbouncer_setting)?
        .build();

    // INI 文字列へ
    let ini_text = config.expr();
    println!("{}", ini_text);

    Ok(())
}
```

入出力（I/O）

- INI 読み込み

```rust,ignore
use std::io::Cursor;
use pgbouncer_config::io::read::Reader;

let ini = "[pgbouncer]\nlisten_addr = 127.0.0.1\nlisten_port = 6432\nauth_type = md5\nmax_client_conn = 100\ndefault_pool_size = 20\npool_mode = session\n";
let mut reader = Reader::new(Cursor::new(ini.as_bytes()));
let cfg = reader.read()?; // pgbouncer.ini から構造体へ
```

- INI/TOML/JSON 書き出し

```rust,ignore
use pgbouncer_config::io::{write::Writer, ConfigFileFormat};
use std::fs::File;
use std::path::Path;

let mut writer_ini = Writer::try_from(pgbouncer_config::io::write::Writers::File(Path::new("./generated/pgbouncer.ini")))?;
writer_ini.write(&cfg)?; // INI として出力

let mut writer_toml = Writer::try_from(pgbouncer_config::io::write::Writers::File(Path::new("./generated/pgbouncer_definition.toml")))?;
writer_toml.write_config(&cfg, ConfigFileFormat::TOML)?; // TOML として出力
```

差分の計算

- 現在の pgbouncer.ini と中間定義（TOML/JSON）から差分を計算し、JSON 表示できます（CLI の diff 参照）。

## ライセンス

- ライブラリ/ツールのライセンスはリポジトリ同梱の LICENSE-mit.md および LICENSE-ap.md を参照してください（Apache-2.0 / MIT のデュアルライセンス）。


## 貢献

- Issue / PR 歓迎です。コード規約やコミットポリシーは一般的な Rust コードスタイルに従ってください。
- テストの追加も歓迎します（特に I/O と差分処理周辺）。


## 謝辞

- PgBouncer プロジェクトおよび Rust コミュニティに感謝します。
