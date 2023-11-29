CREATE TABLE
    IF NOT EXISTS fileInfos (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        path TEXT NOT NULL UNIQUE,
        len INTEGER NOT NULL,
        modified_time TEXT NOT NULL,
        sha1 TEXT
    );

DELETE FROM fileInfos
WHERE
    id = ?;

SELECT
    *
FROM
    fileInfos
WHERE
    path LIKE ?;


ALTER TABLE `fileinfos` CHANGE `old_name` `new_name` new_type NOT NULL;
