use std::cell::RefCell;

struct HookFns {
    malloc_fn: Option<unsafe fn(usize) -> *mut u8>,
    free_fn: Option<unsafe fn(*mut u8)>,
}

thread_local! {
    static HOOKS: RefCell<HookFns> = RefCell::new(HookFns {
        malloc_fn: None,
        free_fn: None,
    });
}

pub struct Hooks {
    pub malloc_fn: Option<unsafe fn(usize) -> *mut u8>,
    pub free_fn: Option<unsafe fn(*mut u8)>,
}

pub fn init_hooks(hooks: Option<&Hooks>) {
    HOOKS.with(|slot| {
        let mut current = slot.borrow_mut();
        match hooks {
            None => {
                current.malloc_fn = None;
                current.free_fn = None;
            }
            Some(hooks) => {
                current.malloc_fn = hooks.malloc_fn;
                current.free_fn = hooks.free_fn;
            }
        }
    });
}

pub fn malloc(size: usize) -> *mut u8 {
    let function = HOOKS.with(|slot| slot.borrow().malloc_fn);
    unsafe {
        match function {
            Some(function) => function(size),
            None => libc::malloc(size) as *mut u8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{free, init_hooks, malloc, Hooks};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static CALLS: AtomicUsize = AtomicUsize::new(0);

    unsafe fn hook_malloc(size: usize) -> *mut u8 {
        CALLS.fetch_add(1, Ordering::SeqCst);
        libc::malloc(size) as *mut u8
    }

    unsafe fn hook_free(pointer: *mut u8) {
        libc::free(pointer as *mut libc::c_void);
    }

    #[test]
    fn malloc_uses_the_installed_hook() {
        CALLS.store(0, Ordering::SeqCst);
        init_hooks(Some(&Hooks {
            malloc_fn: Some(hook_malloc),
            free_fn: Some(hook_free),
        }));
        let pointer = malloc(16);
        assert!(!pointer.is_null());
        free(pointer);
        init_hooks(None);
        assert_eq!(CALLS.load(Ordering::SeqCst), 1);
    }
}

pub fn free(pointer: *mut u8) {
    if pointer.is_null() {
        return;
    }
    let function = HOOKS.with(|slot| slot.borrow().free_fn);
    unsafe {
        match function {
            Some(function) => function(pointer),
            None => libc::free(pointer as *mut libc::c_void),
        }
    }
}
