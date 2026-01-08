---
name: report_from_last_commit
description: 最新コミットで行った実装の仕様書更新と報告書作成

tools: ["execute", "read", "edit", "search"]
---

- 直前のコミットの実装が完了した後に呼び出されます。
- 実装内容をもとに、仕様書の更新と実装報告書の作成を行います。
- 実装内容に関しては git のコマンドにて確認します

## 実行手順

### フェーズ 1: ターミナルで変更内容を確認

実行 shell は `nushell`なので下記コマンドを実行してください。

```nushell
let repo_root = (git rev-parse --show-toplevel | str trim)

git -C $repo_root --no-pager show --name-status HEAD
git -C $repo_root --no-pager show --stat HEAD
git -C $repo_root --no-pager diff HEAD~1 HEAD
```

### フェーズ 2: 変更内容を解析

- 実装内容の概要を抽出
- 達成された成果物を列挙

### フェーズ 3: 仕様書更新

- `.docs/specs/` ディレクトリ内の仕様書を確認
- 仕様書が存在しない場合は新規作成
- 実装結果をもとに仕様書の作成・更新おこなう

### フェーズ 4: 実装報告書の作成

- `.docs/reports/` ディレクトリ内に実装内容の報告書を作成
- 参照コーディング計画: `<コーディング計画のファイル名>` について参照コーディング計画は存在しないので記載しない
