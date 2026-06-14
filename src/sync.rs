//! Mutex abstraction that uses `parking_lot::Mutex` when the `parking_lot` feature
//! is enabled, falling back to `std::sync::Mutex` otherwise.

#[cfg(feature = "parking_lot")]
mod imp {
    /// A mutex backed by `parking_lot::Mutex` for higher performance.
    pub struct Mutex<T>(parking_lot::Mutex<T>);

    impl<T> Mutex<T> {
        pub fn new(val: T) -> Self {
            Self(parking_lot::Mutex::new(val))
        }

        pub fn lock(&self) -> parking_lot::MutexGuard<'_, T> {
            self.0.lock()
        }
    }

    impl<T: std::fmt::Debug> std::fmt::Debug for Mutex<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.fmt(f)
        }
    }
}

#[cfg(not(feature = "parking_lot"))]
mod imp {
    use std::sync::Mutex as StdMutex;
    use std::sync::MutexGuard;

    /// A mutex backed by `std::sync::Mutex`, wrapping its poisoned-lock API
    /// to match `parking_lot::Mutex`'s infallible lock interface.
    pub struct Mutex<T>(StdMutex<T>);

    impl<T> Mutex<T> {
        pub fn new(val: T) -> Self {
            Self(StdMutex::new(val))
        }

        pub fn lock(&self) -> MutexGuard<'_, T> {
            // Ignore poison — we do not track panics across threads.
            self.0.lock().unwrap_or_else(|e| e.into_inner())
        }
    }

    impl<T: std::fmt::Debug> std::fmt::Debug for Mutex<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.fmt(f)
        }
    }
}

pub use imp::Mutex;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutex_basic_lock() {
        let m = Mutex::new(42);
        let guard = m.lock();
        assert_eq!(*guard, 42);
        drop(guard);
    }

    #[test]
    fn test_mutex_mutate() {
        let m = Mutex::new(String::new());
        {
            let mut guard = m.lock();
            guard.push_str("hello");
        }
        assert_eq!(*m.lock(), "hello");
    }

    #[test]
    fn test_mutex_debug() {
        let m = Mutex::new(vec![1, 2, 3]);
        let debug = format!("{:?}", m);
        assert!(!debug.is_empty());
    }

    #[test]
    fn test_mutex_multiple_locks() {
        let m = Mutex::new(0);
        for i in 0..10 {
            let mut guard = m.lock();
            *guard += i;
        }
        assert_eq!(*m.lock(), 45);
    }
}
