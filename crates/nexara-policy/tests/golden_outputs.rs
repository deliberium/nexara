use nexara_policy::{PolicyCompileOptions, PolicyCompiler};

#[test]
fn orders_never_refunds_matches_golden_contract() {
    let intent = include_str!("fixtures/orders_never_refunds.intent").trim();
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/orders_never_refunds.contract.json")).unwrap();
    let compiled = PolicyCompiler::default()
        .compile(intent, PolicyCompileOptions::default())
        .unwrap();
    let actual = serde_json::to_value(&compiled.policy).unwrap();

    assert_eq!(actual, expected);
}
