"""Chat session history — JSON file-based persistence.

Stores sessions under ``data/history/`` as individual JSON files.
"""

from __future__ import annotations

import json
import logging
from datetime import datetime, timezone
from pathlib import Path
from typing import List, Optional

from app.models.schemas import (
    HistoryMessage,
    SessionCreate,
    SessionDetail,
    SessionSummary,
)

logger = logging.getLogger(__name__)

HISTORY_DIR = Path("data/history")


class HistoryService:
    """CRUD operations for chat session history (JSON files)."""

    def __init__(self, base_dir: Path | None = None):
        self.base_dir = base_dir or HISTORY_DIR
        self.base_dir.mkdir(parents=True, exist_ok=True)

    def _session_path(self, session_id: str) -> Path:
        safe_id = session_id.replace("/", "_").replace("\\", "_")
        return self.base_dir / f"{safe_id}.json"

    def _now_iso(self) -> str:
        return datetime.now(timezone.utc).isoformat()

    # ── List ────────────────────────────────────────────

    def list_sessions(self) -> List[SessionSummary]:
        """Return summaries of all saved sessions, newest first."""
        summaries: list[SessionSummary] = []
        for p in sorted(self.base_dir.glob("*.json"), key=lambda f: f.stat().st_mtime, reverse=True):
            try:
                data = json.loads(p.read_text(encoding="utf-8"))
                summaries.append(
                    SessionSummary(
                        id=data.get("id", p.stem),
                        title=data.get("title", "Untitled"),
                        message_count=len(data.get("messages", [])),
                        document_count=len(data.get("documents", [])),
                        created_at=data.get("created_at", ""),
                        updated_at=data.get("updated_at", ""),
                    )
                )
            except Exception as exc:
                logger.warning("Skipping corrupt session file %s: %s", p, exc)
        return summaries

    # ── Get ─────────────────────────────────────────────

    def get_session(self, session_id: str) -> Optional[SessionDetail]:
        p = self._session_path(session_id)
        if not p.exists():
            return None
        try:
            data = json.loads(p.read_text(encoding="utf-8"))
            return SessionDetail(
                id=data["id"],
                title=data.get("title", "Untitled"),
                messages=[HistoryMessage(**m) for m in data.get("messages", [])],
                documents=data.get("documents", []),
                created_at=data.get("created_at", ""),
                updated_at=data.get("updated_at", ""),
            )
        except Exception as exc:
            logger.error("Failed to read session %s: %s", session_id, exc)
            return None

    # ── Save / Update ───────────────────────────────────

    def save_session(self, session: SessionCreate) -> SessionDetail:
        """Create or update a session."""
        p = self._session_path(session.id)
        now = self._now_iso()

        if p.exists():
            try:
                existing = json.loads(p.read_text(encoding="utf-8"))
                created_at = existing.get("created_at", now)
            except Exception:
                created_at = now
        else:
            created_at = now

        payload = {
            "id": session.id,
            "title": session.title,
            "messages": [m.model_dump() for m in session.messages],
            "documents": session.documents,
            "created_at": created_at,
            "updated_at": now,
        }
        p.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")
        logger.info("Saved session %s (%d messages)", session.id, len(session.messages))

        return SessionDetail(
            id=session.id,
            title=session.title,
            messages=session.messages,
            documents=session.documents,
            created_at=created_at,
            updated_at=now,
        )

    # ── Delete ──────────────────────────────────────────

    def delete_session(self, session_id: str) -> bool:
        p = self._session_path(session_id)
        if not p.exists():
            return False

        # Clean up uploaded documents associated with this session
        try:
            data = json.loads(p.read_text(encoding="utf-8"))
            documents = data.get("documents", [])
            for doc_path in documents:
                doc_file = Path(doc_path)
                if doc_file.exists() and doc_file.is_file():
                    doc_file.unlink()
                    logger.info("Deleted associated document: %s", doc_path)
        except Exception as exc:
            logger.warning("Error cleaning up documents for session %s: %s", session_id, exc)

        p.unlink()
        logger.info("Deleted session %s", session_id)
        return True
