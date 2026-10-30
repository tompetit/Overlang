# No duplicate titles

`POST /add` should refuse a title that is already in the list: compare after trimming and ignoring case
(`Buy milk` and ` buy MILK ` are the same). Answer `409` with the body `duplicate title` and do not store
anything. Once the existing todo is deleted the title can be added again. A blank title is still `400`.
