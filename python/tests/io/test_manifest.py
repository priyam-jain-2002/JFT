import copy
import json
import unittest
from dataclasses import FrozenInstanceError, replace
from pathlib import Path
from typing import Any, Dict, Mapping

from jft.io.manifest import (
    ManifestJsonError,
    ManifestValidationError,
    RecordKind,
    TriggerVerificationStatus,
    encode_manifest_bytes,
    load_manifest_bytes,
    load_schema_bytes,
)


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
SCHEMA_PATH = REPOSITORY_ROOT / "schemas/nse_cm/manifest.schema.json"
GOLDEN_PATH = (
    REPOSITORY_ROOT
    / "tests/fixtures/nse_cm/synthetic_nonlicensed/manifest_v1.json"
)


class ManifestConformanceTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.schema = load_schema_bytes(SCHEMA_PATH.read_bytes())
        cls.golden_bytes = GOLDEN_PATH.read_bytes()
        cls.golden = json.loads(cls.golden_bytes)

    def encoded(self, document: Mapping[str, Any]) -> bytes:
        return json.dumps(document, separators=(",", ":")).encode("utf-8") + b"\n"

    def mutated(self) -> Dict[str, Any]:
        return copy.deepcopy(self.golden)

    def test_rust_golden_validates_and_round_trips_byte_identically(self) -> None:
        manifest = load_manifest_bytes(self.golden_bytes, self.schema)

        self.assertEqual(manifest.schema_id, "jft.nse_cm.manifest")
        self.assertEqual(manifest.schema_major, 1)
        self.assertEqual(manifest.source.record_kind, RecordKind.ORDERS)
        self.assertEqual(manifest.trigger.status, TriggerVerificationStatus.VERIFIED)
        self.assertEqual(
            encode_manifest_bytes(manifest, self.schema), self.golden_bytes
        )

    def test_model_is_immutable_and_identity_is_payload_based_and_bounded(self) -> None:
        manifest = load_manifest_bytes(self.golden_bytes, self.schema)

        with self.assertRaises(FrozenInstanceError):
            manifest.schema_major = 2  # type: ignore[misc]
        identity = manifest.record_identity(0)
        self.assertIsNotNone(identity)
        assert identity is not None
        self.assertEqual(identity.decompressed_payload_sha256, manifest.payload.sha256)
        self.assertEqual(identity.record_kind, RecordKind.ORDERS)
        self.assertEqual(manifest.record_identity(-1), None)
        self.assertEqual(manifest.record_identity(1), None)

    def test_strict_json_rejects_duplicate_keys_non_object_and_bad_utf8(self) -> None:
        duplicate = b'{"schema_id":"first","schema_id":"second"}'
        for raw in [duplicate, b"[]", b"\xff"]:
            with self.subTest(raw=raw):
                with self.assertRaises(ManifestJsonError):
                    load_manifest_bytes(raw, self.schema)

    def test_schema_rejects_unknown_fields_versions_paths_and_noncanonical_digest(self) -> None:
        cases = []
        unknown = self.mutated()
        unknown["unknown"] = True
        cases.append(unknown)
        version = self.mutated()
        version["schema_major"] = 2
        cases.append(version)
        path = self.mutated()
        path["source"]["logical_path"] = "nested/../escape.dat"
        cases.append(path)
        digest = self.mutated()
        digest["compressed"]["sha256"] = digest["compressed"]["sha256"].upper()
        cases.append(digest)

        for document in cases:
            with self.subTest(document=document):
                with self.assertRaises(ManifestValidationError):
                    load_manifest_bytes(self.encoded(document), self.schema)

    def test_schema_enforces_rust_integer_ranges(self) -> None:
        for field, value in [
            ("byte_count", 1 << 64),
            ("payload_width", 1 << 32),
        ]:
            document = self.mutated()
            document["payload"][field] = value
            with self.subTest(field=field):
                with self.assertRaisesRegex(
                    ManifestValidationError, "schema violation"
                ):
                    load_manifest_bytes(self.encoded(document), self.schema)

    def test_real_date_validation_is_stricter_than_the_schema_pattern(self) -> None:
        document = self.mutated()
        document["source"]["source_date"] = "2025-02-29"

        with self.assertRaisesRegex(ManifestValidationError, "not a real date"):
            load_manifest_bytes(self.encoded(document), self.schema)

    def test_trigger_declaration_and_status_must_match_compressed_evidence(self) -> None:
        partial = self.mutated()
        partial["trigger"]["declared_md5"] = None
        forged = self.mutated()
        forged["trigger"]["status"] = "md5_mismatch"

        for document in [partial, forged]:
            with self.subTest(document=document):
                with self.assertRaises(ManifestValidationError):
                    load_manifest_bytes(self.encoded(document), self.schema)

    def test_every_trigger_status_is_derived_identically_to_rust(self) -> None:
        correct_md5 = self.golden["compressed"]["md5"]
        wrong_md5 = "d41d8cd98f00b204e9800998ecf8427e"
        cases = [
            ("not_provided", None, None),
            ("verified", 16, correct_md5),
            ("size_mismatch", 17, correct_md5),
            ("md5_mismatch", 16, wrong_md5),
            ("size_and_md5_mismatch", 17, wrong_md5),
        ]

        for status, byte_count, md5 in cases:
            document = self.mutated()
            document["trigger"] = {
                "status": status,
                "declared_byte_count": byte_count,
                "declared_md5": md5,
            }
            with self.subTest(status=status):
                manifest = load_manifest_bytes(self.encoded(document), self.schema)
                self.assertEqual(manifest.trigger.status.value, status)

    def test_payload_counts_bytes_and_jiffy_bounds_fail_closed(self) -> None:
        count = self.mutated()
        count["payload"]["quarantined_records"] = 1
        byte_count = self.mutated()
        byte_count["payload"]["byte_count"] += 1
        missing_bound = self.mutated()
        missing_bound["payload"]["raw_jiffy_maximum"] = None
        reversed_bounds = self.mutated()
        reversed_bounds["payload"]["raw_jiffy_minimum"] = 2
        reversed_bounds["payload"]["raw_jiffy_maximum"] = 1

        for document in [count, byte_count, missing_bound, reversed_bounds]:
            with self.subTest(document=document):
                with self.assertRaises(ManifestValidationError):
                    load_manifest_bytes(self.encoded(document), self.schema)

        manifest = load_manifest_bytes(self.golden_bytes, self.schema)
        invalid_payload = replace(
            manifest.payload, byte_count=manifest.payload.byte_count + 1
        )
        with self.assertRaises(ManifestValidationError):
            encode_manifest_bytes(
                replace(manifest, payload=invalid_payload), self.schema
            )

    def test_container_rename_and_recompression_do_not_change_record_identity(self) -> None:
        first = load_manifest_bytes(self.golden_bytes, self.schema)
        document = self.mutated()
        document["source"]["logical_path"] = "renamed/container.dat.gz"
        document["compressed"] = {
            "byte_count": 0,
            "md5": "d41d8cd98f00b204e9800998ecf8427e",
            "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        }
        document["trigger"] = {
            "status": "not_provided",
            "declared_byte_count": None,
            "declared_md5": None,
        }
        second = load_manifest_bytes(self.encoded(document), self.schema)

        self.assertNotEqual(first.compressed, second.compressed)
        self.assertNotEqual(first.source.logical_path, second.source.logical_path)
        self.assertEqual(first.record_identity(0), second.record_identity(0))


if __name__ == "__main__":
    unittest.main()
