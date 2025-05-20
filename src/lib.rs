pub mod chunker;
use std::io::{Read, Write};

use chunker::divide_into_chunks;
// Import required modules from the LLM library
use llm::{
    builder::{LLMBackend, LLMBuilder},
    chat::ChatMessage,
    LLMProvider,
};

pub enum DobbiKovModels {
    Gemma312b,
}
impl From<DobbiKovModels> for String {
    fn from(value: DobbiKovModels) -> Self {
        match value {
            DobbiKovModels::Gemma312b => "gemma3:12b-it-qat",
        }
        .to_string()
    }
}

#[derive(Clone)]
pub struct OllamaModelBuilder {
    model: String,
    sys_prompt: String,
}

impl OllamaModelBuilder {
    pub fn new(model: impl Into<String>) -> Self {
        let model_str: String = model.into();
        OllamaModelBuilder {
            model: model_str,
            sys_prompt: String::new(),
        }
    }
    pub fn set_system_prompt(mut self, prompt: String) -> OllamaModelBuilder {
        self.sys_prompt = prompt;

        self
    }
    pub fn build(self) -> OllamaModel {
        OllamaModel::build_from_builder(self)
    }
}

pub struct OllamaModel {
    model: String,
    llm: Box<dyn LLMProvider>,
    sys_prompt: String,
}

impl OllamaModel {
    pub fn build_from_builder(builder: OllamaModelBuilder) -> OllamaModel {
        // probably the base_url logic should be changed
        let base_url = std::env::var("OLLAMA_URL").unwrap_or("http://127.0.0.1:11434".into());

        // Initialize and configure the LLM client
        let llm = LLMBuilder::new()
            .backend(LLMBackend::Ollama) // Use Ollama as the LLM backend
            .base_url(base_url) // Set the Ollama server URL
            .model(&builder.model)
            .max_tokens(10000) // Set maximum response length
            .temperature(0.7) // Control response randomness (0.0-1.0)
            .stream(false) // Disable streaming responses
            .build()
            .expect("Failed to build LLM (Ollama)");

        Self {
            model: builder.model,
            llm,
            sys_prompt: builder.sys_prompt,
        }
    }
    pub async fn ask(&self, message: String) -> String {
        let mut messages: Vec<ChatMessage> = vec![];
        if self.sys_prompt.len() > 0 {
            messages.push(ChatMessage::user().content(&self.sys_prompt).build())
        }
        messages.push(ChatMessage::user().content(message).build());
        //
        //// Send chat request and handle the response
        match self.llm.chat(&messages).await {
            Ok(text) => text.to_string(),
            Err(e) => format!("Chat error: {}", e),
        }
    }
}

pub fn read_string_file(path: &str) -> String {
    let mut contents = String::new();
    let mut file = std::fs::File::open(std::path::PathBuf::from(path)).expect("Couldn't open file");
    file.read_to_string(&mut contents);
    contents
}

pub async fn ask_and_write_responses_to_file(llm: OllamaModel, message: String, output_path: &str) {
    let lines_per_chunk: usize = 20;
    let divided = divide_into_chunks(message, lines_per_chunk);
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(output_path)
        .expect("Incorrect path");

    println!("Total number of chunks: {}", divided.len());
    let mut chunk_num = 1;
    for chunk in divided {
        let res = llm.ask(chunk).await;

        let _ = file.write_fmt(format_args!("{}", res));
        println!("Chunked processed: {}", chunk_num);
        chunk_num += 1;
    }
}
