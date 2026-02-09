"""Vector store abstraction — FAISS (default) and Qdrant implementations."""

from __future__ import annotations

import json
import logging
import os
from abc import ABC, abstractmethod
from pathlib import Path
from typing import Any, Dict, List, Tuple

import numpy as np

logger = logging.getLogger(__name__)

try:
    import faiss  # type: ignore[import-untyped]
except ImportError:
    faiss = None


# ── Abstract base ───────────────────────────────────────


class VectorStore(ABC):
    @abstractmethod
    def add_texts(
        self,
        texts: List[str],
        embeddings: List[List[float]],
        metadatas: List[Dict[str, Any]],
    ) -> None: ...

    @abstractmethod
    def search(
        self,
        query_embedding: List[float],
        top_k: int,
        score_threshold: float,
    ) -> List[Tuple[str, float, Dict[str, Any]]]: ...

    @abstractmethod
    def save(self) -> None: ...


# ── FAISS ───────────────────────────────────────────────


class FaissVectorStore(VectorStore):
    """Lightweight FAISS wrapper with JSON metadata persistence."""

    def __init__(self, dim: int, index_path: str, metadata_path: str) -> None:
        if faiss is None:
            raise ImportError("faiss-cpu is required for FaissVectorStore")

        self.dim = dim
        self.index_path = index_path
        self.metadata_path = metadata_path
        self._metadata: List[Dict[str, Any]] = []

        if Path(index_path).exists() and Path(metadata_path).exists():
            logger.info("Loading existing FAISS index from %s", index_path)
            self.index = faiss.read_index(index_path)
            with open(metadata_path, "r", encoding="utf-8") as fh:
                self._metadata = json.load(fh)
        else:
            logger.info("Initialising new FAISS index (dim=%s)", dim)
            self.index = faiss.IndexFlatIP(dim)
            Path(os.path.dirname(index_path) or ".").mkdir(parents=True, exist_ok=True)
            Path(os.path.dirname(metadata_path) or ".").mkdir(parents=True, exist_ok=True)

    def add_texts(
        self,
        texts: List[str],
        embeddings: List[List[float]],
        metadatas: List[Dict[str, Any]],
    ) -> None:
        if not embeddings:
            return
        if len(embeddings) != len(texts) or len(metadatas) != len(texts):
            raise ValueError("Length mismatch between texts, embeddings, and metadata")
        vectors = np.array(embeddings).astype("float32")
        self.index.add(vectors)
        self._metadata.extend(metadatas)

    def search(
        self,
        query_embedding: List[float],
        top_k: int,
        score_threshold: float,
    ) -> List[Tuple[str, float, Dict[str, Any]]]:
        if self.index.ntotal == 0:
            return []
        q = np.array([query_embedding]).astype("float32")
        scores, ids = self.index.search(q, top_k)
        results: List[Tuple[str, float, Dict[str, Any]]] = []
        for score, idx in zip(scores[0], ids[0]):
            if idx == -1 or score < score_threshold:
                continue
            meta = self._metadata[idx]
            results.append((meta.get("text", ""), float(score), meta))
        return results

    def save(self) -> None:
        faiss.write_index(self.index, self.index_path)
        with open(self.metadata_path, "w", encoding="utf-8") as fh:
            json.dump(self._metadata, fh, ensure_ascii=False, indent=2)


# ── Qdrant ──────────────────────────────────────────────


class QdrantVectorStore(VectorStore):
    """Qdrant cloud / self-hosted implementation."""

    def __init__(
        self,
        dim: int,
        url: str,
        collection: str,
        api_key: str | None = None,
    ) -> None:
        from qdrant_client import QdrantClient
        from qdrant_client.http import models as rest

        self.client = QdrantClient(url=url, api_key=api_key)
        self.collection = collection
        self.rest = rest

        existing = [c.name for c in self.client.get_collections().collections]
        if collection not in existing:
            self.client.recreate_collection(
                collection_name=collection,
                vectors_config=rest.VectorParams(size=dim, distance=rest.Distance.COSINE),
            )

    def add_texts(
        self,
        texts: List[str],
        embeddings: List[List[float]],
        metadatas: List[Dict[str, Any]],
    ) -> None:
        if not embeddings:
            return
        import uuid

        points = []
        for text, vec, meta in zip(texts, embeddings, metadatas):
            payload = {**meta, "text": text}
            points.append(
                self.rest.PointStruct(
                    id=str(uuid.uuid4()), vector=vec, payload=payload
                )
            )
        self.client.upsert(collection_name=self.collection, points=points)

    def search(
        self,
        query_embedding: List[float],
        top_k: int,
        score_threshold: float,
    ) -> List[Tuple[str, float, Dict[str, Any]]]:
        hits = self.client.search(
            collection_name=self.collection,
            query_vector=query_embedding,
            limit=top_k,
            score_threshold=score_threshold,
            with_payload=True,
        )
        results: List[Tuple[str, float, Dict[str, Any]]] = []
        for hit in hits:
            payload = hit.payload or {}
            results.append((payload.get("text", ""), float(hit.score), payload))
        return results

    def save(self) -> None:
        pass  # persistence handled by Qdrant server


# ── Factory ─────────────────────────────────────────────


def create_vector_store(
    kind: str,
    dim: int,
    index_path: str = "",
    metadata_path: str = "",
    qdrant_url: str | None = None,
    qdrant_api_key: str | None = None,
    qdrant_collection: str = "nexa_support",
) -> VectorStore:
    kind_lower = kind.lower()
    if kind_lower == "faiss":
        return FaissVectorStore(dim=dim, index_path=index_path, metadata_path=metadata_path)
    if kind_lower == "qdrant":
        if not qdrant_url:
            raise ValueError("QDRANT_URL must be set when VECTOR_STORE=qdrant")
        return QdrantVectorStore(
            dim=dim, url=qdrant_url, collection=qdrant_collection, api_key=qdrant_api_key
        )
    raise ValueError(f"Unsupported vector store: {kind}")
