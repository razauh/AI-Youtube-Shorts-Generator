__all__ = ["generate_shorts"]


def generate_shorts(*args, **kwargs):
    from .pipeline import generate_shorts as _generate_shorts

    return _generate_shorts(*args, **kwargs)
