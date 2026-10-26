//! Turn Windows title-bar close requests into application events before minifb
//! marks its window closed. No messages are injected and no window is recreated.
#[cfg(target_os = "windows")]
mod windows {
    use std::{
        marker::PhantomData,
        rc::Rc,
        sync::atomic::{AtomicBool, Ordering},
    };
    use winapi::{
        shared::{
            minwindef::{LPARAM, LRESULT, UINT, WPARAM},
            windef::HWND,
        },
        um::winuser::WM_CLOSE,
    };

    type SubclassProc =
        unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM, usize, usize) -> LRESULT;

    // Use the existing Windows system DLL directly; no new crate or winapi
    // feature is needed, and minifb retains ownership of GWLP_USERDATA.
    #[link(name = "comctl32")]
    unsafe extern "system" {
        fn SetWindowSubclass(
            owner: HWND,
            callback: Option<SubclassProc>,
            id: usize,
            data: usize,
        ) -> i32;
        fn RemoveWindowSubclass(owner: HWND, callback: Option<SubclassProc>, id: usize) -> i32;
        fn DefSubclassProc(owner: HWND, message: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT;
    }

    unsafe extern "system" fn callback(
        owner: HWND,
        message: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
        _id: usize,
        data: usize,
    ) -> LRESULT {
        if message == WM_CLOSE && data != 0 {
            // The Box is stable until successful removal of this subclass. On
            // removal failure Drop deliberately preserves the allocation.
            let request = unsafe { &*(data as *const AtomicBool) };
            request.store(true, Ordering::Release);
            return 0;
        }
        unsafe { DefSubclassProc(owner, message, wparam, lparam) }
    }

    pub struct CloseGuard {
        owner: HWND,
        id: usize,
        request: Option<Box<AtomicBool>>,
        // Subclasses must be installed/removed on the owning window thread.
        _thread_bound: PhantomData<Rc<()>>,
    }

    impl CloseGuard {
        /// Install after creating Window and keep this guard alive until just
        /// before Window drops. Failure must be reported rather than silently
        /// allowing unsaved annotations to be lost.
        pub fn new(owner: usize) -> Result<Self, String> {
            if owner == 0 {
                return Err("Cannot install close guard without a window handle".into());
            }
            let request = Box::new(AtomicBool::new(false));
            let data = (&*request as *const AtomicBool) as usize;
            let owner = owner as HWND;
            if unsafe { SetWindowSubclass(owner, Some(callback), data, data) } == 0 {
                return Err(format!(
                    "Cannot install window close guard (SetWindowSubclass returned FALSE): {}",
                    std::io::Error::last_os_error()
                ));
            }
            Ok(Self {
                owner,
                id: data,
                request: Some(request),
                _thread_bound: PhantomData,
            })
        }

        /// Coalesce repeated close clicks without growing an event queue.
        pub fn take_request(&self) -> bool {
            self.request
                .as_ref()
                .is_some_and(|request| request.swap(false, Ordering::AcqRel))
        }
    }

    impl Drop for CloseGuard {
        fn drop(&mut self) {
            if unsafe { RemoveWindowSubclass(self.owner, Some(callback), self.id) } == 0 {
                eprintln!(
                    "Window close guard removal failed; preserving callback context to prevent use-after-free"
                );
                if let Some(request) = self.request.take() {
                    let _ = Box::leak(request);
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn close_callback_records_request_without_forwarding_or_creating_a_window() {
            let request = Box::new(AtomicBool::new(false));
            let data = (&*request as *const AtomicBool) as usize;
            // Direct function invocation only: no HWND, window creation,
            // SendMessage/PostMessage, input injection or GUI interaction.
            for _ in 0..3 {
                assert_eq!(
                    unsafe { callback(std::ptr::null_mut(), WM_CLOSE, 0, 0, data, data) },
                    0
                );
            }
            assert!(request.swap(false, Ordering::AcqRel));
            assert!(!request.swap(false, Ordering::AcqRel));
        }

        #[test]
        fn null_window_handle_is_rejected_before_calling_windows() {
            assert!(CloseGuard::new(0).is_err());
        }
    }
}

#[cfg(target_os = "windows")]
pub use windows::CloseGuard;

#[cfg(not(target_os = "windows"))]
pub struct CloseGuard;

#[cfg(not(target_os = "windows"))]
impl CloseGuard {
    pub fn new(_owner: usize) -> Result<Self, String> {
        Err("Native window close confirmation is only supported on Windows".into())
    }
    pub fn take_request(&self) -> bool {
        false
    }
}
