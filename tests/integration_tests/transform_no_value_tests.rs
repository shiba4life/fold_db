use crate::test_data::test_helpers::create_test_node;
use fold_node::datafold_node::loader::load_schema_from_file;

#[test]
fn transform_without_values_does_not_error_on_missing_refs() {
    let mut node = create_test_node();

    // Load sample schemas with a transform
    load_schema_from_file(
        "fold_node/src/datafold_node/samples/data/TransformBase.json",
        &mut node,
    )
    .unwrap();
    load_schema_from_file(
        "fold_node/src/datafold_node/samples/data/TransformSchema.json",
        &mut node,
    )
    .unwrap();

    node.allow_schema("TransformBase").unwrap();
    node.allow_schema("TransformSchema").unwrap();

    let result = node.run_transform("TransformSchema.result");
    if let Err(e) = result {
        let msg = format!("{e:?}");
        assert!(
            !msg.contains("AtomRef not found"),
            "Transform failed with unexpected error: {}",
            msg
        );
    }
}
