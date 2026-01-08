# egui の UI をホットキーで表示・非表示（Alt+Space）

## 概要

- 目的: egui/eframe の UI（ウィンドウ）をホットキーで表示・非表示にしたい
- 結論: eframe/egui 単体では非アクティブ時にも効くグローバルホットキーは基本的に用意されていないため、Windows では Win32 の `RegisterHotKey` を使って `WM_HOTKEY` を受け、`ShowWindow` で表示・非表示をトグルする構成が実装しやすい

## 要件

- ホットキーで UI の表示・非表示を切り替える
- 割り当てたいキー: `Alt + Space`

## 方針

- Win32 のグローバルホットキーを利用する
  - `RegisterHotKey` で登録
  - `GetMessageW` のメッセージループで `WM_HOTKEY` を受信
  - `IsWindowVisible` で状態を見て `ShowWindow(hwnd, SW_HIDE)` / `ShowWindow(hwnd, SW_SHOWNORMAL)` を切り替え
  - 表示時は `SetForegroundWindow(hwnd)` で前面化
- ホットキー受信用に専用スレッドを立てる
  - `RegisterHotKey` は呼び出しスレッドのメッセージキューに `WM_HOTKEY` を投げるため、専用スレッドで待ち受ける
  - 終了時は `PostThreadMessageW(thread_id, WM_QUIT, 0, 0)` でループを抜ける

## 実装案（Windows 限定）

- 依存関係
  - `windows-sys = { version = "0.52", features = [
  "Win32_Foundation",
  "Win32_UI_WindowsAndMessaging",
  "Win32_UI_Input_KeyboardAndMouse",
  "Win32_System_Threading",
] }`
- モジュール構成
  - `src/ui/windows_hotkey.rs` を追加し、ホットキー登録・メッセージループ・トグル処理をまとめる
  - `src/ui/mod.rs` から `#[cfg(windows)] pub mod windows_hotkey;` で公開
- HWND 取得
  - `eframe::winit::platform::windows::WindowExtWindows` を使い、winit window から `hwnd()` を取得する想定
  - 例: `cc.winit_window.as_ref()?.hwnd()`
- `WindowsHotkeyToggle`（概念）
  - `register_alt_space(hwnd: HWND) -> anyhow::Result<Self>` で登録
  - `Drop` で `WM_QUIT` 投げ + `UnregisterHotKey`（二重解除の可能性はあるが、意図としては終了処理を確実にする）
  - 使う Win32 API/定数の例
    - `RegisterHotKey(0, hotkey_id, MOD_ALT, VK_SPACE)`
    - `WM_HOTKEY`
    - `SW_HIDE`, `SW_SHOWNORMAL`
    - `IsWindowVisible`, `ShowWindow`, `SetForegroundWindow`
    - `GetMessageW`, `TranslateMessage`, `DispatchMessageW`
    - `PostThreadMessageW`, `WM_QUIT`
- `LauncherApp` 側の保持
  - ホットキー登録オブジェクトを `LauncherApp` のフィールドとして保持し、アプリ生存中に Drop されないようにする
  - `cfg(windows)` でフィールド・引数・コンストラクタを分岐し、`cfg` 的に不安定な型を避ける

## コード例

- `Cargo.toml`（依存関係追加）

```toml
[dependencies]
# ...existing...
windows-sys = { version = "0.52", features = [
  "Win32_Foundation",
  "Win32_UI_WindowsAndMessaging",
  "Win32_UI_Input_KeyboardAndMouse",
  "Win32_System_Threading",
] }
```

- `src/ui/windows_hotkey.rs`（ホットキー登録とトグル）

```rust
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use log::{error, info};

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_SPACE;
use windows_sys::Win32::UI::WindowsAndMessaging::{
  DispatchMessageW, GetMessageW, IsWindowVisible, PostThreadMessageW, RegisterHotKey,
  SetForegroundWindow, ShowWindow, TranslateMessage, UnregisterHotKey, MOD_ALT, MSG, SW_HIDE,
  SW_SHOWNORMAL, WM_HOTKEY, WM_QUIT,
};

pub struct WindowsHotkeyToggle {
  thread_id: u32,
  hotkey_id: i32,
  handle: Option<JoinHandle<()>>,
}

impl WindowsHotkeyToggle {
  pub fn register_alt_space(hwnd: HWND) -> anyhow::Result<Self> {
    let (tid_tx, tid_rx) = mpsc::channel();
    let hotkey_id: i32 = 1;

    let handle = thread::spawn(move || unsafe {
      let thread_id = GetCurrentThreadId();
      let _ = tid_tx.send(thread_id);

      // Alt+Space は Windows のシステムメニューと競合しやすい
      let ok = RegisterHotKey(0, hotkey_id, MOD_ALT as u32, VK_SPACE as u32);
      if ok == 0 {
        error!("Alt+Space のホットキー登録に失敗しました（OS予約と競合の可能性）");
        return;
      }
      info!("Alt+Space ホットキーを登録しました");

      let mut msg: MSG = std::mem::zeroed();
      while GetMessageW(&mut msg, 0, 0, 0) > 0 {
        if msg.message == WM_HOTKEY {
          let visible = IsWindowVisible(hwnd) != 0;
          if visible {
            ShowWindow(hwnd, SW_HIDE);
            info!("ウィンドウを非表示にしました");
          } else {
            ShowWindow(hwnd, SW_SHOWNORMAL);
            SetForegroundWindow(hwnd);
            info!("ウィンドウを表示しました");
          }
        }

        TranslateMessage(&msg);
        DispatchMessageW(&msg);
      }

      UnregisterHotKey(0, hotkey_id);
      info!("ホットキーを解除しました");
    });

    let thread_id = tid_rx
      .recv()
      .map_err(|e| anyhow::anyhow!("ホットキースレッドの初期化に失敗しました: {e:?}"))?;

    Ok(Self {
      thread_id,
      hotkey_id,
      handle: Some(handle),
    })
  }
}

impl Drop for WindowsHotkeyToggle {
  fn drop(&mut self) {
    unsafe {
      // GetMessageW ループを抜けさせる
      PostThreadMessageW(self.thread_id, WM_QUIT, 0, 0);
      UnregisterHotKey(0, self.hotkey_id);
    }
    if let Some(handle) = self.handle.take() {
      let _ = handle.join();
    }
  }
}
```

- `src/ui/mod.rs`（モジュール公開）

```rust
#[cfg(windows)]
pub mod windows_hotkey;
```

- `src/ui/launcher.rs`（HWND 取得と保持のイメージ）

```rust
#[cfg(windows)]
use crate::ui::windows_hotkey::WindowsHotkeyToggle;

#[cfg(windows)]
use eframe::winit::platform::windows::WindowExtWindows;

pub fn run(settings: Settings) -> anyhow::Result<()> {
  let native_options = eframe::NativeOptions::default();

  eframe::run_native(
    "command-launcher",
    native_options,
    Box::new(|cc| {
      initialize::initialize(&cc.egui_ctx);

      #[cfg(windows)]
      let hotkey = {
        let w = cc
          .winit_window
          .as_ref()
          .ok_or_else(|| anyhow::anyhow!("winit window が取得できません"))?;
        let hwnd = w.hwnd();
        Some(WindowsHotkeyToggle::register_alt_space(hwnd)?)
      };

      #[cfg(windows)]
      let app = LauncherApp::new(settings, hotkey)?;

      #[cfg(not(windows))]
      let app = LauncherApp::new(settings)?;

      Ok(Box::new(app))
    }),
  )
  .map_err(|e| anyhow::Error::msg(format!("UI を起動できません: {e:?}")))?;

  Ok(())
}

struct LauncherApp {
  // ...existing...

  #[cfg(windows)]
  #[allow(dead_code)]
  hotkey: Option<WindowsHotkeyToggle>,
}

impl LauncherApp {
  #[cfg(windows)]
  fn new(settings: Settings, hotkey: Option<WindowsHotkeyToggle>) -> anyhow::Result<Self> {
    Ok(Self {
      // ...existing...
      hotkey,
    })
  }

  #[cfg(not(windows))]
  fn new(settings: Settings) -> anyhow::Result<Self> {
    Ok(Self {
      // ...existing...
    })
  }
}
```

## ログ・初期化

- `println!` ではなく `log` を使う
- ログ初期化は `main` でのみ行う（既存構成どおり）

## 注意点

- `Alt + Space` は Windows のシステムメニューと競合しやすい
  - `RegisterHotKey` が失敗する可能性がある
  - 失敗した場合はログに出して代替キーに切り替えるのが現実的
- `eframe` バージョン差
  - `cc.winit_window` の取り方はバージョンによって異なる可能性がある
  - コンパイルエラーが出た場合はエラー全文を見て取得方法を合わせる

## 代替案

- `Alt+Space` が取れない場合の候補
  - `Ctrl+Space`
  - `Alt+`（バッククォート）
- キー割り当てを設定ファイル化する（必要なら）
