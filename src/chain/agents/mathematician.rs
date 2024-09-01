pub struct MathematicianAgent<'a> {
    _openai_key: &'a String,
}
impl<'a> MathematicianAgent<'a> {
    pub fn new(openai_key: &String) -> MathematicianAgent {
        MathematicianAgent {
            _openai_key: openai_key,
        }
    }

    pub async fn run(&self) -> String {
        "No math notes".to_owned()
    }
}
