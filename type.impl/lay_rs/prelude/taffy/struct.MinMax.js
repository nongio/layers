(function() {
    var type_impls = Object.fromEntries([["lay_rs",[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Clone-for-MinMax%3CMin,+Max%3E\" class=\"impl\"><a href=\"#impl-Clone-for-MinMax%3CMin,+Max%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;Min, Max&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;Min, Max&gt;<div class=\"where\">where\n    Min: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    Max: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone\" class=\"method trait-impl\"><a href=\"#method.clone\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.82.0/core/clone/trait.Clone.html#tymethod.clone\" class=\"fn\">clone</a>(&amp;self) -&gt; <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;Min, Max&gt;</h4></section></summary><div class='docblock'>Returns a copy of the value. <a href=\"https://doc.rust-lang.org/1.82.0/core/clone/trait.Clone.html#tymethod.clone\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone_from\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/1.82.0/src/core/clone.rs.html#174\">source</a></span><a href=\"#method.clone_from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.82.0/core/clone/trait.Clone.html#method.clone_from\" class=\"fn\">clone_from</a>(&amp;mut self, source: &amp;Self)</h4></section></summary><div class='docblock'>Performs copy-assignment from <code>source</code>. <a href=\"https://doc.rust-lang.org/1.82.0/core/clone/trait.Clone.html#method.clone_from\">Read more</a></div></details></div></details>","Clone","lay_rs::prelude::taffy::style::NonRepeatedTrackSizingFunction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-MinMax%3CMin,+Max%3E\" class=\"impl\"><a href=\"#impl-Debug-for-MinMax%3CMin,+Max%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;Min, Max&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;Min, Max&gt;<div class=\"where\">where\n    Min: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>,\n    Max: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.82.0/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/1.82.0/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.82.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.82.0/std/primitive.unit.html\">()</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/1.82.0/core/fmt/struct.Error.html\" title=\"struct core::fmt::Error\">Error</a>&gt;</h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/1.82.0/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","lay_rs::prelude::taffy::style::NonRepeatedTrackSizingFunction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Deserialize%3C'de%3E-for-MinMax%3CMin,+Max%3E\" class=\"impl\"><a href=\"#impl-Deserialize%3C'de%3E-for-MinMax%3CMin,+Max%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'de, Min, Max&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.210/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;Min, Max&gt;<div class=\"where\">where\n    Min: <a class=\"trait\" href=\"https://docs.rs/serde/1.0.210/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt;,\n    Max: <a class=\"trait\" href=\"https://docs.rs/serde/1.0.210/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt;,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.deserialize\" class=\"method trait-impl\"><a href=\"#method.deserialize\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://docs.rs/serde/1.0.210/serde/de/trait.Deserialize.html#tymethod.deserialize\" class=\"fn\">deserialize</a>&lt;__D&gt;(\n    __deserializer: __D,\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.82.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;Min, Max&gt;, &lt;__D as <a class=\"trait\" href=\"https://docs.rs/serde/1.0.210/serde/de/trait.Deserializer.html\" title=\"trait serde::de::Deserializer\">Deserializer</a>&lt;'de&gt;&gt;::<a class=\"associatedtype\" href=\"https://docs.rs/serde/1.0.210/serde/de/trait.Deserializer.html#associatedtype.Error\" title=\"type serde::de::Deserializer::Error\">Error</a>&gt;<div class=\"where\">where\n    __D: <a class=\"trait\" href=\"https://docs.rs/serde/1.0.210/serde/de/trait.Deserializer.html\" title=\"trait serde::de::Deserializer\">Deserializer</a>&lt;'de&gt;,</div></h4></section></summary><div class='docblock'>Deserialize this value from the given Serde deserializer. <a href=\"https://docs.rs/serde/1.0.210/serde/de/trait.Deserialize.html#tymethod.deserialize\">Read more</a></div></details></div></details>","Deserialize<'de>","lay_rs::prelude::taffy::style::NonRepeatedTrackSizingFunction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-FromFlex-for-MinMax%3CMinTrackSizingFunction,+MaxTrackSizingFunction%3E\" class=\"impl\"><a href=\"#impl-FromFlex-for-MinMax%3CMinTrackSizingFunction,+MaxTrackSizingFunction%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"lay_rs/prelude/taffy/style_helpers/trait.FromFlex.html\" title=\"trait lay_rs::prelude::taffy::style_helpers::FromFlex\">FromFlex</a> for <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;<a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MinTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MinTrackSizingFunction\">MinTrackSizingFunction</a>, <a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MaxTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MaxTrackSizingFunction\">MaxTrackSizingFunction</a>&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.from_flex\" class=\"method trait-impl\"><a href=\"#method.from_flex\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"lay_rs/prelude/taffy/style_helpers/trait.FromFlex.html#tymethod.from_flex\" class=\"fn\">from_flex</a>&lt;Input&gt;(\n    flex: Input,\n) -&gt; <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;<a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MinTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MinTrackSizingFunction\">MinTrackSizingFunction</a>, <a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MaxTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MaxTrackSizingFunction\">MaxTrackSizingFunction</a>&gt;<div class=\"where\">where\n    Input: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/convert/trait.Into.html\" title=\"trait core::convert::Into\">Into</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.82.0/std/primitive.f32.html\">f32</a>&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a>,</div></h4></section></summary><div class='docblock'>Converts into an <code>Into&lt;f32&gt;</code> into Self</div></details></div></details>","FromFlex","lay_rs::prelude::taffy::style::NonRepeatedTrackSizingFunction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-FromLength-for-MinMax%3CMinTrackSizingFunction,+MaxTrackSizingFunction%3E\" class=\"impl\"><a href=\"#impl-FromLength-for-MinMax%3CMinTrackSizingFunction,+MaxTrackSizingFunction%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"lay_rs/prelude/taffy/style_helpers/trait.FromLength.html\" title=\"trait lay_rs::prelude::taffy::style_helpers::FromLength\">FromLength</a> for <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;<a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MinTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MinTrackSizingFunction\">MinTrackSizingFunction</a>, <a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MaxTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MaxTrackSizingFunction\">MaxTrackSizingFunction</a>&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.from_length\" class=\"method trait-impl\"><a href=\"#method.from_length\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"lay_rs/prelude/taffy/style_helpers/trait.FromLength.html#tymethod.from_length\" class=\"fn\">from_length</a>&lt;Input&gt;(\n    value: Input,\n) -&gt; <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;<a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MinTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MinTrackSizingFunction\">MinTrackSizingFunction</a>, <a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MaxTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MaxTrackSizingFunction\">MaxTrackSizingFunction</a>&gt;<div class=\"where\">where\n    Input: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/convert/trait.Into.html\" title=\"trait core::convert::Into\">Into</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.82.0/std/primitive.f32.html\">f32</a>&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a>,</div></h4></section></summary><div class='docblock'>Converts into an <code>Into&lt;f32&gt;</code> into Self</div></details></div></details>","FromLength","lay_rs::prelude::taffy::style::NonRepeatedTrackSizingFunction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-FromPercent-for-MinMax%3CMinTrackSizingFunction,+MaxTrackSizingFunction%3E\" class=\"impl\"><a href=\"#impl-FromPercent-for-MinMax%3CMinTrackSizingFunction,+MaxTrackSizingFunction%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"lay_rs/prelude/taffy/style_helpers/trait.FromPercent.html\" title=\"trait lay_rs::prelude::taffy::style_helpers::FromPercent\">FromPercent</a> for <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;<a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MinTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MinTrackSizingFunction\">MinTrackSizingFunction</a>, <a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MaxTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MaxTrackSizingFunction\">MaxTrackSizingFunction</a>&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.from_percent\" class=\"method trait-impl\"><a href=\"#method.from_percent\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"lay_rs/prelude/taffy/style_helpers/trait.FromPercent.html#tymethod.from_percent\" class=\"fn\">from_percent</a>&lt;Input&gt;(\n    percent: Input,\n) -&gt; <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;<a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MinTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MinTrackSizingFunction\">MinTrackSizingFunction</a>, <a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MaxTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MaxTrackSizingFunction\">MaxTrackSizingFunction</a>&gt;<div class=\"where\">where\n    Input: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/convert/trait.Into.html\" title=\"trait core::convert::Into\">Into</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.82.0/std/primitive.f32.html\">f32</a>&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a>,</div></h4></section></summary><div class='docblock'>Converts into an <code>Into&lt;f32&gt;</code> into Self</div></details></div></details>","FromPercent","lay_rs::prelude::taffy::style::NonRepeatedTrackSizingFunction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-MinMax%3CMinTrackSizingFunction,+MaxTrackSizingFunction%3E\" class=\"impl\"><a href=\"#impl-MinMax%3CMinTrackSizingFunction,+MaxTrackSizingFunction%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;<a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MinTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MinTrackSizingFunction\">MinTrackSizingFunction</a>, <a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MaxTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MaxTrackSizingFunction\">MaxTrackSizingFunction</a>&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.min_sizing_function\" class=\"method\"><h4 class=\"code-header\">pub fn <a href=\"lay_rs/prelude/taffy/struct.MinMax.html#tymethod.min_sizing_function\" class=\"fn\">min_sizing_function</a>(&amp;self) -&gt; <a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MinTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MinTrackSizingFunction\">MinTrackSizingFunction</a></h4></section></summary><div class=\"docblock\"><p>Extract the min track sizing function</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.max_sizing_function\" class=\"method\"><h4 class=\"code-header\">pub fn <a href=\"lay_rs/prelude/taffy/struct.MinMax.html#tymethod.max_sizing_function\" class=\"fn\">max_sizing_function</a>(&amp;self) -&gt; <a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MaxTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MaxTrackSizingFunction\">MaxTrackSizingFunction</a></h4></section></summary><div class=\"docblock\"><p>Extract the max track sizing function</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.has_fixed_component\" class=\"method\"><h4 class=\"code-header\">pub fn <a href=\"lay_rs/prelude/taffy/struct.MinMax.html#tymethod.has_fixed_component\" class=\"fn\">has_fixed_component</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.82.0/std/primitive.bool.html\">bool</a></h4></section></summary><div class=\"docblock\"><p>Determine whether at least one of the components (“min” and “max”) are fixed sizing function</p>\n</div></details></div></details>",0,"lay_rs::prelude::taffy::style::NonRepeatedTrackSizingFunction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-PartialEq-for-MinMax%3CMin,+Max%3E\" class=\"impl\"><a href=\"#impl-PartialEq-for-MinMax%3CMin,+Max%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;Min, Max&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> for <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;Min, Max&gt;<div class=\"where\">where\n    Min: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>,\n    Max: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.eq\" class=\"method trait-impl\"><a href=\"#method.eq\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.82.0/core/cmp/trait.PartialEq.html#tymethod.eq\" class=\"fn\">eq</a>(&amp;self, other: &amp;<a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;Min, Max&gt;) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.82.0/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>Tests for <code>self</code> and <code>other</code> values to be equal, and is used by <code>==</code>.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.ne\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/1.82.0/src/core/cmp.rs.html#261\">source</a></span><a href=\"#method.ne\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.82.0/core/cmp/trait.PartialEq.html#method.ne\" class=\"fn\">ne</a>(&amp;self, other: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.82.0/std/primitive.reference.html\">&amp;Rhs</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.82.0/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>Tests for <code>!=</code>. The default implementation is almost always sufficient,\nand should not be overridden without very good reason.</div></details></div></details>","PartialEq","lay_rs::prelude::taffy::style::NonRepeatedTrackSizingFunction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Serialize-for-MinMax%3CMin,+Max%3E\" class=\"impl\"><a href=\"#impl-Serialize-for-MinMax%3CMin,+Max%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;Min, Max&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.210/serde/ser/trait.Serialize.html\" title=\"trait serde::ser::Serialize\">Serialize</a> for <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;Min, Max&gt;<div class=\"where\">where\n    Min: <a class=\"trait\" href=\"https://docs.rs/serde/1.0.210/serde/ser/trait.Serialize.html\" title=\"trait serde::ser::Serialize\">Serialize</a>,\n    Max: <a class=\"trait\" href=\"https://docs.rs/serde/1.0.210/serde/ser/trait.Serialize.html\" title=\"trait serde::ser::Serialize\">Serialize</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.serialize\" class=\"method trait-impl\"><a href=\"#method.serialize\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://docs.rs/serde/1.0.210/serde/ser/trait.Serialize.html#tymethod.serialize\" class=\"fn\">serialize</a>&lt;__S&gt;(\n    &amp;self,\n    __serializer: __S,\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.82.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;&lt;__S as <a class=\"trait\" href=\"https://docs.rs/serde/1.0.210/serde/ser/trait.Serializer.html\" title=\"trait serde::ser::Serializer\">Serializer</a>&gt;::<a class=\"associatedtype\" href=\"https://docs.rs/serde/1.0.210/serde/ser/trait.Serializer.html#associatedtype.Ok\" title=\"type serde::ser::Serializer::Ok\">Ok</a>, &lt;__S as <a class=\"trait\" href=\"https://docs.rs/serde/1.0.210/serde/ser/trait.Serializer.html\" title=\"trait serde::ser::Serializer\">Serializer</a>&gt;::<a class=\"associatedtype\" href=\"https://docs.rs/serde/1.0.210/serde/ser/trait.Serializer.html#associatedtype.Error\" title=\"type serde::ser::Serializer::Error\">Error</a>&gt;<div class=\"where\">where\n    __S: <a class=\"trait\" href=\"https://docs.rs/serde/1.0.210/serde/ser/trait.Serializer.html\" title=\"trait serde::ser::Serializer\">Serializer</a>,</div></h4></section></summary><div class='docblock'>Serialize this value into the given Serde serializer. <a href=\"https://docs.rs/serde/1.0.210/serde/ser/trait.Serialize.html#tymethod.serialize\">Read more</a></div></details></div></details>","Serialize","lay_rs::prelude::taffy::style::NonRepeatedTrackSizingFunction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-TaffyAuto-for-MinMax%3CMinTrackSizingFunction,+MaxTrackSizingFunction%3E\" class=\"impl\"><a href=\"#impl-TaffyAuto-for-MinMax%3CMinTrackSizingFunction,+MaxTrackSizingFunction%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"lay_rs/prelude/taffy/style_helpers/trait.TaffyAuto.html\" title=\"trait lay_rs::prelude::taffy::style_helpers::TaffyAuto\">TaffyAuto</a> for <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;<a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MinTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MinTrackSizingFunction\">MinTrackSizingFunction</a>, <a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MaxTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MaxTrackSizingFunction\">MaxTrackSizingFunction</a>&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedconstant.AUTO\" class=\"associatedconstant trait-impl\"><a href=\"#associatedconstant.AUTO\" class=\"anchor\">§</a><h4 class=\"code-header\">const <a href=\"lay_rs/prelude/taffy/style_helpers/trait.TaffyAuto.html#associatedconstant.AUTO\" class=\"constant\">AUTO</a>: <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;<a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MinTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MinTrackSizingFunction\">MinTrackSizingFunction</a>, <a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MaxTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MaxTrackSizingFunction\">MaxTrackSizingFunction</a>&gt; = _</h4></section></summary><div class='docblock'>The auto value for type implementing TaffyAuto</div></details></div></details>","TaffyAuto","lay_rs::prelude::taffy::style::NonRepeatedTrackSizingFunction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-TaffyFitContent-for-MinMax%3CMinTrackSizingFunction,+MaxTrackSizingFunction%3E\" class=\"impl\"><a href=\"#impl-TaffyFitContent-for-MinMax%3CMinTrackSizingFunction,+MaxTrackSizingFunction%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"lay_rs/prelude/taffy/style_helpers/trait.TaffyFitContent.html\" title=\"trait lay_rs::prelude::taffy::style_helpers::TaffyFitContent\">TaffyFitContent</a> for <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;<a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MinTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MinTrackSizingFunction\">MinTrackSizingFunction</a>, <a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MaxTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MaxTrackSizingFunction\">MaxTrackSizingFunction</a>&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fit_content\" class=\"method trait-impl\"><a href=\"#method.fit_content\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"lay_rs/prelude/taffy/style_helpers/trait.TaffyFitContent.html#tymethod.fit_content\" class=\"fn\">fit_content</a>(\n    argument: <a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.LengthPercentage.html\" title=\"enum lay_rs::prelude::taffy::style::LengthPercentage\">LengthPercentage</a>,\n) -&gt; <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;<a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MinTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MinTrackSizingFunction\">MinTrackSizingFunction</a>, <a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MaxTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MaxTrackSizingFunction\">MaxTrackSizingFunction</a>&gt;</h4></section></summary><div class='docblock'>Converts a LengthPercentage into Self</div></details></div></details>","TaffyFitContent","lay_rs::prelude::taffy::style::NonRepeatedTrackSizingFunction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-TaffyMaxContent-for-MinMax%3CMinTrackSizingFunction,+MaxTrackSizingFunction%3E\" class=\"impl\"><a href=\"#impl-TaffyMaxContent-for-MinMax%3CMinTrackSizingFunction,+MaxTrackSizingFunction%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"lay_rs/prelude/taffy/style_helpers/trait.TaffyMaxContent.html\" title=\"trait lay_rs::prelude::taffy::style_helpers::TaffyMaxContent\">TaffyMaxContent</a> for <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;<a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MinTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MinTrackSizingFunction\">MinTrackSizingFunction</a>, <a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MaxTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MaxTrackSizingFunction\">MaxTrackSizingFunction</a>&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedconstant.MAX_CONTENT\" class=\"associatedconstant trait-impl\"><a href=\"#associatedconstant.MAX_CONTENT\" class=\"anchor\">§</a><h4 class=\"code-header\">const <a href=\"lay_rs/prelude/taffy/style_helpers/trait.TaffyMaxContent.html#associatedconstant.MAX_CONTENT\" class=\"constant\">MAX_CONTENT</a>: <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;<a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MinTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MinTrackSizingFunction\">MinTrackSizingFunction</a>, <a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MaxTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MaxTrackSizingFunction\">MaxTrackSizingFunction</a>&gt; = _</h4></section></summary><div class='docblock'>The max_content value for type implementing TaffyZero</div></details></div></details>","TaffyMaxContent","lay_rs::prelude::taffy::style::NonRepeatedTrackSizingFunction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-TaffyMinContent-for-MinMax%3CMinTrackSizingFunction,+MaxTrackSizingFunction%3E\" class=\"impl\"><a href=\"#impl-TaffyMinContent-for-MinMax%3CMinTrackSizingFunction,+MaxTrackSizingFunction%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"lay_rs/prelude/taffy/style_helpers/trait.TaffyMinContent.html\" title=\"trait lay_rs::prelude::taffy::style_helpers::TaffyMinContent\">TaffyMinContent</a> for <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;<a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MinTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MinTrackSizingFunction\">MinTrackSizingFunction</a>, <a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MaxTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MaxTrackSizingFunction\">MaxTrackSizingFunction</a>&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedconstant.MIN_CONTENT\" class=\"associatedconstant trait-impl\"><a href=\"#associatedconstant.MIN_CONTENT\" class=\"anchor\">§</a><h4 class=\"code-header\">const <a href=\"lay_rs/prelude/taffy/style_helpers/trait.TaffyMinContent.html#associatedconstant.MIN_CONTENT\" class=\"constant\">MIN_CONTENT</a>: <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;<a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MinTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MinTrackSizingFunction\">MinTrackSizingFunction</a>, <a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MaxTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MaxTrackSizingFunction\">MaxTrackSizingFunction</a>&gt; = _</h4></section></summary><div class='docblock'>The min_content value for type implementing TaffyZero</div></details></div></details>","TaffyMinContent","lay_rs::prelude::taffy::style::NonRepeatedTrackSizingFunction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-TaffyZero-for-MinMax%3CMinTrackSizingFunction,+MaxTrackSizingFunction%3E\" class=\"impl\"><a href=\"#impl-TaffyZero-for-MinMax%3CMinTrackSizingFunction,+MaxTrackSizingFunction%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"lay_rs/prelude/taffy/style_helpers/trait.TaffyZero.html\" title=\"trait lay_rs::prelude::taffy::style_helpers::TaffyZero\">TaffyZero</a> for <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;<a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MinTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MinTrackSizingFunction\">MinTrackSizingFunction</a>, <a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MaxTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MaxTrackSizingFunction\">MaxTrackSizingFunction</a>&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedconstant.ZERO\" class=\"associatedconstant trait-impl\"><a href=\"#associatedconstant.ZERO\" class=\"anchor\">§</a><h4 class=\"code-header\">const <a href=\"lay_rs/prelude/taffy/style_helpers/trait.TaffyZero.html#associatedconstant.ZERO\" class=\"constant\">ZERO</a>: <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;<a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MinTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MinTrackSizingFunction\">MinTrackSizingFunction</a>, <a class=\"enum\" href=\"lay_rs/prelude/taffy/style/enum.MaxTrackSizingFunction.html\" title=\"enum lay_rs::prelude::taffy::style::MaxTrackSizingFunction\">MaxTrackSizingFunction</a>&gt; = _</h4></section></summary><div class='docblock'>The zero value for type implementing TaffyZero</div></details></div></details>","TaffyZero","lay_rs::prelude::taffy::style::NonRepeatedTrackSizingFunction"],["<section id=\"impl-Copy-for-MinMax%3CMin,+Max%3E\" class=\"impl\"><a href=\"#impl-Copy-for-MinMax%3CMin,+Max%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;Min, Max&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;Min, Max&gt;<div class=\"where\">where\n    Min: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a>,\n    Max: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a>,</div></h3></section>","Copy","lay_rs::prelude::taffy::style::NonRepeatedTrackSizingFunction"],["<section id=\"impl-Eq-for-MinMax%3CMin,+Max%3E\" class=\"impl\"><a href=\"#impl-Eq-for-MinMax%3CMin,+Max%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;Min, Max&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> for <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;Min, Max&gt;<div class=\"where\">where\n    Min: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a>,\n    Max: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a>,</div></h3></section>","Eq","lay_rs::prelude::taffy::style::NonRepeatedTrackSizingFunction"],["<section id=\"impl-StructuralPartialEq-for-MinMax%3CMin,+Max%3E\" class=\"impl\"><a href=\"#impl-StructuralPartialEq-for-MinMax%3CMin,+Max%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;Min, Max&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/marker/trait.StructuralPartialEq.html\" title=\"trait core::marker::StructuralPartialEq\">StructuralPartialEq</a> for <a class=\"struct\" href=\"lay_rs/prelude/taffy/struct.MinMax.html\" title=\"struct lay_rs::prelude::taffy::MinMax\">MinMax</a>&lt;Min, Max&gt;</h3></section>","StructuralPartialEq","lay_rs::prelude::taffy::style::NonRepeatedTrackSizingFunction"]]]]);
    if (window.register_type_impls) {
        window.register_type_impls(type_impls);
    } else {
        window.pending_type_impls = type_impls;
    }
})()
//{"start":55,"fragment_lengths":[35633]}