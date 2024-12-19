(function() {
    var type_impls = Object.fromEntries([["wasmedge_sys",[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-AsInstance-for-Instance\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#25-29\">Source</a><a href=\"#impl-AsInstance-for-Instance\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"wasmedge_sys/trait.AsInstance.html\" title=\"trait wasmedge_sys::AsInstance\">AsInstance</a> for <a class=\"struct\" href=\"wasmedge_sys/struct.Instance.html\" title=\"struct wasmedge_sys::Instance\">Instance</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.as_ptr\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#26-28\">Source</a><a href=\"#method.as_ptr\" class=\"anchor\">§</a><h4 class=\"code-header\">unsafe fn <a href=\"wasmedge_sys/trait.AsInstance.html#tymethod.as_ptr\" class=\"fn\">as_ptr</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.pointer.html\">*const </a><a class=\"struct\" href=\"wasmedge_sys/ffi/struct.WasmEdge_ModuleInstanceContext.html\" title=\"struct wasmedge_sys::ffi::WasmEdge_ModuleInstanceContext\">WasmEdge_ModuleInstanceContext</a></h4></section></summary><div class='docblock'>Safety <a href=\"wasmedge_sys/trait.AsInstance.html#tymethod.as_ptr\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.name\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#50-59\">Source</a><a href=\"#method.name\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"wasmedge_sys/trait.AsInstance.html#method.name\" class=\"fn\">name</a>(&amp;self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/string/struct.String.html\" title=\"struct alloc::string::String\">String</a>&gt;</h4></section></summary><div class='docblock'>Returns the name of this exported <a href=\"wasmedge_sys/struct.Instance.html\" title=\"struct wasmedge_sys::Instance\">module instance</a>. <a href=\"wasmedge_sys/trait.AsInstance.html#method.name\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.get_table\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#70-88\">Source</a><a href=\"#method.get_table\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"wasmedge_sys/trait.AsInstance.html#method.get_table\" class=\"fn\">get_table</a>(\n    &amp;self,\n    name: impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.AsRef.html\" title=\"trait core::convert::AsRef\">AsRef</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>&gt;,\n) -&gt; <a class=\"type\" href=\"wasmedge_types/type.WasmEdgeResult.html\" title=\"type wasmedge_types::WasmEdgeResult\">WasmEdgeResult</a>&lt;<a class=\"struct\" href=\"wasmedge_sys/instance/struct.InnerRef.html\" title=\"struct wasmedge_sys::instance::InnerRef\">InnerRef</a>&lt;<a class=\"struct\" href=\"wasmedge_sys/struct.Table.html\" title=\"struct wasmedge_sys::Table\">Table</a>, &amp;Self&gt;&gt;<div class=\"where\">where\n    Self: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</div></h4></section></summary><div class='docblock'>Returns the exported <a href=\"wasmedge_sys/struct.Table.html\" title=\"struct wasmedge_sys::Table\">table instance</a> by name. <a href=\"wasmedge_sys/trait.AsInstance.html#method.get_table\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.get_memory_ref\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#99-122\">Source</a><a href=\"#method.get_memory_ref\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"wasmedge_sys/trait.AsInstance.html#method.get_memory_ref\" class=\"fn\">get_memory_ref</a>(\n    &amp;self,\n    name: impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.AsRef.html\" title=\"trait core::convert::AsRef\">AsRef</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>&gt;,\n) -&gt; <a class=\"type\" href=\"wasmedge_types/type.WasmEdgeResult.html\" title=\"type wasmedge_types::WasmEdgeResult\">WasmEdgeResult</a>&lt;<a class=\"struct\" href=\"wasmedge_sys/instance/struct.InnerRef.html\" title=\"struct wasmedge_sys::instance::InnerRef\">InnerRef</a>&lt;<a class=\"struct\" href=\"wasmedge_sys/struct.Memory.html\" title=\"struct wasmedge_sys::Memory\">Memory</a>, &amp;Self&gt;&gt;<div class=\"where\">where\n    Self: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</div></h4></section></summary><div class='docblock'>Returns the exported <a href=\"wasmedge_sys/struct.Memory.html\" title=\"struct wasmedge_sys::Memory\">memory instance</a> by name. <a href=\"wasmedge_sys/trait.AsInstance.html#method.get_memory_ref\">Read more</a></div></details><section id=\"method.get_memory_mut\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#124-150\">Source</a><a href=\"#method.get_memory_mut\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"wasmedge_sys/trait.AsInstance.html#method.get_memory_mut\" class=\"fn\">get_memory_mut</a>(\n    &amp;mut self,\n    name: impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.AsRef.html\" title=\"trait core::convert::AsRef\">AsRef</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>&gt;,\n) -&gt; <a class=\"type\" href=\"wasmedge_types/type.WasmEdgeResult.html\" title=\"type wasmedge_types::WasmEdgeResult\">WasmEdgeResult</a>&lt;<a class=\"struct\" href=\"wasmedge_sys/instance/struct.InnerRef.html\" title=\"struct wasmedge_sys::instance::InnerRef\">InnerRef</a>&lt;<a class=\"struct\" href=\"wasmedge_sys/struct.Memory.html\" title=\"struct wasmedge_sys::Memory\">Memory</a>, &amp;mut Self&gt;&gt;<div class=\"where\">where\n    Self: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</div></h4></section><details class=\"toggle method-toggle\" open><summary><section id=\"method.get_global\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#161-179\">Source</a><a href=\"#method.get_global\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"wasmedge_sys/trait.AsInstance.html#method.get_global\" class=\"fn\">get_global</a>(\n    &amp;self,\n    name: impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.AsRef.html\" title=\"trait core::convert::AsRef\">AsRef</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>&gt;,\n) -&gt; <a class=\"type\" href=\"wasmedge_types/type.WasmEdgeResult.html\" title=\"type wasmedge_types::WasmEdgeResult\">WasmEdgeResult</a>&lt;<a class=\"struct\" href=\"wasmedge_sys/instance/struct.InnerRef.html\" title=\"struct wasmedge_sys::instance::InnerRef\">InnerRef</a>&lt;<a class=\"struct\" href=\"wasmedge_sys/struct.Global.html\" title=\"struct wasmedge_sys::Global\">Global</a>, &amp;Self&gt;&gt;<div class=\"where\">where\n    Self: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</div></h4></section></summary><div class='docblock'>Returns the exported <a href=\"wasmedge_sys/struct.Global.html\" title=\"struct wasmedge_sys::Global\">global instance</a> by name. <a href=\"wasmedge_sys/trait.AsInstance.html#method.get_global\">Read more</a></div></details><section id=\"method.get_global_mut\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#181-202\">Source</a><a href=\"#method.get_global_mut\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"wasmedge_sys/trait.AsInstance.html#method.get_global_mut\" class=\"fn\">get_global_mut</a>(\n    &amp;mut self,\n    name: impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.AsRef.html\" title=\"trait core::convert::AsRef\">AsRef</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>&gt;,\n) -&gt; <a class=\"type\" href=\"wasmedge_types/type.WasmEdgeResult.html\" title=\"type wasmedge_types::WasmEdgeResult\">WasmEdgeResult</a>&lt;<a class=\"struct\" href=\"wasmedge_sys/instance/struct.InnerRef.html\" title=\"struct wasmedge_sys::instance::InnerRef\">InnerRef</a>&lt;<a class=\"struct\" href=\"wasmedge_sys/struct.Global.html\" title=\"struct wasmedge_sys::Global\">Global</a>, &amp;mut Self&gt;&gt;<div class=\"where\">where\n    Self: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</div></h4></section><details class=\"toggle method-toggle\" open><summary><section id=\"method.func_len\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#205-207\">Source</a><a href=\"#method.func_len\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"wasmedge_sys/trait.AsInstance.html#method.func_len\" class=\"fn\">func_len</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u32.html\">u32</a></h4></section></summary><div class='docblock'>Returns the length of the exported <a href=\"wasmedge_sys/struct.Function.html\" title=\"struct wasmedge_sys::Function\">function instances</a> in this module instance.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.func_names\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#210-232\">Source</a><a href=\"#method.func_names\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"wasmedge_sys/trait.AsInstance.html#method.func_names\" class=\"fn\">func_names</a>(&amp;self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/string/struct.String.html\" title=\"struct alloc::string::String\">String</a>&gt;&gt;</h4></section></summary><div class='docblock'>Returns the names of the exported <a href=\"wasmedge_sys/struct.Function.html\" title=\"struct wasmedge_sys::Function\">function instances</a> in this module instance.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.get_func\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#243-258\">Source</a><a href=\"#method.get_func\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"wasmedge_sys/trait.AsInstance.html#method.get_func\" class=\"fn\">get_func</a>(&amp;self, name: &amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>) -&gt; <a class=\"type\" href=\"wasmedge_types/type.WasmEdgeResult.html\" title=\"type wasmedge_types::WasmEdgeResult\">WasmEdgeResult</a>&lt;<a class=\"type\" href=\"wasmedge_sys/type.FuncRef.html\" title=\"type wasmedge_sys::FuncRef\">FuncRef</a>&lt;&amp;<a class=\"struct\" href=\"wasmedge_sys/struct.Instance.html\" title=\"struct wasmedge_sys::Instance\">Instance</a>&gt;&gt;</h4></section></summary><div class='docblock'>Returns the exported <a href=\"wasmedge_sys/struct.Function.html\" title=\"struct wasmedge_sys::Function\">function instance</a> by name. <a href=\"wasmedge_sys/trait.AsInstance.html#method.get_func\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.get_func_mut\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#269-284\">Source</a><a href=\"#method.get_func_mut\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"wasmedge_sys/trait.AsInstance.html#method.get_func_mut\" class=\"fn\">get_func_mut</a>(&amp;mut self, name: &amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>) -&gt; <a class=\"type\" href=\"wasmedge_types/type.WasmEdgeResult.html\" title=\"type wasmedge_types::WasmEdgeResult\">WasmEdgeResult</a>&lt;<a class=\"type\" href=\"wasmedge_sys/type.FuncRef.html\" title=\"type wasmedge_sys::FuncRef\">FuncRef</a>&lt;&amp;mut <a class=\"struct\" href=\"wasmedge_sys/struct.Instance.html\" title=\"struct wasmedge_sys::Instance\">Instance</a>&gt;&gt;</h4></section></summary><div class='docblock'>Returns the exported <a href=\"wasmedge_sys/struct.Function.html\" title=\"struct wasmedge_sys::Function\">function instance</a> by name. <a href=\"wasmedge_sys/trait.AsInstance.html#method.get_func_mut\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.table_len\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#287-289\">Source</a><a href=\"#method.table_len\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"wasmedge_sys/trait.AsInstance.html#method.table_len\" class=\"fn\">table_len</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u32.html\">u32</a></h4></section></summary><div class='docblock'>Returns the length of the exported <a href=\"wasmedge_sys/struct.Table.html\" title=\"struct wasmedge_sys::Table\">table instances</a> in this module instance.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.table_names\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#292-314\">Source</a><a href=\"#method.table_names\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"wasmedge_sys/trait.AsInstance.html#method.table_names\" class=\"fn\">table_names</a>(&amp;self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/string/struct.String.html\" title=\"struct alloc::string::String\">String</a>&gt;&gt;</h4></section></summary><div class='docblock'>Returns the names of the exported <a href=\"wasmedge_sys/struct.Table.html\" title=\"struct wasmedge_sys::Table\">table instances</a> in this module instance.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.mem_len\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#317-319\">Source</a><a href=\"#method.mem_len\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"wasmedge_sys/trait.AsInstance.html#method.mem_len\" class=\"fn\">mem_len</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u32.html\">u32</a></h4></section></summary><div class='docblock'>Returns the length of the exported <a href=\"wasmedge_sys/struct.Memory.html\" title=\"struct wasmedge_sys::Memory\">memory instances</a> in this module instance.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.mem_names\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#322-344\">Source</a><a href=\"#method.mem_names\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"wasmedge_sys/trait.AsInstance.html#method.mem_names\" class=\"fn\">mem_names</a>(&amp;self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/string/struct.String.html\" title=\"struct alloc::string::String\">String</a>&gt;&gt;</h4></section></summary><div class='docblock'>Returns the names of all exported <a href=\"wasmedge_sys/struct.Memory.html\" title=\"struct wasmedge_sys::Memory\">memory instances</a> in this module instance.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.global_len\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#347-349\">Source</a><a href=\"#method.global_len\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"wasmedge_sys/trait.AsInstance.html#method.global_len\" class=\"fn\">global_len</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u32.html\">u32</a></h4></section></summary><div class='docblock'>Returns the length of the exported <a href=\"wasmedge_sys/struct.Global.html\" title=\"struct wasmedge_sys::Global\">global instances</a> in this module instance.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.global_names\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#352-374\">Source</a><a href=\"#method.global_names\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"wasmedge_sys/trait.AsInstance.html#method.global_names\" class=\"fn\">global_names</a>(&amp;self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/string/struct.String.html\" title=\"struct alloc::string::String\">String</a>&gt;&gt;</h4></section></summary><div class='docblock'>Returns the names of the exported <a href=\"wasmedge_sys/struct.Global.html\" title=\"struct wasmedge_sys::Global\">global instances</a> in this module instance.</div></details></div></details>","AsInstance","wasmedge_sys::plugin::PluginModule"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-Instance\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#14\">Source</a><a href=\"#impl-Debug-for-Instance\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"wasmedge_sys/struct.Instance.html\" title=\"struct wasmedge_sys::Instance\">Instance</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#14\">Source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/nightly/core/fmt/type.Result.html\" title=\"type core::fmt::Result\">Result</a></h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","wasmedge_sys::plugin::PluginModule"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Drop-for-Instance\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#18-24\">Source</a><a href=\"#impl-Drop-for-Instance\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"wasmedge_sys/struct.Instance.html\" title=\"struct wasmedge_sys::Instance\">Instance</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.drop\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/module.rs.html#19-23\">Source</a><a href=\"#method.drop\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html#tymethod.drop\" class=\"fn\">drop</a>(&amp;mut self)</h4></section></summary><div class='docblock'>Executes the destructor for this type. <a href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html#tymethod.drop\">Read more</a></div></details></div></details>","Drop","wasmedge_sys::plugin::PluginModule"]]]]);
    if (window.register_type_impls) {
        window.register_type_impls(type_impls);
    } else {
        window.pending_type_impls = type_impls;
    }
})()
//{"start":55,"fragment_lengths":[21822]}