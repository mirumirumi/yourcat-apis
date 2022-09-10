from __future__ import annotations
from typing import NoReturn

from decimal import Decimal


def decimal_to_float(obj: object) -> float | NoReturn:
    if isinstance(obj, Decimal): return float(obj)
    raise TypeError
