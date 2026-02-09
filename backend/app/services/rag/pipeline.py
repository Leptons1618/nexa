"""RAG pipeline — retrieval + augmented generation."""

from __future__ import annotations

import logging
from pathlib import Path
from typing import List, Tuple

from typing import TYPE_CHECKING

if TYPE_CHECKING:  # pragma: no cover
    from app.services.llm.base import LLMClient
    from app.services.rag.embeddings import EmbeddingService
    from app.services.rag.vector_store import VectorStore

logger = logging.getLogger(__name__)

_REFUSAL = (
    "I can only help with questions related to Nexa. "
    "This information is not available in the documentation."
)


class RAGPipeline:
    def __init__(
        self,
        embedder: EmbeddingService,
        store: VectorStore,
        llm: LLMClient,
        system_prompt_path: str,
        rag_prompt_path: str,
        top_k: int = 4,
        similarity_threshold: float = 0.35,
    ) -> None:
        self.embedder = embedder
        self.store = store
        self.llm = llm
        self.system_prompt = Path(system_prompt_path).read_text(encoding="utf-8")
        self.rag_prompt = Path(rag_prompt_path).read_text(encoding="utf-8")
        self.top_k = top_k
        self.similarity_threshold = similarity_threshold

    # ── internal helpers ────────────────────────────────

    def _retrieve(self, query: str) -> List[Tuple[str, float, dict]]:
        query_vec = self.embedder.embed_query(query)
        return self.store.search(
            query_vec, top_k=self.top_k, score_threshold=self.similarity_threshold
        )

    def _build_user_prompt(self, query: str, contexts: List[str]) -> str:
        context_block = "\n---\n".join(contexts)
        return (
            f"{self.rag_prompt}\n"
            f"Context:\n{context_block}\n\n"
            f"User question: {query}\n"
            "Answer concisely using only the context."
        )

    # ── public API ──────────────────────────────────────

    def generate(self, query: str) -> Tuple[str, List[str]]:
        """Run retrieval then generation.  Returns (answer, source_names)."""
        hits = self._retrieve(query)
        if not hits:
            return _REFUSAL, []

        contexts = [h[0] for h in hits]
        sources = list({h[2].get("document_name", "unknown") for h in hits})
        user_prompt = self._build_user_prompt(query, contexts)
        answer = self.llm.generate(user_prompt, system_prompt=self.system_prompt)
        return answer, sources

    def generate_stream(self, query: str):
        """Run retrieval then stream generation tokens.

        Yields ``("sources", sources_list)`` first, then
        ``("token", chunk_str)`` for each token, and finally
        ``("done", "")`` when the stream has finished.
        """
        hits = self._retrieve(query)
        if not hits:
            yield ("sources", [])
            yield ("token", _REFUSAL)
            yield ("done", "")
            return

        contexts = [h[0] for h in hits]
        sources = list({h[2].get("document_name", "unknown") for h in hits})

        # Build source context snippets for hover previews
        source_contexts = []
        for text, score, meta in hits:
            source_contexts.append({
                "document": meta.get("document_name", "unknown"),
                "chunk_id": meta.get("chunk_id", ""),
                "text": text[:500],
                "score": round(score, 3),
            })

        yield ("sources", sources)
        yield ("contexts", source_contexts)

        user_prompt = self._build_user_prompt(query, contexts)
        for chunk in self.llm.generate_stream(
            user_prompt, system_prompt=self.system_prompt
        ):
            yield ("token", chunk)

        yield ("done", "")
