//! Keeps the main window's caption buttons out of the win32k tracking loop.
//!
//! `DefWindowProc` reacts to `WM_NCLBUTTONDOWN` on a caption button by entering
//! a kernel-mode modal tracking loop that waits for the matching
//! `WM_LBUTTONUP` (the buttons are "highlight on press, fire on release").
//! A fullscreen game that re-captures the cursor - the usual "hold alt to get
//! the mouse back" behaviour - swallows that release, so the loop never exits
//! and the whole event loop stalls behind it.
//!
//! The failure is invisible from both ends: no Rust code runs (the press never
//! becomes `WM_CLOSE`, so `CloseRequested` is never emitted) and the webviews
//! only look frozen because queued `EvaluateScript` messages pile up until the
//! loop finally unwinds and delivers them all at once.
//!
//! Turning the press straight into the equivalent system command keeps us out
//! of that loop. Only `WM_NCLBUTTONDOWN` is intercepted, so hit-testing, hover
//! highlighting, Snap Layouts, caption double-click and window dragging all
//! keep their native behaviour.

use log::warn;
use tauri::WebviewWindow;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{
    HTCLOSE, HTMAXBUTTON, HTMINBUTTON, IsZoomed, PostMessageW, SC_MAXIMIZE, SC_MINIMIZE,
    SC_RESTORE, WM_CLOSE, WM_NCDESTROY, WM_NCLBUTTONDOWN, WM_SYSCOMMAND,
};

/// Identifies this subclass on the window so it can be detached again.
const SUBCLASS_ID: usize = 0x7265_736F;

/// Installs the caption-button guard on `window`.
///
/// Must be called from the thread that owns the window. Tauri's `setup` hook
/// runs on the main thread, which is where every window in this app is created.
pub fn install(window: &WebviewWindow) -> Result<(), String> {
    let hwnd = window
        .hwnd()
        .map_err(|error| format!("failed to resolve hwnd: {error}"))?;

    // SAFETY: `hwnd` is a live window owned by the calling thread and
    // `guard_proc` has the signature required by `SUBCLASSPROC`.
    let installed = unsafe { SetWindowSubclass(hwnd, Some(guard_proc), SUBCLASS_ID, 0) };
    if installed.as_bool() {
        Ok(())
    } else {
        Err("SetWindowSubclass rejected the caption guard".to_string())
    }
}

/// Rewrites caption-button presses into system commands, passing every other
/// message through untouched.
unsafe extern "system" fn guard_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    _ref_data: usize,
) -> LRESULT {
    if msg == WM_NCLBUTTONDOWN {
        let posted = match wparam.0 as u32 {
            // Post `WM_CLOSE` rather than `SC_CLOSE` so closing still runs
            // through the single path `on_window_event_fn` already owns
            // (`prevent_close` + `hide`), instead of duplicating that policy.
            HTCLOSE => Some((WM_CLOSE, WPARAM(0))),
            HTMINBUTTON => Some((WM_SYSCOMMAND, WPARAM(SC_MINIMIZE as usize))),
            HTMAXBUTTON => {
                // SAFETY: `hwnd` is the window this proc is attached to.
                let command = if unsafe { IsZoomed(hwnd) }.as_bool() {
                    SC_RESTORE
                } else {
                    SC_MAXIMIZE
                };
                Some((WM_SYSCOMMAND, WPARAM(command as usize)))
            }
            // Anything else on the non-client area (dragging by the caption,
            // the resize borders, the system menu) keeps its native handling.
            _ => None,
        };

        if let Some((message, command)) = posted {
            // SAFETY: `hwnd` is the window this proc is attached to.
            if let Err(error) = unsafe { PostMessageW(Some(hwnd), message, command, LPARAM(0)) } {
                // Swallow the click rather than falling through: reaching
                // DefWindowProc is exactly the stall this guard exists to
                // avoid, and an ignored click is far cheaper than a freeze.
                warn!(target: "app::window", "caption_guard_post_failed message={message:#06x} error={error}");
            }
            return LRESULT(0);
        }
    }

    if msg == WM_NCDESTROY {
        // Detach before the window goes away so the subclass chain stays clean.
        // The window is being destroyed either way, so a failure here is not
        // actionable.
        // SAFETY: removing the subclass we installed, from its own window.
        let _ = unsafe { RemoveWindowSubclass(hwnd, Some(guard_proc), SUBCLASS_ID) };
    }

    // SAFETY: forwarding the original arguments to the next proc in the chain.
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}
