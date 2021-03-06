extern crate iron;
extern crate router;
extern crate bodyparser;
extern crate rustc_serialize;

use iron::status;
use iron::prelude::*;
use router::Router;

use std::sync::Arc;
use std::ops::Deref;
use std::error::Error;

use ::api::{GrokDB, ErrorResponse};
use ::api::decks::reviewable::{ReviewableDeck};
use ::api::decks::restify::{deck_exists};
use ::api::stashes::reviewable::{ReviewableStash};
use ::api::stashes::restify::{stash_exists};
use ::api::cards::restify::{get_card_by_id, card_exists};
use ::api::review::{get_review_card, UpdateCardScore, ReviewableSelection};


// attach review REST endpoints to given router
pub fn restify(router: &mut Router, grokdb: GrokDB) {

    let grokdb = Arc::new(grokdb);

    router.patch("/cards/:card_id/review", {
        let grokdb = grokdb.clone();
        let grokdb_arc = grokdb.clone();
        move |req: &mut Request| -> IronResult<Response> {
            let ref grokdb = grokdb.deref();

            // TODO: refactor; prefer to capture :card_id first before parsing UpdateCardScore
            let update_card_score_request = req.get::<bodyparser::Struct<UpdateCardScore>>();

            // fetch and parse requested card id
            let card_id: &str = req.extensions.get::<Router>().unwrap().find("card_id").unwrap();

            let card_id: i64 = match card_id.parse::<u64>() {
                Ok(card_id) => card_id as i64,
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

            // ensure card exists
            match card_exists(grokdb, card_id) {
                Err(response) => {
                    return response;
                },
                _ => {/* card exists; continue */}
            }

            // parse json

            let update_card_score_request: UpdateCardScore = match update_card_score_request {

                Ok(Some(update_card_score_request)) => {
                    let update_card_score_request: UpdateCardScore = update_card_score_request;
                    update_card_score_request
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

            // ensure card score update is valid
            if !update_card_score_request.should_update() {

                let ref reason = format!("Invalid card score update request.");
                let res_code = status::BadRequest;

                let err_response = ErrorResponse {
                    status: res_code,
                    developerMessage: reason,
                    userMessage: reason,
                }.to_json();

                return Ok(Response::with((res_code, err_response)));
            }

            // if given deck, ensure it exists
            if update_card_score_request.deck.is_some() {

                match deck_exists(grokdb, update_card_score_request.deck.unwrap()) {
                    Err(response) => {
                        return response;
                    },
                    _ => {/* deck exists; continue */}
                }

                // invariant: cached card implies it was once put for review

                // check if cache exists
                // TODO: is this necessary?
            }

            // if given stash, ensure it exists
            if update_card_score_request.stash.is_some() {

                match stash_exists(grokdb, update_card_score_request.stash.unwrap()) {
                    Err(response) => {
                        return response;
                    },
                    _ => {/* stash exists; continue */}
                }

                // check if cache exists
                // TODO: is this necessary?
            }

            // update card score
            match grokdb.review.update_reviewed_card(card_id, update_card_score_request) {
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
                _ => {/* card score updated */}
            }

            // remove cached review card for any container type.
            //
            // rationale:
            // If a card was reviewed, either in a deck or stash (or none),
            // then if the card was cached for review in any other deck or stash,
            // the user shouldn't expect to see that card again if he or she chooses to review
            // another deck or stash (or the same deck/stash he or she was reviewing).
            // However, the exception is that if the card is the only card within a deck or stash,
            // then that same card will be shown for review.
            let deck_selection = ReviewableDeck {
                deck_id: 0, // doesn't matter which deck
                grokdb: grokdb_arc.clone()
            };
            let stash_selection = ReviewableStash {
                stash_id: 0, // doesn't matter which stash
                grokdb: grokdb_arc.clone()
            };

            match deck_selection.remove_cached_card(card_id) {
                Err(err) => {

                    let ref reason = format!("{:?}", err);
                    let res_code = status::InternalServerError;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: err.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                },
                _ => {
                    // cache removed
                }
            }

            match stash_selection.remove_cached_card(card_id) {
                Err(err) => {

                    let ref reason = format!("{:?}", err);
                    let res_code = status::InternalServerError;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: err.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                },
                _ => {
                    // cache removed
                }
            }

            return get_card_by_id(grokdb.clone(), card_id);
        }
    });

    router.get("/decks/:deck_id/review", {
        let grokdb = grokdb.clone();
        let grokdb_arc = grokdb.clone();
        move |req: &mut Request| -> IronResult<Response> {
            let ref grokdb = grokdb.deref();

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

            // ensure deck exists
            match deck_exists(grokdb, deck_id) {
                Err(response) => {
                    return response;
                },
                _ => {/* deck exists; continue */}
            }

            let deck_selection = ReviewableDeck {
                deck_id: deck_id,
                grokdb: grokdb_arc.clone()
            };

            match get_review_card(&deck_selection) {
                Err(why) => {

                    let ref reason = format!("{:?}", why);
                    let res_code = status::InternalServerError;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: why.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                },

                Ok(None) => {

                    let ref reason = format!("No card to review");
                    let res_code = status::NotFound;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: reason,
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                },

                Ok(Some(card_id)) => {
                    return get_card_by_id(grokdb.clone(), card_id);
                }
            }
        }
    });

    router.get("/stashes/:stash_id/review", {
        let grokdb = grokdb.clone();
        let grokdb_arc = grokdb.clone();
        move |req: &mut Request| -> IronResult<Response> {
            let ref grokdb = grokdb.deref();

            let stash_id = req.extensions.get::<Router>().unwrap().find("stash_id").unwrap();

            let stash_id: i64 = match stash_id.parse::<u64>() {
                Ok(stash_id) => stash_id as i64,
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

            // ensure stash exists
            match stash_exists(grokdb, stash_id) {
                Err(response) => {
                    return response;
                },
                _ => {/* stash exists; continue */}
            }

            let stash_selection = ReviewableStash {
                stash_id: stash_id,
                grokdb: grokdb_arc.clone()
            };

            match get_review_card(&stash_selection) {
                Err(why) => {

                    let ref reason = format!("{:?}", why);
                    let res_code = status::InternalServerError;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: why.description(),
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                },

                Ok(None) => {

                    let ref reason = format!("No card to review");
                    let res_code = status::NotFound;

                    let err_response = ErrorResponse {
                        status: res_code,
                        developerMessage: reason,
                        userMessage: reason,
                    }.to_json();

                    return Ok(Response::with((res_code, err_response)));
                },

                Ok(Some(card_id)) => {
                    return get_card_by_id(grokdb.clone(), card_id);
                }
            }
        }
    });
}
