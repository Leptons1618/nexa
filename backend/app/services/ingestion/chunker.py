"""Simple word-based text chunker."""

from __future__ import annotations

from typing import List


def chunk_text(text: str, chunk_size: int = 400, chunk_overlap: int = 80) -> List[str]:
    """Split *text* into overlapping chunks of approximately *chunk_size* words."""
    words = text.split()
    if not words:
        return []

    chunks: List[str] = []
    start = 0
    while start < len(words):
        end = start + chunk_size
        chunks.append(" ".join(words[start:end]))
        if end >= len(words):
            break
        start = max(0, end - chunk_overlap)
    return chunks
