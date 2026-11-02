# Markdown lists

Bug reports are full of steps and bullet points that currently render as one run-on paragraph. Teach the
markdown-lite renderer (spec section 2.5, `md_render`, which renders issue bodies on the issue page and
each comment) bullet and numbered lists. Everything else it does stays as it is, and the text, JSON and
CSV formats keep showing the raw body.

**Item lines.** Only unindented lines (starting at their very first code point) can be items:

- a *bullet item* starts with `- ` or `* ` (the character, then a space U+0020);
- a *numbered item* starts with 1 to 9 ASCII digits, then `. ` (a dot and a space); its number is the
  digits' value.

In both cases the rest of the line after that marker, trimmed, must not be empty; that is the item's
first text. Anything else is not an item: `-x`, `**bold**`, `1.x`, `1) x`, ten or more digits, a marker
followed only by spaces, an indented `- x`.

**Continuation lines.** A non-blank line that starts with at least two spaces and directly follows an
item line (or another continuation line of the same item) belongs to that item. Lists do not nest: an
indented `- x` there is just continuation text.

**Item HTML.** An item's text is its first text and each of its continuation lines (trimmed), joined
with `\n`, rendered with `md_inline`, inside `<li>` and `</li>`. So bold, code spans and links work in
items, and may span the item's lines.

**Lists.** A list is a maximal run of consecutive items of the same kind (bullet or numbered; `-` and `*`
bullets mix freely in one list). A blank line, a non-item line that is not a continuation, or an item of
the other kind ends it. A bullet list renders as `<ul>` + its items + `</ul>`. A numbered list renders as
`<ol>` + its items + `</ol>` when its first item's number is 1, and with `<ol start="{n}">` otherwise,
where `n` is the first item's number in decimal without leading zeros (`007. x` gives `start="7"`,
`0. x` gives `start="0"`). The numbers of later items are ignored.

**Blocks.** `md_render` now produces blocks, still joined with `\n`: lists and paragraphs. A paragraph is
a maximal run of non-blank lines that are neither items nor continuation lines, untrimmed and joined with
`\n` as before; an item line ends the paragraph before it, and a non-item line after a list starts a new
paragraph (no blank line needed). Indented lines that do not follow an item are ordinary paragraph lines
as before. No blocks still gives `""`.

Example (`md_render` input, then output; the output's lines are joined with `\n`):

    Steps:                             <p>Steps:</p>
    1. open the page                   <ol><li>open the page</li><li>click <strong>Login</strong>
    2. click **Login**                 (twice)</li></ol>
       (twice)                         <ul><li>it fails</li><li>with &lt;b&gt;500&lt;/b&gt;</li></ul>
    - it fails                         <p>done</p>
    * with <b>500</b>

    done
