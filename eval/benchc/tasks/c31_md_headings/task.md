# Markdown headings

Issue descriptions are getting long and people want section headings in them. Teach the
markdown-lite renderer (spec section 2.5, `md_render`, which renders issue bodies on the issue page and
each comment) headings. Everything else it does stays as it is, and the text, JSON and CSV formats keep
showing the raw body.

**Heading lines.** A line is a heading line when it starts, at its very first code point (no
indentation), with one, two or three `#` followed by a space (U+0020), and what follows that space,
trimmed, is not empty; that trimmed rest is the heading's text. Any other line that starts with `#` is
ordinary text: four or more `#`, a `#` not followed by a space (`#tag`, `#12`, `#` then a tab), an
indented `#`, or nothing but spaces after `# `.

**Blocks.** `md_render` now produces blocks, still joined with `\n`. A heading line is a block of its
own: it ends the paragraph running before it, and the non-blank lines after it start a new paragraph
(no blank line needed). Paragraphs are otherwise unchanged: each maximal run of non-blank lines that are
not heading lines, untrimmed, joined with `\n`, rendered as `<p>` + `md_inline(...)` + `</p>`. No blocks
still gives `""`.

**Heading HTML.** The page already uses `<h1>` and `<h2>`, so levels are shifted down: `#` renders as
`<h3>`, `##` as `<h4>`, `###` as `<h5>`. The block is `<hN id="{id}">` + `md_inline(text)` + `</hN>`, so
the text is escaped and bold, code spans and links work in it as they do in paragraphs.

**Ids.** The slug of a heading is computed from its text (raw, before markdown or escaping): every ASCII
letter (lowercased) and ASCII digit is kept, every maximal run of any other code points (spaces,
punctuation, `*`, non-ASCII letters, ...) becomes one `-`, and leading and trailing `-` are removed. The
base id is `h-` + slug, or just `h` when the slug is empty. Within one `md_render` call the first
heading with a given base id uses it as is, and the n-th heading with the same base id (n >= 2) gets
`{base}-{n}`. The issue body and each comment are separate calls, so they count separately.

Examples (`md_render` input, then output):

    # Fix **login** & logout!          <h3 id="h-fix-login-logout">Fix <strong>login</strong> &amp; logout!</h3>

    Intro                              <p>Intro</p>
    ## Steps                           <h4 id="h-steps">Steps</h4>
    1. open                            <p>1. open
    #tag                               #tag</p>
    ###   Steps                        <h5 id="h-steps-2">Steps</h5>

(the right column is one string; its lines are joined with `\n`).
