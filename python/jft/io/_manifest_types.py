"""Immutable value types for the NSE CM manifest-v1 boundary."""

from dataclasses import dataclass
from datetime import date
from enum import Enum
from typing import Any, Mapping, Optional


class FilePrefix(str, Enum):
    CM = "cm"
    CASH = "cash"


class Product(str, Enum):
    FULL = "full"
    TRIM = "trim"


class RecordKind(str, Enum):
    ORDERS = "orders"
    TRADES = "trades"


class TriggerVerificationStatus(str, Enum):
    NOT_PROVIDED = "not_provided"
    VERIFIED = "verified"
    SIZE_MISMATCH = "size_mismatch"
    MD5_MISMATCH = "md5_mismatch"
    SIZE_AND_MD5_MISMATCH = "size_and_md5_mismatch"


@dataclass(frozen=True)
class ManifestSource:
    logical_path: str
    prefix: FilePrefix
    product: Product
    record_kind: RecordKind
    source_date: date
    stream: Optional[int]
    layout_id: str


@dataclass(frozen=True)
class CompressedIntegrity:
    byte_count: int
    md5: str
    sha256: str


@dataclass(frozen=True)
class TriggerEvidence:
    status: TriggerVerificationStatus
    declared_byte_count: Optional[int]
    declared_md5: Optional[str]


@dataclass(frozen=True)
class PayloadIntegrity:
    byte_count: int
    sha256: str
    payload_width: int
    delimiter: str
    framed_records: int
    parsed_records: int
    quarantined_records: int
    residue_bytes: int
    raw_jiffy_minimum: Optional[int]
    raw_jiffy_maximum: Optional[int]


@dataclass(frozen=True)
class ProcessingIdentity:
    configuration_id: str
    code_identity: str


@dataclass(frozen=True)
class RecordIdentity:
    decompressed_payload_sha256: str
    record_kind: RecordKind
    zero_based_record_index: int


@dataclass(frozen=True)
class ManifestV1:
    schema_id: str
    schema_major: int
    source: ManifestSource
    compressed: CompressedIntegrity
    trigger: TriggerEvidence
    payload: PayloadIntegrity
    processing: ProcessingIdentity

    def record_identity(self, zero_based_record_index: int) -> Optional[RecordIdentity]:
        if not 0 <= zero_based_record_index < self.payload.framed_records:
            return None
        return RecordIdentity(
            decompressed_payload_sha256=self.payload.sha256,
            record_kind=self.source.record_kind,
            zero_based_record_index=zero_based_record_index,
        )

def manifest_to_mapping(manifest: ManifestV1) -> Mapping[str, Any]:
    return {
        "schema_id": manifest.schema_id,
        "schema_major": manifest.schema_major,
        "source": {
            "logical_path": manifest.source.logical_path,
            "prefix": manifest.source.prefix.value,
            "product": manifest.source.product.value,
            "record_kind": manifest.source.record_kind.value,
            "source_date": manifest.source.source_date.isoformat(),
            "stream": manifest.source.stream,
            "layout_id": manifest.source.layout_id,
        },
        "compressed": {
            "byte_count": manifest.compressed.byte_count,
            "md5": manifest.compressed.md5,
            "sha256": manifest.compressed.sha256,
        },
        "trigger": {
            "status": manifest.trigger.status.value,
            "declared_byte_count": manifest.trigger.declared_byte_count,
            "declared_md5": manifest.trigger.declared_md5,
        },
        "payload": {
            "byte_count": manifest.payload.byte_count,
            "sha256": manifest.payload.sha256,
            "payload_width": manifest.payload.payload_width,
            "delimiter": manifest.payload.delimiter,
            "framed_records": manifest.payload.framed_records,
            "parsed_records": manifest.payload.parsed_records,
            "quarantined_records": manifest.payload.quarantined_records,
            "residue_bytes": manifest.payload.residue_bytes,
            "raw_jiffy_minimum": manifest.payload.raw_jiffy_minimum,
            "raw_jiffy_maximum": manifest.payload.raw_jiffy_maximum,
        },
        "processing": {
            "configuration_id": manifest.processing.configuration_id,
            "code_identity": manifest.processing.code_identity,
        },
    }
