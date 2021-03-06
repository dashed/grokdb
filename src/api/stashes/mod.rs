extern crate rusqlite;
extern crate rustc_serialize;

pub mod restify;
pub mod reviewable;

use std::sync::Arc;

use rusqlite::types::ToSql;
use rusqlite::{SqliteStatement};
use rustc_serialize::json;

use ::database::{DB, QueryError};
pub use self::restify::restify;

pub enum SortBy {
    CreatedAt,
    UpdatedAt,
    Name
    // ReviewedDate, // when stash was last reviewed
    // TimesReviewed // how many times a stash was reviewed
}

pub enum SortOrder {
    Descending,
    Ascending
}

pub struct StashesPageRequest {
    page: i64, // page >= 1
    per_page: i64, // per_page >= 0
    sort_by: SortBy,
    order: SortOrder,
    card: Option<i64> // card id. mark any stash if it contains this card.
}

impl StashesPageRequest {

    pub fn get_offset(&self) -> i64 {
        let offset: i64 = (self.page - 1) * self.per_page;
        return offset;
    }
}


#[derive(Debug, Clone, RustcDecodable)]
pub struct CreateStash {
    name: String,
    description: Option<String>
}

#[derive(Debug, Clone, RustcDecodable)]
pub struct UpdateStash {
    name: Option<String>,
    description: Option<String>
}

impl UpdateStash {

    pub fn valid_name(&self) -> bool {

        if self.name.is_some() {
            let name = self.name.as_ref().unwrap().trim().to_string();

            return name.len() > 0;
        }

        return true;
    }

    #[allow(unused_parens)]
    pub fn should_update(&self) -> bool {
        return (
            self.valid_name() ||
            self.description.is_some()
        );
    }

    // get fields to update.
    // this is a helper to construct the sql update query
    pub fn sqlize(&self) -> (String, Vec<(&str, &ToSql)>) {

        let mut fields: Vec<String> = vec![];
        let mut values: Vec<(&str, &ToSql)> = vec![];

        if self.name.is_some() {
            fields.push(format!("name = :name"));
            let tuple: (&str, &ToSql) = (":name", self.name.as_ref().unwrap());
            values.push(tuple);
        }

        if self.description.is_some() {
            fields.push(format!("description = :description"));
            let tuple: (&str, &ToSql) = (":description", self.description.as_ref().unwrap());
            values.push(tuple);
        }

        return (fields.join(", "), values);
    }
}


#[derive(Debug, RustcEncodable)]
struct Stash {
    id: i64,
    name: String,
    description: String,
    created_at: i64, // unix timestamp
    updated_at: i64  // unix timestamp
}

#[derive(Debug, RustcEncodable)]
struct StashWithCard {
    id: i64,
    name: String,
    description: String,
    created_at: i64, // unix timestamp
    updated_at: i64,  // unix timestamp
    has_card: bool
}

#[derive(Debug, RustcEncodable)]
struct StashResponse {
    id: i64,
    name: String,
    description: String,
    created_at: i64, // unix timestamp
    updated_at: i64  // unix timestamp
}

impl StashResponse {

    pub fn to_json(&self) -> String {
        return json::encode(self).unwrap();
    }
}

#[derive(Debug, RustcEncodable)]
struct StashResponseHasCard {
    id: i64,
    name: String,
    description: String,
    created_at: i64, // unix timestamp
    updated_at: i64,  // unix timestamp
    has_card: bool
}

impl StashResponseHasCard {

    pub fn to_json(&self) -> String {
        return json::encode(self).unwrap();
    }
}

#[derive(Debug, RustcEncodable)]
pub struct StashPaginationInfo {
    num_of_stashes: i64
}

impl StashPaginationInfo {

    pub fn to_json(&self) -> String {
        return json::encode(self).unwrap();
    }
}

#[derive(Debug, Clone)]
pub struct StashesAPI {
    pub db: Arc<DB>,
}

impl StashesAPI {

    pub fn get_response(&self, stash_id: i64) -> Result<StashResponse, QueryError> {

        // get props

        let maybe_stash: Result<Stash, QueryError> = self.get(stash_id);
        let stash: Stash = match maybe_stash {
            Err(why) => {
                // why: QueryError
                return Err(why);
            },
            Ok(stash) => stash,
        };

        let response = StashResponse {
            id: stash.id,
            name: stash.name,
            description: stash.description,
            created_at: stash.created_at,
            updated_at: stash.updated_at
        };

        return Ok(response);
    }

    pub fn get_response_with_card(&self, stash_id: i64, card_id: i64) -> Result<StashResponseHasCard, QueryError> {

        // get props

        let maybe_stash: Result<StashWithCard, QueryError> = self.get_with_card(stash_id, card_id);
        let stash: StashWithCard = match maybe_stash {
            Err(why) => {
                // why: QueryError
                return Err(why);
            },
            Ok(stash) => stash,
        };

        let response = StashResponseHasCard {
            id: stash.id,
            name: stash.name,
            description: stash.description,
            created_at: stash.created_at,
            updated_at: stash.updated_at,
            has_card: stash.has_card
        };

        return Ok(response);
    }

    // pub fn get_list_by_id(&self, page_query: StashesPageRequest) -> Result<Vec<i64>, QueryError> {

    //     let db_conn_guard = self.db.lock().unwrap();
    //     let ref db_conn = *db_conn_guard;

    //     let ref page_query = page_query;

    //     let ref query = get_stashes_query(page_query);

    //     let params: &[(&str, &ToSql)] = &[
    //         (":offset", &(page_query.get_offset())),
    //         (":per_page", &(page_query.per_page))
    //     ];

    //     let maybe_stmt = db_conn.prepare(query);

    //     if maybe_stmt.is_err() {

    //         let why = maybe_stmt.unwrap_err();

    //         let err = QueryError {
    //             sqlite_error: why,
    //             query: query.clone(),
    //         };
    //         return Err(err);
    //     }

    //     let mut stmt: SqliteStatement = maybe_stmt.unwrap();

    //     let maybe_iter = stmt.query_named(params);

    //     match maybe_iter {
    //         Err(why) => {
    //             let err = QueryError {
    //                 sqlite_error: why,
    //                 query: query.clone(),
    //             };
    //             return Err(err);
    //         },
    //         Ok(iter) => {

    //             let mut vec_of_stash_id: Vec<i64> = Vec::new();

    //             for result_row in iter {

    //                 let stash_id: i64 = match result_row {
    //                     Err(why) => {
    //                         let err = QueryError {
    //                             sqlite_error: why,
    //                             query: query.clone(),
    //                         };
    //                         return Err(err);
    //                     },
    //                     Ok(row) => row.get(0)
    //                 };

    //                 vec_of_stash_id.push(stash_id);
    //             }

    //             let vec_of_stash_id = vec_of_stash_id;

    //             return Ok(vec_of_stash_id);
    //         }
    //     };
    // }

    pub fn get_list(&self, page_query: &StashesPageRequest) -> Result<Vec<i64>, QueryError> {

        let db_conn_guard = self.db.lock().unwrap();
        let ref db_conn = *db_conn_guard;

        let ref query = get_stashes_query(page_query);

        let params: &[(&str, &ToSql)] = &[
            (":offset", &(page_query.get_offset())),
            (":per_page", &(page_query.per_page))
        ];

        let maybe_stmt = db_conn.prepare(query);

        if maybe_stmt.is_err() {

            let why = maybe_stmt.unwrap_err();

            let err = QueryError {
                sqlite_error: why,
                query: query.clone(),
            };
            return Err(err);
        }

        let mut stmt: SqliteStatement = maybe_stmt.unwrap();

        let maybe_iter = stmt.query_named(params);

        match maybe_iter {
            Err(why) => {
                let err = QueryError {
                    sqlite_error: why,
                    query: query.clone(),
                };
                return Err(err);
            },
            Ok(iter) => {

                let mut vec_of_stash_id: Vec<i64> = Vec::new();

                for result_row in iter {

                    let stash_id: i64 = match result_row {
                        Err(why) => {
                            let err = QueryError {
                                sqlite_error: why,
                                query: query.clone(),
                            };
                            return Err(err);
                        },
                        Ok(row) => row.get(0)
                    };

                    vec_of_stash_id.push(stash_id);
                }

                let vec_of_stash_id = vec_of_stash_id;

                return Ok(vec_of_stash_id);
            }
        };
    }

    pub fn get(&self, stash_id: i64) -> Result<Stash, QueryError> {

        let db_conn_guard = self.db.lock().unwrap();
        let ref db_conn = *db_conn_guard;

        let ref query = format!("
            SELECT
                stash_id, name, description, created_at, updated_at
            FROM Stashes
            WHERE stash_id = $1 LIMIT 1;
        ");

        let results = db_conn.query_row(query, &[&stash_id], |row| -> Stash {
            return Stash {
                id: row.get(0),
                name: row.get(1),
                description: row.get(2),
                created_at: row.get(3),
                updated_at: row.get(4)
            };
        });

        match results {
            Err(why) => {
                let err = QueryError {
                    sqlite_error: why,
                    query: query.clone(),
                };
                return Err(err);
            },
            Ok(stash) => {
                return Ok(stash);
            }
        };
    }

    pub fn get_with_card(&self, stash_id: i64, card_id: i64) -> Result<StashWithCard, QueryError> {

        let db_conn_guard = self.db.lock().unwrap();
        let ref db_conn = *db_conn_guard;

        let ref query = format!("
            SELECT
                stash_id,
                name,
                description,
                created_at,
                updated_at,
                CASE WHEN
                    sc.card
                    IS NOT NULL
                    THEN 1
                    ELSE 0
                END AS has_card
            FROM Stashes
            LEFT OUTER JOIN
                (
                    SELECT
                        stash, card
                    FROM
                    StashCards
                        WHERE
                    card = :card_id
                ) AS sc
            ON
                sc.stash = stash_id
            WHERE
                stash_id = :stash_id
            LIMIT 1;
        ");

        let params: &[(&str, &ToSql)] = &[
            (":stash_id", &stash_id),
            (":card_id", &card_id)
        ];

        let results = db_conn.query_row_named(query, params, |row| -> StashWithCard {

            let __has_card: i64 = row.get(5);

            return StashWithCard {
                id: row.get(0),
                name: row.get(1),
                description: row.get(2),
                created_at: row.get(3),
                updated_at: row.get(4),
                has_card: __has_card == 1
            };
        });

        match results {
            Err(why) => {
                let err = QueryError {
                    sqlite_error: why,
                    query: query.clone(),
                };
                return Err(err);
            },
            Ok(stash) => {
                return Ok(stash);
            }
        };
    }

    pub fn count(&self) -> Result<i64, QueryError> {

        let db_conn_guard = self.db.lock().unwrap();
        let ref db_conn = *db_conn_guard;

        let ref query = format!("
            SELECT
                COUNT(1)
            FROM Stashes;
        ");

        let maybe_count = db_conn.query_row(query, &[], |row| -> i64 {
            return row.get(0);
        });

        match maybe_count {
            Err(why) => {
                let err = QueryError {
                    sqlite_error: why,
                    query: query.clone(),
                };
                return Err(err);
            },
            Ok(count) => {
                return Ok(count);
            }
        };
    }

    pub fn exists(&self, stash_id: i64) -> Result<bool, QueryError> {

        let db_conn_guard = self.db.lock().unwrap();
        let ref db_conn = *db_conn_guard;

        let ref query = format!("
            SELECT
                COUNT(1)
            FROM Stashes
            WHERE stash_id = $1 LIMIT 1;
        ");

        let stash_exists = db_conn.query_row(query, &[&stash_id], |row| -> bool {
            let count: i64 = row.get(0);
            return count >= 1;
        });

        match stash_exists {
            Err(why) => {
                let err = QueryError {
                    sqlite_error: why,
                    query: query.clone(),
                };
                return Err(err);
            },
            Ok(stash_exists) => {
                return Ok(stash_exists);
            }
        };
    }

    pub fn create(&self, create_stash_request: &CreateStash) -> Result<i64, QueryError> {

        let db_conn_guard = self.db.lock().unwrap();
        let ref db_conn = *db_conn_guard;

        try!(DB::prepare_query(db_conn));

        let description = match create_stash_request.description {
            Some(ref description) => description.clone(),
            None => "".to_string()
        };

        let ref query = format!("INSERT INTO Stashes(name, description) VALUES ($1, $2);");

        let params: &[&ToSql] = &[

            // required
            &create_stash_request.name, // $1

            // optional
            &description // $2
        ];

        match db_conn.execute(query, params) {
            Err(why) => {
                let err = QueryError {
                    sqlite_error: why,
                    query: query.clone(),
                };
                return Err(err);
            },
            _ => {/* query sucessfully executed */},
        }

        let rowid = db_conn.last_insert_rowid();

        return Ok(rowid);
    }

    pub fn update(&self, stash_id: i64, update_stash_request: &UpdateStash) -> Result<(), QueryError> {

        let db_conn_guard = self.db.lock().unwrap();
        let ref db_conn = *db_conn_guard;

        try!(DB::prepare_query(db_conn));

        let (fields, values): (String, Vec<(&str, &ToSql)>) = update_stash_request.sqlize();

        let mut values = values;
        values.push((":stash_id", &stash_id));
        let values = values;

        let ref query_update = format!("
            UPDATE Stashes
            SET
            {fields}
            WHERE stash_id = :stash_id;
        ", fields = fields);

        match db_conn.execute_named(query_update, &values[..]) {
            Err(why) => {
                let err = QueryError {
                    sqlite_error: why,
                    query: query_update.clone(),
                };
                return Err(err);
            },
            _ => {/* query sucessfully executed */},
        }

        return Ok(());
    }

    pub fn delete(&self, stash_id: i64) -> Result<(), QueryError> {

        let db_conn_guard = self.db.lock().unwrap();
        let ref db_conn = *db_conn_guard;

        try!(DB::prepare_query(db_conn));

        let ref query_delete = format!("
            DELETE FROM Stashes WHERE stash_id = :stash_id;
        ");

        let params: &[(&str, &ToSql)] = &[
            (":stash_id", &stash_id)
        ];

        match db_conn.execute_named(query_delete, params) {
            Err(why) => {
                let err = QueryError {
                    sqlite_error: why,
                    query: query_delete.clone(),
                };
                return Err(err);
            },
            _ => {/* query sucessfully executed */},
        }

        return Ok(());
    }

    pub fn add_card_to_stash(&self, stash_id: i64, card_id: i64) -> Result<(), QueryError> {

        let db_conn_guard = self.db.lock().unwrap();
        let ref db_conn = *db_conn_guard;

        try!(DB::prepare_query(db_conn));

        let ref query_insert = format!("
            INSERT OR IGNORE INTO StashCards(stash, card) VALUES (:stash_id, :card_id);
        ");

        let params: &[(&str, &ToSql)] = &[
            (":stash_id", &stash_id),
            (":card_id", &card_id)
        ];

        match db_conn.execute_named(query_insert, params) {
            Err(why) => {
                let err = QueryError {
                    sqlite_error: why,
                    query: query_insert.clone(),
                };
                return Err(err);
            },
            _ => {/* query sucessfully executed */},
        }

        return Ok(());
    }

    pub fn remove_card_from_stash(&self, stash_id: i64, card_id: i64) -> Result<(), QueryError> {

        let db_conn_guard = self.db.lock().unwrap();
        let ref db_conn = *db_conn_guard;

        try!(DB::prepare_query(db_conn));

        let ref query_delete = format!("
            DELETE
            FROM StashCards
            WHERE
                stash = :stash_id
            AND
                card = :card_id;
        ");

        let params: &[(&str, &ToSql)] = &[
            (":stash_id", &stash_id),
            (":card_id", &card_id)
        ];

        match db_conn.execute_named(query_delete, params) {
            Err(why) => {
                let err = QueryError {
                    sqlite_error: why,
                    query: query_delete.clone(),
                };
                return Err(err);
            },
            _ => {/* query sucessfully executed */},
        }

        return Ok(());
    }

    pub fn remove_card_from_all_stashes(&self, card_id: i64) -> Result<(), QueryError> {

        let db_conn_guard = self.db.lock().unwrap();
        let ref db_conn = *db_conn_guard;

        try!(DB::prepare_query(db_conn));

        let ref query_delete = format!("
            DELETE
            FROM StashCards
            WHERE
                card = :card_id;
        ");

        let params: &[(&str, &ToSql)] = &[
            (":card_id", &card_id)
        ];

        match db_conn.execute_named(query_delete, params) {
            Err(why) => {
                let err = QueryError {
                    sqlite_error: why,
                    query: query_delete.clone(),
                };
                return Err(err);
            },
            _ => {/* query sucessfully executed */},
        }

        return Ok(());
    }

    pub fn remove_all_cards_from_stash(&self, stash_id: i64) -> Result<(), QueryError> {

        let db_conn_guard = self.db.lock().unwrap();
        let ref db_conn = *db_conn_guard;

        try!(DB::prepare_query(db_conn));

        let ref query_delete = format!("
            DELETE
            FROM StashCards
            WHERE
                stash = :stash_id;
        ");

        let params: &[(&str, &ToSql)] = &[
            (":stash_id", &stash_id)
        ];

        match db_conn.execute_named(query_delete, params) {
            Err(why) => {
                let err = QueryError {
                    sqlite_error: why,
                    query: query_delete.clone(),
                };
                return Err(err);
            },
            _ => {/* query sucessfully executed */},
        }

        return Ok(());
    }

    pub fn count_by_card(&self, card_id: i64) -> Result<i64, QueryError> {

        let db_conn_guard = self.db.lock().unwrap();
        let ref db_conn = *db_conn_guard;

        let ref query = format!("
            SELECT
                COUNT(1)
            FROM StashCards
            WHERE card = :card_id;
        ");

        let params: &[(&str, &ToSql)] = &[
            (":card_id", &card_id)
        ];

        let maybe_count = db_conn.query_row_named(query, params, |row| -> i64 {
            return row.get(0);
        });

        match maybe_count {
            Err(why) => {
                let err = QueryError {
                    sqlite_error: why,
                    query: query.clone(),
                };
                return Err(err);
            },
            Ok(count) => {
                return Ok(count);
            }
        };
    }

    pub fn get_by_card(&self, card_id: i64, page_query: &StashesPageRequest) -> Result<Vec<i64>, QueryError> {

        let db_conn_guard = self.db.lock().unwrap();
        let ref db_conn = *db_conn_guard;

        let ref query = get_stashes_query_by_card(card_id, page_query);

        let params: &[(&str, &ToSql)] = &[
            (":card_id", &card_id),
            (":offset", &(page_query.get_offset())),
            (":per_page", &(page_query.per_page))
        ];

        let maybe_stmt = db_conn.prepare(query);

        if maybe_stmt.is_err() {

            let why = maybe_stmt.unwrap_err();

            let err = QueryError {
                sqlite_error: why,
                query: query.clone(),
            };
            return Err(err);
        }

        let mut stmt: SqliteStatement = maybe_stmt.unwrap();

        let maybe_iter = stmt.query_named(params);

        match maybe_iter {
            Err(why) => {
                let err = QueryError {
                    sqlite_error: why,
                    query: query.clone(),
                };
                return Err(err);
            },
            Ok(iter) => {

                let mut vec_of_stash_id: Vec<i64> = Vec::new();

                for result_row in iter {

                    let stash_id: i64 = match result_row {
                        Err(why) => {
                            let err = QueryError {
                                sqlite_error: why,
                                query: query.clone(),
                            };
                            return Err(err);
                        },
                        Ok(row) => row.get(0)
                    };

                    vec_of_stash_id.push(stash_id);
                }

                return Ok(vec_of_stash_id);
            }
        };
    }
}

/* helpers */

fn get_stashes_query(page_query: &StashesPageRequest) -> String {

    let sort_order: &str = match page_query.order {
        SortOrder::Descending => "DESC",
        SortOrder::Ascending => "ASC"
    };

    let query = match page_query.sort_by {

        SortBy::CreatedAt => {

            format!("
                SELECT
                    stash_id, name, description, created_at, updated_at
                FROM
                    Stashes
                WHERE oid NOT IN (
                    SELECT
                        oid
                    FROM
                        Stashes
                    ORDER BY
                        created_at
                    {sort_order}
                    LIMIT :offset
                )
                ORDER BY
                    created_at
                {sort_order}
                LIMIT :per_page;
            ", sort_order = sort_order)
        },

        SortBy::UpdatedAt => {

            format!("
                SELECT
                    stash_id, name, description, created_at, updated_at
                FROM
                    Stashes
                WHERE oid NOT IN (
                    SELECT
                        oid
                    FROM
                        Stashes
                    ORDER BY
                        updated_at
                    {sort_order}
                    LIMIT :offset
                )
                ORDER BY
                    updated_at
                {sort_order}
                LIMIT :per_page;
            ", sort_order = sort_order)
        },

        SortBy::Name => {

            format!("
                SELECT
                    stash_id, name, description, created_at, updated_at
                FROM
                    Stashes
                WHERE oid NOT IN (
                    SELECT
                        oid
                    FROM
                        Stashes
                    ORDER BY
                        name
                    {sort_order}
                    LIMIT :offset
                )
                ORDER BY
                    name
                {sort_order}
                LIMIT :per_page;
            ", sort_order = sort_order)
        }
    };

    return query;
}

fn get_stashes_query_by_card(card_id: i64, page_query: &StashesPageRequest) -> String {

    let sort_order: &str = match page_query.order {
        SortOrder::Descending => "DESC",
        SortOrder::Ascending => "ASC"
    };

    let query = match page_query.sort_by {

        SortBy::CreatedAt => {

            format!("
                SELECT
                    stash_id, name, description, created_at, updated_at
                FROM
                    Stashes
                INNER JOIN
                    StashCards
                ON
                    Stashes.stash_id = StashCards.stash
                WHERE Stashes.oid NOT IN (
                    SELECT
                        Stashes.oid
                    FROM
                        Stashes
                    INNER JOIN
                        StashCards
                    ON
                        Stashes.stash_id = StashCards.stash
                    AND StashCards.card = :card_id
                    ORDER BY
                        created_at
                    {sort_order}
                    LIMIT :offset
                )
                AND StashCards.card = :card_id
                ORDER BY
                    created_at
                {sort_order}
                LIMIT :per_page;
            ", sort_order = sort_order)
        },

        SortBy::UpdatedAt => {

            format!("
                SELECT
                    stash_id, name, description, created_at, updated_at
                FROM
                    Stashes
                INNER JOIN
                    StashCards
                ON
                    Stashes.stash_id = StashCards.stash
                WHERE Stashes.oid NOT IN (
                    SELECT
                        Stashes.oid
                    FROM
                        Stashes
                    INNER JOIN
                        StashCards
                    ON
                        Stashes.stash_id = StashCards.stash
                    AND StashCards.card = :card_id
                    ORDER BY
                        updated_at
                    {sort_order}
                    LIMIT :offset
                )
                AND StashCards.card = :card_id
                ORDER BY
                    updated_at
                {sort_order}
                LIMIT :per_page;
            ", sort_order = sort_order)
        },

        SortBy::Name => {

            format!("
                SELECT
                    stash_id, name, description, created_at, updated_at
                FROM
                    Stashes
                INNER JOIN
                    StashCards
                ON
                    Stashes.stash_id = StashCards.stash
                WHERE Stashes.oid NOT IN (
                    SELECT
                        Stashes.oid
                    FROM
                        Stashes
                    INNER JOIN
                        StashCards
                    ON
                        Stashes.stash_id = StashCards.stash
                    AND StashCards.card = :card_id
                    ORDER BY
                        name
                    {sort_order}
                    LIMIT :offset
                )
                AND StashCards.card = :card_id
                ORDER BY
                    name
                {sort_order}
                LIMIT :per_page;
            ", sort_order = sort_order)
        }
    };

    return query;
}
