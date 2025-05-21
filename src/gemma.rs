// Import required modules from the LLM library for Google Gemini integration
use google_genai::datatypes::{Content, GenerateContentParameters, Part};
use llm::{
    builder::{LLMBackend, LLMBuilder}, // Builder pattern components
    chat::ChatMessage,
    LLMProvider, // Chat-related structures
};

pub async fn build_gemma_model(
    sys_prompt: String,
) -> Result<Box<dyn LLMProvider>, Box<dyn std::error::Error>> {
    // Get Google API key from environment variable or use test key as fallback
    let api_key = std::env::var("GOOGLE_API_KEY").unwrap_or("google-key".into());

    // Initialize and configure the LLM client
    let llm = LLMBuilder::new()
        .backend(LLMBackend::Google) // Use Google as the LLM provider
        .api_key(api_key) // Set the API key
        .model("gemma-3-27b-it") // Use Gemini Pro model
        .max_tokens(8512) // Limit response length
        .temperature(0.7) // Control response randomness (0.0-1.0)
        .stream(false) // Disable streaming responses
        // Optional: Set system prompt
        .system(sys_prompt)
        .build()
        .expect("Failed to build LLM (Google)");

    Ok(llm)
}

pub async fn ask_gemma_model(message: String) -> String {
    let api_key =
        std::env::var("GOOGLE_API_KEY").expect("GOOGLEAI_API_KEY environment variable must be set");

    let params = GenerateContentParameters::default()
        .contents(vec![Content {
            parts: Some(vec![Part::default().text(message)]),
            role: Some("user".to_string()),
        }])
        .model("gemma-3-27b-it");

    let request = google_genai::datatypes::GenerateContentReq::default()
        .contents(params.contents.unwrap())
        .model(params.model.unwrap());

    let response = google_genai::generate_content(&api_key, request)
        .await
        .unwrap();

    println!("Response: {:#?}", response);
    String::new()
}
