import sys, pathlib
n = int(sys.argv[1])
out = ["//! GENERATED for the Q3 measurement — not part of the pilot.",
       "#![allow(dead_code)]",
       "use topcoat::{Result, view::{component, view}};", ""]
for i in range(n):
    out.append(f'''#[component]
pub async fn gen_{i}(label: &str) -> Result {{
    view! {{
        <section class="gen">
            <h2>(label)</h2>
            <ul>
                for n in 0..3 {{
                    <li class="row">(format!("item {{n}}"))</li>
                }}
            </ul>
            <p>"generated component {i}"</p>
        </section>
    }}
}}''')
pathlib.Path("src/generated.rs").write_text("\n".join(out) + "\n")
