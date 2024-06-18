/// Gets a dotenv variable. Panics if unbound
pub fn get_dotenv(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("{key} must be set in .env"))
}
