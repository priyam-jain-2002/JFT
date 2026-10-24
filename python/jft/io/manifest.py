"""Fail-closed Python consumer for the Rust NSE CM manifest-v1 contract."""

import json
from datetime import date
from pathlib import Path
from typing import Any, Dict, Mapping, Optional, Sequence, Tuple, cast

from jsonschema import Draft202012Validator
from jsonschema.exceptions import SchemaError, ValidationError

from ._manifest_types import (
    CompressedIntegrity,
    FilePrefix,
    ManifestSource,
    ManifestV1 as ManifestV1,
    PayloadIntegrity,
    ProcessingIdentity,
    Product,
    RecordIdentity as RecordIdentity,
    RecordKind as RecordKind,
    TriggerEvidence,
    TriggerVerificationStatus as TriggerVerificationStatus,
    manifest_to_mapping,
)


class ManifestError(ValueError):
    """Base class for manifest boundary failures."""


class ManifestJsonError(ManifestError):
    """The input is not strict UTF-8 JSON with unique object keys."""


class ManifestSchemaError(ManifestError):
    """The supplied public schema is not a valid Draft 2020-12 schema."""


class ManifestValidationError(ManifestError):
    """The document violates the public schema or cross-field invariants."""


def load_schema_bytes(raw: bytes) -> Mapping[str, Any]:
    schema = _decode_json_object(raw, "manifest schema")
    try:
        Draft202012Validator.check_schema(schema)
    except SchemaError as error:
        raise ManifestSchemaError(f"invalid manifest schema: {error.message}") from error
    return schema


def load_manifest_bytes(raw: bytes, schema: Mapping[str, Any]) -> ManifestV1:
    document = _decode_json_object(raw, "manifest")
    _validate_against_schema(document, schema)
    manifest = _manifest_from_mapping(document)
    _validate_cross_field_invariants(manifest)
    return manifest


def encode_manifest_bytes(
    manifest: ManifestV1, schema: Mapping[str, Any]
) -> bytes:
    """Validate and reproduce the deterministic Rust compact-JSON representation."""
    document = manifest_to_mapping(manifest)
    _validate_against_schema(document, schema)
    _validate_cross_field_invariants(manifest)
    encoded = json.dumps(document, ensure_ascii=False, separators=(",", ":"))
    return encoded.encode("utf-8") + b"\n"


def load_manifest_file(manifest_path: Path, schema_path: Path) -> ManifestV1:
    """Read one manifest and its explicit schema from caller-selected paths."""
    try:
        schema_raw = schema_path.read_bytes()
        manifest_raw = manifest_path.read_bytes()
    except OSError as error:
        raise ManifestError(f"unable to read manifest boundary file: {error}") from error
    return load_manifest_bytes(manifest_raw, load_schema_bytes(schema_raw))


def _decode_json_object(raw: bytes, label: str) -> Dict[str, Any]:
    def unique_object(pairs: Sequence[Tuple[str, Any]]) -> Dict[str, Any]:
        result: Dict[str, Any] = {}
        for key, value in pairs:
            if key in result:
                raise ManifestJsonError(f"{label} contains duplicate key: {key}")
            result[key] = value
        return result

    try:
        text = raw.decode("utf-8")
        value = json.loads(text, object_pairs_hook=unique_object)
    except UnicodeDecodeError as error:
        raise ManifestJsonError(f"{label} is not valid UTF-8") from error
    except json.JSONDecodeError as error:
        raise ManifestJsonError(f"{label} is not valid JSON: {error.msg}") from error

    if not isinstance(value, dict):
        raise ManifestJsonError(f"{label} root must be a JSON object")
    return cast(Dict[str, Any], value)


def _validate_against_schema(
    document: Mapping[str, Any], schema: Mapping[str, Any]
) -> None:
    try:
        Draft202012Validator(schema).validate(document)
    except ValidationError as error:
        location = "/".join(str(component) for component in error.absolute_path)
        where = location if location else "<root>"
        raise ManifestValidationError(
            f"manifest schema violation at {where}: {error.message}"
        ) from error


def _manifest_from_mapping(document: Mapping[str, Any]) -> ManifestV1:
    source = cast(Mapping[str, Any], document["source"])
    compressed = cast(Mapping[str, Any], document["compressed"])
    trigger = cast(Mapping[str, Any], document["trigger"])
    payload = cast(Mapping[str, Any], document["payload"])
    processing = cast(Mapping[str, Any], document["processing"])

    try:
        source_date = date.fromisoformat(cast(str, source["source_date"]))
    except ValueError as error:
        raise ManifestValidationError("manifest source_date is not a real date") from error

    return ManifestV1(
        schema_id=cast(str, document["schema_id"]),
        schema_major=cast(int, document["schema_major"]),
        source=ManifestSource(
            logical_path=cast(str, source["logical_path"]),
            prefix=FilePrefix(cast(str, source["prefix"])),
            product=Product(cast(str, source["product"])),
            record_kind=RecordKind(cast(str, source["record_kind"])),
            source_date=source_date,
            stream=cast(Optional[int], source["stream"]),
            layout_id=cast(str, source["layout_id"]),
        ),
        compressed=CompressedIntegrity(
            byte_count=cast(int, compressed["byte_count"]),
            md5=cast(str, compressed["md5"]),
            sha256=cast(str, compressed["sha256"]),
        ),
        trigger=TriggerEvidence(
            status=TriggerVerificationStatus(cast(str, trigger["status"])),
            declared_byte_count=cast(Optional[int], trigger["declared_byte_count"]),
            declared_md5=cast(Optional[str], trigger["declared_md5"]),
        ),
        payload=PayloadIntegrity(
            byte_count=cast(int, payload["byte_count"]),
            sha256=cast(str, payload["sha256"]),
            payload_width=cast(int, payload["payload_width"]),
            delimiter=cast(str, payload["delimiter"]),
            framed_records=cast(int, payload["framed_records"]),
            parsed_records=cast(int, payload["parsed_records"]),
            quarantined_records=cast(int, payload["quarantined_records"]),
            residue_bytes=cast(int, payload["residue_bytes"]),
            raw_jiffy_minimum=cast(Optional[int], payload["raw_jiffy_minimum"]),
            raw_jiffy_maximum=cast(Optional[int], payload["raw_jiffy_maximum"]),
        ),
        processing=ProcessingIdentity(
            configuration_id=cast(str, processing["configuration_id"]),
            code_identity=cast(str, processing["code_identity"]),
        ),
    )


def _validate_cross_field_invariants(manifest: ManifestV1) -> None:
    expected_trigger = _expected_trigger_status(manifest.compressed, manifest.trigger)
    if manifest.trigger.status is not expected_trigger:
        raise ManifestValidationError(
            "trigger status does not match declared compressed size and MD5"
        )

    payload = manifest.payload
    if payload.parsed_records + payload.quarantined_records != payload.framed_records:
        raise ManifestValidationError(
            "parsed_records + quarantined_records must equal framed_records"
        )
    expected_bytes = payload.framed_records * (payload.payload_width + 1)
    expected_bytes += payload.residue_bytes
    if expected_bytes != payload.byte_count:
        raise ManifestValidationError(
            "payload byte_count must equal framed records including LF plus residue"
        )

    minimum = payload.raw_jiffy_minimum
    maximum = payload.raw_jiffy_maximum
    if (minimum is None) != (maximum is None):
        raise ManifestValidationError("raw jiffy bounds must both be present or absent")
    if payload.parsed_records == 0 and minimum is not None:
        raise ManifestValidationError("empty parsed payload cannot have raw jiffy bounds")
    if payload.parsed_records > 0 and minimum is None:
        raise ManifestValidationError("parsed payload requires raw jiffy bounds")
    if minimum is not None and maximum is not None and minimum > maximum:
        raise ManifestValidationError("raw jiffy minimum cannot exceed maximum")


def _expected_trigger_status(
    compressed: CompressedIntegrity, trigger: TriggerEvidence
) -> TriggerVerificationStatus:
    declared_size = trigger.declared_byte_count
    declared_md5 = trigger.declared_md5
    if declared_size is None and declared_md5 is None:
        return TriggerVerificationStatus.NOT_PROVIDED
    if declared_size is None or declared_md5 is None:
        raise ManifestValidationError(
            "trigger declaration size and MD5 must both be present or absent"
        )

    size_matches = declared_size == compressed.byte_count
    md5_matches = declared_md5 == compressed.md5
    if size_matches and md5_matches:
        return TriggerVerificationStatus.VERIFIED
    if not size_matches and md5_matches:
        return TriggerVerificationStatus.SIZE_MISMATCH
    if size_matches and not md5_matches:
        return TriggerVerificationStatus.MD5_MISMATCH
    return TriggerVerificationStatus.SIZE_AND_MD5_MISMATCH
