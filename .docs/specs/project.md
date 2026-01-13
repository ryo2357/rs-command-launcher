# プロジェクト仕様

## プロジェクト概要

- Windows 向けのコマンドランチャー（常駐してホットキーで呼び出す想定）
- 設定ファイルに定義されたコマンドを実行する
- env ファイルに定義された値を、設定ファイル中の $ 変数として置換してから実行する

## 対象 OS

- Windows を主対象とする

## プロジェクト構造

- src/main.rs
  - エントリーポイント
  - 設定読み込みと簡易 CLI を提供する
- 引数なしの場合はアプリ本体（Controller + Hotkey + UI）を起動する
- src/model/mod.rs
  - ドメインモデルのモジュール定義
- src/model/commands.rs
  - コマンド定義と操作（検索、マージ、重複排除、変数展開）
- src/config.rs
  - 設定ファイルの探索パス解決と setting.yaml / env.yaml の読み込み
  - 読み込み用の構造体（LoadSettings / LoadEnv など）と、UI 向けの Settings への変換
- src/runner.rs
  - 設定に基づくプロセス起動
- src/app/mod.rs
  - アプリ層（UI 以外の常駐処理）
- src/app/endpoint.rs
  - Controller と各コンポーネント間の通信用エンドポイント定義（mpsc）
- src/app/controller.rs
  - 司令塔
  - UI から受け取った HWND を保持し、ホットキーのトグル通知で表示/非表示を切り替える
- src/app/hotkey.rs
  - Windows のグローバルホットキー登録（Alt+Space）
  - 検知結果を Controller へ通知する
- src/app/task_tray.rs
  - タスクトレイ（アイコン + メニュー）実装の置き場
  - 現状は起動経路・Controller 連携が未実装
- src/ui/mod.rs
  - UI 関連モジュール定義
- src/ui/launcher.rs
  - 最小 UI（コマンド名入力と実行）
  - eframe/egui による単一ウィンドウ
  - 初回 update 時に Frame から HWND を取得し Controller へ通知する
- src/ui/native_runner.rs
  - UI 起動処理（eframe::run_native）のエントリーポイント
  - egui の初期化（フォント設定など）

## エントリーポイント構成

- `main` はログ初期化と `app()` の実行、失敗時の終了コード制御を担当する
- 実処理は `app()` に集約し `anyhow::Result<()>` を返す
- `app()` が失敗した場合、`main` はプロセス終了コード 1 で終了する

## ログ

- ログ出力は `log` クレートの `info!` と `error!` を使用する
- ログの初期化は `main` 関数で `init_logger()` を呼び出して行う
- ログレベル
  - Debug ビルドは Info 以上を出力する
  - Release ビルドは Warn 以上を出力する
- 簡易 CLI の出力
  - `list` はコマンド一覧（置換後）をログ出力する
  - `run-first` と `run` は起動したコマンド名をログ出力する

## 設定ファイル

- 設定ディレクトリ
  - `~/.config/command-launcher/`
- setting.yaml
  - コマンド一覧を定義する
- local_commands.yaml
  - ローカル環境専用の追加コマンド一覧を定義する
  - ファイルが存在しない場合は無視する
  - `setting.yaml` と `local_commands.yaml` の両方に同名のコマンドが存在する場合、`local_commands.yaml` 側が優先される
- env.yaml
  - 置換用の変数（キーと値）を定義する
  - YAML は env 配下にマップを持つ

## 設定サンプル

- 設定サンプルは `.docs/.config/` に配置する

## データ構造

- CommandSpec
  - name: コマンド識別子
  - program: 実行ファイル
  - args: 引数配列（省略可）
- Commands
  - CommandSpec の配列を内包する
  - name の重複は排除される
  - name 指定で検索できる
- EnvVars
  - 置換変数のマップ（キーと値）

## 置換仕様

- `setting.yaml` の `program` と `args` の各要素について、先頭が `$` の場合に置換を行う
- 置換名は `$` を除いた文字列とする（例: `$program` は `program` を参照する）
- env に該当キーが存在する場合はその値に置換する
- 未定義のキーは置換せず、そのままの文字列として残す

## 現時点の実装範囲

- 実装済み
  - 設定パス解決
  - YAML 設定読み込み
  - 置換変数の読み込み
  - 設定（置換後）に基づくコマンド起動
  - 最小 UI（src/ui/launcher.rs）を引数なし起動で呼び出す
  - グローバルホットキーによる UI の表示/非表示切り替え（Alt+Space）
- 未実装
  - 常駐
  - タスクトレイ（表示、終了）
  - フルスクリーン判定とホットキー無効化
