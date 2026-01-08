# egui/eframe を Windows タスクトレイ（通知領域）と連携して表示切替・終了

## 概要

- 目的: egui/eframe アプリを非表示にしても起動中と分かるようにタスクトレイへアイコン表示し、そこから終了できるようにする（Windows のみ）
- 結論: egui にタスクトレイ機能は無いので、Win32 の通知領域 API（`Shell_NotifyIconW`）を使い、メッセージ受信用の不可視ウィンドウ + メッセージループを別スレッドで回す構成にする。egui 側は `HWND` を渡し、表示/非表示は `ShowWindow` でトグル、終了はトレイからイベント通知して `frame.close()` で閉じる

## 前提

- 対象 OS は Windows のみ
- egui/eframe 単体ではトレイ常駐やグローバルホットキー等の OS 機能は提供しない

## 実装方針（Win32）

- 通知領域（タスクトレイ）

  - 登録/更新/削除: `Shell_NotifyIconW(NIM_ADD / NIM_MODIFY / NIM_DELETE, ...)`
  - コールバック受信: `NOTIFYICONDATAW.uCallbackMessage` に `WM_APP + n`（例: `WM_APP + 1`）を指定して独自メッセージを受け取る
  - 左クリック: `WM_LBUTTONUP` で「表示/非表示のトグル」
  - 右クリック: `WM_RBUTTONUP` でメニュー表示
    - `CreatePopupMenu` / `AppendMenuW` / `TrackPopupMenu` / `WM_COMMAND`
  - 表示/非表示: `ShowWindow(hwnd, SW_HIDE / SW_SHOWNORMAL)`、必要なら `SetForegroundWindow(hwnd)`
  - 終了: トレイから「終了要求」をアプリへ伝え、eframe 側で `frame.close()`

- メッセージ受信用の不可視ウィンドウ

  - `RegisterClassW` → `CreateWindowExW` で不可視ウィンドウを作る
  - そのスレッドで `GetMessageW` ループを回し `WM_TRAY` や `WM_COMMAND` を処理する

- スレッド分離
  - トレイ処理は専用スレッドに置く（メッセージループを持つため）
  - アプリ終了時は Drop 等でスレッド終了できるようにする設計が楽

## egui/eframe 側との連携（重要ポイント）

- `WindowsTray::start()` を呼ぶ位置

  - `run()` の中で呼ぶが、`eframe::run_native` の「生成クロージャ（`Box::new(|cc| { ... })`）」の中で呼ぶ
  - 理由: `HWND` を得るには eframe のウィンドウ生成後（`CreationContext` が渡るタイミング）が必要

- `HWND` の取得

  - `eframe::winit::platform::windows::WindowExtWindows` を使い、winit window から `hwnd()` を取る
  - 例: `let hwnd = cc.winit_window.as_ref()?.hwnd();`

- トレイオブジェクトの保持

  - `WindowsTray` を `LauncherApp` のフィールドとして保持し、アプリ生存中に Drop されないようにする
  - `cfg(windows)` で型を分ける（非 Windows ビルドを壊さない）

- 終了要求の受け渡し
  - トレイスレッド → eframe へは `std::sync::mpsc` などで `TrayEvent::ExitRequested` を送る
  - `LauncherApp::update()` 内で `rx.try_recv()` を回してイベントを吸い上げ、`frame.close()` を呼ぶ

## API / 定数（会話で触れたもの）

- トレイ

  - `Shell_NotifyIconW`
  - `NOTIFYICONDATAW`
  - `NIM_ADD`, `NIM_MODIFY`, `NIM_DELETE`
  - `NIF_MESSAGE`, `NIF_ICON`, `NIF_TIP`
  - `NOTIFYICONDATAW.uCallbackMessage`

- クリック/メニュー

  - `WM_LBUTTONUP`, `WM_RBUTTONUP`, `WM_COMMAND`
  - `CreatePopupMenu`, `AppendMenuW`, `TrackPopupMenu`, `DestroyMenu`

- ウィンドウ表示
  - `ShowWindow`, `SW_HIDE`, `SW_SHOWNORMAL`
  - `SetForegroundWindow`

## 依存関係（例）

- `windows-sys` を使用
  - 追加候補 feature
    - `Win32_UI_Shell`
    - `Win32_UI_WindowsAndMessaging`
    - `Win32_System_Threading`

## 注意点

- `TaskbarCreated` 対応

  - Explorer 再起動等でトレイが消える場合がある
  - `RegisterWindowMessageW(L"TaskbarCreated")` でメッセージを受けたら `NIM_ADD` し直す実装が堅い

- アイコン

  - 最初は既定アイコンでもよいが、`LoadImageW(..., IMAGE_ICON, ..., LR_LOADFROMFILE)` で `.ico` を読み込む構成が一般的

- 実装の最小化
  - 表示/非表示は OS レベルで `ShowWindow` を叩けばよく、egui 側に特別な「トレイ API」は不要
  - 終了だけは `frame.close()` を呼びたいので「トレイ → eframe のイベント通知」を入れるのが扱いやすい
