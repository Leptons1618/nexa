"""File loader — reads .md, .txt,  and .pdf files."""

from __future__ import annotations

import logging
from pathlib import Path
from typing import List, Tuple

from pypdf import PdfReader

logger = logging.getLogger(__name__)

SUPPORTED_EXTENSIONS = {".md", ".txt", ".pdf"}


def load_file(path: str) -> str:
    ext = Path(path).suffix.lower()
    if ext not in SUPPORTED_EXTENSIONS:
        raise ValueError(f"Unsupported file type: {ext}")

    if ext in {".md", ".txt"}:
        return Path(path).read_text(encoding="utf-8")

    if ext == ".pdf":
        reader = PdfReader(path)
        return "\n".join(page.extract_text() or "" for page in reader.pages)

    raise ValueError(f"Unsupported file type: {ext}")


def gather_documents(paths: List[str]) -> List[Tuple[str, str]]:
    """Return ``[(path, text), ...]``.  Accepts files or directories."""
    docs: List[Tuple[str, str]] = []
    for p in paths:
        path_obj = Path(p)
        if not path_obj.exists():
            logger.warning("Path does not exist, skipping: %s", p)
            continue
        if path_obj.is_dir():
            for child in sorted(path_obj.rglob("*")):
                if child.is_file() and child.suffix.lower() in SUPPORTED_EXTENSIONS:
                    docs.append((str(child), load_file(str(child))))
        else:
            docs.append((str(path_obj), load_file(str(path_obj))))
    return docs
