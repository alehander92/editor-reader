use std::path::PathBuf;

fn build_parsers() {
   let dir: PathBuf = ["parsers/tree-sitter-java", "src"].iter().collect();
   cc::Build::new()
        .include(&dir)
        .file(dir.join("parser.c"))
        // .file(dir.join("scanner.c"))
        .compile("tree-sitter-java");
}

fn main() {
	build_parsers();
}