use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub jwt_access_secret: String,
    pub jwt_refresh_secret: String,
    pub jwt_access_expiry: String,
    pub jwt_refresh_expiry: String,
    pub client_origin: String,
    pub node_env: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let port = env::var("PORT")
            .unwrap_or_else(|_| "5000".to_string())
            .parse::<u16>()
            .expect("PORT must be a valid u16 number");

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in .env");

        let jwt_access_secret = env::var("JWT_ACCESS_SECRET")
            .expect("JWT_ACCESS_SECRET must be set in .env");

        let jwt_refresh_secret = env::var("JWT_REFRESH_SECRET")
            .expect("JWT_REFRESH_SECRET must be set in .env");

        let jwt_access_expiry = env::var("JWT_ACCESS_EXPIRY")
            .unwrap_or_else(|_| "15m".to_string());

        let jwt_refresh_expiry = env::var("JWT_REFRESH_EXPIRY")
            .unwrap_or_else(|_| "7d".to_string());

        let client_origin = env::var("CLIENT_ORIGIN")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());

        let node_env = env::var("NODE_ENV")
            .unwrap_or_else(|_| "development".to_string());

        Config {
            port,
            database_url,
            jwt_access_secret,
            jwt_refresh_secret,
            jwt_access_expiry,
            jwt_refresh_expiry,
            client_origin,
            node_env,
        }
    }

    pub fn is_prod(&self) -> bool {
        self.node_env == "production"
    }
}
