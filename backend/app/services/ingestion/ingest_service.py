"""Ingestion service — loads, chunks, embeds, and stores documents."""

from __future__ import annotations

import logging
import uuid
from typing import List, Optional

from app.services.ingestion.chunker import chunk_text
from app.services.ingestion.loader import gather_documents
from app.services.rag.embeddings import EmbeddingService
from app.services.rag.vector_store import VectorStore

logger = logging.getLogger(__name__)


class IngestionService:
    def __init__(
        self,
        embedder: EmbeddingService,
        store: VectorStore,
        chunk_size: int = 400,
        chunk_overlap: int = 80,
    ) -> None:
        self.embedder = embedder
        self.store = store
        self.chunk_size = chunk_size
        self.chunk_overlap = chunk_overlap

    def ingest_paths(
        self,
        paths: List[str],
        tags: Optional[List[str]] = None,
        version: Optional[str] = None,
    ) -> int:
        documents = gather_documents(paths)
        chunk_texts: List[str] = []
        metadatas: List[dict] = []

        for doc_path, text in documents:
            chunks = chunk_text(text, self.chunk_size, self.chunk_overlap)
            for chunk in chunks:
                meta = {
                    "id": str(uuid.uuid4()),
                    "document_name": doc_path.replace("\\", "/").rsplit("/", 1)[-1],
                    "source_path": doc_path,
                    "version": version,
                    "tags": tags or [],
                    "text": chunk,
                }
                chunk_texts.append(chunk)
                metadatas.append(meta)

        if chunk_texts:
            embeddings = self.embedder.embed_documents(chunk_texts)
            self.store.add_texts(chunk_texts, embeddings, metadatas)
            self.store.save()

        logger.info("Ingested %d chunks from %d paths", len(chunk_texts), len(paths))
        return len(chunk_texts)
