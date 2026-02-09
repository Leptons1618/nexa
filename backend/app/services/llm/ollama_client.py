"""Ollama LLM client — calls the local Ollama HTTP API.

Supports both the ``/api/chat`` (messages) and ``/api/generate`` (raw prompt)
endpoints.  The chat endpoint is used by default because it is better suited
for conversational use-cases with system / user / assistant roles.
"""

from __future__ import annotations

import logging
from typing import Any, Dict, List, Optional

import httpx

from app.services.llm.base import LLMClient

logger = logging.getLogger(__name__)


class OllamaClient(LLMClient):
    """Connect to a running Ollama instance at *base_url*.

    Parameters
    ----------
    base_url:
        Root URL of the Ollama server (default ``http://localhost:11434``).
    model:
        Model tag to use, e.g. ``"mistral"``, ``"llama3"``.
    temperature, top_p, max_tokens:
        Sampling options forwarded in ``"options"`` to Ollama.
    use_chat_api:
        When *True* (default), call ``/api/chat`` (message-array style).
        Set to *False* to fall back to the legacy ``/api/generate`` endpoint.
    """

    def __init__(
        self,
        base_url: str = "http://localhost:11434",
        model: str = "mistral",
        temperature: float = 0.2,
        top_p: float = 0.9,
        max_tokens: int = 512,
        use_chat_api: bool = True,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self.model = model
        self.temperature = temperature
        self.top_p = top_p
        self.max_tokens = max_tokens
        self.use_chat_api = use_chat_api
        self._client = httpx.Client(timeout=120.0)
        logger.info(
            "OllamaClient initialised: url=%s model=%s chat_api=%s",
            self.base_url,
            self.model,
            self.use_chat_api,
        )

    # ── LLMClient interface ─────────────────────────────

    def generate(self, prompt: str, system_prompt: str = "") -> str:
        """Generate a completion.

        Dispatches to ``/api/chat`` or ``/api/generate`` depending on
        ``self.use_chat_api``.
        """
        if self.use_chat_api:
            return self._chat(prompt, system_prompt)
        return self._generate(prompt, system_prompt)

    def health_check(self) -> bool:
        """Ping the Ollama server via ``/api/tags``."""
        try:
            resp = self._client.get(f"{self.base_url}/api/tags")
            return resp.status_code == 200
        except Exception:
            return False

    # ── Ollama-specific helpers ─────────────────────────

    def list_models(self) -> List[Dict[str, Any]]:
        """Return the list of models available on the Ollama server.

        Each entry is a dict with at least ``"name"`` and ``"size"`` keys.
        Returns an empty list when the server is unreachable.
        """
        try:
            resp = self._client.get(f"{self.base_url}/api/tags")
            resp.raise_for_status()
            return resp.json().get("models", [])
        except Exception:
            logger.warning("Failed to list Ollama models", exc_info=True)
            return []

    def model_info(self, model_name: Optional[str] = None) -> Dict[str, Any]:
        """Fetch metadata for a specific model via ``/api/show``.

        Falls back to ``self.model`` when *model_name* is not supplied.
        """
        name = model_name or self.model
        try:
            resp = self._client.post(
                f"{self.base_url}/api/show",
                json={"name": name},
            )
            resp.raise_for_status()
            return resp.json()
        except Exception:
            logger.warning("Failed to fetch model info for '%s'", name, exc_info=True)
            return {}

    # ── Private endpoint wrappers ───────────────────────

    def _options(self) -> Dict[str, Any]:
        return {
            "temperature": self.temperature,
            "top_p": self.top_p,
            "num_predict": self.max_tokens,
        }

    def _chat(self, prompt: str, system_prompt: str = "") -> str:
        """Call ``/api/chat`` — the message-array endpoint."""
        messages: List[Dict[str, str]] = []
        if system_prompt:
            messages.append({"role": "system", "content": system_prompt})
        messages.append({"role": "user", "content": prompt})

        payload: dict = {
            "model": self.model,
            "messages": messages,
            "stream": False,
            "options": self._options(),
        }

        response = self._client.post(f"{self.base_url}/api/chat", json=payload)
        response.raise_for_status()
        data = response.json()
        return data.get("message", {}).get("content", "").strip()

    def _generate(self, prompt: str, system_prompt: str = "") -> str:
        """Call ``/api/generate`` — the legacy raw-prompt endpoint."""
        payload: dict = {
            "model": self.model,
            "prompt": prompt,
            "stream": False,
            "options": self._options(),
        }
        if system_prompt:
            payload["system"] = system_prompt

        response = self._client.post(f"{self.base_url}/api/generate", json=payload)
        response.raise_for_status()
        data = response.json()
        return data.get("response", "").strip()
