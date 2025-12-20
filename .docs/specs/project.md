# プロジェクト仕様

## プロジェクト概要

- Windows 向けのコマンドランチャー（常駐してホットキーで呼び出す想定）
- 設定ファイルに定義されたコマンドを実行する
- env ファイルに定義された環境変数を、コマンド実行時に子プロセスへ渡す

## 対象 OS

- Windows を主対象とする

## プロジェクト構造

- src/main.rs
  - エントリーポイント
  - 設定読み込みと簡易 CLI を提供する
- src/paths.rs
  - 設定ファイルの探索パスを提供する
- src/config.rs
  - setting.yaml と env.yaml のデシリアライズと読み込み
- src/runner.rs
  - 設定に基づくプロセス起動

## 設定ファイル

- 設定ディレクトリ
  - `~/.config/command-launcher/`
- setting.yaml
  - コマンド一覧を定義する
- env.yaml
  - 環境変数（キーと値）を定義する

## データ構造

- Settings
  - commands: CommandSpec の配列
- CommandSpec
  - name: コマンド識別子
  - program: 実行ファイル
  - args: 引数配列（省略可）
  - cwd: 作業ディレクトリ（省略可）
- EnvVars
  - 環境変数のマップ（キーと値）

## 現時点の実装範囲

- 実装済み
  - 設定パス解決
  - YAML 設定読み込み
  - 環境変数の読み込み
  - 設定に基づくコマンド起動
- 未実装
  - 常駐
  - タスクトレイ
  - グローバルホットキー
  - フルスクリーン判定とホットキー無効化
  - UI
