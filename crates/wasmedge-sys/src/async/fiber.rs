//! Defines data structure for WasmEdge async mechanism.

use fiber_for_wasmedge::{Fiber, FiberStack, Suspend};
use std::{
    future::Future,
    pin::Pin,
    ptr,
    task::{Context, Poll},
};

/// Defines a FiberFuture.
pub(crate) struct FiberFuture<'a> {
    fiber: Fiber<'a, Result<(), ()>, (), Result<(), ()>>,
    current_suspend: *mut *const Suspend<Result<(), ()>, (), Result<(), ()>>,
    current_poll_cx: *mut *mut Context<'static>,
}
impl<'a> FiberFuture<'a> {
    /// Create a fiber to execute the given function.
    ///
    /// # Arguments
    ///
    /// * `async_state` - Used to store asynchronous state at run time.
    ///
    /// * `func` - The function to execute.
    ///
    /// # Error
    ///
    /// If fail to create the fiber stack or the fiber fail to resume, then an error is returned.
    pub(crate) async fn on_fiber<R>(
        async_state: &AsyncState,
        func: impl FnOnce() -> R + Send,
    ) -> Result<R, ()> {
        let mut slot = None;
        let future = {
            let current_poll_cx = async_state.current_poll_cx.get();
            let current_suspend = async_state.current_suspend.get();

            let stack = FiberStack::new(2 << 20).map_err(|_e| ())?;
            let slot = &mut slot;
            let fiber = Fiber::new(stack, move |keep_going, suspend| {
                keep_going?;

                unsafe {
                    let _reset = Reset(current_suspend, *current_suspend);
                    *current_suspend = suspend;
                    *slot = Some(func());
                    Ok(())
                }
            })
            .map_err(|_e| ())?;

            FiberFuture {
                fiber,
                current_suspend,
                current_poll_cx,
            }
        };
        future.await?;

        Ok(slot.unwrap())
    }

    /// This is a helper function to call `resume` on the underlying
    /// fiber while correctly managing thread-local data.
    fn resume(&mut self, val: Result<(), ()>) -> Result<Result<(), ()>, ()> {
        let async_cx = AsyncCx {
            current_suspend: self.current_suspend,
            current_poll_cx: self.current_poll_cx,
        };
        ASYNC_CX.set(&async_cx, || self.fiber.resume(val))
    }
}
impl Future for FiberFuture<'_> {
    type Output = Result<(), ()>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            let _reset = Reset(self.current_poll_cx, *self.current_poll_cx);
            *self.current_poll_cx =
                std::mem::transmute::<&mut Context<'_>, *mut Context<'static>>(cx);

            match self.resume(Ok(())) {
                Ok(ret) => Poll::Ready(ret),
                Err(_) => Poll::Pending,
            }
        }
    }
}
unsafe impl Send for FiberFuture<'_> {}
unsafe impl Sync for FiberFuture<'_> {}

type FiberSuspend = Suspend<Result<(), ()>, (), Result<(), ()>>;

impl Drop for FiberFuture<'_> {
    fn drop(&mut self) {
        if !self.fiber.done() {
            let result = self.resume(Err(()));
            // This resumption with an error should always complete the
            // fiber. While it's technically possible for host code to catch
            // the trap and re-resume, we'd ideally like to signal that to
            // callers that they shouldn't be doing that.
            debug_assert!(result.is_ok());
        }
    }
}

/// Defines a TimeoutFiberFuture.
pub(crate) struct TimeoutFiberFuture<'a> {
    fiber: Fiber<'a, Result<(), ()>, (), Result<(), ()>>,
    current_suspend: *mut *const Suspend<Result<(), ()>, (), Result<(), ()>>,
    current_poll_cx: *mut *mut Context<'static>,
    deadline: std::time::SystemTime,
}

impl<'a> TimeoutFiberFuture<'a> {
    /// Create a fiber to execute the given function.
    ///
    /// # Arguments
    ///
    /// * `func` - The function to execute.
    ///
    /// * `async_state` - Used to store asynchronous state at run time.
    ///
    /// * `deadline` - The deadline the function to be run.
    ///
    /// # Error
    ///
    /// If fail to create the fiber stack or the fiber fail to resume, then an error is returned.
    pub(crate) async fn on_fiber<R>(
        async_state: &AsyncState,
        func: impl FnOnce() -> R + Send,
        deadline: std::time::SystemTime,
    ) -> Result<R, ()> {
        let mut slot = None;

        let future = {
            let current_poll_cx = async_state.current_poll_cx.get();
            let current_suspend = async_state.current_suspend.get();

            let stack = FiberStack::new(2 << 20).map_err(|_e| ())?;
            let slot = &mut slot;
            let fiber = Fiber::new(stack, move |keep_going, suspend| {
                keep_going?;

                unsafe {
                    let _reset = Reset(current_suspend, *current_suspend);
                    *current_suspend = suspend;
                    *slot = Some(func());
                    Ok(())
                }
            })
            .map_err(|_e| ())?;

            TimeoutFiberFuture {
                fiber,
                current_suspend,
                current_poll_cx,
                deadline,
            }
        };

        future.await?;

        Ok(slot.unwrap())
    }
}

impl Future for TimeoutFiberFuture<'_> {
    type Output = Result<(), ()>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            crate::executor::init_signal_listen();

            let _reset = Reset(self.current_poll_cx, *self.current_poll_cx);
            *self.current_poll_cx =
                std::mem::transmute::<&mut Context<'_>, *mut Context<'static>>(cx);
            let async_cx = AsyncCx {
                current_suspend: self.current_suspend,
                current_poll_cx: self.current_poll_cx,
            };

            ASYNC_CX.set(&async_cx, || {
                let mut self_thread = libc::pthread_self();
                let mut timerid: libc::timer_t = std::mem::zeroed();
                let mut sev: libc::sigevent = std::mem::zeroed();
                sev.sigev_notify = libc::SIGEV_SIGNAL;
                sev.sigev_signo = crate::executor::timeout_signo();
                sev.sigev_value.sival_ptr = &mut self_thread as *mut _ as *mut libc::c_void;

                if libc::timer_create(libc::CLOCK_REALTIME, &mut sev, &mut timerid) < 0 {
                    return Poll::Ready(Err(()));
                }

                let timeout = match self.deadline.duration_since(std::time::SystemTime::now()) {
                    Ok(timeout) => timeout.max(std::time::Duration::from_millis(100)),
                    Err(_) => return Poll::Ready(Err(())),
                };

                let mut value: libc::itimerspec = std::mem::zeroed();
                value.it_value.tv_sec = timeout.as_secs() as _;
                value.it_value.tv_nsec = timeout.subsec_nanos() as _;
                if libc::timer_settime(timerid, 0, &value, std::ptr::null_mut()) < 0 {
                    libc::timer_delete(timerid);
                    return Poll::Ready(Err(()));
                }

                let mut env: setjmp::sigjmp_buf = std::mem::zeroed();
                let jmp_state = crate::executor::JmpState {
                    sigjmp_buf: &mut env,
                };

                crate::executor::JMP_BUF.set(&jmp_state, || {
                    if setjmp::sigsetjmp(&mut env, 1) == 0 {
                        let r = match self.as_ref().fiber.resume(Ok(())) {
                            Ok(ret) => Poll::Ready(ret),
                            Err(_) => Poll::Pending,
                        };
                        libc::timer_delete(timerid);
                        r
                    } else {
                        libc::timer_delete(timerid);
                        Poll::Ready(Err(()))
                    }
                })
            })
        }
    }
}
unsafe impl Send for TimeoutFiberFuture<'_> {}
unsafe impl Sync for TimeoutFiberFuture<'_> {}

impl Drop for TimeoutFiberFuture<'_> {
    fn drop(&mut self) {
        if !self.fiber.done() {
            let async_cx = AsyncCx {
                current_suspend: self.current_suspend,
                current_poll_cx: self.current_poll_cx,
            };
            let result = ASYNC_CX.set(&async_cx, || self.fiber.resume(Err(())));
            // This resumption with an error should always complete the
            // fiber. While it's technically possible for host code to catch
            // the trap and re-resume, we'd ideally like to signal that to
            // callers that they shouldn't be doing that.
            debug_assert!(result.is_ok());
        }
    }
}

scoped_tls::scoped_thread_local!(static ASYNC_CX: AsyncCx);

/// Defines a async state that contains the pointer to current poll context and current suspend.
#[derive(Debug)]
pub struct AsyncState {
    current_suspend: std::cell::UnsafeCell<*const FiberSuspend>,
    current_poll_cx: std::cell::UnsafeCell<*mut Context<'static>>,
}
impl Default for AsyncState {
    fn default() -> Self {
        Self::new()
    }
}
impl AsyncState {
    /// Creates a new async state.
    pub fn new() -> Self {
        AsyncState {
            current_suspend: std::cell::UnsafeCell::new(std::ptr::null()),
            current_poll_cx: std::cell::UnsafeCell::new(std::ptr::null_mut()),
        }
    }

    /// Returns an async execution context.
    ///
    /// If the pointer of poll context is null, then None is returned.
    pub fn async_cx(&self) -> Option<AsyncCx> {
        let poll_cx_box_ptr = self.current_poll_cx.get();
        if poll_cx_box_ptr.is_null() {
            return None;
        }
        let poll_cx_inner_ptr = unsafe { *poll_cx_box_ptr };
        if poll_cx_inner_ptr.is_null() {
            return None;
        }

        Some(AsyncCx {
            current_suspend: self.current_suspend.get(),
            current_poll_cx: poll_cx_box_ptr,
        })
    }
}
unsafe impl Send for AsyncState {}
unsafe impl Sync for AsyncState {}

/// Defines an async execution context.
#[derive(Debug, Clone, Copy)]
pub struct AsyncCx {
    current_suspend: *mut *const Suspend<Result<(), ()>, (), Result<(), ()>>,
    current_poll_cx: *mut *mut Context<'static>,
}
impl Default for AsyncCx {
    fn default() -> Self {
        Self::new()
    }
}
impl AsyncCx {
    /// Creates a new async execution context.
    pub fn new() -> Self {
        ASYNC_CX.with(|async_cx| *async_cx)
    }

    /// Runs a future to completion.
    ///
    /// # Arguments
    ///
    /// * `future` - The future to run.
    ///
    /// # Error
    ///
    /// If fail to run, then an error is returned.
    pub(crate) unsafe fn block_on<U>(
        &self,
        mut future: Pin<&mut (dyn Future<Output = U> + Send)>,
    ) -> Result<U, ()> {
        let suspend = *self.current_suspend;
        let _reset = Reset(self.current_suspend, suspend);
        *self.current_suspend = ptr::null();
        assert!(!suspend.is_null());

        loop {
            let future_result = {
                let poll_cx = *self.current_poll_cx;
                let _reset = Reset(self.current_poll_cx, poll_cx);
                *self.current_poll_cx = ptr::null_mut();
                assert!(!poll_cx.is_null());
                future.as_mut().poll(&mut *poll_cx)
            };

            match future_result {
                Poll::Ready(t) => break Ok(t),
                Poll::Pending => {}
            }
            let res = (*suspend).suspend(());
            res?;
        }
    }
}

struct Reset<T: Copy>(*mut T, T);
impl<T: Copy> Drop for Reset<T> {
    fn drop(&mut self) {
        unsafe {
            *self.0 = self.1;
        }
    }
}
