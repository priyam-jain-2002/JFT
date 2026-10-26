use std::cell::Cell;

use jft_nse_cm::layouts::SourceDate;
use jft_nse_cm::manifest::v1::{
    CompressedIntegrity, ProcessingIdentity, TRIM_ORDER_LAYOUT_ID, TriggerDeclaration,
};
use jft_nse_cm::manifest::{
    CmFilePrefix, CmLogicalGroupKey, CmPairedStreamSet, CmProduct, CmRecordKind, CmSourceIdentity,
    CmSourceIdentityError,
};
use jft_nse_cm::reconcile::{
    CmStreamSource, CmStreamSourceSetError, IdentifiedCmStreamSource, ManifestCompletionEvidence,
    TradeCandidateStreamSetError, TrimManifestError, TrimOrderTradeEvidenceError,
    profile_identified_trim_order_trade_evidence,
    profile_identified_trim_order_trade_evidence_with_trade_observer,
    profile_trim_order_trade_evidence,
};

const ORDER_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_entry.dat");
const TRADE_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_trade.dat");

fn delivery(record_kind: CmRecordKind) -> CmPairedStreamSet {
    CmPairedStreamSet {
        prefix: CmFilePrefix::Cm,
        key: CmLogicalGroupKey {
            product: CmProduct::Trim,
            record_kind,
            source_date: SourceDate::new(2026, 5, 18).expect("valid date"),
        },
        members: vec![None],
    }
}

fn identified_sources<'a>(
    order: &'a [u8],
    trade: &'a [u8],
) -> (IdentifiedCmStreamSource<'a>, IdentifiedCmStreamSource<'a>) {
    (
        IdentifiedCmStreamSource::new(
            None,
            order,
            "delivery/CM_Orders_T_18052026.DAT.gz",
            CmRecordKind::Orders,
        )
        .expect("valid order identity"),
        IdentifiedCmStreamSource::new(
            None,
            trade,
            "delivery/CM_Trades_T_18052026.DAT.gz",
            CmRecordKind::Trades,
        )
        .expect("valid trade identity"),
    )
}

#[test]
fn source_identity_is_derived_from_exact_bytes_and_is_path_safe() {
    assert_eq!(
        CmSourceIdentity::from_decompressed_bytes(
            "../CM_Orders_T_18052026.DAT.gz",
            CmRecordKind::Orders,
            ORDER_FIXTURE,
        ),
        Err(CmSourceIdentityError::InvalidLogicalPath)
    );

    let original = CmSourceIdentity::from_decompressed_bytes(
        "first/CM_Orders_T_18052026.DAT.gz",
        CmRecordKind::Orders,
        ORDER_FIXTURE,
    )
    .expect("valid identity");
    let renamed = CmSourceIdentity::from_decompressed_bytes(
        "renamed/CM_Orders_T_18052026.DAT.gz",
        CmRecordKind::Orders,
        ORDER_FIXTURE,
    )
    .expect("valid renamed identity");
    let mut mutated_bytes = ORDER_FIXTURE.to_vec();
    mutated_bytes[0] ^= 1;
    let mutated = CmSourceIdentity::from_decompressed_bytes(
        "first/CM_Orders_T_18052026.DAT.gz",
        CmRecordKind::Orders,
        &mutated_bytes,
    )
    .expect("valid mutated identity");

    assert_ne!(original.logical_path(), renamed.logical_path());
    assert_eq!(
        original.decompressed_payload_sha256(),
        renamed.decompressed_payload_sha256()
    );
    assert_eq!(original.record_identity(0), renamed.record_identity(0));
    assert_ne!(
        original.decompressed_payload_sha256(),
        mutated.decompressed_payload_sha256()
    );
    assert_ne!(original.record_identity(0), mutated.record_identity(0));
}

#[test]
fn rejects_identity_and_filename_mismatches_before_observation() {
    let order_delivery = delivery(CmRecordKind::Orders);
    let trade_delivery = delivery(CmRecordKind::Trades);
    let (order, _) = identified_sources(ORDER_FIXTURE, TRADE_FIXTURE);
    let wrong_kind = IdentifiedCmStreamSource::new(
        None,
        TRADE_FIXTURE,
        "delivery/CM_Trades_T_18052026.DAT.gz",
        CmRecordKind::Orders,
    )
    .expect("structurally valid identity");
    let observations = Cell::new(0);
    let error = profile_identified_trim_order_trade_evidence_with_trade_observer(
        &order_delivery,
        &[order],
        &trade_delivery,
        &[wrong_kind],
        |_| observations.set(observations.get() + 1),
    )
    .expect_err("identity kind must match delivery");
    assert_eq!(observations.get(), 0);
    assert_eq!(
        error,
        TrimOrderTradeEvidenceError::TradeProfile(TradeCandidateStreamSetError::TradeSourceSet(
            CmStreamSourceSetError::IdentityRecordKindMismatch {
                member: None,
                expected: CmRecordKind::Trades,
                actual: CmRecordKind::Orders,
            }
        ))
    );

    let (_, valid_trade) = identified_sources(ORDER_FIXTURE, TRADE_FIXTURE);
    let wrong_member = IdentifiedCmStreamSource::new(
        Some(1),
        ORDER_FIXTURE,
        "delivery/CM_Orders_T_18052026_01.DAT.gz",
        CmRecordKind::Orders,
    )
    .expect("structurally valid member identity");
    for invalid_orders in [Vec::new(), vec![order, order], vec![wrong_member]] {
        let observations = Cell::new(0);
        let error = profile_identified_trim_order_trade_evidence_with_trade_observer(
            &order_delivery,
            &invalid_orders,
            &trade_delivery,
            &[valid_trade],
            |_| observations.set(observations.get() + 1),
        )
        .expect_err("missing, duplicate, and wrong members must fail");
        assert_eq!(observations.get(), 0);
        assert!(matches!(
            error,
            TrimOrderTradeEvidenceError::OrderReplay(
                CmStreamSourceSetError::StreamMembersMismatch { .. }
            )
        ));
    }

    let wrong_declared_member = IdentifiedCmStreamSource::new(
        None,
        ORDER_FIXTURE,
        "delivery/CM_Orders_T_18052026_01.DAT.gz",
        CmRecordKind::Orders,
    )
    .expect("structurally valid filename identity");
    let observations = Cell::new(0);
    let error = profile_identified_trim_order_trade_evidence_with_trade_observer(
        &order_delivery,
        &[wrong_declared_member],
        &trade_delivery,
        &[valid_trade],
        |_| observations.set(observations.get() + 1),
    )
    .expect_err("filename member must match source member");
    assert_eq!(observations.get(), 0);
    assert!(matches!(
        error,
        TrimOrderTradeEvidenceError::OrderReplay(
            CmStreamSourceSetError::LogicalPathMemberMismatch { .. }
        )
    ));

    let wrong_name = IdentifiedCmStreamSource::new(
        None,
        TRADE_FIXTURE,
        "delivery/CM_Orders_T_18052026.DAT.gz",
        CmRecordKind::Trades,
    )
    .expect("structurally valid identity");
    let error = profile_identified_trim_order_trade_evidence(
        &order_delivery,
        &[order],
        &trade_delivery,
        &[wrong_name],
    )
    .expect_err("filename kind must match delivery");
    assert!(matches!(
        error,
        TrimOrderTradeEvidenceError::TradeProfile(TradeCandidateStreamSetError::TradeSourceSet(
            CmStreamSourceSetError::LogicalPathRecordKindMismatch { .. }
        ))
    ));
}

#[test]
fn identified_and_fixture_paths_have_identical_replay_results() {
    let order_delivery = delivery(CmRecordKind::Orders);
    let trade_delivery = delivery(CmRecordKind::Trades);
    let (order, trade) = identified_sources(ORDER_FIXTURE, TRADE_FIXTURE);
    let identified = profile_identified_trim_order_trade_evidence(
        &order_delivery,
        &[order],
        &trade_delivery,
        &[trade],
    )
    .expect("identified evidence");
    let fixture = profile_trim_order_trade_evidence(
        &order_delivery,
        &[CmStreamSource {
            member: None,
            source: ORDER_FIXTURE,
        }],
        &trade_delivery,
        &[CmStreamSource {
            member: None,
            source: TRADE_FIXTURE,
        }],
    )
    .expect("fixture evidence");

    assert_eq!(identified.orders.delivery, fixture.orders.delivery);
    assert_eq!(
        identified.orders.streams[0].replay,
        fixture.orders.streams[0].replay
    );
    assert_eq!(identified.order_summary, fixture.order_summary);
    assert_eq!(identified.trades.delivery, fixture.trades.delivery);
    assert_eq!(
        identified.trades.streams[0].source,
        fixture.trades.streams[0].source
    );
    assert_eq!(
        identified.trades.combined_value_summary,
        fixture.trades.combined_value_summary
    );
    assert!(identified.orders.streams[0].source_identity().is_some());
    assert!(identified.trades.streams[0].source_identity().is_some());
    assert!(fixture.orders.streams[0].source_identity().is_none());
    assert!(fixture.trades.streams[0].source_identity().is_none());
}

#[test]
fn quarantines_and_manifests_are_bound_to_the_same_record_identity() {
    let mut bad_order = ORDER_FIXTURE.to_vec();
    bad_order[37] = b'X';
    let mut bad_trade = TRADE_FIXTURE.to_vec();
    bad_trade[0] = b'X';
    let order_delivery = delivery(CmRecordKind::Orders);
    let trade_delivery = delivery(CmRecordKind::Trades);
    let (order, trade) = identified_sources(&bad_order, &bad_trade);
    let evidence = profile_identified_trim_order_trade_evidence(
        &order_delivery,
        &[order],
        &trade_delivery,
        &[trade],
    )
    .expect("quarantined evidence remains valid");
    let quarantines = evidence.quarantine_evidence();

    assert_eq!(quarantines.len(), 2);
    assert_eq!(
        quarantines[0].record_identity(),
        evidence.orders.streams[0]
            .source_identity()
            .expect("order identity")
            .record_identity(0)
    );
    assert_eq!(
        quarantines[1].record_identity(),
        evidence.trades.streams[0]
            .source_identity()
            .expect("trade identity")
            .record_identity(0)
    );

    let compressed_bytes = b"immutable compressed order evidence";
    let compressed = CompressedIntegrity::from_bytes(compressed_bytes);
    let completion = ManifestCompletionEvidence {
        compressed_bytes,
        trigger_declaration: Some(TriggerDeclaration {
            byte_count: compressed.byte_count,
            md5: compressed.md5,
        }),
        processing: ProcessingIdentity {
            configuration_id: "identified-evidence-test-v1".to_owned(),
            code_identity: "test-commit".to_owned(),
        },
    };
    let stream = &evidence.orders.streams[0];
    let manifest = stream
        .assemble_manifest_v1(&order_delivery, &completion)
        .expect("derived manifest");

    assert_eq!(
        manifest.source.logical_path,
        order.identity().logical_path()
    );
    assert_eq!(manifest.source.layout_id, TRIM_ORDER_LAYOUT_ID);
    assert_eq!(manifest.compressed, compressed);
    assert_eq!(manifest.payload.byte_count, bad_order.len() as u64);
    assert_eq!(manifest.payload.framed_records, 1);
    assert_eq!(manifest.payload.parsed_records, 0);
    assert_eq!(manifest.payload.quarantined_records, 1);
    assert_eq!(
        manifest.payload.sha256,
        order.identity().decompressed_payload_sha256()
    );
    assert_eq!(
        manifest.record_identity(0),
        quarantines[0].record_identity()
    );
    assert_eq!(
        stream.verify_manifest_v1(&order_delivery, &completion, &manifest),
        Ok(())
    );

    let mut forged = manifest.clone();
    forged.processing.code_identity = "forged-code".to_owned();
    assert_eq!(
        stream.verify_manifest_v1(&order_delivery, &completion, &forged),
        Err(TrimManifestError::ExistingManifestMismatch)
    );
    let mut forged = manifest;
    forged.payload.sha256 = trade.identity().decompressed_payload_sha256();
    assert_eq!(
        stream.verify_manifest_v1(&order_delivery, &completion, &forged),
        Err(TrimManifestError::ExistingManifestMismatch)
    );

    let mut order_with_residue = ORDER_FIXTURE.to_vec();
    order_with_residue.extend_from_slice(b"tail");
    let (order_with_residue, valid_trade) = identified_sources(&order_with_residue, TRADE_FIXTURE);
    let residue_evidence = profile_identified_trim_order_trade_evidence(
        &order_delivery,
        &[order_with_residue],
        &trade_delivery,
        &[valid_trade],
    )
    .expect("framing residue is retained as evidence");
    let residue_manifest = residue_evidence.orders.streams[0]
        .assemble_manifest_v1(&order_delivery, &completion)
        .expect("manifest accounts for parsed records and residue");
    assert_eq!(residue_manifest.payload.parsed_records, 1);
    assert_eq!(residue_manifest.payload.quarantined_records, 0);
    assert_eq!(residue_manifest.payload.residue_bytes, 4);
    assert_eq!(
        residue_manifest.payload.raw_jiffy_minimum,
        Some(91_200_611_817_221)
    );
    assert_eq!(
        residue_manifest.payload.raw_jiffy_maximum,
        Some(91_200_611_817_221)
    );
}
