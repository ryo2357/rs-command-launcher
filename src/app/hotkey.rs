use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use log::{error, info};

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    MOD_ALT, RegisterHotKey, UnregisterHotKey, VK_SPACE,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, PostMessageW, PostThreadMessageW, TranslateMessage,
    WM_HOTKEY, WM_NULL, WM_QUIT,
};

use crate::app::endpoint::{HotkeyCmd, HotkeyEndpoint, HotkeyEvent};

// Alt+Space のホットキー検知と通知を行う
// 複数のホットキーを扱う場合は改修が必要
pub struct Hotkey {
    endpoint: HotkeyEndpoint,
    hwnd: Option<HWND>,
    handle: Option<JoinHandle<()>>,
    hotkey_rx: Option<mpsc::Receiver<()>>,
    thread_id: Option<u32>,
    hotkey_id: Option<i32>,
}

impl Hotkey {
    pub fn new(endpoint: HotkeyEndpoint) -> anyhow::Result<Self> {
        Ok(Self {
            endpoint,
            hwnd: None,
            handle: None,
            hotkey_rx: None,
            thread_id: None,
            hotkey_id: None,
        })
    }

    pub fn run(&mut self) {
        loop {
            // コマンド受信
            if let Ok(cmd) = self.endpoint.rx.try_recv() {
                info!("コントローラーからのコマンド受信");
                match cmd {
                    HotkeyCmd::Register(hwnd) => match self.set_hwnd(hwnd) {
                        Ok(_) => {
                            let _ = self.endpoint.tx.send(HotkeyEvent::RegisterResult(true));
                        }
                        Err(e) => {
                            error!("ホットキー登録に失敗しました: {e:?}");
                            let _ = self.endpoint.tx.send(HotkeyEvent::RegisterResult(false));
                        }
                    },
                    HotkeyCmd::Finish => {
                        break;
                    }
                }
            }

            // hotkey 検知
            if let Some(rx) = &self.hotkey_rx
                && rx.try_recv().is_ok()
            {
                info!("ホットキー検知: Alt+Space");
                let _ = self.endpoint.tx.send(HotkeyEvent::Toggle);
            }
            // CPU負荷を抑えるために短いスリープを挿入
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        // info!("Hotkey スレッド終了");
        let Some(thread_id) = self.thread_id else {
            return;
        };
        let Some(hotkey_id) = self.hotkey_id else {
            return;
        };
        unsafe {
            PostThreadMessageW(thread_id, WM_QUIT, 0, 0);
            UnregisterHotKey(0, hotkey_id);
        }
        info!("Hotkey 登録解除");
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    pub fn set_hwnd(&mut self, hwnd: HWND) -> anyhow::Result<()> {
        let (tid_tx, tid_rx) = mpsc::channel();
        let (tx, rx) = mpsc::channel::<()>();
        let hotkey_id: i32 = 1;

        let handle = thread::spawn(move || unsafe {
            let thread_id = GetCurrentThreadId();
            let _ = tid_tx.send(thread_id);

            let modifiers = MOD_ALT;
            let vk = VK_SPACE;

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
                    let _ = tx.send(());
                }
            }
        });
        let thread_id = tid_rx
            .recv()
            .map_err(|e| anyhow::anyhow!("ホットキースレッドの初期化に失敗しました: {e:?}"))?;

        self.handle = Some(handle);
        self.hwnd = Some(hwnd);
        self.hotkey_rx = Some(rx);
        self.thread_id = Some(thread_id);
        self.hotkey_id = Some(hotkey_id);
        Ok(())
    }
}
