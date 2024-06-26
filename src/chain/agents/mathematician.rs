pub struct MathematicianAgent<'a> {
    openai_key: &'a String,
}
impl<'a> MathematicianAgent<'a> {
    pub fn new(openai_key: &String) -> MathematicianAgent {
        MathematicianAgent { openai_key }
    }

    pub async fn run(&self) -> String {
        return "No math notes".to_string();
    }
}
