"""File loader — reads .md, .txt, .pdf, .html, .json, .xml, .csv, .rst, .doc, .docx files."""

from __future__ import annotations

import csv
import io
import json
import logging
from pathlib import Path
from typing import List, Tuple

from pypdf import PdfReader

logger = logging.getLogger(__name__)

SUPPORTED_EXTENSIONS = {
    ".md", ".txt", ".pdf", ".html", ".htm",
    ".json", ".xml", ".csv", ".rst", ".doc", ".docx",
}


def load_file(path: str) -> str:
    ext = Path(path).suffix.lower()
    if ext not in SUPPORTED_EXTENSIONS:
        raise ValueError(f"Unsupported file type: {ext}")

    if ext in {".md", ".txt", ".rst", ".xml", ".html", ".htm"}:
        return Path(path).read_text(encoding="utf-8", errors="replace")

    if ext == ".pdf":
        reader = PdfReader(path)
        return "\n".join(page.extract_text() or "" for page in reader.pages)

    if ext == ".json":
        raw = Path(path).read_text(encoding="utf-8", errors="replace")
        try:
            data = json.loads(raw)
            return json.dumps(data, indent=2, ensure_ascii=False)
        except json.JSONDecodeError:
            return raw

    if ext == ".csv":
        raw = Path(path).read_text(encoding="utf-8", errors="replace")
        reader_csv = csv.reader(io.StringIO(raw))
        rows = list(reader_csv)
        return "\n".join(", ".join(row) for row in rows)

    if ext in {".doc", ".docx"}:
        # Best-effort: extract text from docx (ZIP-based) or return raw text
        try:
            import zipfile
            import xml.etree.ElementTree as ET

            with zipfile.ZipFile(path) as z:
                if "word/document.xml" in z.namelist():
                    xml_content = z.read("word/document.xml")
                    tree = ET.fromstring(xml_content)
                    ns = {"w": "http://schemas.openxmlformats.org/wordprocessingml/2006/main"}
                    texts = [node.text for node in tree.iter("{%s}t" % ns["w"]) if node.text]
                    return " ".join(texts)
        except Exception:
            pass
        # Fallback: read as bytes and decode
        try:
            return Path(path).read_text(encoding="utf-8", errors="replace")
        except Exception:
            return ""

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
