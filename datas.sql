CREATE TABLE
    IF NOT EXISTS fileInfos (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        path TEXT NOT NULL UNIQUE,
        len INTEGER NOT NULL,
        modified_time TEXT NOT NULL,
        sha1 TEXT
    );

DELETE FROM fileInfo
WHERE
    id = ?;

SELECT
    *
FROM
    fileInfo
WHERE
    path LIKE ?;


ALTER TABLE `fileinfo` CHANGE `old_name` `new_name` new_type NOT NULL;
