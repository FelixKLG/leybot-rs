use chrono::{DateTime, Days, Utc};
use error_stack::{Context, IntoReport, Report, Result, ResultExt};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct HttpClientError;

impl std::fmt::Display for HttpClientError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("HTTP Error: An error occurred while building the HTTP client")
    }
}

impl Context for HttpClientError {}

#[derive(Debug)]
pub struct LinkClientHTTPError;

impl std::fmt::Display for LinkClientHTTPError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("HTTP Error: An error occurred in the Link Client")
    }
}

impl Context for LinkClientHTTPError {}

#[derive(Debug)]
pub struct GMSClientHTTPError;

impl std::fmt::Display for GMSClientHTTPError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("HTTP Error: An error occurred in the Link Client")
    }
}

impl Context for GMSClientHTTPError {}

#[derive(Debug)]
pub struct CouponBuilderError;

impl std::fmt::Display for CouponBuilderError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("An error occurred whilst building the coupon")
    }
}

impl Context for CouponBuilderError {}

pub struct HttpClient {
    pub link_client: LinkClient,
    pub gmod_store_client: GmodStoreClient,
}

impl HttpClient {
    pub fn new() -> Result<Self, HttpClientError> {
        let link_client = LinkClient::new()?;
        let gmod_store_client = GmodStoreClient::new()?;
        Ok(Self {
            link_client,
            gmod_store_client,
        })
    }
}

pub struct LinkClient {
    client: Client,
    pub url: String,
}

impl LinkClient {
    pub fn new() -> Result<Self, HttpClientError> {
        let api_key = Self::get_token()?;
        let api_url = Self::get_url()?;

        let mut api_headers = reqwest::header::HeaderMap::new();
        api_headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", api_key))
                .into_report()
                .attach_printable_lazy(|| format!("Failed to parse API key: {}", api_key))
                .change_context(HttpClientError)?,
        );

        let api_http_builder = reqwest::Client::builder().default_headers(api_headers);

        let api_http = api_http_builder
            .build()
            .into_report()
            .attach_printable("Failed to build HTTP client")
            .change_context(HttpClientError)?;
        Ok(Self {
            client: api_http,
            url: api_url,
        })
    }

    fn get_url() -> Result<String, HttpClientError> {
        let url = crate::misc::get_env("API_ENDPOINT")
            .attach_printable("Failed to read environment variable: API_ENDPOINT")
            .change_context(HttpClientError)?;

        let url_chars = url.chars().last();

        match url_chars {
            Some('/') => Err(Report::new(HttpClientError)
                .attach_printable("API_ENDPOINT ends with a slash ('/') REMOVE IT!")),
            None => {
                Err(Report::new(HttpClientError).attach_printable("API_ENDPOINT is not defined"))
            }
            _ => Ok(url),
        }
    }

    fn get_token() -> Result<String, HttpClientError> {
        let token = crate::misc::get_env("API_TOKEN")
            .attach_printable("Failed to read environment variable: API_TOKEN")
            .change_context(HttpClientError)?;
        Ok(token)
    }
}

pub struct GmodStoreClient {
    client: Client,
    url: String,
}

impl GmodStoreClient {
    pub fn new() -> Result<Self, HttpClientError> {
        let api_key = Self::get_token()?;
        let api_url = Self::get_url();

        let mut api_headers = reqwest::header::HeaderMap::new();
        api_headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", api_key))
                .into_report()
                .attach_printable_lazy(|| format!("Failed to parse API key: {}", api_key))
                .change_context(HttpClientError)?,
        );

        let api_http_builder = reqwest::Client::builder().default_headers(api_headers);

        let api_http = api_http_builder
            .build()
            .into_report()
            .attach_printable("Failed to build GmodStore HTTP client")
            .change_context(HttpClientError)?;
        Ok(Self {
            client: api_http,
            url: api_url,
        })
    }

    fn get_token() -> Result<String, HttpClientError> {
        let token = crate::misc::get_env("GMS_PAT")
            .attach_printable("Failed to read environment variable: GMS_PAT")
            .change_context(HttpClientError)?;
        Ok(token)
    }

    fn get_url() -> String {
        String::from("https://www.gmodstore.com/api/v3")
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiUserResponse {
    pub data: ApiUserObject,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiUserObject {
    pub uuid: String,
    pub name: Option<String>,
    #[serde(rename = "steamId")]
    pub steam_id: u64,
    #[serde(rename = "discordId")]
    pub discord_id: Option<u64>,
    #[serde(rename = "gmodStoreId")]
    pub gmod_store_id: Option<String>,
    pub avatar: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiPurchasesResponse {
    pub data: ApiPurchaseObject,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiPurchaseObject {
    #[serde(rename = "LSAC")]
    pub lsac: bool,
    #[serde(rename = "SwiftAC")]
    pub swift_ac: bool,
    #[serde(rename = "HitReg")]
    pub hit_reg: bool,
    #[serde(rename = "ScreenGrabs")]
    pub screen_grabs: bool,
    #[serde(rename = "WorkshopDL")]
    pub workshop_dl: bool,
    #[serde(rename = "SexyErrors")]
    pub sexy_errors: bool,
}

pub struct User<'a> {
    pub uuid: String,
    pub name: Option<String>,
    pub steam_id: u64,
    pub discord_id: Option<u64>,
    pub gmod_store_id: Option<String>,
    pub avatar: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    http: &'a LinkClient,
}

impl LinkClient {
    pub async fn get_user_by_discord(
        &self,
        discord_id: u64,
    ) -> Result<Option<User>, LinkClientHTTPError> {
        let url = format!("{}/api/users/discord/{}", self.url, discord_id);

        let response = self
            .client
            .get(url)
            .send()
            .await
            .into_report()
            .attach_printable("An error occurred while fetching from the API")
            .change_context(LinkClientHTTPError)?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let user_api = response
            .json::<ApiUserResponse>()
            .await
            .into_report()
            .attach_printable("An error occurred whilst serializing the API response")
            .change_context(LinkClientHTTPError)?
            .data;

        Ok(Some(User {
            uuid: user_api.uuid,
            name: user_api.name,
            steam_id: user_api.steam_id,
            discord_id: user_api.discord_id,
            gmod_store_id: user_api.gmod_store_id,
            avatar: user_api.avatar,
            created_at: user_api.created_at,
            updated_at: user_api.updated_at,
            http: self,
        }))
    }
}

impl User<'_> {
    pub async fn get_purchases(&self) -> Result<ApiPurchaseObject, LinkClientHTTPError> {
        let url = format!("{}/api/users/{}/purchases", self.http.url, self.uuid);

        let response = self
            .http
            .client
            .get(url)
            .send()
            .await
            .into_report()
            .attach_printable("An error occurred while fetching from the API")
            .change_context(LinkClientHTTPError)?;

        Ok(response
            .json::<ApiPurchasesResponse>()
            .await
            .into_report()
            .attach_printable("An error occurred whilst serializing the API response")
            .change_context(LinkClientHTTPError)?
            .data)
    }

    pub async fn delete(&self) -> Result<(), LinkClientHTTPError> {
        let url = format!("{}/api/users/{}", self.http.url, self.uuid);

        self.http
            .client
            .delete(url)
            .send()
            .await
            .into_report()
            .attach_printable("Failed to send delete request to API")
            .change_context(LinkClientHTTPError)?;

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GMSCursorsObject {
    pub previous: Option<String>,
    pub next: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GMSMetaObject {
    #[serde(rename = "perPage")]
    pub per_age: u8,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GMSCouponObject {
    pub id: String,
    pub code: String,
    pub percent: u8,
    #[serde(rename = "maxUses")]
    pub max_uses: u8,
    #[serde(rename = "boundUser")]
    pub bound_user: Option<String>,
    #[serde(rename = "expiresAt")]
    pub expires_at: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GMSCouponsResponse {
    pub data: Vec<GMSCouponObject>,
    pub connections: Vec<String>,
    pub cursors: GMSCursorsObject,
    pub meta: Option<GMSMetaObject>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GMSCouponCreateResponse {
    pub data: GMSCouponObject,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CouponBuilder {
    pub code: String,
    pub percent: u8,
    #[serde(rename = "maxUses")]
    pub max_uses: u8,
    #[serde(rename = "boundUserId")]
    pub bound_user_id: Option<String>,
    #[serde(rename = "expiresAt")]
    pub expires_at: String,
}

impl CouponBuilder {
    pub fn new(
        code: String,
        percent: u8,
        max_uses: u8,
        bound_user_id: Option<String>,
    ) -> Result<Self, CouponBuilderError> {
        if percent > 90 {
            return Err(Report::new(CouponBuilderError)
                .attach_printable("'percent' argument must be less than or equal to 90"));
        }
        if percent == 0 {
            return Err(Report::new(CouponBuilderError)
                .attach_printable("'percent' argument must be greater than 0"));
        }
        if max_uses > 100 {
            return Err(Report::new(CouponBuilderError)
                .attach_printable("'max_uses' argument must be less than or equal to 100"));
        }
        if max_uses == 0 {
            return Err(Report::new(CouponBuilderError)
                .attach_printable("'max_uses' argument must be greater than 0"));
        }
        if code.len() > 64 {
            return Err(Report::new(CouponBuilderError)
                .attach_printable("'code' argument must be less than or equal to 64 characters"));
        }
        if code.is_empty() {
            return Err(Report::new(CouponBuilderError)
                .attach_printable("'code' argument must be greater than 0 characters"));
        }

        let now = Utc::now();
        let expiry = now + Days::new(7);

        Ok(Self {
            code,
            percent,
            max_uses,
            bound_user_id,
            expires_at: expiry.to_rfc3339(),
        })
    }
}

impl GmodStoreClient {
    pub async fn get_coupons_by_user(
        &self,
        user: &User<'_>,
        addon: &str,
    ) -> Result<Option<Vec<GMSCouponObject>>, GMSClientHTTPError> {
        // Format URL
        let url = format!("{}/products/{}/coupons", self.url, addon);

        let user_id = match &user.gmod_store_id {
            Some(id) => id,
            None => {
                return Err(Report::new(GMSClientHTTPError)
                    .attach_printable("Failed to get user's GmodStore ID"))
            }
        };

        // Send Request
        let response = self
            .client
            .get(url) // Get request
            .query(&[("filter[boundUserId]", user_id)]) // Filter coupons by user ID
            .send()
            .await
            .into_report()
            .attach_printable("An error occurred while fetching from the API")
            .change_context(GMSClientHTTPError)?;

        // Init empty vec of user coupons
        let mut coupons: Vec<GMSCouponObject> = Vec::new();

        // Deserialize response
        let response = response
            .json::<GMSCouponsResponse>()
            .await
            .into_report()
            .attach_printable("An error occurred whilst deserializing the API response")
            .change_context(GMSClientHTTPError)?;

        // Validate the coupon is not expired
        for x in response.data {
            let expiry: DateTime<Utc> = DateTime::parse_from_rfc3339(&x.expires_at)
                .into_report()
                .attach_printable("An error occurred whilst parsing the expiry date")
                .change_context(GMSClientHTTPError)?
                .into();

            if expiry > Utc::now() {
                coupons.push(x);
            }
        }

        if coupons.is_empty() {
            return Ok(None);
        }

        Ok(Some(coupons))
    }

    pub async fn create_coupon(
        &self,
        addon: &str,
        coupon: CouponBuilder,
    ) -> Result<GMSCouponObject, GMSClientHTTPError> {
        // Format URL
        let url = format!("{}/products/{}/coupons", self.url, addon);

        // Send Request
        let response = self
            .client
            .post(url)
            .json(&coupon)
            .send()
            .await
            .into_report()
            .attach_printable("An error occurred while fetching from the API")
            .change_context(GMSClientHTTPError)?;

        let return_value = response
            .json::<GMSCouponCreateResponse>()
            .await
            .into_report()
            .attach_printable("An error occurred whilst deserializing the API response")
            .change_context(GMSClientHTTPError)?;

        Ok(return_value.data)
    }
}
