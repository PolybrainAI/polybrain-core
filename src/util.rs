
/// Gets a dotenv variable. Panics if unbound
pub fn get_dotenv(key: &str) -> String{
    std::env::var(key).expect(format!("{key} must be set in .env").as_str())
}