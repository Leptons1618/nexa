"""Health controller — service readiness checks."""

from __future__ import annotations

import logging

from typing import TYPE_CHECKING

from app.models.schemas import HealthResponse

if TYPE_CHECKING:  # pragma: no cover
    from app.services.llm.base import LLMClient

logger = logging.getLogger(__name__)


class HealthController:
    def __init__(self, llm: LLMClient) -> None:
        self.llm = llm

    def check_health(self) -> HealthResponse:
        llm_ok = self.llm.health_check()
        status = "ok" if llm_ok else "degraded"
        detail = "All systems operational" if llm_ok else "LLM service unreachable"
        logger.info("Health check: status=%s llm=%s", status, llm_ok)
        return HealthResponse(status=status, llm_connected=llm_ok, detail=detail)
