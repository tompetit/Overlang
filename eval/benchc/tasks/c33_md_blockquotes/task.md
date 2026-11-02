# Markdown blockquotes

People reply to comments by quoting them, e-mail style, and the `>` lines currently render as one
paragraph of `&gt;`. Teach the markdown-lite renderer (spec section 2.5, `md_render`, which renders issue
bodies on the issue page and each comment) blockquotes. Everything else it does stays as it is, and the
text, JSON and CSV formats keep showing the raw body.

**Quote lines.** A line is a quote line when its very first code point is `>` (no indentation). This is
about the raw text the user typed: a line that starts with the four characters `&gt;` is not a quote
line, and neither is ` > x`.

**Quote blocks.** A maximal run of consecutive quote lines is one quote block. Its inner text is the run's
lines, each with its leading `>` removed and then at most one space (U+0020) after it removed, joined with
`\n`. The block renders as `<blockquote>` + `md_render(inner text)` + `</blockquote>`: the inner text is
rendered with these same rules, so a line that is just `>` separates paragraphs inside the quote, and
`>>` or `> >` nests a quote inside a quote. A quote whose inner text renders as `""` is
`<blockquote></blockquote>`.

**Depth limit.** Quotes nest at most 4 deep: when rendering the inner text of a quote that is already
inside three others, quote lines are not recognised and are ordinary paragraph text there.

**Blocks.** `md_render` now produces blocks, still joined with `\n`: quote blocks and paragraphs. A quote
line ends the paragraph before it, and a non-blank line after a quote block that is not a quote line
starts a new paragraph (no blank line needed). A blank line ends a quote block, so two runs separated by
a blank line are two blockquotes. Paragraphs are otherwise unchanged: maximal runs of non-blank lines
that are not quote lines, untrimmed, joined with `\n`. No blocks still gives `""`.

Example (`md_render` input, then output; the output's lines are joined with `\n`):

    > alice wrote:                     <blockquote><p>alice wrote:</p>
    >                                  <blockquote><p>it is <strong>broken</strong></p></blockquote></blockquote>
    >> it is **broken**                <p>Fixed in v2, a &gt; b.</p>
    Fixed in v2, a > b.
