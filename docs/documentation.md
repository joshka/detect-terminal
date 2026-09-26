# Writing Documentation

Write so someone unfamiliar with the implementation can make the next decision: which entry point to
call, what the result means, or how to investigate a wrong match. A shorter page is useful only if
it preserves the conditions and reasoning the reader needs.

## Start with the Contract

Before drafting a claim, trace the code that makes it true and find the test or external reference
that supports it. A plausible environment variable is not evidence that a terminal sets it. A unit
test with an invented variable proves how the detector responds, not compatibility with a product.

For detection claims, name the input, precedence, result, and uncertainty. For example:

> `TERM_PROGRAM=ghostty` selects Ghostty before `TERM` fallbacks. If only `TERM=xterm-256color` is
> present, the result identifies xterm emulation; many different applications use that value.

That gives a caller something to act on. "Accurately detects your terminal" supplies no conditions
and overstates what inherited environment hints can establish.

Keep facts, proposals, and untested assumptions distinguishable. Describe the current code, and put
future work in an issue or release note rather than writing it into today's API contract. When a
reason is unclear, investigate before supplying a persuasive explanation.

## Put Explanations Where Readers Need Them

| Surface            | What belongs there                                                                |
| ------------------ | --------------------------------------------------------------------------------- |
| README             | Purpose, first use, support status, setup, and links to deeper material           |
| Crate Rustdoc      | The detection model, entry points, precedence, side effects, and limitations      |
| Item Rustdoc       | The local caller contract: accepted input, output, defaults, and failure behavior |
| Source comments    | Why ordering, representation, or a surprising implementation choice matters       |
| Contributor guides | Repeatable development practices and the reasons for them                         |
| Release notes      | Changes users must account for when adopting a version                            |

Explain shared behavior once at its owner and link to it. Someone landing directly on a field or
function should still understand its local meaning without reading the whole crate first.

## Write Connected Explanations

Lead with the operation or decision. Give each paragraph one subject, then explain its conditions
and consequences in reading order. Use lists for separate steps or alternatives, and tables when
comparing the same properties across several things.

Replace general praise with observable behavior. "Probe failures leave version metadata absent and
appear in `command_probes`" says more than "gracefully handles errors." Likewise, "Use `--no-env` to
omit the diagnostic map; identifiers remain visible" is more useful than "enable privacy mode."

Use the same term for the same concept. In this crate, a terminal application, a terminfo name, and
a multiplexer describe different things. Avoid rotating synonyms that blur those distinctions.
Explain a necessary domain term before relying on it.

Trim repeated introductions and sentences that merely announce what the page will cover. Repair
vague or choppy sentences before deleting useful context. Read the result as a caller: can they
identify the actor, input, outcome, and exception without filling in missing steps?

## Review Substance Before Polish

1. Check every changed behavioral claim against the implementation and regression evidence.
1. Check the defaults, side effects, precedence, and failure cases together; these often drift when
   the happy path still looks correct.
1. Run examples and commands with their stated inputs. Use controlled environments for examples that
   promise a specific result.
1. Check nearby pages for contradictory claims, then remove duplication or link to the owner.
1. Read for clarity: replace unsupported certainty, empty praise, invented labels, and page
   narration with the actual mechanism. Preserve qualifications that affect the caller's decision.
1. Run the relevant checks from [Rustdoc contracts](rustdoc.md) and `just fmt-md-check`.

These are review questions, not a vocabulary ban or a demand for uniform prose. Stop editing when
the explanation is accurate, useful, and readable; swapping equivalent wording is not a quality
gate.
