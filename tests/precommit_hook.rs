#[test]
fn precommit_contains_npm_test() {
    let script = std::fs::read_to_string("pre-commit").expect("read pre-commit");
    assert!(script.contains("npm test"), "pre-commit hook should run npm tests");
}
