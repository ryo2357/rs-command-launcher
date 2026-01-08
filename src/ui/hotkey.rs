use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use log::{error, info};

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    MOD_ALT, RegisterHotKey, UnregisterHotKey, VK_SPACE,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, IsWindowVisible, MSG, PostThreadMessageW, SW_HIDE,
    SW_SHOWNORMAL, SetForegroundWindow, ShowWindow, TranslateMessage, WM_HOTKEY, WM_QUIT,
};

pub struct HotkeyToggle {
    thread_id: u32,
    hotkey_id: i32,
    handle: Option<JoinHandle<()>>,
}

impl HotkeyToggle {
    pub fn register(hwnd: HWND) -> anyhow::Result<Self> {
        // 将来的に設定からホットキーを変更できるようにする
        let (tid_tx, tid_rx) = mpsc::channel();
        let hotkey_id: i32 = 1;

        let handle = thread::spawn(move || unsafe {
            let thread_id = GetCurrentThreadId();
            let _ = tid_tx.send(thread_id);

            let modifiers = MOD_ALT;
            let vk = VK_SPACE;

            // Alt+Space は Windows のシステムメニューと競合しやすい
            #[allow(clippy::unnecessary_cast)]
            let ok = RegisterHotKey(0, hotkey_id, modifiers as u32, vk as u32);
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
                    } else {
                        ShowWindow(hwnd, SW_SHOWNORMAL);
                        SetForegroundWindow(hwnd);
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

impl Drop for HotkeyToggle {
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
