"""Abstract base class for all LLM backends."""

from __future__ import annotations

from abc import ABC, abstractmethod


class LLMClient(ABC):
    """Unified interface for local (Ollama) and cloud LLM providers."""

    @abstractmethod
    def generate(self, prompt: str, system_prompt: str = "") -> str:
        """Generate a completion from *prompt* (with optional system prompt)."""
        ...

    @abstractmethod
    def health_check(self) -> bool:
        """Return ``True`` when the LLM backend is reachable."""
        ...
