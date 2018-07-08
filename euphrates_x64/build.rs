extern crate cc;

fn main() {
    std::env::set_var("CC", "clang");

    cc::Build::new()
        .file("src/pattern_to_palette_indices.s")
        .compile("pattern_to_palette_indices");
}
