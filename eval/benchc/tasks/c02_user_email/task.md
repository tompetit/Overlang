# Email addresses for users

Users get an optional email address. Store it in a new `email` column of the `users` table (a
fresh database is fine; no migration of existing files is needed). Many users have no email, and
no two users may have the same one.

**Valid emails.** After trimming (emails are line fields), an email is valid when it has at most
254 code points and is `local@domain` with exactly one `@`, where `local` is one or more ASCII
letters, digits, `.`, `_`, `%`, `+` or `-`, and `domain` is two or more labels separated by `.`,
each label one or more ASCII letters, digits or `-`. Emails are stored with ASCII uppercase letters
turned into lowercase, so two emails that differ only in case are the same email.

**Writing it.**

- `POST /users` takes an optional form field `email`. Empty or absent means no email. The checks,
  in order: the existing three (`name required`, `bad name`, `user exists`), then, when the email
  is not empty, 422 `bad email` if it is not valid and 422 `email exists` if another user already
  has it. Nothing is stored on an error.
- New route `POST /users/{NAME}/email` with the form field `email`. `{NAME}` is the path segment
  used verbatim. An unknown user is 404 `no such user: {NAME}` (checked first). An empty email
  removes the user's email; otherwise the same two checks as above (`bad email`, then
  `email exists`; setting a user's own current email again, in any case, is not an error). On
  success it answers with the HTML users page. Any other method on this path is 404 `not found`.

**Showing it.**

- `users_text` (`/users.txt`): `{id} {name} {email}` per user, with `-` for no email:
  `1 alice alice@example.org` and `2 bob -`.
- Users page: a user with an email is `<li>{name} <a href="mailto:{email}">{email}</a></li>`, one
  without is `<li>{name}</li>` as before. The form gets ` <input name="email" placeholder="email">`
  right after the `name` input.
- Issue JSON: a key `"assignee_email"` right after `"assignee"`: the assignee's current email as a
  string, or `null` when the issue is unassigned or the assignee has no email:
  `..."assignee":"alice","assignee_email":"alice@example.org","labels":...`.
- Issue page: in the meta line, an assignee with an email is shown as
  `Assignee: <a href="mailto:{email}">{name}</a> |`; without one it stays `Assignee: {name} |`
  (and `Assignee: - |` when unassigned).

Changing a user's email changes what every issue assigned to them shows. Nothing else changes:
`issues.txt`, the CSV, filters and comments look exactly as before.
