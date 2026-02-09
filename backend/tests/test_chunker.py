"""Tests for the chunker utility."""

import pytest

from app.services.ingestion.chunker import chunk_text


class TestChunker:
    def test_empty_text(self):
        assert chunk_text("") == []

    def test_short_text_single_chunk(self):
        text = "one two three"
        chunks = chunk_text(text, chunk_size=10, chunk_overlap=2)
        assert len(chunks) == 1
        assert chunks[0] == text

    def test_overlap_produces_extra_chunks(self):
        words = " ".join(f"w{i}" for i in range(20))
        chunks = chunk_text(words, chunk_size=10, chunk_overlap=3)
        assert len(chunks) >= 2
        # overlap means some words appear in multiple chunks
        assert chunks[0].split()[-1] in chunks[1]

    def test_no_overlap(self):
        words = " ".join(f"w{i}" for i in range(20))
        chunks = chunk_text(words, chunk_size=10, chunk_overlap=0)
        assert len(chunks) == 2

    def test_chunk_size_larger_than_text(self):
        text = "short"
        assert chunk_text(text, chunk_size=1000) == ["short"]
