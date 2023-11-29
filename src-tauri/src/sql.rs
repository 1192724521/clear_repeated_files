pub const GET_DATAS: &str = r#"
SELECT
    *
FROM
    fileInfos
WHERE
    path LIKE ?||'%'"#;

pub const GET_FILE_INFOS_BY_PATH: &str = r#"
SELECT
    *
FROM
    fileInfos
WHERE
    path LIKE ?||'%'"#;

pub const SET_FILEINFOS_SHA1_BY_ID: &str = r#"
UPDATE fileInfos
SET
    sha1 = ?
WHERE
    id = ?"#;
