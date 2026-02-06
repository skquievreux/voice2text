// Keyboard Hook for PTT
// Only intercepts F8 and Ctrl+F12. Everything else is passed through.

use std::sync::atomic::{AtomicBool, Ordering};
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::shared::windef::HHOOK;
use winapi::um::winuser::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, GetMessageW, TranslateMessage, DispatchMessageW, PostThreadMessageW,
    HC_ACTION, WH_KEYBOARD_LL, KBDLLHOOKSTRUCT, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP, MSG, WM_QUIT
};

static mut HOOK_HANDLE: HHOOK = std::ptr::null_mut();
static RECORDING: AtomicBool = AtomicBool::new(false);

// Virtual Key Codes
const VK_F8: i32 = 0x77;
const VK_F12: i32 = 0x7B;
const VK_LCONTROL: i32 = 0xA2;
const VK_RCONTROL: i32 = 0xA3;

unsafe extern "system" fn keyboard_hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code == HC_ACTION {
        let kb = *(l_param as *const KBDLLHOOKSTRUCT);
        let vk_code = kb.vkCode as i32;
        let event_type = w_param as u32;
        
        // PTT Logic: F8
        if vk_code == VK_F8 {
            if event_type == WM_KEYDOWN || event_type == WM_SYSKEYDOWN {
                if !RECORDING.load(Ordering::SeqCst) {
                    println!("[HOOK] F8 PRESSED -> START");
                    RECORDING.store(true, Ordering::SeqCst);
                    winapi::um::utilapiset::Beep(1500, 50); // Feedback
                }
                return 1; // Block key
            } else if event_type == WM_KEYUP || event_type == WM_SYSKEYUP {
                if RECORDING.load(Ordering::SeqCst) {
                    println!("[HOOK] F8 RELEASED -> STOP");
                    RECORDING.store(false, Ordering::SeqCst);
                    winapi::um::utilapiset::Beep(1200, 50); // Feedback
                }
                return 1; // Block key
            }
        }
        
        // PTT Logic: Ctrl+F12
        if vk_code == VK_F12 {
             let ctrl = (winapi::um::winuser::GetAsyncKeyState(VK_LCONTROL) as u16 & 0x8000 != 0) || 
                        (winapi::um::winuser::GetAsyncKeyState(VK_RCONTROL) as u16 & 0x8000 != 0);
             if ctrl {
                if event_type == WM_KEYDOWN || event_type == WM_SYSKEYDOWN {
                    if !RECORDING.load(Ordering::SeqCst) {
                        println!("[HOOK] Ctrl+F12 PRESSED");
                        RECORDING.store(true, Ordering::SeqCst);
                        winapi::um::utilapiset::Beep(1500, 50);
                    }
                    return 1;
                } else if event_type == WM_KEYUP || event_type == WM_SYSKEYUP {
                    if RECORDING.load(Ordering::SeqCst) {
                        println!("[HOOK] Ctrl+F12 RELEASED");
                        RECORDING.store(false, Ordering::SeqCst);
                        winapi::um::utilapiset::Beep(1200, 50);
                    }
                    return 1;
                }
             }
        }
    }
    CallNextHookEx(HOOK_HANDLE, code, w_param, l_param)
}

pub fn install_keyboard_hook() -> Result<(), String> {
    std::thread::spawn(|| {
        unsafe {
            let hook_id = SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(keyboard_hook_proc),
                std::ptr::null_mut(),
                0,
            );
            
            if hook_id.is_null() {
                println!("[HOOK] Failed to install hook (Error: {})", winapi::um::errhandlingapi::GetLastError());
                return;
            }
            
            HOOK_HANDLE = hook_id;
            println!("[HOOK] Hook installed. Starting Message Loop...");
            
            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    });
    Ok(())
}

pub fn is_recording() -> bool {
    RECORDING.load(Ordering::SeqCst)
}

#[allow(dead_code)]
pub fn uninstall_keyboard_hook() {
    unsafe {
        if !HOOK_HANDLE.is_null() {
            UnhookWindowsHookEx(HOOK_HANDLE);
            HOOK_HANDLE = std::ptr::null_mut();
            println!("[HOOK] Keyboard hook uninstalled");
        }
    }
}
