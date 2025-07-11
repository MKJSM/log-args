#[test]
fn trybuild() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-all-args.rs");
    t.pass("tests/02-selected-fields.rs");
    t.pass("tests/03-subfield-logging.rs");
    t.pass("tests/04-custom-fields.rs");
    t.pass("tests/05-async-fn.rs");
    t.pass("tests/07-all-log-levels.rs");
    t.compile_fail("tests/06-invalid-input.rs");
}
