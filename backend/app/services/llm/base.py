"""Abstract base class for all LLM backends."""

from __future__ import annotations

from abc import ABC, abstractmethod
from typing import Generator


class LLMClient(ABC):
    """Unified interface for local (Ollama) and cloud LLM providers."""

    @abstractmethod
    def generate(self, prompt: str, system_prompt: str = "") -> str:
        """Generate a completion from *prompt* (with optional system prompt)."""
        ...

    def generate_stream(self, prompt: str, system_prompt: str = "") -> Generator[str, None, None]:
        """Stream tokens one chunk at a time.  Default falls back to generate()."""
        yield self.generate(prompt, system_prompt)

    @abstractmethod
    def health_check(self) -> bool:
        """Return ``True`` when the LLM backend is reachable."""
        ...
