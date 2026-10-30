# Clear completed todos

Add `POST /clear-done` that deletes every todo marked done and answers with the re-rendered page (`200`),
like the other mutating routes. Open todos keep their ids. Calling it when nothing is done is fine and also
answers `200`. Put the database part in a function `clear_done(db)` that returns the number of rows removed.
