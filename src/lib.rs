use std::{
    net::IpAddr,
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use worker::*;

mod utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub colo: String,
    pub asn: u32,
    pub country: Option<String>,
    pub city: Option<String>,
    pub continent: Option<String>,
    pub coordinates: Option<(f32, f32)>,
    pub postal_code: Option<String>,
    pub metro_code: Option<String>,
    pub region: Option<String>,
    pub region_code: Option<String>,
    pub http_version: String,
    #[serde(with = "serde_millis")]
    pub time: SystemTime,
}

pub fn date_to_system_time(date: Date) -> SystemTime {
    let millis = date.as_millis();
    let duration = Duration::from_millis(millis);
    UNIX_EPOCH + duration
}

impl From<&Cf> for Location {
    fn from(cf: &Cf) -> Self {
        let colo = cf.colo();
        let asn = cf.asn();
        let country = cf.country();
        let city = cf.city();
        let continent = cf.continent();
        let coordinates = cf.coordinates();
        let postal_code = cf.postal_code();
        let metro_code = cf.metro_code();
        let region = cf.region();
        let region_code = cf.region_code();
        let http_version = cf.http_protocol();
        let time = date_to_system_time(Date::now());
        Self {
            colo,
            asn,
            country,
            city,
            continent,
            coordinates,
            postal_code,
            metro_code,
            region,
            region_code,
            http_version,
            time,
        }
    }
}

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    router
        .get_async("/", |req, _| async move {
            let cache = Cache::default();
            if let Some(resp) = cache.get(&req, false).await? {
                console_log!("Cached response");
                return Ok(resp);
            }
            const HTML_CONTENT: &'static str = include_str!("../index.html");
            let mut headers = Headers::new();
            headers.set("Cache-Control", "public,max-age=1000")?;
            let mut resp = Response::from_html(HTML_CONTENT)?
                .with_headers(headers);
            cache.put(&req, resp.cloned()?).await?;
            Ok(resp)
        })
        .get_async("/index.js", |req, _| async move {
            let cache = Cache::default();
            if let Some(resp) = cache.get(&req, false).await? {
                console_log!("Cached response");
                return Ok(resp);
            }
            const JS_CONTENT: &'static str = include_str!("../index.js");
            let body = JS_CONTENT.as_bytes().to_vec();
            let mut headers = Headers::new();
            headers.set("Cache-Control", "public,max-age=1000")?;
            headers.set("Content-Type", "text/javascript")?;
            let mut resp = Response::from_body(ResponseBody::Body(body))?
                .with_headers(headers);
            cache.put(&req, resp.cloned()?).await?;
            Ok(resp)
        })
        .get_async("/location", |req, ctx| async move {
            let cf = req.cf();
            let loc = Location::from(cf);
            let req_headers = req.headers();
            if let Some(ip_string) = req_headers.get("x-real-ip")? {
                match IpAddr::from_str(&ip_string) {
                    Ok(ip) => {
                        let store = ctx.kv("IP_LOCATIONS")?;
                        let ip_key_str = ip.to_string();
                        let builder = store.put(&ip_key_str, &loc)?;
                        builder.execute().await?;
                    }
                    Err(e) => {
                        console_error!("Failed to parse ip address: {:?}", e);
                    }
                }
            }
            let location_headers = {
                let mut ret = Headers::new();
                ret.set("Cache-Control", "no-store, no-transform").unwrap();
                ret
            };
            let response = Response::from_json(&loc)?.with_headers(location_headers);
            Ok(response)
        })
        .get("/worker-version", |_, ctx| {
            let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
            Response::ok(version)
        })
        .run(req, env)
        .await
}
