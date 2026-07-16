//! Defines WasmEdge Statistics struct.

use crate::{WasmEdgeResult, ffi};
use std::sync::Arc;
use wasmedge_types::error::WasmEdgeError;

#[derive(Debug, Clone)]
/// Struct of WasmEdge Statistics.
pub struct Statistics {
    pub(crate) inner: Arc<InnerStat>,
}
impl Statistics {
    /// Creates a new [Statistics].
    ///
    /// # Error
    ///
    /// If fail to create a [Statistics], then an error is returned.
    pub fn create() -> WasmEdgeResult<Self> {
        let ctx = unsafe { ffi::WasmEdge_StatisticsCreate() };
        if ctx.is_null() {
            Err(Box::new(WasmEdgeError::StatisticsCreate))
        } else {
            Ok(Statistics {
                inner: Arc::new(InnerStat(ctx)),
            })
        }
    }

    /// Returns the instruction count in execution.
    pub fn instr_count(&self) -> u64 {
        unsafe { ffi::WasmEdge_StatisticsGetInstrCount(self.inner.0) }
    }

    /// Returns the instruction count per second in execution.
    ///
    /// # Notice
    ///
    /// For the following cases,
    /// * [Statistics] is not enabled, or
    /// * the total execution time is 0
    ///
    /// The instructions per second could be `NaN`, which represents `divided-by-zero`.
    /// Use the `is_nan` function of F64 to check the return value before use it,
    /// for example,
    ///
    /// ```
    /// use wasmedge_sys::Statistics;
    ///
    /// // create a Statistics instance
    /// let stat = Statistics::create().expect("fail to create a Statistics");
    ///
    /// // check instruction count per second
    /// assert!(stat.instr_per_sec().is_nan());
    /// ```
    pub fn instr_per_sec(&self) -> f64 {
        unsafe { ffi::WasmEdge_StatisticsGetInstrPerSecond(self.inner.0) }
    }

    /// Returns the total cost in execution.
    pub fn cost_in_total(&self) -> u64 {
        unsafe { ffi::WasmEdge_StatisticsGetTotalCost(self.inner.0) }
    }

    /// Sets the cost of instructions.
    ///
    /// # Arguments
    ///
    /// * `cost_table` - The slice of cost table.
    pub fn set_cost_table(&mut self, cost_table: impl AsRef<[u64]>) {
        unsafe {
            ffi::WasmEdge_StatisticsSetCostTable(
                self.inner.0,
                cost_table.as_ref().as_ptr() as *mut _,
                cost_table.as_ref().len() as u32,
            )
        }
    }

    /// Sets the cost limit in execution.
    ///
    /// # Arguments
    ///
    /// * `limit` - The cost limit.
    pub fn set_cost_limit(&mut self, limit: u64) {
        unsafe { ffi::WasmEdge_StatisticsSetCostLimit(self.inner.0, limit) }
    }

    /// Clears the data in this statistics.
    pub fn clear(&mut self) {
        unsafe { ffi::WasmEdge_StatisticsClear(self.inner.0) }
    }

    /// Provides a raw pointer to the inner Statistics context.
    #[cfg(feature = "ffi")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ffi")))]
    pub fn as_ptr(&self) -> *const ffi::WasmEdge_StatisticsContext {
        self.inner.0 as *const _
    }
}
#[derive(Debug)]
pub(crate) struct InnerStat(pub(crate) *mut ffi::WasmEdge_StatisticsContext);
impl Drop for InnerStat {
    fn drop(&mut self) {
        // `Statistics` shares this handle through `Arc<InnerStat>`, and an
        // `Executor` may hold another `Arc` to keep it alive. Deleting the context
        // on the reference-counted inner — not on the cloneable outer `Statistics`
        // — guarantees the last owner deletes it exactly once. A `Drop for
        // Statistics` double-freed whenever the struct was cloned.
        unsafe { ffi::WasmEdge_StatisticsDelete(self.0) }
    }
}
// SAFETY: (assumed, pre-existing) owns an opaque `*mut WasmEdge_StatisticsContext`.
// `Send` is sound: a move transfers sole ownership of a thread-agnostic handle.
// `Sync` is the assumed half (concurrent `&self` C calls) — WasmEdge documents
// no thread-safety for this context, so it is an unverified, inherited invariant.
unsafe impl Send for InnerStat {}
unsafe impl Sync for InnerStat {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Executor;

    // Cloning a `Statistics` shares one `Arc<InnerStat>`; dropping the clones must
    // not delete the underlying context until the last owner is gone. Before the
    // fix, `Drop for Statistics` deleted the raw context on every clone, so this
    // exercised a double-free (which aborts the test binary).
    #[test]
    fn test_statistics_clone_no_double_free() {
        let stat = Statistics::create().unwrap();
        assert_eq!(Arc::strong_count(&stat.inner), 1);

        let c1 = stat.clone();
        let c2 = stat.clone();
        assert_eq!(Arc::strong_count(&stat.inner), 3);

        // Dropping clones must not free the shared context.
        drop(c1);
        drop(c2);
        assert_eq!(Arc::strong_count(&stat.inner), 1);

        // Original still points at a live context (a premature free would be a UAF).
        let _ = stat.instr_count();
        // `stat` drops here and deletes the context exactly once.
    }

    // `Executor::create` stores the statistics context but does not own it. The
    // executor must keep the `Arc<InnerStat>` alive; otherwise dropping the passed
    // `Statistics` would leave the executor with a dangling stat pointer and make
    // the probe read below a use-after-free.
    #[test]
    fn test_executor_keeps_statistics_alive() {
        let stat = Statistics::create().unwrap();
        // Keep our own handle to the same context.
        let stat_probe = stat.clone();
        assert_eq!(Arc::strong_count(&stat_probe.inner), 2);

        // The executor takes ownership of one `Arc` and must keep it alive.
        let executor = Executor::create(None, Some(stat)).unwrap();

        // Executor's retained `Arc` + our probe == 2 strong refs.
        assert_eq!(Arc::strong_count(&stat_probe.inner), 2);

        // Would be a use-after-free under the old dangling-pointer behavior.
        let _ = stat_probe.instr_count();

        drop(executor);
        // Executor released its `Arc`; only our probe remains.
        assert_eq!(Arc::strong_count(&stat_probe.inner), 1);
    }
}
