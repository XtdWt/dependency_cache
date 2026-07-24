from .automagic_parser import automagically_dependency_cached
from .dependency_cache import (
    DependencyCacheBase,
    dependency_cached,
)

__all__ = [
    "dependency_cached",
    "DependencyCacheBase",
    "automagically_dependency_cached",
]
