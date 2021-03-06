//! This is a sample program for running a Rocket.rs server. This redirects
//! users to Discord's authorization page, requesting the `identity` scope.
//!
//! Once they have authorized, it will take the `code` given and then exchange
//! it for an access token, which can be used to access the user's identity.
//!
//! This example requires the following environment variables, both available
//! from your Discord application's settings:
//!
//! - `DISCORD_CLIENT_ID`
//! - `DISCORD_CLIENT_SECRET`
//!
//! You will also need to register a redirect URI. Running this locally would
//! cause the redirect URI to be `http://localhost:8000` for example. This can
//! be registered in your application's settings.
//!
//! Example of how to run this:
//!
//! `$ git clone https://github.com/serenity-rs/oauth`
//! `$ cd oauth`
//! `$ DISCORD_CLIENT_SECRET=my_secret DISCORD_CLIENT_ID=my_client_id cargo run --example rocket`

#![feature(custom_derive, plugin)]
#![plugin(rocket_codegen)]

extern crate hyper;
extern crate hyper_native_tls;
extern crate serenity_oauth;
extern crate rocket;

use hyper::net::HttpsConnector;
use hyper::Client as HyperClient;
use hyper_native_tls::NativeTlsClient;
use rocket::response::Redirect;
use serenity_oauth::model::AccessTokenExchangeRequest;
use serenity_oauth::{DiscordOAuthHyperRequester, Scope};
use std::env;
use std::error::Error;

#[derive(Debug, FromForm)]
struct Params {
    code: String,
}

fn get_client_id() -> u64 {
    env::var("DISCORD_CLIENT_ID")
        .expect("No DISCORD_CLIENT_ID present")
        .parse::<u64>()
        .expect("Error parsing DISCORD_CLIENT_ID into u64")
}

fn get_client_secret() -> String {
    env::var("DISCORD_CLIENT_SECRET")
        .expect("No DISCORD_CLIENT_SECRET present")
}

#[get("/callback?<params>")]
fn get_callback(params: Params) -> Result<String, Box<Error>> {
    // Exchange the code for an access token.
    let ssl = NativeTlsClient::new()?;
    let connector = HttpsConnector::new(ssl);
    let client = HyperClient::with_connector(connector);

    let response = client.exchange_code(&AccessTokenExchangeRequest::new(
        get_client_id(),
        get_client_secret(),
        params.code,
        "http://localhost:8000/callback",
    ))?;

    Ok(format!("The user's access token is: {}", response.access_token))
}

#[get("/")]
fn get_redirect() -> Redirect {
    // Although this example does not use a state, you _should always_ use one
    // in production for security purposes.
    let url = serenity_oauth::utils::authorization_code_grant_url(
        get_client_id(),
        &[Scope::Identify],
        None,
        "http://localhost:8000/callback",
    );

    Redirect::to(&url)
}

fn main() {
    rocket::ignite()
        .mount("/", routes![
            get_callback,
            get_redirect,
        ]).launch();
}
