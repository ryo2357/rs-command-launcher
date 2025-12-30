# 実装報告書: local_commands を設定可能に

作成日時: 2025-12-30 16:32

## 実装内容の概要

- 設定ディレクトリ配下に local_commands.yaml を追加し、ローカル環境用のコマンド定義を読み込めるようにした
- local_commands.yaml は任意ファイルとして扱い、存在しない場合は従来どおり setting.yaml のみで動作する
- 同名コマンドが存在する場合は local_commands.yaml 側の定義が優先される

## 実装した機能

- local_commands.yaml の読み込み
  - `~/.config/command-launcher/local_commands.yaml` を読み込む
  - YAML 形式は setting.yaml と同様に commands 配下へ配列で定義する
- コマンド定義のマージ
  - setting.yaml の読み込み結果に対して local_commands.yaml の定義を追加する
  - name が重複する場合は最後に追加された定義を採用するため、local_commands.yaml が優先される

## 変更点

- 設定サンプル
  - .docs/.config/local_commands.yaml を追加した
  - .docs/.config/setting.yaml の改行を調整した
- 設定ロード処理
  - src/config.rs に local_commands.yaml の探索と読み込み処理を追加した
- エントリーポイント
  - src/main.rs のログ初期化処理を調整した

## 影響範囲と互換性

- local_commands.yaml は任意ファイルのため、既存環境への影響はない
- 同名コマンドが両方に定義されている場合、優先される定義が変わるため注意が必要

## 動作確認

- 実装報告書作成時点では未実施
