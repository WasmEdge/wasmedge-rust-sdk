(function() {
    var type_impls = Object.fromEntries([["wasmedge_sys",[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Clone-for-InnerRef%3CD,+%26mut+Ref%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/mod.rs.html#80-87\">Source</a><a href=\"#impl-Clone-for-InnerRef%3CD,+%26mut+Ref%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;D: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>, Ref&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"wasmedge_sys/instance/struct.InnerRef.html\" title=\"struct wasmedge_sys::instance::InnerRef\">InnerRef</a>&lt;D, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;mut Ref</a>&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/mod.rs.html#81-86\">Source</a><a href=\"#method.clone\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone\" class=\"fn\">clone</a>(&amp;self) -&gt; Self</h4></section></summary><div class='docblock'>Returns a copy of the value. <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone_from\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/clone.rs.html#174\">Source</a></span><a href=\"#method.clone_from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from\" class=\"fn\">clone_from</a>(&amp;mut self, source: &amp;Self)</h4></section></summary><div class='docblock'>Performs copy-assignment from <code>source</code>. <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from\">Read more</a></div></details></div></details>","Clone","wasmedge_sys::instance::function::FuncRef"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-InnerRef%3CD,+Ref%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/mod.rs.html#67-71\">Source</a><a href=\"#impl-Debug-for-InnerRef%3CD,+Ref%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;D: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>, Ref&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"wasmedge_sys/instance/struct.InnerRef.html\" title=\"struct wasmedge_sys::instance::InnerRef\">InnerRef</a>&lt;D, Ref&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/mod.rs.html#68-70\">Source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/nightly/core/fmt/type.Result.html\" title=\"type core::fmt::Result\">Result</a></h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","wasmedge_sys::instance::function::FuncRef"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Deref-for-InnerRef%3CD,+%26Ref%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/mod.rs.html#73-78\">Source</a><a href=\"#impl-Deref-for-InnerRef%3CD,+%26Ref%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;D, Ref&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"wasmedge_sys/instance/struct.InnerRef.html\" title=\"struct wasmedge_sys::instance::InnerRef\">InnerRef</a>&lt;D, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;Ref</a>&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedtype.Target\" class=\"associatedtype trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/mod.rs.html#74\">Source</a><a href=\"#associatedtype.Target\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html#associatedtype.Target\" class=\"associatedtype\">Target</a> = D</h4></section></summary><div class='docblock'>The resulting type after dereferencing.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.deref\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/mod.rs.html#75-77\">Source</a><a href=\"#method.deref\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html#tymethod.deref\" class=\"fn\">deref</a>(&amp;self) -&gt; &amp;Self::<a class=\"associatedtype\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html#associatedtype.Target\" title=\"type core::ops::deref::Deref::Target\">Target</a></h4></section></summary><div class='docblock'>Dereferences the value.</div></details></div></details>","Deref","wasmedge_sys::instance::function::FuncRef"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Deref-for-InnerRef%3CD,+%26mut+Ref%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/mod.rs.html#89-94\">Source</a><a href=\"#impl-Deref-for-InnerRef%3CD,+%26mut+Ref%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;D, Ref&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"wasmedge_sys/instance/struct.InnerRef.html\" title=\"struct wasmedge_sys::instance::InnerRef\">InnerRef</a>&lt;D, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;mut Ref</a>&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedtype.Target\" class=\"associatedtype trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/mod.rs.html#90\">Source</a><a href=\"#associatedtype.Target\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html#associatedtype.Target\" class=\"associatedtype\">Target</a> = D</h4></section></summary><div class='docblock'>The resulting type after dereferencing.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.deref\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/mod.rs.html#91-93\">Source</a><a href=\"#method.deref\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html#tymethod.deref\" class=\"fn\">deref</a>(&amp;self) -&gt; &amp;Self::<a class=\"associatedtype\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html#associatedtype.Target\" title=\"type core::ops::deref::Deref::Target\">Target</a></h4></section></summary><div class='docblock'>Dereferences the value.</div></details></div></details>","Deref","wasmedge_sys::instance::function::FuncRef"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-DerefMut-for-InnerRef%3CD,+%26mut+Ref%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/mod.rs.html#96-100\">Source</a><a href=\"#impl-DerefMut-for-InnerRef%3CD,+%26mut+Ref%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;D, Ref&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.DerefMut.html\" title=\"trait core::ops::deref::DerefMut\">DerefMut</a> for <a class=\"struct\" href=\"wasmedge_sys/instance/struct.InnerRef.html\" title=\"struct wasmedge_sys::instance::InnerRef\">InnerRef</a>&lt;D, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;mut Ref</a>&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.deref_mut\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/mod.rs.html#97-99\">Source</a><a href=\"#method.deref_mut\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.DerefMut.html#tymethod.deref_mut\" class=\"fn\">deref_mut</a>(&amp;mut self) -&gt; &amp;mut Self::<a class=\"associatedtype\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html#associatedtype.Target\" title=\"type core::ops::deref::Deref::Target\">Target</a></h4></section></summary><div class='docblock'>Mutably dereferences the value.</div></details></div></details>","DerefMut","wasmedge_sys::instance::function::FuncRef"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-InnerRef%3CD,+%26Ref%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/mod.rs.html#31-47\">Source</a><a href=\"#impl-InnerRef%3CD,+%26Ref%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;D, Ref: ?<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>&gt; <a class=\"struct\" href=\"wasmedge_sys/instance/struct.InnerRef.html\" title=\"struct wasmedge_sys::instance::InnerRef\">InnerRef</a>&lt;D, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;Ref</a>&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.create_from_ref\" class=\"method\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/mod.rs.html#35-38\">Source</a><h4 class=\"code-header\">pub unsafe fn <a href=\"wasmedge_sys/instance/struct.InnerRef.html#tymethod.create_from_ref\" class=\"fn\">create_from_ref</a>(value: <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/mem/manually_drop/struct.ManuallyDrop.html\" title=\"struct core::mem::manually_drop::ManuallyDrop\">ManuallyDrop</a>&lt;D&gt;, _r: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;Ref</a>) -&gt; Self</h4></section></summary><div class=\"docblock\"><h5 id=\"safety\"><a class=\"doc-anchor\" href=\"#safety\">§</a>Safety</h5>\n<p>The return value type of this function should ensure the correctness of lifetimes.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.create_ref\" class=\"method\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/mod.rs.html#43-46\">Source</a><h4 class=\"code-header\">pub unsafe fn <a href=\"wasmedge_sys/instance/struct.InnerRef.html#tymethod.create_ref\" class=\"fn\">create_ref</a>(value: <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/mem/manually_drop/struct.ManuallyDrop.html\" title=\"struct core::mem::manually_drop::ManuallyDrop\">ManuallyDrop</a>&lt;D&gt;) -&gt; Self</h4></section></summary><div class=\"docblock\"><h5 id=\"safety-1\"><a class=\"doc-anchor\" href=\"#safety-1\">§</a>Safety</h5>\n<p>The return value type of this function should ensure the correctness of lifetimes.</p>\n</div></details></div></details>",0,"wasmedge_sys::instance::function::FuncRef"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-InnerRef%3CD,+%26mut+Ref%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/mod.rs.html#49-65\">Source</a><a href=\"#impl-InnerRef%3CD,+%26mut+Ref%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;D, Ref: ?<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>&gt; <a class=\"struct\" href=\"wasmedge_sys/instance/struct.InnerRef.html\" title=\"struct wasmedge_sys::instance::InnerRef\">InnerRef</a>&lt;D, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;mut Ref</a>&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.create_from_mut\" class=\"method\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/mod.rs.html#53-56\">Source</a><h4 class=\"code-header\">pub unsafe fn <a href=\"wasmedge_sys/instance/struct.InnerRef.html#tymethod.create_from_mut\" class=\"fn\">create_from_mut</a>(value: <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/mem/manually_drop/struct.ManuallyDrop.html\" title=\"struct core::mem::manually_drop::ManuallyDrop\">ManuallyDrop</a>&lt;D&gt;, _r: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;mut Ref</a>) -&gt; Self</h4></section></summary><div class=\"docblock\"><h5 id=\"safety\"><a class=\"doc-anchor\" href=\"#safety\">§</a>Safety</h5>\n<p>The return value type of this function should ensure the correctness of lifetimes.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.create_mut\" class=\"method\"><a class=\"src rightside\" href=\"src/wasmedge_sys/instance/mod.rs.html#61-64\">Source</a><h4 class=\"code-header\">pub unsafe fn <a href=\"wasmedge_sys/instance/struct.InnerRef.html#tymethod.create_mut\" class=\"fn\">create_mut</a>(value: <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/mem/manually_drop/struct.ManuallyDrop.html\" title=\"struct core::mem::manually_drop::ManuallyDrop\">ManuallyDrop</a>&lt;D&gt;) -&gt; Self</h4></section></summary><div class=\"docblock\"><h5 id=\"safety-1\"><a class=\"doc-anchor\" href=\"#safety-1\">§</a>Safety</h5>\n<p>The return value type of this function should ensure the correctness of lifetimes.</p>\n</div></details></div></details>",0,"wasmedge_sys::instance::function::FuncRef"]]]]);
    if (window.register_type_impls) {
        window.register_type_impls(type_impls);
    } else {
        window.pending_type_impls = type_impls;
    }
})()
//{"start":55,"fragment_lengths":[15059]}