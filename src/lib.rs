pub mod chunker;
pub mod gemma;
use loggit::debug;
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
    Aya8B,
}
impl From<DobbiKovModels> for String {
    fn from(value: DobbiKovModels) -> Self {
        match value {
            DobbiKovModels::Gemma312b => "gemma3:12b-it-qat",
            DobbiKovModels::Aya8B => "aya:8b",
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
        let mut fin_mess = String::new();
        if self.sys_prompt.len() > 0 {
            //messages.push(ChatMessage::user().content(&self.sys_prompt).build())
            //fin_mess.push_str(&self.sys_prompt);
        }
        fin_mess.push_str(&message);
        messages.push(ChatMessage::user().content(&fin_mess).build());
        debug!("---Full prompt: \n{}", &fin_mess);
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
    translate_prompt: String,
    fixer_prompt: String,
    message: String,
    output_path: &str,
    lines_per_chunk: usize,
) {
    let divided = divide_into_chunks(message, lines_per_chunk);
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(output_path)
        .expect("Incorrect path");

    println!("Total number of chunks: {}", divided.len());
    let mut chunk_num = 1;
    let _ = loggit::logger::set_file("llm_debug_output.txt");
    let _ = loggit::logger::set_log_level(loggit::Level::DEBUG);
    let _ = loggit::logger::set_print_to_terminal(false);
    for chunk in divided {
        let mut request = String::new();
        request.push_str(&translate_prompt);
        request.push_str("<document>\n");
        request.push_str(&chunk);
        request.push_str("</document>");

        let response = llm.ask(request).await;

        debug!("\nChunk {}, contents:", chunk_num);
        debug!("\n{}", response);
        let res = chunker::extract_translated_from_response(response);

        let corr_res = compare_and_return_fixed(&llm, &fixer_prompt, chunk, res).await;

        let _ = file.write_fmt(format_args!("{}", corr_res));
        println!("Chunked processed: {}", chunk_num);
        chunk_num += 1;
    }
}

async fn compare_and_return_fixed(
    llm: &OllamaModel,
    fixer_prompt: &str,
    original: String,
    output: String,
) -> String {
    let mut request = String::new();
    request.push_str(fixer_prompt);
    request.push_str("<document>\n");
    request.push_str(&original);
    request.push_str("</document>\n");
    request.push_str("<translated>\n");
    request.push_str(&output);
    request.push_str("</translated>");
    println!("{}", &request);

    let response = llm.ask(request).await;
    println!("----");
    println!("{}", &response);
    let res = chunker::extract_translated_from_response(response);
    res
}
