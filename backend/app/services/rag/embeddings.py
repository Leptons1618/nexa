"""Embedding service — wraps sentence-transformers."""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING, List

logger = logging.getLogger(__name__)

if TYPE_CHECKING:  # pragma: no cover
    from sentence_transformers import SentenceTransformer


class EmbeddingService:
    """Generate embeddings for documents and queries."""

    def __init__(
        self,
        model_name: str = "sentence-transformers/all-MiniLM-L6-v2",
        batch_size: int = 32,
        device: str | None = None,
    ) -> None:
        from sentence_transformers import SentenceTransformer  # lazy import

        self.model_name = model_name
        self.batch_size = batch_size
        self.device = device or "cpu"
        logger.info("Loading embedding model %s on %s", model_name, self.device)
        self.model = SentenceTransformer(model_name, device=self.device)

    @property
    def dimension(self) -> int:
        return self.model.get_sentence_embedding_dimension()

    def embed_documents(self, texts: List[str]) -> List[List[float]]:
        import numpy as np

        if not texts:
            return []
        embeddings = self.model.encode(
            texts,
            batch_size=self.batch_size,
            show_progress_bar=False,
            normalize_embeddings=True,
        )
        return embeddings.tolist() if isinstance(embeddings, np.ndarray) else embeddings

    def embed_query(self, text: str) -> List[float]:
        import numpy as np

        embedding = self.model.encode([text], normalize_embeddings=True)[0]
        if isinstance(embedding, np.ndarray):
            return embedding.tolist()
        return embedding
