use syntect::{dumps::dump_to_uncompressed_file, parsing::SyntaxSet};

fn main() {
    let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
    builder.add_from_folder(".", true).unwrap();
    let ss = builder.build();
    dump_to_uncompressed_file(&ss, "newlines.packdump").unwrap();
}
