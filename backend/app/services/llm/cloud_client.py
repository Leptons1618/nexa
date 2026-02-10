"""Cloud LLM client — works with any OpenAI-compatible API."""

from __future__ import annotations

import logging

import httpx

from app.services.llm.base import LLMClient

logger = logging.getLogger(__name__)


class CloudLLMClient(LLMClient):
    """Connect to an OpenAI-compatible chat-completion endpoint."""

    def __init__(
        self,
        api_key: str,
        base_url: str = "https://api.openai.com/v1",
        model: str = "gpt-4",
        temperature: float = 0.2,
        top_p: float = 0.9,
        max_tokens: int = 512,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self.model = model
        self.api_key = api_key
        self.temperature = temperature
        self.top_p = top_p
        self.max_tokens = max_tokens
        self._client = httpx.Client(
            timeout=60.0,
            headers={
                "Authorization": f"Bearer {self.api_key}",
                "Content-Type": "application/json",
            },
        )
        logger.info("CloudLLMClient initialised: url=%s model=%s", self.base_url, self.model)

    def generate(self, prompt: str, system_prompt: str = "") -> str:
        messages = []
        if system_prompt:
            messages.append({"role": "system", "content": system_prompt})
        messages.append({"role": "user", "content": prompt})

        payload = {
            "model": self.model,
            "messages": messages,
            "temperature": self.temperature,
            "top_p": self.top_p,
            "max_tokens": self.max_tokens,
        }
        response = self._client.post(
            f"{self.base_url}/chat/completions",
            json=payload,
        )
        response.raise_for_status()
        data = response.json()
        return data["choices"][0]["message"]["content"].strip()

    def health_check(self) -> bool:
        try:
            resp = self._client.get(f"{self.base_url}/models")
            return resp.status_code == 200
        except Exception:
            return False

    def list_models(self) -> list[dict]:
        """Fetch available models from the cloud provider.

        Returns a list of dicts with at least an ``id`` key.
        """
        try:
            resp = self._client.get(f"{self.base_url}/models")
            resp.raise_for_status()
            data = resp.json()
            # OpenAI-compatible APIs return {"data": [{"id": ...}, ...]}
            models = data.get("data", [])
            return [{"id": m.get("id", "unknown"), "owned_by": m.get("owned_by", "")} for m in models]
        except Exception as exc:
            logger.warning("Failed to list cloud models: %s", exc)
            return []

    def test_connection(self) -> dict:
        """Test the connection by hitting the models endpoint.

        Returns a dict with ``success``, ``message``, and optionally ``models_count``.
        """
        try:
            resp = self._client.get(f"{self.base_url}/models")
            if resp.status_code == 200:
                data = resp.json()
                count = len(data.get("data", []))
                return {
                    "success": True,
                    "message": f"Connected successfully. {count} model(s) available.",
                    "models_count": count,
                }
            elif resp.status_code == 401:
                return {"success": False, "message": "Authentication failed. Check your API key."}
            else:
                return {"success": False, "message": f"Server returned status {resp.status_code}."}
        except httpx.ConnectError:
            return {"success": False, "message": "Could not connect. Check the base URL."}
        except Exception as exc:
            return {"success": False, "message": f"Connection error: {exc}"}
