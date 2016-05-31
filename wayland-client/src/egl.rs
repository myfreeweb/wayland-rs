//! EGL utilities
//!
//! This module contains bindings to the `libwayland-egl.so` library.
//!
//! This library is used to interface with the OpenGL stack, and creating
//! EGL surfaces from a wayland surface.
//!
//! See WlEglSurface documentation for details.

use std::os::raw::c_void;
use std::ops::Deref;

use wayland_sys::egl::*;
use sys::wayland::client::WlSurface;
use Proxy;

/// Checks if the wayland-egl lib is available and can be used
///
/// Trying to create an `WlEglSurface` while this function returns
/// `false` will result in a panic.
pub fn is_available() -> bool {
    is_lib_available()
}

unsafe impl Send for WlEglSurface {}
unsafe impl Sync for WlEglSurface {}

/// EGL surface
///
/// This object is a simple wrapper around a `WlSurface` to add the EGL
/// capabilities. Just use the `egl_surface_ptr` once this object is created
/// to get the window pointer your OpenGL library is needing to initialize the
/// EGL context (you'll most likely need the display ptr as well, that you can
/// get via the `ptr` method of the `Proxy` trait on the `WlDisplay` object).
///
/// It'll allow you to acces the underlying `WlSurface` to call its methods
/// or define its role wia a `Deref` implementation.
///
/// If you let it go out of scope, it'll destroy the underlying `WlSurface`. If you
/// need to get it back, use the `destroy` method.
pub struct WlEglSurface {
    ptr: *mut wl_egl_window,
    surface: WlSurface
}

impl WlEglSurface {
    /// Create an EGL surface from a wayland surface
    pub fn new(surface: WlSurface, width: i32, height: i32) -> WlEglSurface {
        let ptr = unsafe { ffi_dispatch!(WAYLAND_EGL_HANDLE, wl_egl_window_create,
            surface.ptr(), width, height) };
        WlEglSurface {
            ptr: ptr,
            surface: surface
        }
    }

    /// Destroy the EGL surface, giving back the original wayland surface
    pub fn destroy(mut self) -> WlSurface {
        unsafe { ffi_dispatch!(WAYLAND_EGL_HANDLE, wl_egl_window_destroy, self.ptr); }
        let surface = ::std::mem::replace(&mut self.surface, unsafe { ::std::mem::uninitialized() });
        ::std::mem::forget(self);
        surface
    }

    /// Fetch current size of the EGL surface
    pub fn get_size(&self) -> (i32, i32) {
        let mut w = 0i32;
        let mut h = 0i32;
        unsafe { ffi_dispatch!(WAYLAND_EGL_HANDLE, wl_egl_window_get_attached_size,
            self.ptr, &mut w as *mut i32, &mut h as *mut i32); }
        (w, h)
    }

    /// Resize the EGL surface
    ///
    /// The two first arguments `(width, height)` are the new size of
    /// the surface, the two others `(dx, dy)` represent the displacement
    /// of the top-left corner of the surface. It allows you to control the
    /// direction of the resizing if necessary.
    pub fn resize(&self, width: i32, height: i32, dx: i32, dy: i32) {
        unsafe { ffi_dispatch!(WAYLAND_EGL_HANDLE, wl_egl_window_resize,
            self.ptr, width, height, dx, dy) }
    }

    /// Raw pointer to the EGL surface
    ///
    /// You'll need this pointer to initialize the EGL context in your
    /// favourite OpenGL lib.
    pub fn egl_surface_ptr(&self) -> *const c_void {
        self.ptr as *const c_void
    }

    /// Raw pointer to the EGL surface
    #[deprecated(since="0.6", note="use egl_surface_ptr")]
    pub unsafe fn egl_surfaceptr(&self) -> *mut c_void {
        self.ptr as *mut c_void
    }
}

impl Drop for WlEglSurface {
    fn drop(&mut self) {
        unsafe { ffi_dispatch!(WAYLAND_EGL_HANDLE, wl_egl_window_destroy, self.ptr); }
    }
}

impl Deref for WlEglSurface {
    type Target = WlSurface;
    fn deref(&self) -> &WlSurface {
        &self.surface
    }
}
