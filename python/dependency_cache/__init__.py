from .automagic_parser import automagic_dependency_cache
from .dependency_cache import (
    DependencyCacheBase,
    dependency_cache,
)

__all__ = [
    "dependency_cache",
    "DependencyCacheBase",
    "automagic_dependency_cache",
]
