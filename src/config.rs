//! Defines the structs used to construct configurations.

use std::sync::Arc;

use crate::WasmEdgeResult;
#[cfg(feature = "aot")]
use crate::{CompilerOptimizationLevel, CompilerOutputFormat};
use wasmedge_sys as sys;
use wasmedge_types::{RunMode, WasmStandard};

/// Defines a builder for creating a [Config].
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    common_config: CommonConfigOptions,
    stat_config: Option<StatisticsConfigOptions>,
    #[cfg(feature = "aot")]
    compiler_config: Option<CompilerConfigOptions>,
    runtime_config: Option<RuntimeConfigOptions>,
}
impl ConfigBuilder {
    /// Creates a new [ConfigBuilder] with the given [CommonConfigOptions] setting.
    pub fn new(options: CommonConfigOptions) -> Self {
        Self {
            common_config: options,
            stat_config: None,
            #[cfg(feature = "aot")]
            compiler_config: None,
            runtime_config: None,
        }
    }

    /// Sets the [StatisticsConfigOptions] for the [ConfigBuilder].
    ///
    /// # Argument
    ///
    /// - `options` specifies the [StatisticsConfigOptions] settings to set.
    pub fn with_statistics_config(self, options: StatisticsConfigOptions) -> Self {
        Self {
            stat_config: Some(options),
            ..self
        }
    }

    /// Sets the [RuntimeConfigOptions] for the [ConfigBuilder].
    ///
    /// # Argument
    ///
    /// - `options` specifies the [RuntimeConfigOptions] settings to set.
    pub fn with_runtime_config(self, options: RuntimeConfigOptions) -> Self {
        Self {
            runtime_config: Some(options),
            ..self
        }
    }

    /// Sets the [CompilerConfigOptions] for the [ConfigBuilder].
    ///
    /// # Argument
    ///
    /// - `options` specifies the [CompilerConfigOptions] settings to set.
    #[cfg(feature = "aot")]
    #[cfg_attr(docsrs, doc(cfg(feature = "aot")))]
    pub fn with_compiler_config(self, options: CompilerConfigOptions) -> Self {
        Self {
            compiler_config: Some(options),
            ..self
        }
    }

    /// Creates a new [Config] from the [ConfigBuilder].
    ///
    /// # Errors
    ///
    /// If fail to create a [Config], then an error is returned.
    pub fn build(self) -> WasmEdgeResult<Config> {
        let mut inner = sys::Config::create()?;
        inner.mutable_globals(self.common_config.mutable_globals);
        inner.non_trap_conversions(self.common_config.non_trap_conversions);
        inner.sign_extension_operators(self.common_config.sign_extension_operators);
        inner.multi_value(self.common_config.multi_value);
        inner.bulk_memory_operations(self.common_config.bulk_memory_operations);
        inner.simd(self.common_config.simd);
        inner.relax_simd(self.common_config.relax_simd);
        inner.multi_memories(self.common_config.multi_memories);
        inner.threads(self.common_config.threads);
        inner.tail_call(self.common_config.tail_call);
        inner.extended_const(self.common_config.extended_const);
        inner.annotations(self.common_config.annotations);
        inner.memory64(self.common_config.memory64);
        inner.exception_handling(self.common_config.exception_handling);
        inner.component_model(self.common_config.component_model);
        // The GC proposal depends on FunctionReferences, and both depend on ReferenceTypes.
        // Apply the dependents first, so that disabling a dependency is not silently ignored
        // by the runtime while a dependent is still enabled.
        inner.gc(self.common_config.gc);
        inner.function_references(self.common_config.function_references);
        inner.reference_types(self.common_config.reference_types);
        inner.set_run_mode(self.common_config.run_mode);

        if let Some(stat_config) = self.stat_config {
            inner.count_instructions(stat_config.count_instructions);
            inner.measure_cost(stat_config.measure_cost);
            inner.measure_time(stat_config.measure_time);
        }
        #[cfg(feature = "aot")]
        if let Some(compiler_config) = self.compiler_config {
            inner.set_aot_compiler_output_format(compiler_config.out_format);
            inner.set_aot_optimization_level(compiler_config.opt_level);
            inner.dump_ir(compiler_config.dump_ir);
            inner.generic_binary(compiler_config.generic_binary);
            inner.interruptible(compiler_config.interruptible);
        }
        if let Some(runtim_config) = self.runtime_config {
            inner.set_max_memory_pages(runtim_config.max_memory_pages);
            inner.allow_afunix(runtim_config.allow_afunix);
        }

        Ok(Config {
            inner: Arc::new(inner),
        })
    }
}

/// Defines [Config] struct used to check/set the configuration options.
///
/// # Example
///
/// The following code shows how to create a [Config] with [ConfigBuilder].
///
/// ```rust
///
/// use wasmedge_sdk::{config::{Config, ConfigBuilder, CommonConfigOptions, StatisticsConfigOptions, RuntimeConfigOptions}};
/// use wasmedge_types::{CompilerOutputFormat, CompilerOptimizationLevel};
///
/// let common_options = CommonConfigOptions::default()
///     .bulk_memory_operations(true)
///     .multi_value(true)
///     .mutable_globals(true)
///     .non_trap_conversions(true)
///     .reference_types(true)
///     .sign_extension_operators(true)
///     .simd(true);
///
/// let stat_options = StatisticsConfigOptions::default()
///     .count_instructions(true)
///     .measure_cost(true)
///     .measure_time(true);
///
/// let runtime_options = RuntimeConfigOptions::default().max_memory_pages(1024);
///
///
/// let result = ConfigBuilder::new(common_options)
///     .with_statistics_config(stat_options)
///     .with_runtime_config(runtime_options)
///     .build();
/// assert!(result.is_ok());
/// let config = result.unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) inner: Arc<sys::Config>,
}
impl Config {
    /// Returns the number of the memory pages available.
    pub fn max_memory_pages(&self) -> u32 {
        self.inner.get_max_memory_pages()
    }

    /// Checks if the ImportExportMutGlobals option turns on or not.
    pub fn mutable_globals_enabled(&self) -> bool {
        self.inner.mutable_globals_enabled()
    }

    /// Checks if the NonTrapFloatToIntConversions option turns on or not.
    pub fn non_trap_conversions_enabled(&self) -> bool {
        self.inner.non_trap_conversions_enabled()
    }

    /// Checks if the SignExtensionOperators option turns on or not.
    pub fn sign_extension_operators_enabled(&self) -> bool {
        self.inner.sign_extension_operators_enabled()
    }

    /// Checks if the MultiValue option turns on or not.
    pub fn multi_value_enabled(&self) -> bool {
        self.inner.multi_value_enabled()
    }

    /// Checks if the BulkMemoryOperations option turns on or not.
    pub fn bulk_memory_operations_enabled(&self) -> bool {
        self.inner.bulk_memory_operations_enabled()
    }

    /// Checks if the ReferenceTypes option turns on or not.
    pub fn reference_types_enabled(&self) -> bool {
        self.inner.reference_types_enabled()
    }

    /// Checks if the SIMD option turns on or not.
    pub fn simd_enabled(&self) -> bool {
        self.inner.simd_enabled()
    }

    /// Checks if the RelaxSIMD option turns on or not.
    pub fn relax_simd_enabled(&self) -> bool {
        self.inner.relax_simd_enabled()
    }

    /// Checks if the MultiMemories option turns on or not.
    pub fn multi_memories_enabled(&self) -> bool {
        self.inner.multi_memories_enabled()
    }

    /// Checks if the Threads option turns on or not.
    pub fn threads_enabled(&self) -> bool {
        self.inner.threads_enabled()
    }

    /// Checks if the TailCall option turns on or not.
    pub fn tail_call_enabled(&self) -> bool {
        self.inner.tail_call_enabled()
    }

    /// Checks if the ExtendedConst option turns on or not.
    pub fn extended_const_enabled(&self) -> bool {
        self.inner.extended_const_enabled()
    }

    /// Checks if the Annotations option turns on or not.
    pub fn annotations_enabled(&self) -> bool {
        self.inner.annotations_enabled()
    }

    /// Checks if the Memory64 option turns on or not.
    pub fn memory64_enabled(&self) -> bool {
        self.inner.memory64_enabled()
    }

    /// Checks if the ExceptionHandling option turns on or not.
    pub fn exception_handling_enabled(&self) -> bool {
        self.inner.exception_handling_enabled()
    }

    /// Checks if the GC option turns on or not.
    pub fn gc_enabled(&self) -> bool {
        self.inner.gc_enabled()
    }

    /// Checks if the FunctionReferences option turns on or not.
    pub fn function_references_enabled(&self) -> bool {
        self.inner.function_references_enabled()
    }

    /// Checks if the Component option turns on or not.
    pub fn component_model_enabled(&self) -> bool {
        self.inner.component_model_enabled()
    }

    /// Checks if the `AF_UNIX` sockets option turns on or not.
    pub fn allow_afunix_enabled(&self) -> bool {
        self.inner.allow_afunix_enabled()
    }

    /// Checks if the `ForceInterpreter` option turns on or not.
    #[deprecated(note = "use `run_mode` instead")]
    pub fn interpreter_mode_enabled(&self) -> bool {
        #[allow(deprecated)]
        self.inner.interpreter_mode_enabled()
    }

    /// Returns the execution mode.
    pub fn run_mode(&self) -> RunMode {
        self.inner.get_run_mode()
    }

    /// Returns the optimization level of AOT compiler.
    #[cfg(feature = "aot")]
    #[cfg_attr(docsrs, doc(cfg(feature = "aot")))]
    pub fn optimization_level(&self) -> CompilerOptimizationLevel {
        self.inner.get_aot_optimization_level()
    }

    /// Returns the output binary format of AOT compiler.
    #[cfg(feature = "aot")]
    #[cfg_attr(docsrs, doc(cfg(feature = "aot")))]
    pub fn out_format(&self) -> CompilerOutputFormat {
        self.inner.get_aot_compiler_output_format()
    }

    /// Checks if the dump IR option turns on or not.
    #[cfg(feature = "aot")]
    #[cfg_attr(docsrs, doc(cfg(feature = "aot")))]
    pub fn dump_ir_enabled(&self) -> bool {
        self.inner.dump_ir_enabled()
    }

    /// Checks if the generic binary option of AOT compiler turns on or not.
    #[cfg(feature = "aot")]
    #[cfg_attr(docsrs, doc(cfg(feature = "aot")))]
    pub fn generic_binary_enabled(&self) -> bool {
        self.inner.generic_binary_enabled()
    }

    /// Checks if the `Interruptible` option of AOT Compiler turns on or not.
    #[cfg(feature = "aot")]
    #[cfg_attr(docsrs, doc(cfg(feature = "aot")))]
    pub fn interruptible_enabled(&self) -> bool {
        self.inner.interruptible_enabled()
    }

    /// Checks if the instruction counting option turns on or not.
    pub fn instruction_counting_enabled(&self) -> bool {
        self.inner.is_instruction_counting()
    }

    /// Checks if the cost measuring option turns on or not.
    pub fn cost_measuring_enabled(&self) -> bool {
        self.inner.is_cost_measuring()
    }

    /// Checks if the cost measuring option turns on or not.
    pub fn time_measuring_enabled(&self) -> bool {
        self.inner.is_time_measuring()
    }
}

/// Defines the common configuration options.
///
/// [CommonConfigOptions] is used to set the common configuration options, which are
///     
///  - `ImportExportMutGlobals` supports mutable imported and exported globals.
///
///    Also see [Import/Export Mutable Globals Proposal](https://github.com/WebAssembly/mutable-global/blob/master/proposals/mutable-global/Overview.md#importexport-mutable-globals).
///
///  - `NonTrapFloatToIntConversions` supports the non-trapping float-to-int conversion.
///
///    Also see [Non-trapping Float-to-int Conversions Proposal](https://github.com/WebAssembly/spec/blob/main/proposals/nontrapping-float-to-int-conversion/Overview.md).
///
///  - `SignExtensionOperators` supports new integer instructions for sign-extending 8-bit, 16-bit, and 32-bit values.
///     
///    Also see [Sign-extension Operators Proposal](https://github.com/WebAssembly/spec/blob/main/proposals/sign-extension-ops/Overview.md).
///
///  - `MultiValue` supports functions and instructions with multiple return values, and blocks with inputs.
///     
///    Also see [Multi-value Extension](https://github.com/WebAssembly/spec/blob/main/proposals/multi-value/Overview.md).
///
///  - `BulkMemoryOperations` supports bulk memory operations.
///
///    Also see [Bulk Memory Operations Proposal](https://github.com/WebAssembly/spec/blob/main/proposals/bulk-memory-operations/Overview.md#motivation-for-bulk-memory-operations).
///
///  - `ReferenceTypes` supports reference types.
///
///    Also see [Reference Types Proposal](https://github.com/WebAssembly/spec/blob/main/proposals/reference-types/Overview.md).
///
///  - `SIMD` supports 128-bit packed SIMD extension to WebAssembly.
///
///    Also see [SIMD Proposal](https://github.com/WebAssembly/spec/blob/main/proposals/simd/SIMD.md).
///
///  - `RelaxSIMD` supports the relaxed SIMD instructions.
///
///    Also see [Relaxed SIMD Proposal](https://github.com/WebAssembly/relaxed-simd/blob/main/proposals/relaxed-simd/Overview.md).
///
///  - `MultiMemories` enables the use of multiple memories within a single Wasm module.
///
///    Also see [Multiple Memories Proposal](https://github.com/WebAssembly/multi-memory/blob/main/proposals/multi-memory/Overview.md).
///
///  - `Threads` supports the threading feature.
///
///    Also see [Threading Proposal](https://github.com/WebAssembly/threads/blob/main/proposals/threads/Overview.md).
///
///  - `TailCall` supports tail call optimization.
///
///    Also see [Tail Call Proposal](https://github.com/WebAssembly/tail-call/blob/master/proposals/tail-call/Overview.md).
///
///  - `ExtendedConst` supports extended constant expressions.
///
///    Also see [Extended Const Expressions Proposal](https://github.com/WebAssembly/extended-const/blob/main/proposals/extended-const/Overview.md).
///
///  - `Annotations` supports annotations in the WASM text format.
///
///    Also see [Annotations Proposal](https://github.com/WebAssembly/annotations/blob/master/proposals/annotations/Overview.md).
///
///  - `Memory64` supports 64-bit memory indexes.
///
///    Also see [Memory64 Proposal](https://github.com/WebAssembly/memory64/blob/main/proposals/memory64/Overview.md).
///
///  - `ExceptionHandling` supports exception handling.
///
///    Also see [Exception Handling Proposal](https://github.com/WebAssembly/exception-handling/blob/main/proposals/exception-handling/Exceptions.md).
///
///  - `FunctionReferences` supports typed function references for WebAssembly.
///
///    Also see [Function References Proposal](https://github.com/WebAssembly/function-references/blob/master/proposals/function-references/Overview.md).
///
///  - `GC` supports garbage collection.
///
///    Also see [GC Proposal](https://github.com/WebAssembly/gc/blob/main/proposals/gc/Overview.md).
///
///  - `Component` supports the WebAssembly component model. The support is experimental.
///
///    Also see [Component Model Proposal](https://github.com/WebAssembly/component-model/blob/main/design/mvp/Explainer.md).
#[derive(Debug, Clone, Copy)]
pub struct CommonConfigOptions {
    mutable_globals: bool,
    non_trap_conversions: bool,
    sign_extension_operators: bool,
    multi_value: bool,
    bulk_memory_operations: bool,
    reference_types: bool,
    simd: bool,
    relax_simd: bool,
    multi_memories: bool,
    threads: bool,
    gc: bool,
    tail_call: bool,
    extended_const: bool,
    annotations: bool,
    memory64: bool,
    exception_handling: bool,
    function_references: bool,
    component_model: bool,
    run_mode: RunMode,
}
impl CommonConfigOptions {
    /// Creates a new instance of [CommonConfigOptions].
    ///
    /// The default options match the default proposal set of WasmEdge 0.17.1, which enables all
    /// the proposals of the WebAssembly 3.0 standard:
    /// * mutable_globals: true,
    /// * non_trap_conversions: true,
    /// * sign_extension_operators: true,
    /// * multi_value: true,
    /// * bulk_memory_operations: true,
    /// * reference_types: true,
    /// * simd: true,
    /// * relax_simd: true,
    /// * multi_memories: true,
    /// * threads: false,
    /// * gc: true,
    /// * tail_call: true,
    /// * extended_const: true,
    /// * annotations: false,
    /// * memory64: true,
    /// * exception_handling: true,
    /// * function_references: true,
    /// * component_model: false,
    /// * run_mode: RunMode::Interpreter,
    pub fn new() -> Self {
        Self {
            mutable_globals: true,
            non_trap_conversions: true,
            sign_extension_operators: true,
            multi_value: true,
            bulk_memory_operations: true,
            reference_types: true,
            simd: true,
            relax_simd: true,
            multi_memories: true,
            threads: false,
            gc: true,
            tail_call: true,
            extended_const: true,
            annotations: false,
            memory64: true,
            exception_handling: true,
            function_references: true,
            component_model: false,
            run_mode: RunMode::Interpreter,
        }
    }

    /// Creates a new instance of [CommonConfigOptions] with the proposal preset of the given
    /// WebAssembly standard.
    ///
    /// The proposals that do not belong to the given standard, such as `Annotations`, `Threads`,
    /// and `Component`, are disabled.
    ///
    /// # Argument
    ///
    /// - `standard` specifies the WebAssembly standard whose proposal preset is applied.
    pub fn from_wasm_standard(standard: WasmStandard) -> Self {
        let wasm1 = Self {
            mutable_globals: true,
            non_trap_conversions: false,
            sign_extension_operators: false,
            multi_value: false,
            bulk_memory_operations: false,
            reference_types: false,
            simd: false,
            relax_simd: false,
            multi_memories: false,
            threads: false,
            gc: false,
            tail_call: false,
            extended_const: false,
            annotations: false,
            memory64: false,
            exception_handling: false,
            function_references: false,
            component_model: false,
            run_mode: RunMode::Interpreter,
        };
        match standard {
            WasmStandard::Wasm1 => wasm1,
            WasmStandard::Wasm2 => Self {
                non_trap_conversions: true,
                sign_extension_operators: true,
                multi_value: true,
                bulk_memory_operations: true,
                reference_types: true,
                simd: true,
                ..wasm1
            },
            WasmStandard::Wasm3 => Self {
                non_trap_conversions: true,
                sign_extension_operators: true,
                multi_value: true,
                bulk_memory_operations: true,
                reference_types: true,
                simd: true,
                relax_simd: true,
                tail_call: true,
                extended_const: true,
                function_references: true,
                gc: true,
                multi_memories: true,
                exception_handling: true,
                memory64: true,
                ..wasm1
            },
        }
    }

    /// Enables or disables the ImportExportMutGlobals option.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if the option turns on or not.
    pub fn mutable_globals(self, enable: bool) -> Self {
        Self {
            mutable_globals: enable,
            ..self
        }
    }

    /// Enables or disables the NonTrapFloatToIntConversions option.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if the option turns on or not.
    pub fn non_trap_conversions(self, enable: bool) -> Self {
        Self {
            non_trap_conversions: enable,
            ..self
        }
    }

    /// Enables or disables the SignExtensionOperators option.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if the option turns on or not.
    pub fn sign_extension_operators(self, enable: bool) -> Self {
        Self {
            sign_extension_operators: enable,
            ..self
        }
    }

    /// Enables or disables the MultiValue option.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if the option turns on or not.
    pub fn multi_value(self, enable: bool) -> Self {
        Self {
            multi_value: enable,
            ..self
        }
    }

    /// Enables or disables the BulkMemoryOperations option.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if the option turns on or not.
    pub fn bulk_memory_operations(self, enable: bool) -> Self {
        Self {
            bulk_memory_operations: enable,
            ..self
        }
    }

    /// Enables or disables the ReferenceTypes option.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if the option turns on or not.
    pub fn reference_types(self, enable: bool) -> Self {
        Self {
            reference_types: enable,
            ..self
        }
    }

    /// Enables or disables the SIMD option.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if the option turns on or not.
    pub fn simd(self, enable: bool) -> Self {
        Self {
            simd: enable,
            ..self
        }
    }

    /// Enables or disables the RelaxSIMD option.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if the option turns on or not.
    pub fn relax_simd(self, enable: bool) -> Self {
        Self {
            relax_simd: enable,
            ..self
        }
    }

    /// Enables or disables the MultiMemories option.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if the option turns on or not.
    pub fn multi_memories(self, enable: bool) -> Self {
        Self {
            multi_memories: enable,
            ..self
        }
    }

    /// Enables or disables the Threads option.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if the option turns on or not.
    pub fn threads(self, enable: bool) -> Self {
        Self {
            threads: enable,
            ..self
        }
    }

    /// Enables or disables the GC option.
    ///
    /// The GC proposal depends on the FunctionReferences and ReferenceTypes proposals: while GC
    /// is enabled, the runtime keeps both dependencies enabled and ignores disabling them.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if the option turns on or not.
    pub fn gc(self, enable: bool) -> Self {
        Self { gc: enable, ..self }
    }

    /// Enables or disables the TailCall option.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if the option turns on or not.
    pub fn tail_call(self, enable: bool) -> Self {
        Self {
            tail_call: enable,
            ..self
        }
    }

    /// Enables or disables the ExtendedConst option.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if the option turns on or not.
    pub fn extended_const(self, enable: bool) -> Self {
        Self {
            extended_const: enable,
            ..self
        }
    }

    /// Enables or disables the Annotations option.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if the option turns on or not.
    pub fn annotations(self, enable: bool) -> Self {
        Self {
            annotations: enable,
            ..self
        }
    }

    /// Enables or disables the Memory64 option.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if the option turns on or not.
    pub fn memory64(self, enable: bool) -> Self {
        Self {
            memory64: enable,
            ..self
        }
    }

    /// Enables or disables the ExceptionHandling option.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if the option turns on or not.
    pub fn exception_handling(self, enable: bool) -> Self {
        Self {
            exception_handling: enable,
            ..self
        }
    }

    /// Enables or disables the FunctionReferences option.
    ///
    /// The FunctionReferences proposal depends on the ReferenceTypes proposal: while
    /// FunctionReferences is enabled, the runtime keeps ReferenceTypes enabled and ignores
    /// disabling it. Disabling FunctionReferences is ignored while the GC proposal stays enabled.
    ///
    /// # Argument
    ///
    /// * `enable` - Whether the option turns on or not.
    pub fn function_references(self, enable: bool) -> Self {
        Self {
            function_references: enable,
            ..self
        }
    }

    /// Enables or disables the Component option.
    ///
    /// Notice that the WebAssembly component model support in WasmEdge is experimental.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if the option turns on or not.
    pub fn component_model(self, enable: bool) -> Self {
        Self {
            component_model: enable,
            ..self
        }
    }

    /// Enables or disables the `ForceInterpreter` option.
    ///
    /// # Argument
    ///
    /// * `enable` - Whether the option turns on or not.
    #[deprecated(
        note = "use `run_mode` instead; since WasmEdge 0.17.0, passing `false` is a no-op"
    )]
    pub fn interpreter_mode(self, enable: bool) -> Self {
        // Keep the historical "don't force" semantic: only `true` selects the
        // interpreter; `false` leaves the current mode untouched.
        if enable {
            self.run_mode(RunMode::Interpreter)
        } else {
            self
        }
    }

    /// Sets the execution mode: interpreter, JIT, or AOT. By default, the mode is
    /// [RunMode::Interpreter].
    ///
    /// Since WasmEdge 0.17.0, only [RunMode::Aot] loads the AOT custom sections from a universal
    /// WASM file or `dlopen`s a shared-library WASM artifact; in the other modes, the AOT data is
    /// ignored and the WASM runs in the selected engine.
    ///
    /// # Argument
    ///
    /// * `mode` - The execution mode to set.
    pub fn run_mode(self, mode: RunMode) -> Self {
        Self {
            run_mode: mode,
            ..self
        }
    }
}
impl Default for CommonConfigOptions {
    /// Creates a new default instance of [CommonConfigOptions].
    ///
    /// The default options match the default proposal set of WasmEdge 0.17.1, which enables all
    /// the proposals of the WebAssembly 3.0 standard:
    /// * mutable_globals: true,
    /// * non_trap_conversions: true,
    /// * sign_extension_operators: true,
    /// * multi_value: true,
    /// * bulk_memory_operations: true,
    /// * reference_types: true,
    /// * simd: true,
    /// * relax_simd: true,
    /// * multi_memories: true,
    /// * threads: false,
    /// * gc: true,
    /// * tail_call: true,
    /// * extended_const: true,
    /// * annotations: false,
    /// * memory64: true,
    /// * exception_handling: true,
    /// * function_references: true,
    /// * component_model: false,
    /// * run_mode: RunMode::Interpreter,
    fn default() -> Self {
        Self::new()
    }
}

/// Defines a group of configuration options for AOT compiler.
///
/// [CompilerConfigOptions] is used to set the AOT compiler related configuration options, which are
///
///  - Compiler Optimization Levels
///    - `O0` performs as many optimizations as possible.
///    
///    - `O1` optimizes quickly without destroying debuggability  
///    - `02` optimizes for fast execution as much as possible without triggering significant incremental
///      compile time or code size growth  
///    - `O3` optimizes for fast execution as much as possible  
///    - `Os` optimizes for small code size as much as possible without triggering significant incremental
///      compile time or execution time slowdowns  
///    - `Oz` optimizes for small code size as much as possible  
///  - Compiler Output Formats
///    - `Native` specifies the output format is native dynamic library (`*.wasm.so`)  
///    - `Wasm` specifies the output format is WebAssembly with AOT compiled codes in custom section (`*.wasm`).
///  
///  - `dump_ir` determines if AOT compiler generates IR or not  
///  - `generic_binary` determines if AOT compiler generates the generic binary or not.
///  - `interruptible` determines if AOT compiler generates interruptible binary or not.
///  
///  The configuration options above are only effective to [AOT compiler](crate::Compiler).
#[cfg(feature = "aot")]
#[cfg_attr(docsrs, doc(cfg(feature = "aot")))]
#[derive(Debug, Clone, Copy)]
pub struct CompilerConfigOptions {
    out_format: CompilerOutputFormat,
    opt_level: CompilerOptimizationLevel,
    dump_ir: bool,
    generic_binary: bool,
    interruptible: bool,
}
#[cfg(feature = "aot")]
#[cfg_attr(docsrs, doc(cfg(feature = "aot")))]
impl CompilerConfigOptions {
    /// Creates a new instance of [CompilerConfigOptions].
    pub fn new() -> Self {
        Self {
            out_format: CompilerOutputFormat::Wasm,
            opt_level: CompilerOptimizationLevel::O3,
            dump_ir: false,
            generic_binary: false,
            interruptible: false,
        }
    }

    /// Sets the output binary format of AOT compiler.
    ///
    /// # Argument
    ///
    /// - `format` specifies the format of the output binary.
    pub fn out_format(self, format: CompilerOutputFormat) -> Self {
        Self {
            out_format: format,
            ..self
        }
    }

    /// Sets the optimization level of AOT compiler.
    ///
    /// # Argument
    ///
    /// - `level` specifies the optimization level of AOT compiler.
    pub fn optimization_level(self, level: CompilerOptimizationLevel) -> Self {
        Self {
            opt_level: level,
            ..self
        }
    }

    /// Sets the dump IR option of AOT compiler.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if dump ir or not.
    pub fn dump_ir(self, enable: bool) -> Self {
        Self {
            dump_ir: enable,
            ..self
        }
    }

    /// Sets the generic binary option of AOT compiler.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if generate the generic binary or not when perform AOT compilation.
    pub fn generic_binary(self, enable: bool) -> Self {
        Self {
            generic_binary: enable,
            ..self
        }
    }

    /// Enables or Disables the `Interruptible` option of AOT compiler.
    ///
    /// This option determines to generate interruptible binary or not when compilation in AOT compiler.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if turn on the `Interruptible` option.
    pub fn interruptible(self, enable: bool) -> Self {
        Self {
            interruptible: enable,
            ..self
        }
    }
}
#[cfg(feature = "aot")]
#[cfg_attr(docsrs, doc(cfg(feature = "aot")))]
impl Default for CompilerConfigOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Defines a group of runtime configuration options.
///
/// [RuntimeConfigOptions] is used to set the runtime configuration options, which are
///
/// - `maximum_memory_page` limits the page size of [Memory](crate::Memory). This option is only effective to
///   [Executor](crate::Executor).
///
/// - `allow_afunix` allows the use of `AF_UNIX` sockets in the WASI socket implementation.
#[derive(Debug, Clone, Copy)]
pub struct RuntimeConfigOptions {
    max_memory_pages: u32,
    allow_afunix: bool,
}
impl RuntimeConfigOptions {
    /// Creates a new instance of [RuntimeConfigOptions].
    ///
    /// The default options are:
    /// * max_memory_pages: 65536,
    /// * allow_afunix: false,
    pub fn new() -> Self {
        Self {
            max_memory_pages: 65536,
            allow_afunix: false,
        }
    }

    /// Sets the maximum number of the memory pages available.
    ///
    /// # Argument
    ///
    /// - `count` specifies the page count (64KB per page).
    pub fn max_memory_pages(self, count: u32) -> Self {
        Self {
            max_memory_pages: count,
            ..self
        }
    }

    /// Allows or disallows the use of `AF_UNIX` sockets in the WASI socket implementation.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if the option turns on or not.
    pub fn allow_afunix(self, enable: bool) -> Self {
        Self {
            allow_afunix: enable,
            ..self
        }
    }
}
impl Default for RuntimeConfigOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Defines a group of the statistics configuration options.
///
/// [StatisticsConfigOptions] is used to set the statistics configuration options, which are
///
///  - `count_instructions` determines if measuring the count of instructions when running a compiled or pure WASM.
///   
///  - `measure_cost` determines if measuring the instruction costs when running a compiled or pure WASM.
///   
///  - `measure_time` determines if measuring the running time when running a compiled or pure WASM.
#[derive(Debug, Default, Clone, Copy)]
pub struct StatisticsConfigOptions {
    count_instructions: bool,
    measure_cost: bool,
    measure_time: bool,
}
impl StatisticsConfigOptions {
    /// Creates a new instance of [StatisticsConfigOptions].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the instruction counting option.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if support instruction counting or not when execution after AOT compilation.
    pub fn count_instructions(self, enable: bool) -> Self {
        Self {
            count_instructions: enable,
            ..self
        }
    }

    /// Sets the cost measuring option.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if support cost measuring or not when execution after AOT compilation.
    pub fn measure_cost(self, enable: bool) -> Self {
        Self {
            measure_cost: enable,
            ..self
        }
    }

    /// Sets the time measuring option.
    ///
    /// # Argument
    ///
    /// - `enable` specifies if support time measuring or not when execution after AOT compilation.
    pub fn measure_time(self, enable: bool) -> Self {
        Self {
            measure_time: enable,
            ..self
        }
    }
}

#[cfg(test)]
mod proposal_tests {
    use super::*;

    /// The default configuration must match the default proposal set of WasmEdge 0.17.1.
    #[test]
    fn test_config_defaults_match_runtime() {
        let config = ConfigBuilder::new(CommonConfigOptions::default())
            .build()
            .unwrap();

        // enabled by default in WasmEdge 0.17.1
        assert!(config.mutable_globals_enabled());
        assert!(config.non_trap_conversions_enabled());
        assert!(config.sign_extension_operators_enabled());
        assert!(config.multi_value_enabled());
        assert!(config.bulk_memory_operations_enabled());
        assert!(config.reference_types_enabled());
        assert!(config.simd_enabled());
        assert!(config.relax_simd_enabled());
        assert!(config.tail_call_enabled());
        assert!(config.extended_const_enabled());
        assert!(config.function_references_enabled());
        assert!(config.gc_enabled());
        assert!(config.multi_memories_enabled());
        assert!(config.exception_handling_enabled());
        assert!(config.memory64_enabled());

        // disabled by default in WasmEdge 0.17.1
        assert!(!config.threads_enabled());
        assert!(!config.annotations_enabled());
        assert!(!config.component_model_enabled());
        assert!(!config.allow_afunix_enabled());

        assert_eq!(config.run_mode(), RunMode::Interpreter);
        assert_eq!(config.max_memory_pages(), 65536);

        // the built defaults must be identical to a fresh runtime configuration
        let runtime_defaults = sys::Config::create().unwrap();
        assert_eq!(
            config.mutable_globals_enabled(),
            runtime_defaults.mutable_globals_enabled()
        );
        assert_eq!(
            config.non_trap_conversions_enabled(),
            runtime_defaults.non_trap_conversions_enabled()
        );
        assert_eq!(
            config.sign_extension_operators_enabled(),
            runtime_defaults.sign_extension_operators_enabled()
        );
        assert_eq!(
            config.multi_value_enabled(),
            runtime_defaults.multi_value_enabled()
        );
        assert_eq!(
            config.bulk_memory_operations_enabled(),
            runtime_defaults.bulk_memory_operations_enabled()
        );
        assert_eq!(
            config.reference_types_enabled(),
            runtime_defaults.reference_types_enabled()
        );
        assert_eq!(config.simd_enabled(), runtime_defaults.simd_enabled());
        assert_eq!(
            config.relax_simd_enabled(),
            runtime_defaults.relax_simd_enabled()
        );
        assert_eq!(
            config.tail_call_enabled(),
            runtime_defaults.tail_call_enabled()
        );
        assert_eq!(
            config.extended_const_enabled(),
            runtime_defaults.extended_const_enabled()
        );
        assert_eq!(
            config.function_references_enabled(),
            runtime_defaults.function_references_enabled()
        );
        assert_eq!(config.gc_enabled(), runtime_defaults.gc_enabled());
        assert_eq!(
            config.multi_memories_enabled(),
            runtime_defaults.multi_memories_enabled()
        );
        assert_eq!(
            config.exception_handling_enabled(),
            runtime_defaults.exception_handling_enabled()
        );
        assert_eq!(
            config.memory64_enabled(),
            runtime_defaults.memory64_enabled()
        );
        assert_eq!(config.threads_enabled(), runtime_defaults.threads_enabled());
        assert_eq!(
            config.annotations_enabled(),
            runtime_defaults.annotations_enabled()
        );
        assert_eq!(
            config.component_model_enabled(),
            runtime_defaults.component_model_enabled()
        );
        assert_eq!(
            config.allow_afunix_enabled(),
            runtime_defaults.allow_afunix_enabled()
        );
    }

    /// Disabling ReferenceTypes together with its dependents must not be silently ignored:
    /// the dependents (GC, FunctionReferences) must be disabled before the dependency.
    #[test]
    fn test_config_disable_proposals_in_dependency_order() {
        let config = ConfigBuilder::new(
            CommonConfigOptions::default()
                .gc(false)
                .function_references(false)
                .reference_types(false),
        )
        .build()
        .unwrap();

        assert!(!config.gc_enabled());
        assert!(!config.function_references_enabled());
        assert!(!config.reference_types_enabled());
    }

    /// While GC stays enabled, the runtime keeps its dependencies enabled.
    #[test]
    fn test_config_gc_keeps_dependencies_enabled() {
        let config = ConfigBuilder::new(
            CommonConfigOptions::default()
                .gc(true)
                .function_references(false)
                .reference_types(false),
        )
        .build()
        .unwrap();

        assert!(config.gc_enabled());
        assert!(config.function_references_enabled());
        assert!(config.reference_types_enabled());
    }

    #[test]
    fn test_config_wasm_standard_presets() {
        // WASM 1.0
        let config =
            ConfigBuilder::new(CommonConfigOptions::from_wasm_standard(WasmStandard::Wasm1))
                .build()
                .unwrap();
        assert!(config.mutable_globals_enabled());
        assert!(!config.non_trap_conversions_enabled());
        assert!(!config.sign_extension_operators_enabled());
        assert!(!config.multi_value_enabled());
        assert!(!config.bulk_memory_operations_enabled());
        assert!(!config.reference_types_enabled());
        assert!(!config.simd_enabled());
        assert!(!config.relax_simd_enabled());
        assert!(!config.tail_call_enabled());
        assert!(!config.extended_const_enabled());
        assert!(!config.function_references_enabled());
        assert!(!config.gc_enabled());
        assert!(!config.multi_memories_enabled());
        assert!(!config.exception_handling_enabled());
        assert!(!config.memory64_enabled());

        // WASM 2.0
        let config =
            ConfigBuilder::new(CommonConfigOptions::from_wasm_standard(WasmStandard::Wasm2))
                .build()
                .unwrap();
        assert!(config.mutable_globals_enabled());
        assert!(config.non_trap_conversions_enabled());
        assert!(config.sign_extension_operators_enabled());
        assert!(config.multi_value_enabled());
        assert!(config.bulk_memory_operations_enabled());
        assert!(config.reference_types_enabled());
        assert!(config.simd_enabled());
        assert!(!config.relax_simd_enabled());
        assert!(!config.tail_call_enabled());
        assert!(!config.gc_enabled());
        assert!(!config.memory64_enabled());

        // WASM 3.0: matches the WasmEdge 0.17.1 defaults
        let config =
            ConfigBuilder::new(CommonConfigOptions::from_wasm_standard(WasmStandard::Wasm3))
                .build()
                .unwrap();
        let defaults = ConfigBuilder::new(CommonConfigOptions::default())
            .build()
            .unwrap();
        assert_eq!(
            config.mutable_globals_enabled(),
            defaults.mutable_globals_enabled()
        );
        assert_eq!(
            config.non_trap_conversions_enabled(),
            defaults.non_trap_conversions_enabled()
        );
        assert_eq!(
            config.sign_extension_operators_enabled(),
            defaults.sign_extension_operators_enabled()
        );
        assert_eq!(config.multi_value_enabled(), defaults.multi_value_enabled());
        assert_eq!(
            config.bulk_memory_operations_enabled(),
            defaults.bulk_memory_operations_enabled()
        );
        assert_eq!(
            config.reference_types_enabled(),
            defaults.reference_types_enabled()
        );
        assert_eq!(config.simd_enabled(), defaults.simd_enabled());
        assert_eq!(config.relax_simd_enabled(), defaults.relax_simd_enabled());
        assert_eq!(config.tail_call_enabled(), defaults.tail_call_enabled());
        assert_eq!(
            config.extended_const_enabled(),
            defaults.extended_const_enabled()
        );
        assert_eq!(
            config.function_references_enabled(),
            defaults.function_references_enabled()
        );
        assert_eq!(config.gc_enabled(), defaults.gc_enabled());
        assert_eq!(
            config.multi_memories_enabled(),
            defaults.multi_memories_enabled()
        );
        assert_eq!(
            config.exception_handling_enabled(),
            defaults.exception_handling_enabled()
        );
        assert_eq!(config.memory64_enabled(), defaults.memory64_enabled());
        assert_eq!(config.threads_enabled(), defaults.threads_enabled());
        assert_eq!(config.annotations_enabled(), defaults.annotations_enabled());
        assert_eq!(
            config.component_model_enabled(),
            defaults.component_model_enabled()
        );
    }

    #[test]
    fn test_config_toggle_new_proposals() {
        let config = ConfigBuilder::new(
            CommonConfigOptions::default()
                .relax_simd(false)
                .extended_const(false)
                .exception_handling(false)
                .memory64(false)
                .annotations(true)
                .component_model(true),
        )
        .with_runtime_config(RuntimeConfigOptions::default().allow_afunix(true))
        .build()
        .unwrap();

        assert!(!config.relax_simd_enabled());
        assert!(!config.extended_const_enabled());
        assert!(!config.exception_handling_enabled());
        assert!(!config.memory64_enabled());
        assert!(config.annotations_enabled());
        assert!(config.component_model_enabled());
        assert!(config.allow_afunix_enabled());
    }
}

#[cfg(test)]
#[cfg(feature = "aot")]
mod tests {
    use super::*;

    #[test]
    fn test_config_create() {
        let common_options = CommonConfigOptions::default()
            .bulk_memory_operations(true)
            .multi_value(true)
            .mutable_globals(true)
            .non_trap_conversions(true)
            .reference_types(true)
            .sign_extension_operators(true)
            .simd(true)
            .multi_memories(true)
            .run_mode(RunMode::Aot);

        let compiler_options = CompilerConfigOptions::default()
            .dump_ir(true)
            .generic_binary(true)
            .interruptible(true)
            .optimization_level(CompilerOptimizationLevel::O0)
            .out_format(CompilerOutputFormat::Native);

        let stat_options = StatisticsConfigOptions::default()
            .count_instructions(true)
            .measure_cost(true)
            .measure_time(true);

        let runtime_options = RuntimeConfigOptions::default().max_memory_pages(1024);

        let result = ConfigBuilder::new(common_options)
            .with_statistics_config(stat_options)
            .with_compiler_config(compiler_options)
            .with_runtime_config(runtime_options)
            .build();
        assert!(result.is_ok());
        let config = result.unwrap();

        // check common config options
        assert!(config.bulk_memory_operations_enabled());
        assert!(config.multi_value_enabled());
        assert!(config.mutable_globals_enabled());
        assert!(config.non_trap_conversions_enabled());
        assert!(config.reference_types_enabled());
        assert!(config.sign_extension_operators_enabled());
        assert!(config.simd_enabled());
        assert!(config.multi_memories_enabled());
        assert_eq!(config.run_mode(), RunMode::Aot);

        // check compiler config options
        assert!(config.dump_ir_enabled());
        assert!(config.generic_binary_enabled());
        assert!(config.interruptible_enabled());
        assert_eq!(config.optimization_level(), CompilerOptimizationLevel::O0);
        assert_eq!(config.out_format(), CompilerOutputFormat::Native);

        // check statistics config options
        assert!(config.instruction_counting_enabled());
        assert!(config.cost_measuring_enabled());
        assert!(config.time_measuring_enabled());

        // check runtime config options
        assert_eq!(config.max_memory_pages(), 1024);
    }

    #[test]
    fn test_config_copy() {
        let common_config = CommonConfigOptions::default()
            .simd(false)
            .multi_memories(true);
        let compiler_config =
            CompilerConfigOptions::default().optimization_level(CompilerOptimizationLevel::O0);
        let stat_config = StatisticsConfigOptions::default().measure_time(false);
        let runtime_config = RuntimeConfigOptions::default().max_memory_pages(1024);

        let result = ConfigBuilder::new(common_config)
            .with_statistics_config(stat_config)
            .with_compiler_config(compiler_config)
            .with_runtime_config(runtime_config)
            .build();
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(!config.simd_enabled());
        assert!(config.multi_memories_enabled());
        assert_eq!(config.optimization_level(), CompilerOptimizationLevel::O0);
        assert!(!config.time_measuring_enabled());
        assert_eq!(config.max_memory_pages(), 1024);

        // make a copy
        let config_copied = config.clone();
        assert!(!config_copied.simd_enabled());
        assert!(config_copied.multi_memories_enabled());
        assert_eq!(
            config_copied.optimization_level(),
            CompilerOptimizationLevel::O0
        );
        assert!(!config.time_measuring_enabled());
        assert_eq!(config_copied.max_memory_pages(), 1024);
    }
}
