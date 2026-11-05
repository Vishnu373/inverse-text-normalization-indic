"""Decode denominator-first fractions emitted in spoken order by the graph."""


def reorder_fractions(text: str) -> str:
    parts = text.split("\ue000")
    output = [parts[0]]
    for part in parts[1:]:
        denominator, middle, rest = part.partition("\ue001")
        numerator, end, suffix = rest.partition("\ue002")
        if not middle or not end or any(
            not operand.isascii() or not operand.isdecimal()
            for operand in (denominator, numerator)
        ):
            raise ValueError("Malformed fraction markers")
        output.extend((numerator, "/", denominator, suffix))
    result = "".join(output)
    if "\ue001" in result or "\ue002" in result:
        raise ValueError("Malformed fraction markers")
    return result
