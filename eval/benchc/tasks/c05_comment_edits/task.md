# Editing comments

Authors want to fix their comments. Add a new `edits` column to the `comments` table (an integer,
default 0: how many times the comment was edited; a fresh database is fine, no migration of
existing files is needed).

**Route.** New `POST /p/{KEY}/{N}/comments/{C}/edit` with the form fields `author` and `body`.
`{C}` is a comment id (the `id` column), a segment of ASCII digits with value >= 1 exactly like
`{N}`; any other segment falls through to 404 `not found`, as does any other method on this path.
After the usual unknown-project and unknown-issue checks, in order:

1. 404 `no such comment: {C}` when there is no comment with that id on this issue (a comment of
   another issue counts as missing);
2. `author` empty: 422 `author required`;
3. `author` is not an existing user: 422 `no such user: {author}`;
4. `author` is not the comment's author: 422 `not the author`;
5. `body` (a text field, cleaned like the comment body of `POST /p/{KEY}/{N}/comments`) empty:
   422 `body required`.

Then, if the cleaned body equals the comment's current body, nothing changes. Otherwise the body is
replaced, `edits` goes up by one, and an activity entry is logged with kind `edited` and the
author's name as detail (`activity.txt` shows `edited bob`). The answer is the HTML issue page.

**Showing it.**

- `comments_text`: an edited comment's line ends with ` (edited)`: `bob: Fixed in v3 (edited)`.
- Issue page: an edited comment is
  `<li><b>{author}</b> <span class="edited">(edited)</span> {!md_render(body)}</li>`; one never
  edited stays `<li><b>{author}</b> {!md_render(body)}</li>`.
- Issue JSON: each comment object gets its id first and its edit count last:
  `{"id":1,"author":"bob","body":"ok","edits":0}`.
