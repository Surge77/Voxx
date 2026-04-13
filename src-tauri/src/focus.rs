use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForegroundTarget(isize);

impl ForegroundTarget {
    pub fn from_raw(raw: isize) -> Option<Self> {
        if raw == 0 {
            return None;
        }

        Some(Self(raw))
    }

    pub fn as_raw(self) -> isize {
        self.0
    }
}

#[cfg(windows)]
pub fn capture_foreground_target() -> Option<ForegroundTarget> {
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    let hwnd = unsafe { GetForegroundWindow() };
    ForegroundTarget::from_raw(hwnd.0 as isize)
}

#[cfg(not(windows))]
pub fn capture_foreground_target() -> Option<ForegroundTarget> {
    None
}

#[cfg(windows)]
pub fn restore_foreground_target(target: Option<ForegroundTarget>) -> bool {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
    use windows::Win32::UI::Input::KeyboardAndMouse::{SetActiveWindow, SetFocus};
    use windows::Win32::UI::WindowsAndMessaging::{
        BringWindowToTop, GetForegroundWindow, GetWindowThreadProcessId, IsIconic, IsWindow, SetForegroundWindow,
        ShowWindow, SW_RESTORE,
    };

    let Some(target) = target else {
        return false;
    };
    let hwnd = HWND(target.as_raw() as *mut core::ffi::c_void);

    unsafe {
        if !IsWindow(hwnd).as_bool() {
            return false;
        }

        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }

        let current_thread_id = GetCurrentThreadId();
        let target_thread_id = GetWindowThreadProcessId(hwnd, None);
        let foreground = GetForegroundWindow();
        let foreground_thread_id = GetWindowThreadProcessId(foreground, None);

        let attach_target = target_thread_id != 0 && target_thread_id != current_thread_id;
        let attach_foreground = foreground_thread_id != 0 && foreground_thread_id != current_thread_id;

        if attach_target {
            let _ = AttachThreadInput(current_thread_id, target_thread_id, true);
        }
        if attach_foreground && foreground_thread_id != target_thread_id {
            let _ = AttachThreadInput(current_thread_id, foreground_thread_id, true);
        }

        let _ = BringWindowToTop(hwnd);
        let _ = SetActiveWindow(hwnd);
        let _ = SetFocus(hwnd);
        let restored = SetForegroundWindow(hwnd).as_bool();

        if attach_foreground && foreground_thread_id != target_thread_id {
            let _ = AttachThreadInput(current_thread_id, foreground_thread_id, false);
        }
        if attach_target {
            let _ = AttachThreadInput(current_thread_id, target_thread_id, false);
        }

        std::thread::sleep(Duration::from_millis(160));
        restored
    }
}

#[cfg(not(windows))]
pub fn restore_foreground_target(_target: Option<ForegroundTarget>) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::ForegroundTarget;

    #[test]
    fn foreground_target_ignores_null_handle() {
        assert!(ForegroundTarget::from_raw(0).is_none());
    }

    #[test]
    fn foreground_target_preserves_nonzero_handle() {
        let target = ForegroundTarget::from_raw(42).expect("target");

        assert_eq!(target.as_raw(), 42);
    }
}
