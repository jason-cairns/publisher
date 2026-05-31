use publisher::*;

#[test]
fn inspect_api_sketch_snapshot_is_stable() {
    let publication = parse_publication("examples/api_sketch_site/index.typ").unwrap();
    let report = publication.validate();
    let options = InspectOptions::new().source_root("examples/api_sketch_site");

    assert!(
        report.errors.is_empty(),
        "expected valid fixture, got {:#?}",
        report.errors
    );

    insta::assert_snapshot!(inspect_publication(&publication, &report, &options));
}

#[test]
fn inspect_prd003_snapshot_is_stable() {
    let publication = parse_publication("examples/prd003_discovery_site/index.typ").unwrap();
    let report = publication.validate();
    let options = InspectOptions::new().source_root("examples/prd003_discovery_site");

    assert!(
        report.errors.is_empty(),
        "expected valid fixture, got {:#?}",
        report.errors
    );

    insta::assert_snapshot!(inspect_publication(&publication, &report, &options));
}
