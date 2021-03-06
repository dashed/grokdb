extern crate iron;
extern crate router;
extern crate bodyparser;
extern crate rustc_serialize;

use iron::status;
use iron::prelude::*;
use iron::mime::Mime;
use router::Router;
use urlencoded::{UrlEncodedQuery, QueryMap};
use rustc_serialize::json;
use regex::Regex;


use std::sync::Arc;
use std::ops::Deref;
use std::error::Error;

use ::api::{GrokDB, ErrorResponse};
use ::api::decks::{CreateDeck, UpdateDeck, DeckResponse};
use ::database::QueryError;

// attach decks REST endpoints to given router
pub fn restify(router: &mut Router, grokdb: GrokDB) {

    let grokdb = Arc::new(grokdb);

    let decks_list_re = Regex::new(r"^[1-9]\d*(,[1-9]\d*)*$").unwrap();

    router.get("/decks", {
        let grokdb = grokdb.clone();
        move |req: &mut Request| -> IronResult<Response> {
            let ref grokdb = grokdb.deref();

            let list_deck_ids: Vec<i64> = match req.get_ref::<UrlEncodedQuery>() {

                Err(why) => {

                    let ref reason = format!("{:?}", why);
                    let res_code = status::BadRequest;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: why.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                },

                Ok(ref hashmap) => {
                    let hashmap: &QueryMap = hashmap;

                    let decks: Vec<i64> = match hashmap.contains_key("decks") {
                        true => {
                            let maybe_decks: &Vec<String> = hashmap.get("decks").unwrap();

                            if maybe_decks.len() <= 0 {
                                vec![]
                            } else {

                                let ref decks_str: String = maybe_decks[0];

                                if decks_list_re.is_match(decks_str) {
                                    let decks = decks_str.split(",").map(
                                        |x: &str| -> i64 {
                                            x.parse::<i64>().unwrap()
                                    });

                                    decks.collect::<Vec<i64>>()
                                } else {

                                    let ref reason = format!("Invalid list of deck ids");
                                    let res_code = status::BadRequest;

                                    let err_response = ErrorResponse {
                                        status: res_code,
                                        developerMessage: reason,
                                        userMessage: reason,
                                    }.to_json();

                                    return Ok(Response::with((res_code, err_response)));
                                }
                            }
                        },

                        _ => vec![]
                    };

                    decks
                }
            };

            let mut decks: Vec<DeckResponse> = vec![];

            for deck_id in list_deck_ids {

                let maybe_deck: Result<DeckResponse, QueryError> = grokdb.decks.get_response(deck_id);

                let deck: DeckResponse = match maybe_deck {

                    Err(why) => {
                        // why: QueryError

                        let ref reason = format!("{:?}", why);
                        let res_code = status::NotFound;

                        let err_response = ErrorResponse {
                            status: res_code,
                            developerMessage: reason,
                            userMessage: why.description(),
                        }.to_json();

                        return Ok(Response::with((res_code, err_response)));
                    },

                    Ok(deck) => deck,
                };

                decks.push(deck);
            }

            let ref decks = decks;

            let response = json::encode(decks).unwrap();

            let content_type = "application/json".parse::<Mime>().unwrap();

            return Ok(Response::with((content_type, status::Ok, response)));
        }
    });

    router.head("/decks/:deck_id", {
        let grokdb = grokdb.clone();
        move |req: &mut Request| -> IronResult<Response> {
            let ref grokdb = grokdb.deref();

            // fetch and parse requested deck id

            let deck_id: &str = req.extensions.get::<Router>().unwrap().find("deck_id").unwrap();

            let deck_id: i64 = match deck_id.parse::<u64>() {
                Ok(deck_id) => deck_id as i64,
                Err(why) => {

                    let ref reason = format!("{:?}", why);
                    let res_code = status::BadRequest;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: why.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                }
            };

            match grokdb.decks.exists(deck_id) {

                Err(why) => {
                    // why: QueryError

                    let ref reason = format!("{:?}", why);
                    let res_code = status::InternalServerError;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: why.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                },

                Ok(false) => {
                    return Ok(Response::with((status::NotFound, "")));
                },

                Ok(true) => {
                    return Ok(Response::with((status::Ok, "")));
                }
            }
        }
    });

    router.get("/decks/:deck_id", {
        let grokdb = grokdb.clone();
        move |req: &mut Request| -> IronResult<Response> {
            let ref grokdb = grokdb.deref();

            // fetch and parse requested deck id

            let deck_id = req.extensions.get::<Router>().unwrap().find("deck_id").unwrap();

            let deck_id: i64 = match deck_id.parse::<u64>() {
                Ok(deck_id) => deck_id as i64,
                Err(why) => {

                    let ref reason = format!("{:?}", why);
                    let res_code = status::BadRequest;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: why.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                }
            };

            return get_deck_by_id(grokdb, deck_id);
        }
    });

    router.get("/decks/:deck_id/ancestors/name", {
        let grokdb = grokdb.clone();
        move |req: &mut Request| -> IronResult<Response> {
            let ref grokdb = grokdb.deref();

            // fetch and parse requested deck id

            let deck_id = req.extensions.get::<Router>().unwrap().find("deck_id").unwrap();

            let deck_id: i64 = match deck_id.parse::<u64>() {
                Ok(deck_id) => deck_id as i64,
                Err(why) => {

                    let ref reason = format!("{:?}", why);
                    let res_code = status::BadRequest;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: why.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                }
            };

            match grokdb.decks.exists(deck_id) {

                Err(why) => {
                    // why: QueryError

                    let ref reason = format!("{:?}", why);
                    let res_code = status::InternalServerError;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: why.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                },

                Ok(false) => {
                    return Ok(Response::with((status::NotFound, "")));
                },

                Ok(true) => {
                    /* deck exists; continue */
                }
            }

            let maybe_ancestors: Result<Vec<String>, QueryError> = grokdb.decks.ancestors_by_name(deck_id);

            let response: String = match maybe_ancestors {
                Err(why) => {
                    // why: QueryError

                    let ref reason = format!("{:?}", why);
                    let res_code = status::InternalServerError;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: why.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));

                },
                Ok(ancestors) => {
                    let ref ancestors = ancestors;
                    json::encode(ancestors).unwrap()
                },
            };

            let content_type = "application/json".parse::<Mime>().unwrap();

            return Ok(Response::with((content_type, status::Ok, response)));
        }
    });

    router.get("/decks/:deck_id/ancestors/id", {
        let grokdb = grokdb.clone();
        move |req: &mut Request| -> IronResult<Response> {
            let ref grokdb = grokdb.deref();

            // fetch and parse requested deck id

            let deck_id = req.extensions.get::<Router>().unwrap().find("deck_id").unwrap();

            let deck_id: i64 = match deck_id.parse::<u64>() {
                Ok(deck_id) => deck_id as i64,
                Err(why) => {

                    let ref reason = format!("{:?}", why);
                    let res_code = status::BadRequest;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: why.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                }
            };

            match grokdb.decks.exists(deck_id) {

                Err(why) => {
                    // why: QueryError

                    let ref reason = format!("{:?}", why);
                    let res_code = status::InternalServerError;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: why.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                },

                Ok(false) => {
                    return Ok(Response::with((status::NotFound, "")));
                },

                Ok(true) => {
                    /* deck exists; continue */
                }
            }

            let maybe_ancestors: Result<Vec<i64>, QueryError> = grokdb.decks.ancestors(deck_id);

            let response: String = match maybe_ancestors {
                Err(why) => {
                    // why: QueryError

                    let ref reason = format!("{:?}", why);
                    let res_code = status::InternalServerError;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: why.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));

                },
                Ok(ancestors) => {
                    let ref ancestors = ancestors;
                    json::encode(ancestors).unwrap()
                },
            };

            let content_type = "application/json".parse::<Mime>().unwrap();

            return Ok(Response::with((content_type, status::Ok, response)));
        }
    });

    router.delete("/decks/:deck_id", {
        let grokdb = grokdb.clone();
        move |req: &mut Request| -> IronResult<Response> {
            let ref grokdb = grokdb.deref();

            // fetch and parse requested deck id

            let deck_id = req.extensions.get::<Router>().unwrap().find("deck_id").unwrap();

            let deck_id: i64 = match deck_id.parse::<u64>() {
                Ok(deck_id) => deck_id as i64,
                Err(why) => {

                    let ref reason = format!("{:?}", why);
                    let res_code = status::BadRequest;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: why.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                }
            };

            // delete deck

            match grokdb.decks.delete(deck_id) {
                Err(why) => {
                    // why: QueryError
                    let ref reason = format!("{:?}", why);
                    let res_code = status::InternalServerError;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: why.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                },
                _ => {/* deck sucessfully deleted */},
            };

            return Ok(Response::with((status::Ok)));
        }
    });

    router.post("/decks", {
        let grokdb = grokdb.clone();
        move |req: &mut Request| -> IronResult<Response> {
            let ref grokdb = grokdb.deref();

            // parse json

            let create_deck_request = req.get::<bodyparser::Struct<CreateDeck>>();

            let create_deck_request: CreateDeck = match create_deck_request {

                Ok(Some(create_deck_request)) => {

                    let create_deck_request: CreateDeck = create_deck_request;

                    // ensure parent deck (if given) exists; otherwise bail early
                    match create_deck_request.parent {
                        Some(parent_deck_id) => {
                            match deck_exists(grokdb, parent_deck_id) {
                                Err(response) => {
                                    return response;
                                },
                                _ => {/* noop; continue */}
                            }
                        },
                        _ => {/* noop; continue */}
                    }
                    create_deck_request
                },

                Ok(None) => {

                    let reason = "no JSON given";
                    let res_code = status::BadRequest;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: reason,
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                },

                Err(err) => {

                    let ref reason = format!("{:?}", err);
                    let res_code = status::BadRequest;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: err.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                }
            };

            // ensure deck title is non-empty string when trimmed
            let mut create_deck_request = create_deck_request;
            create_deck_request.name = create_deck_request.name.trim().to_string();

            if create_deck_request.name.len() <= 0 {
                let ref reason = format!("deck name should be non-empty string when trimmed");
                let res_code = status::BadRequest;

                let err_response = ErrorResponse {
                    status: res_code,
                    developerMessage: reason,
                    userMessage: reason,
                }.to_json();

                return Ok(Response::with((res_code, err_response)));
            }

            let create_deck_request = create_deck_request;

            // create deck

            let deck_id: i64 = match grokdb.decks.create(&create_deck_request) {
                Err(why) => {
                    // why: QueryError

                    let ref reason = format!("{:?}", why);
                    let res_code = status::InternalServerError;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: why.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                },
                Ok(rowid) => {
                    // rowid: i64
                    /* deck sucessfully created */
                    rowid
                },
            };

            // connect new deck to parent (if given).
            // invariant: parent deck exists
            match create_deck_request.parent {
                Some(parent_deck_id) => {

                    let parent_deck_id: i64 = parent_deck_id;

                    match grokdb.decks.connect_decks(deck_id, parent_deck_id) {
                        Err(why) => {
                            // why: QueryError
                            let ref reason = format!("{:?}", why);
                            let res_code = status::InternalServerError;

                            let err_response = ErrorResponse {
                                status: res_code,
                                developerMessage: reason,
                                userMessage: why.description(),
                            }.to_json();

                            return Ok(Response::with((res_code, err_response)));
                        },
                        _ => {/* connected new deck with parent; continue */}
                    }
                },
                _ => {/* no parent specified; continue */}
            }

            return get_deck_by_id(grokdb.clone(), deck_id);
        }
    });

    router.patch("/decks/:deck_id", {
        let grokdb = grokdb.clone();
        move |req: &mut Request| -> IronResult<Response> {
            let ref grokdb = grokdb.deref();

            let update_deck_request = req.get::<bodyparser::Struct<UpdateDeck>>();

            // fetch and parse requested deck id

            let deck_id: &str = req.extensions.get::<Router>().unwrap().find("deck_id").unwrap();

            let deck_id: i64 = match deck_id.parse::<u64>() {
                Ok(deck_id) => deck_id as i64,
                Err(why) => {

                    let ref reason = format!("{:?}", why);
                    let res_code = status::BadRequest;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: why.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                }
            };

            // parse deck patch request

            let ref update_deck_request: UpdateDeck = match update_deck_request {

                Ok(Some(update_deck_request)) => {
                    let update_deck_request: UpdateDeck = update_deck_request;
                    update_deck_request
                },

                Ok(None) => {

                    let reason = "no JSON given";
                    let res_code = status::BadRequest;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: reason,
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                },

                Err(err) => {

                    let ref reason = format!("{:?}", err);
                    let res_code = status::BadRequest;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: err.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                }
            };

            // ensure there is at least one attribute to update
            if !update_deck_request.should_update() {

                let ref reason = format!("Invalid deck update request.");
                let res_code = status::BadRequest;

                let err_response = ErrorResponse {
                    status: res_code,
                    developerMessage: reason,
                    userMessage: reason,
                }.to_json();

                return Ok(Response::with((res_code, err_response)));
            }

            // ensure deck to be updated exists
            match deck_exists(grokdb, deck_id) {
                Err(response) => {
                    return response;
                },
                _ => {/* updating deck exists; continue */}
            }

            // if deck is to be moved to a new parent, check if parent exists
            match update_deck_request.parent {
                Some(parent_deck_id) => {

                    match deck_exists(grokdb, parent_deck_id) {
                        Err(response) => {
                            return response;
                        },
                        _ => {/* noop */}
                    }

                    let should_move: bool = match grokdb.decks.has_parent(deck_id) {

                        Err(why) => {
                            // why: QueryError

                            let ref reason = format!("{:?}", why);
                            let res_code = status::InternalServerError;

                            let err_response = ErrorResponse {
                                status: res_code,
                                developerMessage: reason,
                                userMessage: why.description(),
                            }.to_json();

                            return Ok(Response::with((res_code, err_response)));
                        },

                        Ok(false) => true,

                        Ok(true) => {

                            let deck_parent: i64 = match grokdb.decks.get_parent(deck_id) {
                                Err(why) => {
                                    // why: QueryError

                                    let ref reason = format!("{:?}", why);
                                    let res_code = status::InternalServerError;

                                    let err_response = ErrorResponse {
                                        status: res_code,
                                        developerMessage: reason,
                                        userMessage: why.description(),
                                    }.to_json();

                                    return Ok(Response::with((res_code, err_response)));
                                },
                                Ok(parent) => parent
                            };

                            deck_parent != parent_deck_id
                        }
                    };

                    if should_move {

                        // parent deck different, move deck to new parent
                        match grokdb.decks.connect_decks(deck_id, parent_deck_id) {
                            Err(why) => {
                                // why: QueryError
                                let ref reason = format!("{:?}", why);
                                let res_code = status::InternalServerError;

                                let err_response = ErrorResponse {
                                    status: res_code,
                                    developerMessage: reason,
                                    userMessage: why.description(),
                                }.to_json();

                                return Ok(Response::with((res_code, err_response)));
                            },
                            _ => {/* connected new deck with parent; continue */}
                        }
                    }
                },
                _ => {/* noop; continue */}
            }

            // update deck props
            if update_deck_request.should_update_deck_props() {

                match grokdb.decks.update(deck_id, update_deck_request) {
                    Err(why) => {
                        // why: QueryError

                        let ref reason = format!("{:?}", why);
                        let res_code = status::InternalServerError;

                        let err_response = ErrorResponse {
                            status: res_code,
                            developerMessage: reason,
                            userMessage: why.description(),
                        }.to_json();

                        return Ok(Response::with((res_code, err_response)));
                    },
                    _ => {/* deck updated */}
                }

            }

            return get_deck_by_id(grokdb.clone(), deck_id);
        }
    });

    router.delete("/decks/:deck_id/parent", {
        let grokdb = grokdb.clone();
        move |req: &mut Request| -> IronResult<Response> {
            let ref grokdb = grokdb.deref();

            // fetch and parse requested deck id

            let deck_id = req.extensions.get::<Router>().unwrap().find("deck_id").unwrap();

            let deck_id: i64 = match deck_id.parse::<u64>() {
                Ok(deck_id) => deck_id as i64,
                Err(why) => {

                    let ref reason = format!("{:?}", why);
                    let res_code = status::BadRequest;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: why.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                }
            };

            // delete deck

            match grokdb.decks.remove_parent(deck_id) {
                Err(why) => {
                    // why: QueryError
                    let ref reason = format!("{:?}", why);
                    let res_code = status::InternalServerError;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: why.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                },
                _ => {/* deck's parent sucessfully disconnected */},
            };

            return get_deck_by_id(grokdb.clone(), deck_id);
        }
    });

}

/* helpers */

fn get_deck_by_id(grokdb: &GrokDB, deck_id: i64) -> IronResult<Response> {

    let maybe_deck: Result<DeckResponse, QueryError> = grokdb.decks.get_response(deck_id);

    // TODO: check if deck exists
    // TODO: add param option to skip the deck check

    let deck: DeckResponse = match maybe_deck {

        Err(why) => {
            // why: QueryError

            let ref reason = format!("{:?}", why);
            let res_code = status::NotFound;

            let err_response = ErrorResponse {
                status: res_code,
                developerMessage: reason,
                userMessage: why.description(),
            }.to_json();

            return Ok(Response::with((res_code, err_response)));
        },

        Ok(deck) => deck,
    };

    let response = deck.to_json();

    let content_type = "application/json".parse::<Mime>().unwrap();

    return Ok(Response::with((content_type, status::Ok, response)));
}

pub fn deck_exists(grokdb: &GrokDB, deck_id: i64) -> Result<(), IronResult<Response>> {

    match grokdb.decks.exists(deck_id) {

        Err(why) => {
            // why: QueryError

            let ref reason = format!("{:?}", why);
            let res_code = status::InternalServerError;

            let err_response = ErrorResponse {
                status: res_code,
                developerMessage: reason,
                userMessage: why.description(),
            }.to_json();

            let res = Ok(Response::with((res_code, err_response)));
            return Err(res);
        },

        Ok(false) => {
            let ref reason = format!("given deck id does not exist: {}", deck_id);
            let res_code = status::NotFound;

            let err_response = ErrorResponse {
                status: res_code,
                developerMessage: reason,
                userMessage: reason,
            }.to_json();

            let res = Ok(Response::with((res_code, err_response)));
            return Err(res);
        },

        _ => {
            return Ok(());
        }
    }
}
