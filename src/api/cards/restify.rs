extern crate iron;
extern crate router;
extern crate bodyparser;
extern crate rustc_serialize;

use iron::status;
use iron::prelude::*;
use router::Router;
use rustc_serialize::json;

use std::sync::Arc;
use std::ops::Deref;
use std::error::Error;

use ::api::{GrokDB, ErrorResponse};
use ::api::cards::{CreateCard, Card, CardResponse};
use ::api::decks::restify::deck_exists;
use ::database::QueryError;

// attach decks REST endpoints to given router
pub fn restify(router: &mut Router, grokdb: GrokDB) {

    let grokdb = Arc::new(grokdb);

    router.post("/cards", {
        let grokdb = grokdb.clone();
        move |req: &mut Request| -> IronResult<Response> {
            let ref grokdb = grokdb.deref();

            // parse json

            let create_card_request = req.get::<bodyparser::Struct<CreateCard>>();

            let create_card_request: CreateCard = match create_card_request {

                Ok(Some(create_card_request)) => {

                    let create_card_request: CreateCard = create_card_request;

                    // ensure deck exists; otherwise bail early
                    match deck_exists(grokdb, create_card_request.deck) {
                        Err(response) => {
                            return response;
                        },
                        _ => {/* noop; continue */}
                    }
                    create_card_request
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

            // create card

            let card_id: i64 = match grokdb.cards.create(&create_card_request) {
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
                Ok(card_id) => {
                    // card_id: i64
                    /* card sucessfully created */
                    card_id
                },
            };

            return get_card_by_id(grokdb.clone(), card_id);
        }
    });

}

/* helpers */

fn get_card_by_id(grokdb: &GrokDB, card_id: i64) -> IronResult<Response> {

    let maybe_card: Result<CardResponse, QueryError> = grokdb.cards.get_response(card_id);

    let card: CardResponse = match maybe_card {

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

        Ok(card) => card,
    };

    let response = card.to_json();

    return Ok(Response::with((status::Ok, response)));
}
