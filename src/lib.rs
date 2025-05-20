pub mod chunker;
use std::io::{Read, Write};

use chunker::divide_into_chunks;
// Import required modules from the LLM library
use llm::{
    builder::{LLMBackend, LLMBuilder},
    chat::ChatMessage,
    LLMProvider,
};

/// The models preinstalled on my PC
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
/// Model builder struct
pub struct OllamaModelBuilder {
    /// Model name
    model: String,
    /// System prompt (can be empty)
    sys_prompt: String,
}

impl OllamaModelBuilder {
    /// Initializes builder struct
    pub fn new(model: impl Into<String>) -> Self {
        let model_str: String = model.into();
        OllamaModelBuilder {
            model: model_str,
            sys_prompt: String::new(),
        }
    }
    /// Sets system prompt that will be provided to the LLM before each request
    pub fn set_system_prompt(mut self, prompt: String) -> OllamaModelBuilder {
        self.sys_prompt = prompt;

        self
    }
    /// Build's the [OllamaModel] struct
    pub fn build(self) -> OllamaModel {
        OllamaModel::build_from_builder(self)
    }
}

/// Struct containing a model and it's system prompt
pub struct OllamaModel {
    /// LLM provider
    llm: Box<dyn LLMProvider>,
    /// System prompt that will be set for the model before each request
    sys_prompt: String,
}

impl OllamaModel {
    fn build_from_builder(builder: OllamaModelBuilder) -> OllamaModel {
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
            llm,
            sys_prompt: builder.sys_prompt,
        }
    }

    /// Provides a requests to the model and returns it's response
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

/// Asks the provided LLM, get's it's response, extract the contents and writes it to the given
/// file
pub async fn ask_estract_contents_and_write_responses_to_file(
    llm: OllamaModel,
    message: String,
    output_path: &str,
) {
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
        let response = llm.ask(chunk).await;
        let res = chunker::extract_translated_from_response(response);

        let _ = file.write_fmt(format_args!("{}", res));
        println!("Chunked processed: {}", chunk_num);
        chunk_num += 1;
    }
}
